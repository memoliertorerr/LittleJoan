//! What the command bar suggests (pure, tested; ui/command/rows.ts orders and shows it).
//!
//! Nothing typed: the pinned sites first (LJ50, settings' `pinnedSites`, in the order pinned), then
//! the top sites by frecency (history.rs) — leaving out the pinned sites' hosts and the page that is
//! open —, at most 8 rows in all; four generic starters when there is neither. Typing: the history
//! records that match every word, and one row that searches for the text with the chosen engine.

use serde::Serialize;
use url::Url;

use crate::history::{self, History};
use crate::input::SearchEngine;
use crate::settings::PinnedSite;

/// The most top sites, and the most history matches, one answer holds.
pub const MAX_ROWS: usize = 8;

/// Shown while there are no top sites yet: (title, address, site).
pub const STARTERS: [(&str, &str, &str); 4] = [
    ("Wikipedia", "https://www.wikipedia.org/", "wikipedia.org"),
    ("YouTube", "https://www.youtube.com/", "youtube.com"),
    ("GitHub", "https://github.com/", "github.com"),
    ("Google Maps", "https://www.google.com/maps", "google.com"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    /// A site the user pinned (LJ50).
    Pinned,
    TopSite,
    History,
    /// Search for the typed text: `title` is the engine's name, `url` its results page.
    Search,
    Starter,
}

/// One suggestion: what to open, what to show, whose favicon, how it ranked (0 when unranked).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Suggestion {
    pub kind: Kind,
    pub title: String,
    pub url: String,
    pub host: String,
    pub score: f64,
}

/// The suggestions for `query`. `current` is the page that is open (its site isn't suggested as a
/// top site while nothing is typed); `pinned` the user's pinned sites (settings' `pinnedSites`).
pub fn suggestions(
    history: &History,
    query: &str,
    current: Option<&Url>,
    engine: SearchEngine,
    now: u64,
    pinned: &[PinnedSite],
) -> Vec<Suggestion> {
    let query = query.trim();
    if query.is_empty() {
        let pins = pinned_rows(pinned);
        let mut exclude: Vec<String> = current
            .filter(|url| matches!(url.scheme(), "http" | "https"))
            .map(history::host_key)
            .into_iter()
            .collect();
        exclude.extend(pins.iter().map(|row| row.host.clone()));
        let exclude: Vec<&str> = exclude.iter().map(String::as_str).collect();
        let top: Vec<Suggestion> = history
            .top_sites(MAX_ROWS.saturating_sub(pins.len()), &exclude, now)
            .into_iter()
            .map(|row| Suggestion {
                kind: Kind::TopSite,
                title: row.title,
                url: row.url,
                host: row.host,
                score: row.score,
            })
            .collect();
        if pins.is_empty() && top.is_empty() {
            return starters();
        }
        return pins.into_iter().chain(top).collect();
    }
    let mut rows: Vec<Suggestion> = history
        .search(query, MAX_ROWS, now)
        .into_iter()
        .map(|row| Suggestion {
            kind: Kind::History,
            title: row.title,
            url: row.url,
            host: row.host,
            score: row.score,
        })
        .collect();
    let search = engine.search_url(query);
    rows.push(Suggestion {
        kind: Kind::Search,
        title: engine.name().to_owned(),
        host: history::host_key(&search),
        url: search.into(),
        score: 0.0,
    });
    rows
}

/// The pinned sites as rows, in the order pinned (at most `MAX_ROWS`; one that isn't an address
/// any more is left out).
fn pinned_rows(pinned: &[PinnedSite]) -> Vec<Suggestion> {
    pinned
        .iter()
        .filter_map(|site| {
            let url = Url::parse(&site.url).ok()?;
            Some(Suggestion {
                kind: Kind::Pinned,
                title: site.title.clone(),
                host: history::host_key(&url),
                url: site.url.clone(),
                score: 0.0,
            })
        })
        .take(MAX_ROWS)
        .collect()
}

/// The four generic starters.
pub fn starters() -> Vec<Suggestion> {
    STARTERS
        .iter()
        .map(|(title, url, host)| Suggestion {
            kind: Kind::Starter,
            title: (*title).to_owned(),
            url: (*url).to_owned(),
            host: (*host).to_owned(),
            score: 0.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::VisitKind;
    use crate::keys;

    const NOW: u64 = 1_800_000_000_000;
    const HOUR: u64 = 3_600_000;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    fn visited(pages: &[(&str, &str, u32)]) -> History {
        let mut history = History::default();
        for (address, title, visits) in pages {
            for i in 0..*visits {
                history.record(
                    &url(address),
                    Some(title),
                    VisitKind::Typed,
                    NOW - u64::from(i + 1) * HOUR,
                    &keys::app_origin(),
                );
            }
        }
        history
    }

    #[test]
    fn empty_history_offers_the_four_starters() {
        let got = suggestions(
            &History::default(),
            "  ",
            None,
            SearchEngine::Google,
            NOW,
            &[],
        );
        assert_eq!(got.len(), 4);
        assert!(got.iter().all(|s| s.kind == Kind::Starter));
        let titles: Vec<&str> = got.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, ["Wikipedia", "YouTube", "GitHub", "Google Maps"]);
        assert!(got.iter().all(|s| s.url.starts_with("https://")));
    }

    #[test]
    fn nothing_typed_offers_top_sites_without_the_open_page() {
        let history = visited(&[
            ("http://127.0.0.1:5000/smoke.html", "LJ-SMOKE ready", 3),
            ("https://example.com/", "Example", 1),
        ]);
        let got = suggestions(&history, "", None, SearchEngine::Google, NOW, &[]);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].kind, Kind::TopSite);
        assert_eq!(got[0].host, "127.0.0.1:5000");
        assert_eq!(got[0].url, "http://127.0.0.1:5000/smoke.html");
        assert_eq!(got[1].host, "example.com");

        let open = url("http://127.0.0.1:5000/other");
        let got = suggestions(&history, "", Some(&open), SearchEngine::Google, NOW, &[]);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].host, "example.com");
    }

    #[test]
    fn the_starters_fill_in_when_the_only_site_is_the_open_page() {
        let history = visited(&[("https://example.com/", "Example", 1)]);
        let open = url("https://www.example.com/a");
        let got = suggestions(&history, "", Some(&open), SearchEngine::Google, NOW, &[]);
        assert!(got.iter().all(|s| s.kind == Kind::Starter));
        // The app's own start page is never "the open page" of any site.
        let start = url("tauri://localhost/index.html");
        let got = suggestions(&history, "", Some(&start), SearchEngine::Google, NOW, &[]);
        assert_eq!(got[0].host, "example.com");
    }

    #[test]
    fn typing_offers_matches_then_one_search_row() {
        let history = visited(&[
            ("http://127.0.0.1:5000/smoke.html", "LJ-SMOKE ready", 3),
            ("https://example.com/", "Example", 1),
        ]);
        let got = suggestions(&history, "127", None, SearchEngine::DuckDuckGo, NOW, &[]);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].kind, Kind::History);
        assert_eq!(got[0].host, "127.0.0.1:5000");
        assert!(got[0].score > 0.0);
        let search = &got[1];
        assert_eq!(search.kind, Kind::Search);
        assert_eq!(search.title, "DuckDuckGo");
        assert_eq!(search.url, "https://duckduckgo.com/?q=127");
        assert_eq!(search.host, "duckduckgo.com");
    }

    #[test]
    fn a_query_with_no_match_still_searches() {
        let got = suggestions(
            &History::default(),
            " rust lang ",
            None,
            SearchEngine::Google,
            NOW,
            &[],
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].kind, Kind::Search);
        assert_eq!(got[0].url, "https://www.google.com/search?q=rust+lang");
        assert_eq!(got[0].host, "google.com");
    }

    #[test]
    fn at_most_eight_matches() {
        let pages: Vec<(String, String)> = (0..12)
            .map(|i| (format!("https://site{i}.test/"), format!("Site {i}")))
            .collect();
        let refs: Vec<(&str, &str, u32)> = pages
            .iter()
            .map(|(u, t)| (u.as_str(), t.as_str(), 1))
            .collect();
        let history = visited(&refs);
        let got = suggestions(&history, "site", None, SearchEngine::Google, NOW, &[]);
        assert_eq!(got.iter().filter(|s| s.kind == Kind::History).count(), 8);
        assert_eq!(got.last().map(|s| s.kind), Some(Kind::Search));
        assert_eq!(
            suggestions(&history, "", None, SearchEngine::Google, NOW, &[]).len(),
            8
        );
    }

    fn pins(urls: &[&str]) -> Vec<PinnedSite> {
        urls.iter()
            .map(|u| crate::settings::pinned::site(u, "Pinned").expect("a site"))
            .collect()
    }

    #[test]
    fn pinned_sites_lead_the_empty_state_then_top_sites_without_their_hosts() {
        let history = visited(&[
            ("http://127.0.0.1:5000/smoke.html", "LJ-SMOKE ready", 5),
            ("https://example.com/", "Example", 3),
            ("https://news.test/", "News", 1),
        ]);
        let pinned = pins(&[
            "https://news.test/today",
            "http://127.0.0.1:5000/pinned.html",
        ]);
        let got = suggestions(&history, "", None, SearchEngine::Google, NOW, &pinned);
        let kinds: Vec<Kind> = got.iter().map(|s| s.kind).collect();
        assert_eq!(kinds, [Kind::Pinned, Kind::Pinned, Kind::TopSite]);
        // In the order pinned, with their own addresses.
        assert_eq!(got[0].url, "https://news.test/today");
        assert_eq!(got[0].host, "news.test");
        assert_eq!(got[1].host, "127.0.0.1:5000");
        assert_eq!(got[0].title, "Pinned");
        // The top sites leave the pinned hosts out.
        assert_eq!(got[2].host, "example.com");
        // The open page's site stays pinned (only top sites leave it out).
        let open = url("https://news.test/other");
        let got = suggestions(
            &history,
            "",
            Some(&open),
            SearchEngine::Google,
            NOW,
            &pinned,
        );
        assert_eq!(got[0].host, "news.test");
    }

    #[test]
    fn pinned_sites_alone_need_no_starters_and_the_rows_stay_eight() {
        let pinned = pins(&["http://127.0.0.1:5000/smoke.html"]);
        let got = suggestions(
            &History::default(),
            "",
            None,
            SearchEngine::Google,
            NOW,
            &pinned,
        );
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].kind, Kind::Pinned);
        assert_eq!(got[0].host, "127.0.0.1:5000");
        let pages: Vec<(String, String)> = (0..12)
            .map(|i| (format!("https://site{i}.test/"), format!("Site {i}")))
            .collect();
        let refs: Vec<(&str, &str, u32)> = pages
            .iter()
            .map(|(u, t)| (u.as_str(), t.as_str(), 1))
            .collect();
        let history = visited(&refs);
        let three = pins(&["https://a.test/", "https://b.test/", "https://c.test/"]);
        let got = suggestions(&history, "", None, SearchEngine::Google, NOW, &three);
        assert_eq!(got.len(), MAX_ROWS);
        assert_eq!(got.iter().filter(|s| s.kind == Kind::Pinned).count(), 3);
        let eight: Vec<String> = (0..8).map(|i| format!("https://p{i}.test/")).collect();
        let eight: Vec<&str> = eight.iter().map(String::as_str).collect();
        let got = suggestions(&history, "", None, SearchEngine::Google, NOW, &pins(&eight));
        assert!(got.iter().all(|s| s.kind == Kind::Pinned));
        assert_eq!(got.len(), MAX_ROWS);
    }

    #[test]
    fn typing_never_offers_pins() {
        let pinned = pins(&["https://news.test/"]);
        let got = suggestions(
            &History::default(),
            "news",
            None,
            SearchEngine::Google,
            NOW,
            &pinned,
        );
        assert!(got.iter().all(|s| s.kind != Kind::Pinned));
        assert_eq!(serde_json::to_value(Kind::Pinned).expect("json"), "pinned");
    }

    #[test]
    fn rows_serialize_for_the_bar() {
        let row = &starters()[0];
        let json = serde_json::to_value(row).expect("json");
        assert_eq!(json["kind"], "starter");
        assert_eq!(json["host"], "wikipedia.org");
        let search = serde_json::to_value(Kind::TopSite).expect("json");
        assert_eq!(search, "topSite");
    }
}
