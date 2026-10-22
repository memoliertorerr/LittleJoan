//! How long a summon takes (LJ19.1), for the smoke log only: from the request — the shortcut, the
//! tray's Open, a forwarded `--show`, a launch's own flags — to the command bar on screen, with a
//! mark for each step on the way. Recorded only while smoke logging is on (`smoke::enabled`): in
//! normal use nothing is measured, kept or logged, and the bar is never asked for its part.
//!
//! One line per summon that built the window, once its last mark is in (or the window closes
//! first, `"partial":true`):
//! `summon-timing {"source":"cli","url":false,"at":<unix ms>,"partial":false,"marks":{...}}`,
//! each mark in milliseconds after the request, on the monotonic clock (`Instant`). The marks, in
//! the order they normally come:
//!
//! | Mark | When |
//! |---|---|
//! | `dispatched` | the window code runs (off the main thread, after the lifecycle lock) |
//! | `windowBuilt` | the native window exists (hidden) |
//! | `historyLoaded` | history.json read and parsed |
//! | `webviewsAdded` | the bar and the page webviews exist |
//! | `cookiesRestored` | the kept session cookies are set again (Windows; nothing elsewhere) |
//! | `agentSet` | the page's user agent for its width (Windows: a DevTools call) |
//! | `shown` | the window is on screen and focused |
//! | `created` | the window code is done (with a URL: its navigation started) |
//! | `barLoaded` | the bar's page finished loading (the engine's load event) |
//! | `barScript` | the bar's script runs (its early theme report) |
//! | `barReady` | the bar asked `start_info` |
//! | `barPainted` | the bar's first frame was painted (the bar says so) |
//! | `commandBar` | the command bar's first rows were painted (the bar says so) |
//! | `pageStarted` | with a URL: the web page started loading |
//!
//! A summon without a URL is complete at `commandBar`; one with a URL at `barPainted` and
//! `pageStarted` (the command bar closes for the page, so it draws no rows).

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::smoke;

/// Where a request came from (the smoke log's `source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The global shortcut (global-hotkey, or the XDG portal's `Activated`).
    Shortcut,
    /// The tray / menu-bar menu's Open.
    Tray,
    /// A second launch's arguments, forwarded by single-instance.
    Cli,
    /// The launch's own flags, or the first launch's welcome.
    Launch,
    /// Little Joan itself (closing the window, a page action).
    App,
}

impl Source {
    pub fn name(self) -> &'static str {
        match self {
            Source::Shortcut => "shortcut",
            Source::Tray => "tray",
            Source::Cli => "cli",
            Source::Launch => "launch",
            Source::App => "app",
        }
    }
}

/// One step of a summon (the table above).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Dispatched,
    WindowBuilt,
    HistoryLoaded,
    WebviewsAdded,
    CookiesRestored,
    AgentSet,
    Shown,
    Created,
    BarLoaded,
    BarScript,
    BarReady,
    BarPainted,
    CommandBar,
    PageStarted,
}

impl Mark {
    pub fn name(self) -> &'static str {
        match self {
            Mark::Dispatched => "dispatched",
            Mark::WindowBuilt => "windowBuilt",
            Mark::HistoryLoaded => "historyLoaded",
            Mark::WebviewsAdded => "webviewsAdded",
            Mark::CookiesRestored => "cookiesRestored",
            Mark::AgentSet => "agentSet",
            Mark::Shown => "shown",
            Mark::Created => "created",
            Mark::BarLoaded => "barLoaded",
            Mark::BarScript => "barScript",
            Mark::BarReady => "barReady",
            Mark::BarPainted => "barPainted",
            Mark::CommandBar => "commandBar",
            Mark::PageStarted => "pageStarted",
        }
    }
}

/// When a request arrived: taken where it is received (joan::dispatch), before anything else runs.
#[derive(Debug, Clone, Copy)]
pub struct Request {
    pub source: Source,
    pub received: Instant,
    /// The same moment on the wall clock, so CI can line it up with its own (unix ms).
    pub at_unix_ms: u128,
}

impl Request {
    pub fn now(source: Source) -> Request {
        Request {
            source,
            received: Instant::now(),
            at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis()),
        }
    }
}

/// One summon's marks (pure: the times are handed in).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summon {
    source: Source,
    url: bool,
    at_unix_ms: u128,
    marks: Vec<(Mark, u64)>,
}

impl Summon {
    pub fn new(source: Source, url: bool, at_unix_ms: u128) -> Summon {
        Summon {
            source,
            url,
            at_unix_ms,
            marks: Vec::new(),
        }
    }

    /// Records `mark` at `ms` after the request; the first time counts. Returns whether it did.
    pub fn mark(&mut self, mark: Mark, ms: u64) -> bool {
        if self.has(mark) {
            return false;
        }
        self.marks.push((mark, ms));
        true
    }

    pub fn has(&self, mark: Mark) -> bool {
        self.marks.iter().any(|(m, _)| *m == mark)
    }

    /// Whether the last mark it waits for is in (the module's doc).
    pub fn complete(&self) -> bool {
        if self.url {
            self.has(Mark::BarPainted) && self.has(Mark::PageStarted)
        } else {
            self.has(Mark::CommandBar)
        }
    }

    /// The smoke log's JSON: the marks in the order they were recorded.
    pub fn to_json(&self, partial: bool) -> String {
        let marks: Vec<String> = self
            .marks
            .iter()
            .map(|(mark, ms)| format!("\"{}\":{ms}", mark.name()))
            .collect();
        format!(
            "{{\"source\":\"{}\",\"url\":{},\"at\":{},\"partial\":{partial},\"marks\":{{{}}}}}",
            self.source.name(),
            self.url,
            self.at_unix_ms,
            marks.join(",")
        )
    }
}

/// Milliseconds from `from` to `to` (0 if `to` came first).
pub fn millis_between(from: Instant, to: Instant) -> u64 {
    u64::try_from(to.saturating_duration_since(from).as_millis()).unwrap_or(u64::MAX)
}

/// The moment `ago_ms` before `now`, never before `floor` (the bar's reports: "that long ago").
/// `None` for a value that isn't a sane duration.
pub fn moment_ago(now: Instant, ago_ms: f64, floor: Instant) -> Option<Instant> {
    if !ago_ms.is_finite() || !(0.0..=MAX_AGO_MS).contains(&ago_ms) {
        return None;
    }
    let ago = Duration::from_secs_f64(ago_ms / 1000.0);
    Some(now.checked_sub(ago).map_or(floor, |at| at.max(floor)))
}

/// The longest "ago" the bar may report (a minute).
const MAX_AGO_MS: f64 = 60_000.0;

/// The summon being measured: its request time and its marks.
static CURRENT: Mutex<Option<(Instant, Summon)>> = Mutex::new(None);

fn current() -> std::sync::MutexGuard<'static, Option<(Instant, Summon)>> {
    CURRENT.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The window is being built for `request` (with a URL to open, or not): measuring starts, at
/// `dispatched`. A summon still open from before is logged as it is first. Smoke mode only.
pub fn begin(request: Request, url: bool) {
    if !smoke::enabled() {
        return;
    }
    let now = Instant::now();
    let mut slot = current();
    if let Some((_, old)) = slot.take() {
        smoke::log("summon-timing", &old.to_json(true));
    }
    let mut summon = Summon::new(request.source, url, request.at_unix_ms);
    summon.mark(Mark::Dispatched, millis_between(request.received, now));
    *slot = Some((request.received, summon));
}

/// `mark` happened now.
pub fn mark(mark: Mark) {
    mark_at(mark, Instant::now());
}

/// The bar's report: `mark` happened `ago_ms` milliseconds before now.
pub fn mark_ago(mark: Mark, ago_ms: f64) {
    let now = Instant::now();
    let received = current().as_ref().map(|(received, _)| *received);
    if let Some(at) = received.and_then(|received| moment_ago(now, ago_ms, received)) {
        mark_at(mark, at);
    }
}

fn mark_at(mark: Mark, at: Instant) {
    let mut slot = current();
    let Some((received, summon)) = slot.as_mut() else {
        return;
    };
    if summon.mark(mark, millis_between(*received, at)) && summon.complete() {
        let done = summon.to_json(false);
        *slot = None;
        drop(slot);
        smoke::log("summon-timing", &done);
    }
}

/// The window closed: a summon still waiting for its last mark is logged as it is.
pub fn end() {
    let open = current().take();
    if let Some((_, summon)) = open {
        smoke::log("summon-timing", &summon.to_json(true));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_summon_without_a_url_is_complete_at_the_command_bar() {
        let mut summon = Summon::new(Source::Shortcut, false, 1_000);
        assert!(summon.mark(Mark::Dispatched, 1));
        assert!(summon.mark(Mark::Shown, 40));
        assert!(summon.mark(Mark::BarPainted, 120));
        assert!(!summon.complete());
        assert!(summon.mark(Mark::CommandBar, 150));
        assert!(summon.complete());
    }

    #[test]
    fn a_summon_with_a_url_waits_for_the_bar_and_the_page() {
        let mut summon = Summon::new(Source::Cli, true, 1_000);
        summon.mark(Mark::PageStarted, 90);
        assert!(!summon.complete());
        summon.mark(Mark::BarPainted, 130);
        assert!(summon.complete());
        // The command bar's rows aren't needed (it closes for the page).
        let mut painted_only = Summon::new(Source::Cli, true, 1_000);
        painted_only.mark(Mark::CommandBar, 100);
        assert!(!painted_only.complete());
    }

    #[test]
    fn the_first_time_of_a_mark_counts() {
        let mut summon = Summon::new(Source::Tray, false, 5);
        assert!(summon.mark(Mark::BarLoaded, 70));
        assert!(!summon.mark(Mark::BarLoaded, 900));
        assert_eq!(
            summon.to_json(false),
            r#"{"source":"tray","url":false,"at":5,"partial":false,"marks":{"barLoaded":70}}"#
        );
    }

    #[test]
    fn the_json_keeps_the_order_of_the_marks() {
        let mut summon = Summon::new(Source::Launch, true, 1_760_000_000_123);
        summon.mark(Mark::Dispatched, 0);
        summon.mark(Mark::WindowBuilt, 12);
        summon.mark(Mark::PageStarted, 200);
        let json = summon.to_json(true);
        assert_eq!(
            json,
            r#"{"source":"launch","url":true,"at":1760000000123,"partial":true,"marks":{"dispatched":0,"windowBuilt":12,"pageStarted":200}}"#
        );
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["marks"]["windowBuilt"], 12);
    }

    #[test]
    fn every_mark_and_source_has_a_json_name() {
        let marks = [
            Mark::Dispatched,
            Mark::WindowBuilt,
            Mark::HistoryLoaded,
            Mark::WebviewsAdded,
            Mark::CookiesRestored,
            Mark::AgentSet,
            Mark::Shown,
            Mark::Created,
            Mark::BarLoaded,
            Mark::BarScript,
            Mark::BarReady,
            Mark::BarPainted,
            Mark::CommandBar,
            Mark::PageStarted,
        ];
        let mut names: Vec<&str> = marks.iter().map(|m| m.name()).collect();
        assert!(
            names
                .iter()
                .all(|n| n.chars().all(|c| c.is_ascii_alphabetic()))
        );
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), marks.len());
        for source in [
            Source::Shortcut,
            Source::Tray,
            Source::Cli,
            Source::Launch,
            Source::App,
        ] {
            assert!(source.name().chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn the_bars_reports_land_between_the_request_and_now() {
        let request = Instant::now();
        let now = request + Duration::from_millis(500);
        let at = moment_ago(now, 200.0, request).expect("a sane value");
        assert_eq!(millis_between(request, at), 300);
        // Longer ago than the request: the request itself.
        let early = moment_ago(now, 2_000.0, request).expect("a sane value");
        assert_eq!(millis_between(request, early), 0);
        for bad in [f64::NAN, f64::INFINITY, -1.0, 61_000.0] {
            assert!(moment_ago(now, bad, request).is_none(), "{bad}");
        }
    }
}
