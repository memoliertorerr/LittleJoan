//! How long a summon takes (LJ19.1, LJ19.4), for the smoke log only: from the request — the
//! shortcut, the tray's Open, a forwarded `--show`, a launch's own flags — to the command bar on
//! screen with its field taking keys, with a mark for each step on the way. Recorded only while
//! smoke logging is on (`smoke::enabled`): in normal use nothing is measured, kept or logged, and
//! the bar is never asked for its part.
//!
//! One line per summon, once its last mark is in (or the window closes first, `"partial":true`):
//! `summon-timing {"source":"cli","kind":"ready","mode":"ready","url":false,"at":<unix ms>,
//! "nap":…,"partial":false,"marks":{...}}`, each mark in milliseconds after the request, on the
//! monotonic clock (`Instant`). `kind` is `cold` (the window and the bar were built for it) or
//! `ready` (the hidden window and its bar were there already, LJ19.4); `mode` is the "After
//! closing" setting; `nap` is macOS's App Nap state at the request (`platform::nap_state`; `null`
//! elsewhere). The marks, in the order they normally come:
//!
//! | Mark | When | Kind |
//! |---|---|---|
//! | `dispatched` | the window code runs (off the main thread, after the lifecycle lock) | both |
//! | `windowBuilt` | the native window exists (hidden) | cold |
//! | `webviewsAdded` | the bar's webview exists (since LJ19.4 the page comes after the bar) | cold |
//! | `shown` | the window is on screen and focused | both |
//! | `created` | the window code is done (with a URL: its navigation started) | both |
//! | `historyLoaded` | history.json read and parsed (on its own thread, any time after `dispatched`) | both |
//! | `pageBuilt` | the page webview exists (after the bar is on screen, or at once for a URL) | both |
//! | `cookiesRestored` | the kept session cookies are set again (Windows; nothing elsewhere) | both |
//! | `agentSet` | the page's user agent for its width (Windows: a DevTools call) | both |
//! | `barNavigation` | the bar's navigation started (the engine's timing; the bar says so) | cold |
//! | `barHtml` | bar.html arrived (the bar says so) | cold |
//! | `barLoaded` | the bar's page finished loading (the engine's load event) | cold |
//! | `barModule` | ui/bar.ts runs (the bar says so) | cold |
//! | `barScript` | its early theme report reached Rust | cold |
//! | `barThemeAnswered` | … and its answer reached the bar (the bar says so) | cold |
//! | `barReady` | the bar asked `start_info` | cold |
//! | `barStartInfo` | … and its answer reached the bar (the bar says so) | cold |
//! | `barPainted` | the bar's first frame (a ready summon: its first frame after the summon) | both |
//! | `barFocused` | the bar's field has the keyboard: its document has focus (the bar says so) | both |
//! | `barSuggestions` | the first suggestions were drawn into the command bar (the bar says so) | both |
//! | `commandBar` | the command bar's first rows were painted (the bar says so) | both |
//! | `pageStarted` | with a URL: the web page started loading | both |
//!
//! A summon is logged once every mark its kind waits for is in (`EXPECTED_COLD` or
//! `EXPECTED_READY`, plus `commandBar` and `barFocused` without a URL, or `pageStarted` with one —
//! the command bar closes for the page, so it draws no rows and the keyboard goes to the page),
//! whatever order they came in: they come from the
//! window code, the history thread, the page's builder, the engine's events and the bar, and on
//! Windows the bar loads, runs and can even paint its rows before `add_child` returns. One that
//! misses a mark is logged as it is (`"partial":true`) when the window closes, or `GIVE_UP` after
//! the request. The bar reports its marks once per summon, in one call (commands/timing.rs); its
//! diagnostic ones (`barNavigation` …) aren't waited for.

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
    PageBuilt,
    BarNavigation,
    BarHtml,
    BarLoaded,
    BarModule,
    BarScript,
    BarThemeAnswered,
    BarReady,
    BarStartInfo,
    BarPainted,
    BarFocused,
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
            Mark::PageBuilt => "pageBuilt",
            Mark::BarNavigation => "barNavigation",
            Mark::BarHtml => "barHtml",
            Mark::BarLoaded => "barLoaded",
            Mark::BarModule => "barModule",
            Mark::BarScript => "barScript",
            Mark::BarThemeAnswered => "barThemeAnswered",
            Mark::BarReady => "barReady",
            Mark::BarStartInfo => "barStartInfo",
            Mark::BarPainted => "barPainted",
            Mark::BarFocused => "barFocused",
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

/// What a summon started from (the smoke log's `kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The window and its bar were built for this summon.
    Cold,
    /// The hidden window and its bar were ready (LJ19.4): shown, the page built after the bar.
    Ready,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Cold => "cold",
            Kind::Ready => "ready",
        }
    }

    /// The marks a summon of this kind waits for before it is logged (with `commandBar` or
    /// `pageStarted`).
    pub fn expected(self) -> &'static [Mark] {
        match self {
            Kind::Cold => &EXPECTED_COLD,
            Kind::Ready => &EXPECTED_READY,
        }
    }
}

/// A cold summon's marks (with `commandBar` or `pageStarted`).
pub const EXPECTED_COLD: [Mark; 13] = [
    Mark::Dispatched,
    Mark::WindowBuilt,
    Mark::WebviewsAdded,
    Mark::Shown,
    Mark::Created,
    Mark::HistoryLoaded,
    Mark::PageBuilt,
    Mark::CookiesRestored,
    Mark::AgentSet,
    Mark::BarLoaded,
    Mark::BarScript,
    Mark::BarReady,
    Mark::BarPainted,
];

/// A summon from ready's marks (with `commandBar` or `pageStarted`): no window, bar or early
/// theme to wait for — they were there.
pub const EXPECTED_READY: [Mark; 8] = [
    Mark::Dispatched,
    Mark::Shown,
    Mark::Created,
    Mark::HistoryLoaded,
    Mark::PageBuilt,
    Mark::CookiesRestored,
    Mark::AgentSet,
    Mark::BarPainted,
];

/// How long after its request a summon still missing a mark is logged as it is (`partial`).
const GIVE_UP: Duration = Duration::from_secs(20);

/// One summon's marks (pure: the times are handed in).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summon {
    source: Source,
    kind: Kind,
    /// The "After closing" setting's name at the request.
    mode: &'static str,
    url: bool,
    at_unix_ms: u128,
    /// App Nap's state at the request (macOS), as `platform::nap_state` words it.
    nap: Option<String>,
    marks: Vec<(Mark, u64)>,
}

impl Summon {
    pub fn new(source: Source, kind: Kind, url: bool, at_unix_ms: u128) -> Summon {
        Summon {
            source,
            kind,
            mode: "ready",
            url,
            at_unix_ms,
            nap: None,
            marks: Vec::new(),
        }
    }

    /// With the "After closing" setting's name (`ready` until told).
    pub fn with_mode(mut self, mode: &'static str) -> Summon {
        self.mode = mode;
        self
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
        let last: &[Mark] = if self.url {
            &[Mark::PageStarted]
        } else {
            &[Mark::CommandBar, Mark::BarFocused]
        };
        self.kind
            .expected()
            .iter()
            .chain(last)
            .all(|&mark| self.has(mark))
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
            "{{\"source\":\"{}\",\"kind\":\"{}\",\"mode\":\"{}\",\"url\":{},\"at\":{},\"nap\":{nap},\"partial\":{partial},\"marks\":{{{}}}}}",
            self.source.name(),
            self.kind.name(),
            self.mode,
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

/// A summon of `kind` (with a URL to open, or not) runs for `request`, with "After closing" on
/// `mode`: measuring starts, at `dispatched`. A summon still open from before is logged as it is
/// first. Smoke mode only.
pub fn begin(request: Request, kind: Kind, mode: &'static str, url: bool) {
    if !smoke::enabled() {
        return;
    }
    let now = Instant::now();
    let mut slot = current();
    if let Some((_, old)) = slot.take() {
        smoke::log("summon-timing", &old.to_json(true));
    }
    let mut summon = Summon::new(request.source, kind, url, request.at_unix_ms)
        .with_mode(mode)
        .with_nap(request.nap);
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
        for kind in [Kind::Cold, Kind::Ready] {
            let mut summon = Summon::new(Source::Shortcut, kind, false, 1_000);
            let done = mark_all(&mut summon, kind.expected());
            assert!(done.iter().all(|d| !d), "{kind:?}: not before its rows");
            assert!(summon.mark(Mark::CommandBar, 400));
            assert!(
                !summon.complete(),
                "{kind:?}: not before the field has the keyboard"
            );
            assert!(summon.mark(Mark::BarFocused, 410));
            assert!(summon.complete(), "{kind:?}");
        }
    }

    #[test]
    fn a_summon_from_ready_needs_none_of_the_cold_marks() {
        let mut summon = Summon::new(Source::Shortcut, Kind::Ready, false, 1_000);
        mark_all(&mut summon, &EXPECTED_READY);
        summon.mark(Mark::BarFocused, 60);
        summon.mark(Mark::CommandBar, 90);
        assert!(summon.complete());
        for cold_only in [
            Mark::WindowBuilt,
            Mark::WebviewsAdded,
            Mark::BarLoaded,
            Mark::BarScript,
            Mark::BarReady,
        ] {
            assert!(EXPECTED_COLD.contains(&cold_only), "{cold_only:?}");
            assert!(!EXPECTED_READY.contains(&cold_only), "{cold_only:?}");
        }
        // Every ready mark is a cold one too: a ready summon is a cold one minus the building.
        assert!(
            EXPECTED_READY
                .iter()
                .all(|mark| EXPECTED_COLD.contains(mark))
        );
        // Both wait for the page webview (and, without a URL, the field taking keys: above).
        for kind in [Kind::Cold, Kind::Ready] {
            assert!(kind.expected().contains(&Mark::PageBuilt));
        }
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
            Mark::BarFocused,
            Mark::CommandBar,
            Mark::WebviewsAdded,
            Mark::Shown,
            Mark::Created,
            Mark::PageBuilt,
            Mark::CookiesRestored,
            Mark::AgentSet,
        ];
        let mut summon = Summon::new(Source::Cli, Kind::Cold, false, 1_000);
        let done = mark_all(&mut summon, &order);
        assert_eq!(done.iter().filter(|d| **d).count(), 1);
        assert_eq!(done.last(), Some(&true));
        // Any mark missing keeps it open, whichever it is.
        for missing in order {
            let mut summon = Summon::new(Source::Cli, Kind::Cold, false, 1_000);
            let rest: Vec<Mark> = order.iter().copied().filter(|m| *m != missing).collect();
            mark_all(&mut summon, &rest);
            assert!(!summon.complete(), "{missing:?}");
        }
    }

    #[test]
    fn a_summon_with_a_url_waits_for_the_page_not_the_rows() {
        let mut summon = Summon::new(Source::Cli, Kind::Cold, true, 1_000);
        mark_all(&mut summon, &EXPECTED_COLD);
        assert!(!summon.complete());
        summon.mark(Mark::CommandBar, 300);
        assert!(!summon.complete(), "the command bar closes for the page");
        // The keyboard goes to the page: the bar's field taking it isn't waited for.
        summon.mark(Mark::PageStarted, 310);
        assert!(summon.complete());
    }

    #[test]
    fn the_bars_diagnostic_marks_are_not_waited_for() {
        let mut summon = Summon::new(Source::Cli, Kind::Cold, false, 1_000);
        mark_all(&mut summon, &EXPECTED_COLD);
        summon.mark(Mark::CommandBar, 300);
        summon.mark(Mark::BarFocused, 300);
        assert!(summon.complete());
        for diagnostic in [
            Mark::BarNavigation,
            Mark::BarHtml,
            Mark::BarModule,
            Mark::BarThemeAnswered,
            Mark::BarStartInfo,
            Mark::BarSuggestions,
        ] {
            assert!(!EXPECTED_COLD.contains(&diagnostic));
            assert!(!EXPECTED_READY.contains(&diagnostic));
            assert!(!summon.has(diagnostic));
        }
    }

    #[test]
    fn the_first_time_of_a_mark_counts() {
        let mut summon = Summon::new(Source::Tray, Kind::Ready, false, 5);
        assert!(summon.mark(Mark::BarLoaded, 70));
        assert!(!summon.mark(Mark::BarLoaded, 900));
        assert_eq!(
            summon.to_json(false),
            r#"{"source":"tray","kind":"ready","mode":"ready","url":false,"at":5,"nap":null,"partial":false,"marks":{"barLoaded":70}}"#
        );
    }

    #[test]
    fn the_json_keeps_the_order_of_the_marks() {
        let mut summon = Summon::new(Source::Launch, Kind::Cold, true, 1_760_000_000_123)
            .with_mode("hibernate")
            .with_nap(Some("active=1 \"x\"".to_owned()));
        summon.mark(Mark::Dispatched, 0);
        summon.mark(Mark::WindowBuilt, 12);
        summon.mark(Mark::PageStarted, 200);
        let json = summon.to_json(true);
        assert_eq!(
            json,
            r#"{"source":"launch","kind":"cold","mode":"hibernate","url":true,"at":1760000000123,"nap":"active=1 \"x\"","partial":true,"marks":{"dispatched":0,"windowBuilt":12,"pageStarted":200}}"#
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
            Mark::PageBuilt,
            Mark::BarNavigation,
            Mark::BarHtml,
            Mark::BarLoaded,
            Mark::BarModule,
            Mark::BarScript,
            Mark::BarThemeAnswered,
            Mark::BarReady,
            Mark::BarStartInfo,
            Mark::BarPainted,
            Mark::BarFocused,
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
        for kind in [Kind::Cold, Kind::Ready] {
            assert!(kind.name().chars().all(|c| c.is_ascii_lowercase()));
        }
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
