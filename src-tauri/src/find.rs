//! Find in page (LJ49), the pure part: the find state of the page window — its query, what the
//! engine reported, and what the toolbar's find strip shows. The engines do the finding natively
//! (platform/find.rs: WebView2's `ICoreWebView2Find`, WKWebView's
//! `findString:withConfiguration:completionHandler:`, WebKitGTK's `WebKitFindController`); the page
//! gets no script and no IPC (Ground Rules 10 and 18). The bar drives it through one command,
//! `find` (commands/find.rs), and Rust pushes the result back (bar/find.rs).
//!
//! What each engine can say differs: WebView2 counts the matches and knows the active one;
//! WebKitGTK counts them (`counted-matches`) but never says which is active, so it is tracked here
//! — 1 after a search, one on or back with next / previous, wrapping at the ends —; WKWebView says
//! only whether a match was found. Every find is case-insensitive and wraps around the page's end.

use serde::{Deserialize, Serialize};

/// The longest query, in characters: longer text is cut there.
pub const MAX_QUERY: usize = 1_000;
/// WebKitGTK counts and highlights at most this many matches (more: `counted-matches` says
/// `u32::MAX`).
pub const MAX_MATCHES: u32 = 1_000;
/// Every find ignores case.
pub const CASE_SENSITIVE: bool = false;
/// Every find goes on from the page's start after its end (and back from its end).
pub const WRAPS: bool = true;

/// What the bar asks for (the `find` command): exactly one of these, nothing else.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum FindAction {
    // Empty structs, not unit variants: serde's internally tagged unit variants ignore extra
    // fields despite deny_unknown_fields (as commands/quick.rs found; a test sends one).
    /// Find `query` from the start (an empty query clears the find; the strip stays open).
    Start { query: String },
    /// The next match (Enter, ↓).
    Next {},
    /// The previous match (Shift+Enter, ↑).
    Previous {},
    /// The strip closes (Esc, ×): the find ends and the page gets the keyboard.
    Stop {},
}

impl FindAction {
    /// For the smoke log: the kind, and a query's length — never its text.
    pub fn log_detail(&self) -> String {
        match self {
            FindAction::Start { query } => format!("start length={}", query.chars().count()),
            FindAction::Next {} => "next".to_owned(),
            FindAction::Previous {} => "previous".to_owned(),
            FindAction::Stop {} => "stop".to_owned(),
        }
    }
}

/// The page's engine, and what it can report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// WebView2: the match count and the active match (1-based), both through events.
    WebView2,
    /// WebKitGTK: the match count and found / not found; the active match is tracked here.
    WebKitGtk,
    /// WKWebView: found or not, nothing more.
    WebKit,
}

impl Engine {
    /// This OS's engine.
    pub const fn current() -> Engine {
        if cfg!(windows) {
            Engine::WebView2
        } else if cfg!(target_os = "macos") {
            Engine::WebKit
        } else {
            Engine::WebKitGtk
        }
    }
}

/// What the engine is asked to do (platform/find.rs). Next and previous carry the query, which
/// WKWebView needs for every call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineOp {
    Start(String),
    Next(String),
    Previous(String),
    Stop,
}

/// One thing an engine reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Report {
    /// How many matches (WebView2's `MatchCount`, WebKitGTK's `counted-matches`; `u32::MAX`: more
    /// than WebKitGTK was asked to count).
    Count(u32),
    /// The active match, 1-based (WebView2's `ActiveMatchIndex`); 0: none.
    Active(u32),
    /// A match was found, or not (WKWebView's `matchFound`; WebKitGTK's `found-text` /
    /// `failed-to-find-text`).
    Found(bool),
    /// The engine has no find (a WebView2 runtime older than `ICoreWebView2_28`).
    Unavailable,
}

/// What the find strip shows, pushed to the bar as `window.littleJoan.setFindResult(<json>)`:
/// `{ matches, active, found }` (`matches` / `active` null where the engine doesn't say), and
/// `unavailable: true` when the engine has no find.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct FindResult {
    pub matches: Option<u32>,
    pub active: Option<u32>,
    pub found: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unavailable: bool,
}

impl FindResult {
    /// The smoke log's `find-result` detail: `matches=3 active=1` where the engine counts,
    /// `found=true` where it doesn't, `unavailable`.
    pub fn log_line(&self) -> String {
        if self.unavailable {
            return "unavailable".to_owned();
        }
        match (self.matches, self.active) {
            (Some(matches), Some(active)) => format!("matches={matches} active={active}"),
            (Some(matches), None) => format!("matches={matches}"),
            (None, _) => format!("found={}", self.found),
        }
    }
}

/// How a find ended (`FindState::end`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ended {
    /// The strip was open: the bar closes it.
    pub was_open: bool,
    /// A query was being found: the engine stops.
    pub stop_engine: bool,
}

/// The page window's find: whether the strip is open, its query, and what the engine said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindState {
    engine: Engine,
    open: bool,
    query: Option<String>,
    /// Bumped with every new query: reports that say which find they are for (WKWebView's) are
    /// dropped when they are for an older one.
    generation: u64,
    matches: Option<u32>,
    active: Option<u32>,
    found: Option<bool>,
    unavailable: bool,
}

/// `text` cut to `MAX_QUERY` characters.
pub fn cut_query(text: &str) -> String {
    text.chars().take(MAX_QUERY).collect()
}

impl FindState {
    pub const fn new(engine: Engine) -> FindState {
        FindState {
            engine,
            open: false,
            query: None,
            generation: 0,
            matches: None,
            active: None,
            found: None,
            unavailable: false,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// The current find's number (reports carry it where the engine allows).
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether the engine has said anything about the current query yet.
    pub fn known(&self) -> bool {
        self.query.is_some()
            && (self.unavailable
                || self.matches.is_some()
                || self.active.is_some()
                || self.found.is_some())
    }

    /// Ctrl/Cmd+F: the strip opens (or stays open, its query kept).
    pub fn open(&mut self) {
        self.open = true;
    }

    fn clear_results(&mut self) {
        self.matches = None;
        self.active = None;
        self.found = None;
        self.unavailable = false;
    }

    /// What `action` asks of the engine (`None`: nothing). Only while the strip is open — a
    /// query typed just before the strip closed (a page load) finds nothing.
    pub fn act(&mut self, action: FindAction) -> Option<EngineOp> {
        match action {
            FindAction::Start { query } => {
                if !self.open {
                    return None;
                }
                let query = cut_query(&query);
                self.generation += 1;
                self.clear_results();
                if query.is_empty() {
                    // Clears the find; the strip stays.
                    return self.query.take().map(|_| EngineOp::Stop);
                }
                self.query = Some(query.clone());
                Some(EngineOp::Start(query))
            }
            FindAction::Next {} | FindAction::Previous {} => {
                let query = self.query.clone().filter(|_| self.open)?;
                let forward = action == FindAction::Next {};
                if self.engine == Engine::WebKitGtk {
                    self.active = self
                        .active
                        .map(|active| step(active, self.matches, forward));
                }
                Some(if forward {
                    EngineOp::Next(query)
                } else {
                    EngineOp::Previous(query)
                })
            }
            FindAction::Stop {} => {
                let ended = self.end();
                ended.stop_engine.then_some(EngineOp::Stop)
            }
        }
    }

    /// The find ends: the strip closes (Esc, a new page, the command bar, the window closing).
    pub fn end(&mut self) -> Ended {
        let ended = Ended {
            was_open: self.open,
            stop_engine: self.query.is_some(),
        };
        self.open = false;
        self.query = None;
        self.generation += 1;
        self.clear_results();
        ended
    }

    /// An engine's report for find `generation` (`None`: the engine can't say which — the
    /// current one). Dropped when it is for an older find, or no find runs. Returns the new
    /// result when it changed.
    pub fn report(&mut self, generation: Option<u64>, report: Report) -> Option<FindResult> {
        if self.query.is_none() || generation.is_some_and(|g| g != self.generation) {
            return None;
        }
        let before = (self.known(), self.result());
        match report {
            Report::Unavailable => self.unavailable = true,
            Report::Count(u32::MAX) if self.engine == Engine::WebKitGtk => {
                // More than it was asked to count: found, but no number.
                self.matches = None;
                self.found = Some(true);
            }
            Report::Count(count) => {
                self.matches = Some(count);
                if count == 0 {
                    self.active = None;
                    self.found = Some(false);
                } else if self.engine == Engine::WebKitGtk && self.active.is_none() {
                    // A search selects the first match after it.
                    self.active = Some(1);
                }
            }
            Report::Active(active) => {
                self.active = (active > 0).then_some(active);
            }
            Report::Found(found) => {
                self.found = Some(found);
                if self.engine == Engine::WebKitGtk {
                    if !found {
                        self.active = None;
                    } else if self.active.is_none() {
                        self.active = Some(1);
                    }
                }
            }
        }
        let after = (self.known(), self.result());
        (after != before).then_some(after.1)
    }

    /// What the strip shows now: merged from what this engine can say.
    pub fn result(&self) -> FindResult {
        if self.unavailable {
            return FindResult {
                unavailable: true,
                ..FindResult::default()
            };
        }
        match self.engine {
            Engine::WebKit => FindResult {
                found: self.found.unwrap_or(false),
                ..FindResult::default()
            },
            Engine::WebView2 | Engine::WebKitGtk => {
                let found = match (self.found, self.matches) {
                    (Some(found), _) => found,
                    (None, Some(matches)) => matches > 0,
                    (None, None) => false,
                };
                let active = self
                    .active
                    .filter(|_| found && self.matches.is_none_or(|m| m > 0));
                FindResult {
                    matches: self.matches,
                    active,
                    found,
                    unavailable: false,
                }
            }
        }
    }
}

/// The active match one on (`forward`) or back from `active`, wrapping at the ends when the count
/// is known.
pub fn step(active: u32, matches: Option<u32>, forward: bool) -> u32 {
    match (matches.filter(|m| *m > 0), forward) {
        (Some(matches), true) if active >= matches => 1,
        (Some(matches), false) if active <= 1 => matches,
        (_, true) => active.saturating_add(1),
        (_, false) => active.saturating_sub(1).max(1),
    }
}

#[cfg(test)]
mod tests;
