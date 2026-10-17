//! What the command bar suggests (pure, tested; ui/command/rows.ts orders and shows it).
//!
//! Nothing typed: the top sites by frecency (history.rs), leaving out the page that is open, or
//! four generic starters when there is no site to show yet. Typing: the history records that
//! match every word, and one row that searches for the text with the chosen engine.

use serde::Serialize;
use url::Url;

use crate::history::{self, History};
use crate::input::SearchEngine;

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

/// The suggestions for `query`. `current` is the page that is open (its site isn't suggested
/// while nothing is typed).
pub fn suggestions(
    history: &History,
    query: &str,
    current: Option<&Url>,
    engine: SearchEngine,
    now: u64,
) -> Vec<Suggestion> {
    let query = query.trim();
    if query.is_empty() {
        let exclude: Vec<String> = current
            .filter(|url| matches!(url.scheme(), "http" | "https"))
            .map(history::host_key)
            .into_iter()
            .collect();
        let exclude: Vec<&str> = exclude.iter().map(String::as_str).collect();
        let top: Vec<Suggestion> = history
            .top_sites(MAX_ROWS, &exclude, now)
            .into_iter()
            .map(|row| Suggestion {
                kind: Kind::TopSite,
                title: row.title,
                url: row.url,
                host: row.host,
                score: row.score,
            })
            .collect();
        return if top.is_empty() { starters() } else { top };
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
        let got = suggestions(&History::default(), "  ", None, SearchEngine::Google, NOW);
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
        let got = suggestions(&history, "", None, SearchEngine::Google, NOW);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].kind, Kind::TopSite);
        assert_eq!(got[0].host, "127.0.0.1:5000");
        assert_eq!(got[0].url, "http://127.0.0.1:5000/smoke.html");
        assert_eq!(got[1].host, "example.com");

        let open = url("http://127.0.0.1:5000/other");
        let got = suggestions(&history, "", Some(&open), SearchEngine::Google, NOW);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].host, "example.com");
    }

    #[test]
    fn the_starters_fill_in_when_the_only_site_is_the_open_page() {
        let history = visited(&[("https://example.com/", "Example", 1)]);
        let open = url("https://www.example.com/a");
        let got = suggestions(&history, "", Some(&open), SearchEngine::Google, NOW);
        assert!(got.iter().all(|s| s.kind == Kind::Starter));
        // The app's own start page is never "the open page" of any site.
        let start = url("tauri://localhost/index.html");
        let got = suggestions(&history, "", Some(&start), SearchEngine::Google, NOW);
        assert_eq!(got[0].host, "example.com");
    }

    #[test]
    fn typing_offers_matches_then_one_search_row() {
        let history = visited(&[
            ("http://127.0.0.1:5000/smoke.html", "LJ-SMOKE ready", 3),
            ("https://example.com/", "Example", 1),
        ]);
        let got = suggestions(&history, "127", None, SearchEngine::DuckDuckGo, NOW);
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
        let got = suggestions(&history, "site", None, SearchEngine::Google, NOW);
        assert_eq!(got.iter().filter(|s| s.kind == Kind::History).count(), 8);
        assert_eq!(got.last().map(|s| s.kind), Some(Kind::Search));
        assert_eq!(
            suggestions(&history, "", None, SearchEngine::Google, NOW).len(),
            8
        );
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
