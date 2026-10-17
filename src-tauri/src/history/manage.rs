//! What the history view does with history (pure, tested): list every page by its latest visit,
//! and clear the visits of a time range. history_store.rs writes the result to disk at once.

use serde::{Deserialize, Serialize};

use super::History;

const HOUR_MS: u64 = 3_600_000;
const DAY_MS: u64 = 24 * HOUR_MS;

/// How much "Clear history" removes: the visits of the last hour, the last 24 hours, the last 7
/// days, or everything. Rolling windows from now (Rust has no local calendar; the view's "Today"
/// heading is the bar's local day).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClearRange {
    Hour,
    Day,
    Week,
    All,
}

impl ClearRange {
    /// The earliest visit time (unix ms) the range removes; `None` = everything.
    pub fn since(self, now: u64) -> Option<u64> {
        match self {
            ClearRange::Hour => Some(now.saturating_sub(HOUR_MS)),
            ClearRange::Day => Some(now.saturating_sub(DAY_MS)),
            ClearRange::Week => Some(now.saturating_sub(7 * DAY_MS)),
            ClearRange::All => None,
        }
    }
}

/// One page in the history view: the record's address, title, site, latest visit and count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listed {
    pub url: String,
    pub title: String,
    pub host: String,
    /// Unix milliseconds.
    pub last_visit: u64,
    pub visit_count: u64,
}

impl History {
    /// Every record, latest visit first (on a tie, by address), for the history view.
    pub fn list(&self) -> Vec<Listed> {
        let mut pages: Vec<Listed> = self
            .entries
            .iter()
            .map(|entry| Listed {
                url: entry.url.clone(),
                title: entry.title.clone(),
                host: entry.host.clone(),
                last_visit: entry.last_visit(),
                visit_count: entry.visit_count,
            })
            .collect();
        pages.sort_by(|a, b| b.last_visit.cmp(&a.last_visit).then(a.url.cmp(&b.url)));
        pages
    }

    /// Forgets the visits of `range` (at or after its start, so a visit stamped in the future
    /// goes too). A record left without visits goes; one that keeps older visits stays, its
    /// count lowered by the visits removed. Returns how many records went.
    pub fn clear(&mut self, range: ClearRange, now: u64) -> usize {
        let before = self.entries.len();
        let Some(since) = range.since(now) else {
            self.entries.clear();
            return before;
        };
        for entry in &mut self.entries {
            let kept = entry.visits.len();
            entry.visits.retain(|visit| visit.at < since);
            let removed = (kept - entry.visits.len()) as u64;
            entry.visit_count = entry
                .visit_count
                .saturating_sub(removed)
                .max(entry.visits.len() as u64);
        }
        self.entries.retain(|entry| !entry.visits.is_empty());
        before - self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::{Visit, VisitKind};
    use url::Url;

    /// A fixed clock: day 1000 of the unix epoch.
    const NOW: u64 = 1_000 * DAY_MS;

    fn history(records: &[(&str, &[u64])]) -> History {
        let origin = Url::parse("tauri://localhost/").expect("valid");
        let mut history = History::default();
        for (address, visits) in records {
            let url = Url::parse(address).expect("valid");
            for at in *visits {
                assert!(history.record(&url, Some("T"), VisitKind::Link, *at, &origin));
            }
        }
        history
    }

    fn urls(history: &History) -> Vec<&str> {
        history.entries().iter().map(|e| e.url.as_str()).collect()
    }

    #[test]
    fn ranges_reach_back_an_hour_a_day_a_week_or_all_the_way() {
        assert_eq!(ClearRange::Hour.since(NOW), Some(NOW - HOUR_MS));
        assert_eq!(ClearRange::Day.since(NOW), Some(NOW - DAY_MS));
        assert_eq!(ClearRange::Week.since(NOW), Some(NOW - 7 * DAY_MS));
        assert_eq!(ClearRange::All.since(NOW), None);
        assert_eq!(ClearRange::Week.since(5), Some(0), "no underflow");
        for (name, range) in [
            ("\"hour\"", ClearRange::Hour),
            ("\"day\"", ClearRange::Day),
            ("\"week\"", ClearRange::Week),
            ("\"all\"", ClearRange::All),
        ] {
            assert_eq!(serde_json::from_str::<ClearRange>(name).ok(), Some(range));
        }
        assert!(serde_json::from_str::<ClearRange>("\"month\"").is_err());
    }

    #[test]
    fn the_list_is_every_page_latest_visit_first() {
        let h = history(&[
            ("https://a.test/", &[NOW - 3 * DAY_MS, NOW - DAY_MS]),
            ("https://b.test/", &[NOW - 2 * DAY_MS]),
            ("https://c.test/", &[NOW - DAY_MS]),
        ]);
        let list = h.list();
        let order: Vec<&str> = list.iter().map(|p| p.url.as_str()).collect();
        assert_eq!(
            order,
            ["https://a.test/", "https://c.test/", "https://b.test/"]
        );
        assert_eq!(
            list[0],
            Listed {
                url: "https://a.test/".into(),
                title: "T".into(),
                host: "a.test".into(),
                last_visit: NOW - DAY_MS,
                visit_count: 2,
            }
        );
        let json = serde_json::to_string(&list[0]).expect("json");
        assert!(json.contains(r#""lastVisit":86313600000"#), "{json}");
        assert!(json.contains(r#""visitCount":2"#), "{json}");
    }

    #[test]
    fn clearing_the_last_hour_keeps_older_visits() {
        let mut h = history(&[
            ("https://recent.test/", &[NOW - 10 * 60_000]),
            ("https://both.test/", &[NOW - 2 * HOUR_MS, NOW - 60_000]),
            ("https://old.test/", &[NOW - 2 * HOUR_MS]),
            ("https://future.test/", &[NOW + 60_000]),
        ]);
        assert_eq!(h.clear(ClearRange::Hour, NOW), 2);
        assert_eq!(urls(&h), ["https://both.test/", "https://old.test/"]);
        let both = &h.entries()[0];
        assert_eq!(
            both.visits,
            [Visit {
                at: NOW - 2 * HOUR_MS,
                kind: VisitKind::Link
            }]
        );
        assert_eq!(both.visit_count, 1);
    }

    #[test]
    fn clearing_a_day_or_a_week_reaches_further() {
        let records: &[(&str, &[u64])] = &[
            ("https://today.test/", &[NOW - 5 * HOUR_MS]),
            ("https://days.test/", &[NOW - 3 * DAY_MS]),
            ("https://weeks.test/", &[NOW - 20 * DAY_MS]),
        ];
        let mut day = history(records);
        assert_eq!(day.clear(ClearRange::Day, NOW), 1);
        assert_eq!(urls(&day), ["https://days.test/", "https://weeks.test/"]);
        let mut week = history(records);
        assert_eq!(week.clear(ClearRange::Week, NOW), 2);
        assert_eq!(urls(&week), ["https://weeks.test/"]);
    }

    #[test]
    fn clearing_everything_leaves_nothing() {
        let mut h = history(&[("https://a.test/", &[1]), ("https://b.test/", &[NOW])]);
        assert_eq!(h.clear(ClearRange::All, NOW), 2);
        assert!(h.entries().is_empty());
        assert_eq!(h.clear(ClearRange::All, NOW), 0);
    }

    #[test]
    fn a_count_beyond_the_kept_visits_drops_by_the_visits_removed() {
        let mut h = History::default();
        let origin = Url::parse("tauri://localhost/").expect("valid");
        let url = Url::parse("https://busy.test/").expect("valid");
        // 25 visits: the record keeps the last 20 and counts 25.
        for i in 0..25u64 {
            h.record(
                &url,
                None,
                VisitKind::Link,
                NOW - (25 - i) * HOUR_MS,
                &origin,
            );
        }
        assert_eq!(h.entries()[0].visit_count, 25);
        // An hour back from an hour ago: the visits 2 h and 1 h ago.
        assert_eq!(h.clear(ClearRange::Hour, NOW - HOUR_MS), 0);
        let entry = &h.entries()[0];
        assert_eq!(entry.visits.len(), 18);
        assert_eq!(entry.visit_count, 23);
    }
}
