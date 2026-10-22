//! How long a summon takes (LJ19.1), for the smoke log only: from the request — the shortcut, the
//! tray's Open, a forwarded `--show`, a launch's own flags — to the command bar on screen, with a
//! mark for each step on the way. Recorded only while smoke logging is on (`smoke::enabled`): in
//! normal use nothing is measured, kept or logged, and the bar is never asked for its part.
//!
//! One line per summon that built the window, once its last mark is in (or the window closes
//! first, `"partial":true`):
//! `summon-timing {"source":"cli","url":false,"at":<unix ms>,"nap":…,"partial":false,"marks":{...}}`,
//! each mark in milliseconds after the request, on the monotonic clock (`Instant`); `nap` is
//! macOS's App Nap state at the request (`platform::nap_state`; `null` elsewhere). The marks, in
//! the order they normally come (since LJ19.1's changes):
//!
//! | Mark | When |
//! |---|---|
//! | `dispatched` | the window code runs (off the main thread, after the lifecycle lock) |
//! | `windowBuilt` | the native window exists (hidden) |
//! | `webviewsAdded` | the bar and the page webviews exist |
//! | `shown` | the window is on screen and focused |
//! | `cookiesRestored` | the kept session cookies are set again (Windows; nothing elsewhere) |
//! | `agentSet` | the page's user agent for its width (Windows: a DevTools call) |
//! | `created` | the window code is done (with a URL: its navigation started) |
//! | `historyLoaded` | history.json read and parsed (on its own thread, any time after `dispatched`) |
//! | `barNavigation` | the bar's navigation started (the engine's timing; the bar says so) |
//! | `barHtml` | bar.html arrived (the bar says so) |
//! | `barLoaded` | the bar's page finished loading (the engine's load event) |
//! | `barModule` | ui/bar.ts runs (the bar says so) |
//! | `barScript` | its early theme report reached Rust |
//! | `barThemeAnswered` | … and its answer reached the bar (the bar says so) |
//! | `barReady` | the bar asked `start_info` |
//! | `barStartInfo` | … and its answer reached the bar (the bar says so) |
//! | `barPainted` | the bar's first frame was painted (the bar says so) |
//! | `barSuggestions` | the first suggestions were drawn into the command bar (the bar says so) |
//! | `commandBar` | the command bar's first rows were painted (the bar says so) |
//! | `pageStarted` | with a URL: the web page started loading |
//!
//! A summon is logged once every mark it waits for is in (`EXPECTED`, plus `commandBar` without a
//! URL or `pageStarted` with one — the command bar closes for the page, so it draws no rows),
//! whatever order they came in: they come from the window code, the history thread, the engine's
//! events and the bar, and on Windows the bar loads, runs and can even paint its rows before
//! `add_child` returns. One that misses a mark is logged as it is (`"partial":true`) when the
//! window closes, or `GIVE_UP` after the request. The bar reports its marks once, in one call, at
//! the end (commands/timing.rs); its diagnostic ones (`barNavigation` …) aren't waited for.

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
    BarNavigation,
    BarHtml,
    BarLoaded,
    BarModule,
    BarScript,
    BarThemeAnswered,
    BarReady,
    BarStartInfo,
    BarPainted,
    BarSuggestions,
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
            Mark::BarNavigation => "barNavigation",
            Mark::BarHtml => "barHtml",
            Mark::BarLoaded => "barLoaded",
            Mark::BarModule => "barModule",
            Mark::BarScript => "barScript",
            Mark::BarThemeAnswered => "barThemeAnswered",
            Mark::BarReady => "barReady",
            Mark::BarStartInfo => "barStartInfo",
            Mark::BarPainted => "barPainted",
            Mark::BarSuggestions => "barSuggestions",
            Mark::CommandBar => "commandBar",
            Mark::PageStarted => "pageStarted",
        }
    }
}

/// When a request arrived: taken where it is received (joan::dispatch), before anything else runs.
#[derive(Debug, Clone)]
pub struct Request {
    pub source: Source,
    pub received: Instant,
    /// The same moment on the wall clock, so CI can line it up with its own (unix ms).
    pub at_unix_ms: u128,
    /// Smoke mode, macOS: App Nap's state at that moment (`platform::nap_state`).
    pub nap: Option<String>,
}

impl Request {
    pub fn now(source: Source) -> Request {
        Request {
            source,
            received: Instant::now(),
            at_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis()),
            nap: smoke::enabled().then(crate::platform::nap_state).flatten(),
        }
    }
}

/// The marks every summon waits for before it is logged (with `commandBar` or `pageStarted`).
pub const EXPECTED: [Mark; 12] = [
    Mark::Dispatched,
    Mark::WindowBuilt,
    Mark::WebviewsAdded,
    Mark::Shown,
    Mark::CookiesRestored,
    Mark::AgentSet,
    Mark::Created,
    Mark::HistoryLoaded,
    Mark::BarLoaded,
    Mark::BarScript,
    Mark::BarReady,
    Mark::BarPainted,
];

/// How long after its request a summon still missing a mark is logged as it is (`partial`).
const GIVE_UP: Duration = Duration::from_secs(20);

/// One summon's marks (pure: the times are handed in).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summon {
    source: Source,
    url: bool,
    at_unix_ms: u128,
    /// App Nap's state at the request (macOS), as `platform::nap_state` words it.
    nap: Option<String>,
    marks: Vec<(Mark, u64)>,
}

impl Summon {
    pub fn new(source: Source, url: bool, at_unix_ms: u128) -> Summon {
        Summon {
            source,
            url,
            at_unix_ms,
            nap: None,
            marks: Vec::new(),
        }
    }

    /// With App Nap's state at the request.
    pub fn with_nap(mut self, nap: Option<String>) -> Summon {
        self.nap = nap;
        self
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

    /// Whether every mark it waits for is in, in whatever order they came (the module's doc).
    pub fn complete(&self) -> bool {
        let last = if self.url {
            Mark::PageStarted
        } else {
            Mark::CommandBar
        };
        EXPECTED.iter().all(|&mark| self.has(mark)) && self.has(last)
    }

    /// The smoke log's JSON: the marks in the order they were recorded.
    pub fn to_json(&self, partial: bool) -> String {
        let marks: Vec<String> = self
            .marks
            .iter()
            .map(|(mark, ms)| format!("\"{}\":{ms}", mark.name()))
            .collect();
        let nap = self.nap.as_ref().map_or_else(
            || "null".to_owned(),
            |nap| serde_json::to_string(nap).unwrap_or_else(|_| "null".to_owned()),
        );
        format!(
            "{{\"source\":\"{}\",\"url\":{},\"at\":{},\"nap\":{nap},\"partial\":{partial},\"marks\":{{{}}}}}",
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
    let mut summon = Summon::new(request.source, url, request.at_unix_ms).with_nap(request.nap);
    summon.mark(Mark::Dispatched, millis_between(request.received, now));
    let received = request.received;
    *slot = Some((received, summon));
    drop(slot);
    // A mark that never comes: the summon is logged as it got, GIVE_UP after its request (unless
    // it was logged, or a newer one replaced it, meanwhile).
    let _ = std::thread::Builder::new()
        .name("little-joan-summon-timing".into())
        .spawn(move || {
            std::thread::sleep(GIVE_UP.saturating_sub(received.elapsed()));
            let mut slot = current();
            if slot.as_ref().is_some_and(|(at, _)| *at == received)
                && let Some((_, summon)) = slot.take()
            {
                drop(slot);
                smoke::log("summon-timing", &summon.to_json(true));
            }
        });
}

/// `mark` happened now.
pub fn mark(mark: Mark) {
    mark_at(mark, Instant::now());
}

/// The bar's report: each mark happened its `ago` milliseconds before now.
pub fn marks_ago(marks: &[(Mark, f64)]) {
    let now = Instant::now();
    let Some(received) = current().as_ref().map(|(received, _)| *received) else {
        return;
    };
    let at: Vec<(Mark, Instant)> = marks
        .iter()
        .filter_map(|&(mark, ago)| Some((mark, moment_ago(now, ago, received)?)))
        .collect();
    record(&at);
}

fn mark_at(mark: Mark, at: Instant) {
    record(&[(mark, at)]);
}

/// Records each mark (the first time counts); logs the summon once it is complete.
fn record(marks: &[(Mark, Instant)]) {
    let mut slot = current();
    let Some((received, summon)) = slot.as_mut() else {
        return;
    };
    let mut added = false;
    for &(mark, at) in marks {
        added |= summon.mark(mark, millis_between(*received, at));
    }
    if added && summon.complete() {
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

    /// Marks `marks` at increasing times; returns whether the summon was complete after each.
    fn mark_all(summon: &mut Summon, marks: &[Mark]) -> Vec<bool> {
        marks
            .iter()
            .enumerate()
            .map(|(i, &mark)| {
                summon.mark(mark, u64::try_from(i).unwrap_or(0) * 10);
                summon.complete()
            })
            .collect()
    }

    #[test]
    fn a_summon_without_a_url_waits_for_every_mark_and_the_command_bar() {
        let mut summon = Summon::new(Source::Shortcut, false, 1_000);
        let done = mark_all(&mut summon, &EXPECTED);
        assert!(done.iter().all(|d| !d), "not before its rows");
        assert!(summon.mark(Mark::CommandBar, 400));
        assert!(summon.complete());
    }

    #[test]
    fn marks_in_windows_order_complete_only_with_the_last_one() {
        // CI's Windows run (LJ19.1): WebView2 loads and runs the bar — and it paints its rows —
        // before add_child returns, so the window code's marks come last.
        let order = [
            Mark::Dispatched,
            Mark::WindowBuilt,
            Mark::HistoryLoaded,
            Mark::BarLoaded,
            Mark::BarScript,
            Mark::BarReady,
            Mark::BarPainted,
            Mark::CommandBar,
            Mark::WebviewsAdded,
            Mark::Shown,
            Mark::CookiesRestored,
            Mark::AgentSet,
            Mark::Created,
        ];
        let mut summon = Summon::new(Source::Cli, false, 1_000);
        let done = mark_all(&mut summon, &order);
        assert_eq!(done.iter().filter(|d| **d).count(), 1);
        assert_eq!(done.last(), Some(&true));
        // Any mark missing keeps it open, whichever it is.
        for missing in order {
            let mut summon = Summon::new(Source::Cli, false, 1_000);
            let rest: Vec<Mark> = order.iter().copied().filter(|m| *m != missing).collect();
            mark_all(&mut summon, &rest);
            assert!(!summon.complete(), "{missing:?}");
        }
    }

    #[test]
    fn a_summon_with_a_url_waits_for_the_page_not_the_rows() {
        let mut summon = Summon::new(Source::Cli, true, 1_000);
        mark_all(&mut summon, &EXPECTED);
        assert!(!summon.complete());
        summon.mark(Mark::CommandBar, 300);
        assert!(!summon.complete(), "the command bar closes for the page");
        summon.mark(Mark::PageStarted, 310);
        assert!(summon.complete());
    }

    #[test]
    fn the_bars_diagnostic_marks_are_not_waited_for() {
        let mut summon = Summon::new(Source::Cli, false, 1_000);
        mark_all(&mut summon, &EXPECTED);
        summon.mark(Mark::CommandBar, 300);
        assert!(summon.complete());
        for diagnostic in [
            Mark::BarNavigation,
            Mark::BarHtml,
            Mark::BarModule,
            Mark::BarThemeAnswered,
            Mark::BarStartInfo,
            Mark::BarSuggestions,
        ] {
            assert!(!EXPECTED.contains(&diagnostic));
            assert!(!summon.has(diagnostic));
        }
    }

    #[test]
    fn the_first_time_of_a_mark_counts() {
        let mut summon = Summon::new(Source::Tray, false, 5);
        assert!(summon.mark(Mark::BarLoaded, 70));
        assert!(!summon.mark(Mark::BarLoaded, 900));
        assert_eq!(
            summon.to_json(false),
            r#"{"source":"tray","url":false,"at":5,"nap":null,"partial":false,"marks":{"barLoaded":70}}"#
        );
    }

    #[test]
    fn the_json_keeps_the_order_of_the_marks() {
        let mut summon = Summon::new(Source::Launch, true, 1_760_000_000_123)
            .with_nap(Some("active=1 \"x\"".to_owned()));
        summon.mark(Mark::Dispatched, 0);
        summon.mark(Mark::WindowBuilt, 12);
        summon.mark(Mark::PageStarted, 200);
        let json = summon.to_json(true);
        assert_eq!(
            json,
            r#"{"source":"launch","url":true,"at":1760000000123,"nap":"active=1 \"x\"","partial":true,"marks":{"dispatched":0,"windowBuilt":12,"pageStarted":200}}"#
        );
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert_eq!(parsed["marks"]["windowBuilt"], 12);
        assert_eq!(parsed["nap"], "active=1 \"x\"");
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
            Mark::BarNavigation,
            Mark::BarHtml,
            Mark::BarLoaded,
            Mark::BarModule,
            Mark::BarScript,
            Mark::BarThemeAnswered,
            Mark::BarReady,
            Mark::BarStartInfo,
            Mark::BarPainted,
            Mark::BarSuggestions,
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
