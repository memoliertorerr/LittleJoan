//! Browsing history, local only: what is recorded, frecency, top sites and typed matching (pure,
//! tested with an injected clock). history_store.rs keeps it on disk and in memory.
//!
//! One record per URL (the fragment dropped, the query kept): its title, host (`www.` stripped,
//! lower case, with a non-default port), all-time visit count, and its last 20 visits (time +
//! kind). At most 5,000 URLs; the least frecent go first. Never recorded: anything but
//! http(s), the app's own pages, Tauri's custom-scheme hosts on Windows (`ipc`), and extension
//! panels in any form (`ljext:`, WebView2's `http://ljext.<id>.localhost`).
//! The history view's list and "Clear history" are in history/manage.rs.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use url::Url;

use crate::keys;

pub const HISTORY_FILE: &str = "history.json";
pub const HISTORY_VERSION: u32 = 1;
pub const MAX_URLS: usize = 5_000;
/// Visits kept per URL, and visits counted per host by frecency.
pub const MAX_VISITS: usize = 20;
/// Only visits this recent count for frecency.
pub const WINDOW_DAYS: f64 = 90.0;
/// A visit's weight halves every 14 days.
pub const HALF_LIFE_DAYS: f64 = 14.0;
/// Titles longer than this are cut (characters), so one page can't bloat the file.
pub const MAX_TITLE_CHARS: usize = 300;
/// Longer URLs (data stuffed into the query) aren't recorded.
pub const MAX_URL_BYTES: usize = 4096;
/// On Windows, Tauri serves custom schemes as `http://<scheme>.localhost` (extensions as
/// `http://ljext.<id>.localhost`: extensions::urls::is_ext_like).
const SCHEME_HOSTS: [&str; 2] = ["tauri.localhost", "ipc.localhost"];
const DAY_MS: f64 = 86_400_000.0;

/// How a visit started. `Typed`: the user typed or picked it (open_input, also for a picked
/// suggestion); `Reload`: the reload page action; `Link`: everything else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisitKind {
    Typed,
    Link,
    Reload,
}

impl VisitKind {
    /// Frecency weight: 2 for a visit the user chose, 1 for a link, 0 for a reload.
    pub fn weight(self) -> f64 {
        match self {
            VisitKind::Typed => 2.0,
            VisitKind::Link => 1.0,
            VisitKind::Reload => 0.0,
        }
    }
}

/// One visit: unix milliseconds and kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Visit {
    pub at: u64,
    pub kind: VisitKind,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub url: String,
    pub title: String,
    pub host: String,
    pub visit_count: u64,
    /// Oldest first, at most MAX_VISITS.
    pub visits: Vec<Visit>,
}

impl Entry {
    fn last_visit(&self) -> u64 {
        self.visits.last().map_or(0, |v| v.at)
    }
}

/// A suggestion row for the bar: what to open, what to show, how it ranked.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    pub url: String,
    pub title: String,
    pub host: String,
    pub score: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct History {
    entries: Vec<Entry>,
}

/// The address history keeps for `url` (fragment dropped), or `None` when it is never recorded.
pub fn recordable(url: &Url, app_origin: &Url) -> Option<Url> {
    let host = url.host_str()?.to_ascii_lowercase();
    if !matches!(url.scheme(), "http" | "https")
        || keys::is_app_url(app_origin, url)
        || SCHEME_HOSTS.contains(&host.as_str())
        || crate::extensions::urls::is_ext_like(url)
        || url.as_str().len() > MAX_URL_BYTES
    {
        return None;
    }
    let mut kept = url.clone();
    kept.set_fragment(None);
    Some(kept)
}

/// The site a URL belongs to: its host, lower case, `www.` stripped, with a non-default port.
pub fn host_key(url: &Url) -> String {
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let host = host.strip_prefix("www.").unwrap_or(&host).to_owned();
    match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    }
}

/// Σ w · 2^(−age / 14 days) over the most recent 20 of `visits` within 90 days of `now`.
pub fn frecency<'a>(visits: impl IntoIterator<Item = &'a Visit>, now: u64) -> f64 {
    let age_days = |visit: &Visit| now.saturating_sub(visit.at) as f64 / DAY_MS;
    let mut recent: Vec<&Visit> = visits
        .into_iter()
        .filter(|visit| age_days(visit) <= WINDOW_DAYS)
        .collect();
    recent.sort_by_key(|visit| std::cmp::Reverse(visit.at));
    recent
        .iter()
        .take(MAX_VISITS)
        .map(|visit| visit.kind.weight() * (-age_days(visit) / HALF_LIFE_DAYS).exp2())
        .sum()
}

fn clean_title(title: &str) -> String {
    title.trim().chars().take(MAX_TITLE_CHARS).collect()
}

/// Higher score first; on a tie, the later visit.
fn by_rank(a: (f64, u64), b: (f64, u64)) -> Ordering {
    b.0.total_cmp(&a.0).then(b.1.cmp(&a.1))
}

impl History {
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Records a visit to `url` at `now` (with the page's title if it is known already). Returns
    /// whether anything was recorded.
    pub fn record(
        &mut self,
        url: &Url,
        title: Option<&str>,
        kind: VisitKind,
        now: u64,
        app_origin: &Url,
    ) -> bool {
        let Some(url) = recordable(url, app_origin) else {
            return false;
        };
        let index = match self.entries.iter().position(|e| e.url == url.as_str()) {
            Some(index) => index,
            None => {
                self.entries.push(Entry {
                    url: url.as_str().to_owned(),
                    title: String::new(),
                    host: host_key(&url),
                    visit_count: 0,
                    visits: Vec::new(),
                });
                self.entries.len() - 1
            }
        };
        if let Some(entry) = self.entries.get_mut(index) {
            entry.visit_count = entry.visit_count.saturating_add(1);
            entry.visits.push(Visit { at: now, kind });
            let excess = entry.visits.len().saturating_sub(MAX_VISITS);
            entry.visits.drain(..excess);
            if let Some(title) = title.map(clean_title).filter(|t| !t.is_empty()) {
                entry.title = title;
            }
        }
        self.cap(now);
        true
    }

    /// The page at `url` changed its title: updates that URL's record. Returns whether one was
    /// updated (a page that isn't recorded, or an empty title, changes nothing).
    pub fn set_title(&mut self, url: &Url, title: &str, app_origin: &Url) -> bool {
        let title = clean_title(title);
        let Some(url) = recordable(url, app_origin) else {
            return false;
        };
        match self.entries.iter_mut().find(|e| e.url == url.as_str()) {
            Some(entry) if !title.is_empty() && entry.title != title => {
                entry.title = title;
                true
            }
            _ => false,
        }
    }

    /// Forgets the record of `url` (an address as the bar got it in a suggestion). Returns
    /// whether there was one.
    pub fn remove(&mut self, url: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.url != url);
        self.entries.len() != before
    }

    /// Keeps the MAX_URLS most frecent URLs (on a tie, the most recently visited).
    fn cap(&mut self, now: u64) {
        if self.entries.len() <= MAX_URLS {
            return;
        }
        let mut ranked: Vec<(f64, u64, Entry)> = std::mem::take(&mut self.entries)
            .into_iter()
            .map(|e| (frecency(&e.visits, now), e.last_visit(), e))
            .collect();
        ranked.sort_by(|a, b| by_rank((a.0, a.1), (b.0, b.1)));
        ranked.truncate(MAX_URLS);
        self.entries = ranked.into_iter().map(|(_, _, e)| e).collect();
    }

    /// Up to `n` sites by frecency (per host, over the host's last 20 visits), skipping the
    /// hosts in `exclude` and sites with no recent visit. Each opens the host's most-visited URL
    /// with at most one path segment and no query, else the site's origin.
    pub fn top_sites(&self, n: usize, exclude: &[&str], now: u64) -> Vec<Row> {
        let excluded: HashSet<String> = exclude
            .iter()
            .map(|h| {
                let h = h.trim().to_ascii_lowercase();
                h.strip_prefix("www.").unwrap_or(&h).to_owned()
            })
            .collect();
        let mut sites: HashMap<&str, Vec<&Entry>> = HashMap::new();
        for entry in &self.entries {
            if !excluded.contains(&entry.host) {
                sites.entry(entry.host.as_str()).or_default().push(entry);
            }
        }
        let mut rows: Vec<(f64, u64, Row)> = sites
            .into_iter()
            .filter_map(|(host, entries)| {
                let score = frecency(entries.iter().flat_map(|e| &e.visits), now);
                let last = entries.iter().map(|e| e.last_visit()).max().unwrap_or(0);
                (score > 0.0).then(|| (score, last, site_row(host, &entries, score)))
            })
            .collect();
        rows.sort_by(|a, b| by_rank((a.0, a.1), (b.0, b.1)));
        rows.into_iter().take(n).map(|(_, _, row)| row).collect()
    }

    /// Up to `limit` records matching every token of `query`, by points × the URL's frecency
    /// (on a tie, the later visit). Points per token, the best that applies: host prefix 10,
    /// host word boundary 8, title word prefix 6, path word boundary 4, substring 1.
    pub fn search(&self, query: &str, limit: usize, now: u64) -> Vec<Row> {
        let tokens: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        if tokens.is_empty() {
            return Vec::new();
        }
        let mut rows: Vec<(f64, u64, &Entry)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                let points = match_points(entry, &tokens)?;
                Some((
                    points * frecency(&entry.visits, now),
                    entry.last_visit(),
                    entry,
                ))
            })
            .collect();
        rows.sort_by(|a, b| by_rank((a.0, a.1), (b.0, b.1)));
        rows.into_iter()
            .take(limit)
            .map(|(score, _, entry)| Row {
                url: entry.url.clone(),
                title: entry.title.clone(),
                host: entry.host.clone(),
                score,
            })
            .collect()
    }

    /// history.json: `{ "version": 1, "entries": [ { url, title, host, visitCount, visits:
    /// [ { at, kind } ] } ] }`.
    pub fn to_json(&self) -> String {
        json!({ "version": HISTORY_VERSION, "entries": self.entries }).to_string()
    }

    /// Reads history.json. A bad file is empty history; a bad entry or visit is skipped alone.
    /// Hosts are recomputed, URLs that are never recorded are dropped, duplicates keep the first.
    pub fn parse(text: &str, now: u64, app_origin: &Url) -> History {
        let Ok(Value::Object(file)) = serde_json::from_str::<Value>(text) else {
            return History::default();
        };
        let Some(Value::Array(list)) = file.get("entries") else {
            return History::default();
        };
        let mut seen = HashSet::new();
        let entries = list
            .iter()
            .filter_map(|value| parse_entry(value, app_origin))
            .filter(|entry| seen.insert(entry.url.clone()))
            .collect();
        let mut history = History { entries };
        history.cap(now);
        history
    }
}

/// The row for one site: its most-visited short URL (≤ 1 path segment, no query), else origin.
fn site_row(host: &str, entries: &[&Entry], score: f64) -> Row {
    let short = entries
        .iter()
        .filter_map(|e| Url::parse(&e.url).ok().map(|url| (e, url)))
        .filter(|(_, url)| {
            url.query().is_none()
                && url
                    .path_segments()
                    .is_some_and(|s| s.filter(|p| !p.is_empty()).count() <= 1)
        })
        .max_by(|(a, _), (b, _)| {
            a.visit_count
                .cmp(&b.visit_count)
                .then(a.last_visit().cmp(&b.last_visit()))
        });
    let (url, title) = match short {
        Some((entry, _)) => (entry.url.clone(), entry.title.clone()),
        None => {
            let origin = entries
                .first()
                .and_then(|e| Url::parse(&e.url).ok())
                .map(|url| format!("{}/", url.origin().ascii_serialization()))
                .unwrap_or_default();
            (origin, String::new())
        }
    };
    let title = if title.is_empty() {
        host.to_owned()
    } else {
        title
    };
    Row {
        url,
        title,
        host: host.to_owned(),
        score,
    }
}

fn match_points(entry: &Entry, tokens: &[String]) -> Option<f64> {
    let url = Url::parse(&entry.url).ok()?;
    let host = entry.host.as_str();
    let title = entry.title.to_lowercase();
    let path = format!(
        "{}{}",
        url.path(),
        url.query().map(|q| format!("?{q}")).unwrap_or_default()
    )
    .to_lowercase();
    let whole = entry.url.to_lowercase();
    let mut points = 0u32;
    for token in tokens {
        points += if host.starts_with(token.as_str()) {
            10
        } else if at_word_start(host, token) {
            8
        } else if title
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word.starts_with(token.as_str()))
        {
            6
        } else if at_word_start(&path, token) {
            4
        } else if whole.contains(token.as_str()) || title.contains(token.as_str()) {
            1
        } else {
            return None;
        };
    }
    Some(f64::from(points))
}

/// Whether `token` occurs in `text` at the start of a word (the start, or after a character
/// that isn't a letter or digit).
fn at_word_start(text: &str, token: &str) -> bool {
    text.match_indices(token).any(|(at, _)| {
        text[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric())
    })
}

fn parse_entry(value: &Value, app_origin: &Url) -> Option<Entry> {
    let url = Url::parse(value.get("url")?.as_str()?).ok()?;
    let url = recordable(&url, app_origin)?;
    let mut visits: Vec<Visit> = value
        .get("visits")?
        .as_array()?
        .iter()
        .filter_map(|v| {
            Some(Visit {
                at: v.get("at")?.as_u64()?,
                kind: serde_json::from_value(v.get("kind")?.clone()).ok()?,
            })
        })
        .collect();
    if visits.is_empty() {
        return None;
    }
    visits.sort_by_key(|v| v.at);
    let excess = visits.len().saturating_sub(MAX_VISITS);
    visits.drain(..excess);
    let count = value.get("visitCount").and_then(Value::as_u64).unwrap_or(0);
    Some(Entry {
        url: url.as_str().to_owned(),
        title: clean_title(
            value
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ),
        host: host_key(&url),
        visit_count: count.max(visits.len() as u64),
        visits,
    })
}

mod manage;

pub use manage::{ClearRange, Listed};

#[cfg(test)]
mod tests;
