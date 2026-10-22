//! History on disk and in memory (history.rs is the pure part).
//!
//! `history.json` lives next to settings.json. It is loaded when the page window opens — on a
//! thread of its own, while the window and its webviews are built (LJ19.1: a big history took
//! 20–50 ms off every summon); everything that uses the window's history waits for that load
//! first (`wait_loaded`) —, kept in `AppState` while the window exists, saved atomically when the
//! window closes and every 30 s while it has changes, and dropped when Little Joan hibernates —
//! nothing extra is held at rest. Visits are recorded natively from the page webview's load and title events (bar.rs),
//! so pages need no IPC; nothing is recorded while the "Remember history" setting is off.
//! Removing a page or clearing a range (the history view, Settings → Privacy) rewrites the file
//! at once — with no window open, straight on disk. History never leaves the machine.

use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Runtime};
use url::Url;

use crate::history::{ClearRange, HISTORY_FILE, History, Listed, VisitKind};
use crate::input::SearchEngine;
use crate::keys;
use crate::smoke;
use crate::state;
use crate::suggest::{self, Suggestion};
use crate::summon_timing::{self, Mark};

/// How often unsaved visits are written while the window is open.
const AUTOSAVE_EVERY: Duration = Duration::from_secs(30);

/// Held across every read-modify-write of history.json and every change of the window's history
/// (open, close, autosave, remove, clear), so writes never overlap and an older copy can't
/// overwrite a newer one (an autosave's snapshot landing after a clear). Always taken before the
/// app state's lock, never inside it.
static FILE: Mutex<()> = Mutex::new(());
/// Bumped for every window, so an old autosave loop stops.
static GENERATION: AtomicU64 = AtomicU64::new(0);
/// Bumped when a window opens and when it closes: a load that finishes after its window closed
/// keeps nothing (nothing may be held at rest).
static WINDOW: AtomicU64 = AtomicU64::new(0);
/// How many reads of history.json are under way (one per window that opened; an old window's
/// read can still be finishing when the next window opens).
static LOADING: Loading = Loading::new();
/// The longest anything waits for that read (a stuck disk mustn't hang the window for good).
const LOAD_WAIT: Duration = Duration::from_secs(10);

/// Reads under way, and a wait for them with a time limit.
struct Loading {
    busy: Mutex<u32>,
    done: Condvar,
}

impl Loading {
    const fn new() -> Self {
        Loading {
            busy: Mutex::new(0),
            done: Condvar::new(),
        }
    }

    fn started(&self) {
        *self.busy.lock().unwrap_or_else(PoisonError::into_inner) += 1;
    }

    fn finished(&self) {
        let mut busy = self.busy.lock().unwrap_or_else(PoisonError::into_inner);
        *busy = busy.saturating_sub(1);
        if *busy == 0 {
            self.done.notify_all();
        }
    }

    /// Blocks until no read is under way or `limit` passed. Never call it holding the file lock
    /// or the app state's lock (a read takes both).
    fn wait(&self, limit: Duration) {
        let busy = self.busy.lock().unwrap_or_else(PoisonError::into_inner);
        let (busy, timeout) = self
            .done
            .wait_timeout_while(busy, limit, |busy| *busy > 0)
            .unwrap_or_else(PoisonError::into_inner);
        if *busy > 0 && timeout.timed_out() {
            eprintln!("little-joan: history.json is taking long to load; going on without it");
        }
    }
}

/// Waits for the open window's history to be loaded (at once when it is, or with no window).
fn wait_loaded() {
    LOADING.wait(LOAD_WAIT);
}

/// The history of the open window.
#[derive(Debug, Default)]
pub struct OpenHistory {
    history: History,
    /// Changed since the last save.
    dirty: bool,
    /// The kind of the next page load's visit (`Link` unless the app started it).
    next_kind: Option<VisitKind>,
    /// The page's last title, for a visit recorded after its title arrived.
    last_title: Option<(Url, String)>,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Reads `dir/history.json`; a missing or bad file is empty history.
pub fn load(dir: &Path) -> History {
    fs::read_to_string(dir.join(HISTORY_FILE)).map_or_else(
        |_| History::default(),
        |text| History::parse(&text, now_millis(), &keys::app_origin()),
    )
}

fn file_lock() -> MutexGuard<'static, ()> {
    FILE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Writes `dir/history.json` atomically (a temp file, then a rename over the old one).
pub fn save(dir: &Path, history: &History) -> io::Result<()> {
    let _file = file_lock();
    save_unlocked(dir, history)
}

/// `save` for callers that hold the file lock already.
fn save_unlocked(dir: &Path, history: &History) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{HISTORY_FILE}.tmp"));
    fs::write(&tmp, history.to_json())?;
    fs::rename(&tmp, dir.join(HISTORY_FILE))
}

/// The window opened: starts loading history (its own thread; whatever needs it waits) and the
/// autosave loop. Returns at once.
pub fn open<R: Runtime>(app: &AppHandle<R>) {
    let window = WINDOW.fetch_add(1, Ordering::SeqCst) + 1;
    LOADING.started();
    let loader = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-history-load".into())
        .spawn(move || {
            load_into(&loader, window);
            LOADING.finished();
        });
    if spawned.is_err() {
        // No thread: load it here, as before LJ19.1.
        load_into(app, window);
        LOADING.finished();
    }
    autosave(app);
}

/// Reads history.json into the window `window`'s state — unless that window closed meanwhile.
fn load_into<R: Runtime>(app: &AppHandle<R>, window: u64) {
    let _file = file_lock();
    let dir = state::with(app, |s| s.data_dir.clone());
    let history = load(&dir);
    state::with(app, |s| {
        if WINDOW.load(Ordering::SeqCst) == window {
            s.history = Some(OpenHistory {
                history,
                ..OpenHistory::default()
            });
        }
    });
    summon_timing::mark(Mark::HistoryLoaded);
}

/// The window closes: saves what changed and drops the history from memory.
pub fn close<R: Runtime>(app: &AppHandle<R>) {
    GENERATION.fetch_add(1, Ordering::Relaxed);
    // A load still running keeps nothing now; one that finished is saved below.
    WINDOW.fetch_add(1, Ordering::SeqCst);
    wait_loaded();
    let _file = file_lock();
    let (dir, open) = state::with(app, |s| (s.data_dir.clone(), s.history.take()));
    if let Some(open) = open.filter(|open| open.dirty) {
        write(&dir, &open.history);
    }
}

/// Saves (the caller holds the file lock) and logs the outcome.
fn write(dir: &Path, history: &History) {
    match save_unlocked(dir, history) {
        Ok(()) => smoke::log("history-saved", &history.entries().len().to_string()),
        Err(err) => eprintln!("little-joan: could not save the history: {err}"),
    }
}

/// Saves every 30 s while the window has unsaved visits; stops within a second of the window
/// closing (it checks once a second).
fn autosave<R: Runtime>(app: &AppHandle<R>) {
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-history".into())
        .spawn(move || {
            let mut waited = Duration::ZERO;
            loop {
                std::thread::sleep(Duration::from_secs(1));
                if GENERATION.load(Ordering::Relaxed) != generation {
                    break;
                }
                waited += Duration::from_secs(1);
                if waited < AUTOSAVE_EVERY {
                    continue;
                }
                waited = Duration::ZERO;
                let _file = file_lock();
                let pending = state::with(&app, |s| {
                    let dir = s.data_dir.clone();
                    let open = s.history.as_mut().filter(|open| open.dirty)?;
                    open.dirty = false;
                    Some((dir, open.history.clone()))
                });
                if let Some((dir, history)) = pending {
                    write(&dir, &history);
                }
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not start saving the history: {err}");
    }
}

/// The app is about to load something in the page: its visit gets `kind` (open_input: typed;
/// the reload page action: reload).
pub fn set_next_kind<R: Runtime>(app: &AppHandle<R>, kind: VisitKind) {
    wait_loaded();
    state::with(app, |s| {
        if let Some(open) = s.history.as_mut() {
            open.next_kind = Some(kind);
        }
    });
}

/// The page finished loading `url`: one visit, unless history is off or the page is the app's
/// own (or not a web page at all).
pub fn page_loaded<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    wait_loaded();
    let recorded = state::with(app, |s| {
        let remember = s.settings.remember_history;
        let open = s.history.as_mut()?;
        let kind = open.next_kind.take().unwrap_or(VisitKind::Link);
        if !remember {
            return None;
        }
        let title = open
            .last_title
            .as_ref()
            .filter(|(at, _)| same_page(at, url))
            .map(|(_, title)| title.clone());
        let changed = open.history.record(
            url,
            title.as_deref(),
            kind,
            now_millis(),
            &keys::app_origin(),
        );
        open.dirty |= changed;
        changed.then_some(kind)
    });
    if let Some(kind) = recorded {
        smoke::log("history-visit", &format!("{kind:?} {url}"));
    }
}

/// The same page: equal but for the fragment (history keeps no fragments).
fn same_page(a: &Url, b: &Url) -> bool {
    let (mut a, mut b) = (a.clone(), b.clone());
    a.set_fragment(None);
    b.set_fragment(None);
    a == b
}

/// The page at `url` is now titled `title`.
pub fn title_changed<R: Runtime>(app: &AppHandle<R>, url: &Url, title: &str) {
    wait_loaded();
    state::with(app, |s| {
        let remember = s.settings.remember_history;
        let Some(open) = s.history.as_mut() else {
            return;
        };
        if !remember {
            return;
        }
        open.last_title = Some((url.clone(), title.to_owned()));
        open.dirty |= open.history.set_title(url, title, &keys::app_origin());
    });
}

/// Forgets the record of `url` (Shift+Delete on a suggestion, × in the history view) and
/// rewrites history.json at once. Returns whether there was one.
pub fn remove<R: Runtime>(app: &AppHandle<R>, url: &str) -> io::Result<bool> {
    edit(app, removing(url))
}

/// The change `remove` makes: (whether there was a record, whether to rewrite the file).
fn removing(url: &str) -> impl FnOnce(&mut History, u64) -> (bool, bool) + '_ {
    move |history, _| {
        let removed = history.remove(url);
        (removed, removed)
    }
}

/// Forgets the visits of `range` (the history view's "Clear history…", Settings → Privacy) and
/// rewrites history.json at once — also with no window open, and always for everything (a file
/// that couldn't be read goes too). Returns how many pages went.
pub fn clear<R: Runtime>(app: &AppHandle<R>, range: ClearRange) -> io::Result<usize> {
    edit(app, clearing(range))
}

/// The change `clear` makes: (pages removed, whether to rewrite the file).
fn clearing(range: ClearRange) -> impl FnOnce(&mut History, u64) -> (usize, bool) {
    move |history, now| {
        let removed = history.clear(range, now);
        (removed, removed > 0 || range == ClearRange::All)
    }
}

/// Every page for the history view, latest visit first (empty while no window is open).
pub fn list<R: Runtime>(app: &AppHandle<R>) -> Vec<Listed> {
    wait_loaded();
    state::with(app, |s| {
        s.history
            .as_ref()
            .map(|open| open.history.list())
            .unwrap_or_default()
    })
}

/// Applies `change` under the file lock (so the window can't open or close meanwhile).
fn edit<R: Runtime, T>(
    app: &AppHandle<R>,
    change: impl FnOnce(&mut History, u64) -> (T, bool),
) -> io::Result<T> {
    wait_loaded();
    let _file = file_lock();
    state::with(app, |s| {
        let dir = s.data_dir.clone();
        edit_in(&mut s.history, &dir, now_millis(), change)
    })
}

/// Applies `change` to the open window's history (`slot`) or, with no window, to the file in
/// `dir`, and writes history.json at once when `change` says so (its second value). The caller
/// holds the file lock.
fn edit_in<T>(
    slot: &mut Option<OpenHistory>,
    dir: &Path,
    now: u64,
    change: impl FnOnce(&mut History, u64) -> (T, bool),
) -> io::Result<T> {
    let mut on_disk;
    let (history, open) = match slot.as_mut() {
        Some(open) => (&mut open.history, Some(&mut open.dirty)),
        None => {
            on_disk = load(dir);
            (&mut on_disk, None)
        }
    };
    let (outcome, rewrite) = change(history, now);
    if rewrite {
        // Until the file is written, the window still has unsaved changes.
        if let Some(dirty) = open {
            *dirty = true;
            save_unlocked(dir, history)?;
            *dirty = false;
        } else {
            save_unlocked(dir, history)?;
        }
        smoke::log("history-saved", &history.entries().len().to_string());
    }
    Ok(outcome)
}

/// The command bar's suggestions for `query` (suggest.rs); with no window open, those of empty
/// history.
pub fn suggest<R: Runtime>(
    app: &AppHandle<R>,
    query: &str,
    current: Option<&Url>,
    engine: SearchEngine,
) -> Vec<Suggestion> {
    wait_loaded();
    state::with(app, |s| match s.history.as_ref() {
        Some(open) => suggest::suggestions(&open.history, query, current, engine, now_millis()),
        None => suggest::suggestions(&History::default(), query, current, engine, now_millis()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "little-joan-test-history-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn waiting_for_the_load_ends_when_every_read_is_over() {
        let loading = std::sync::Arc::new(Loading::new());
        // Nothing under way: no wait at all.
        let start = std::time::Instant::now();
        loading.wait(Duration::from_secs(5));
        assert!(start.elapsed() < Duration::from_secs(1));
        // Two reads (an old window's still finishing): the wait lasts until both are over.
        loading.started();
        loading.started();
        let reads = std::sync::Arc::clone(&loading);
        let finisher = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            reads.finished();
            std::thread::sleep(Duration::from_millis(50));
            reads.finished();
        });
        let start = std::time::Instant::now();
        loading.wait(Duration::from_secs(5));
        assert!(start.elapsed() >= Duration::from_millis(90));
        assert!(start.elapsed() < Duration::from_secs(4));
        finisher.join().expect("the finishing thread");
        // A read that never ends: the wait gives up at its limit.
        loading.started();
        let start = std::time::Instant::now();
        loading.wait(Duration::from_millis(60));
        assert!(start.elapsed() >= Duration::from_millis(60));
        loading.finished();
        // More ends than starts never wraps around.
        loading.finished();
        let start = std::time::Instant::now();
        loading.wait(Duration::from_secs(5));
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn a_title_seen_before_the_load_belongs_to_the_same_page() {
        let url = |s: &str| Url::parse(s).expect("url");
        assert!(same_page(
            &url("https://a.test/x#one"),
            &url("https://a.test/x")
        ));
        assert!(!same_page(
            &url("https://a.test/x?q=1"),
            &url("https://a.test/x")
        ));
    }

    #[test]
    fn a_missing_or_bad_file_loads_as_empty_history() {
        let dir = temp_dir("bad");
        assert_eq!(load(&dir), History::default());
        fs::create_dir_all(&dir).expect("create");
        fs::write(dir.join(HISTORY_FILE), "{ not json").expect("write");
        assert_eq!(load(&dir), History::default());
        let _ = fs::remove_dir_all(&dir);
    }

    fn two_pages(dir: &Path) -> History {
        let mut history = History::default();
        for (address, at) in [("https://a.test/", now_millis()), ("https://b.test/", 1)] {
            let url = Url::parse(address).expect("url");
            history.record(&url, Some("T"), VisitKind::Link, at, &keys::app_origin());
        }
        save(dir, &history).expect("save");
        history
    }

    /// How many records history.json holds, read as plain JSON (`None`: not a history file).
    fn file_entries(dir: &Path) -> Option<usize> {
        let text = fs::read_to_string(dir.join(HISTORY_FILE)).ok()?;
        let value: serde_json::Value = serde_json::from_str(&text).ok()?;
        Some(value.get("entries")?.as_array()?.len())
    }

    fn urls(history: &History) -> Vec<&str> {
        history.entries().iter().map(|e| e.url.as_str()).collect()
    }

    #[test]
    fn removing_and_clearing_change_the_open_history_and_rewrite_the_file() {
        let dir = temp_dir("edit-open");
        let history = two_pages(&dir);
        let mut slot = Some(OpenHistory {
            history,
            dirty: true,
            ..OpenHistory::default()
        });
        let now = now_millis();
        let removed = edit_in(&mut slot, &dir, now, removing("https://a.test/")).expect("edit");
        assert!(removed);
        let open = slot.as_ref().expect("still open");
        assert_eq!(urls(&open.history), ["https://b.test/"], "in memory");
        assert!(!open.dirty, "everything is on disk");
        assert_eq!(urls(&load(&dir)), ["https://b.test/"], "on disk");
        let again = edit_in(&mut slot, &dir, now, removing("https://a.test/")).expect("edit");
        assert!(!again, "already gone");

        let cleared = edit_in(&mut slot, &dir, now, clearing(ClearRange::All)).expect("clear");
        assert_eq!(cleared, 1);
        assert!(slot.as_ref().expect("open").history.entries().is_empty());
        assert!(load(&dir).entries().is_empty());
        assert_eq!(file_entries(&dir), Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn with_no_window_the_file_itself_is_rewritten() {
        let dir = temp_dir("edit-closed");
        two_pages(&dir);
        let mut slot = None;
        // The last hour: a.test was visited just now, b.test long ago.
        let cleared =
            edit_in(&mut slot, &dir, now_millis(), clearing(ClearRange::Hour)).expect("clear");
        assert_eq!(cleared, 1);
        assert!(slot.is_none(), "nothing is loaded into memory");
        assert_eq!(urls(&load(&dir)), ["https://b.test/"]);
        let removed =
            edit_in(&mut slot, &dir, now_millis(), removing("https://b.test/")).expect("remove");
        assert!(removed);
        assert!(load(&dir).entries().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clearing_everything_replaces_a_bad_file() {
        let dir = temp_dir("edit-bad");
        fs::create_dir_all(&dir).expect("create");
        fs::write(dir.join(HISTORY_FILE), "{ not json").expect("write");
        let cleared =
            edit_in(&mut None, &dir, now_millis(), clearing(ClearRange::All)).expect("clear");
        assert_eq!(cleared, 0);
        assert_eq!(file_entries(&dir), Some(0));
        // A smaller range leaves an untouched file alone.
        fs::write(dir.join(HISTORY_FILE), "{ not json").expect("write");
        edit_in(&mut None, &dir, now_millis(), clearing(ClearRange::Hour)).expect("clear");
        assert_eq!(
            fs::read_to_string(dir.join(HISTORY_FILE)).expect("read"),
            "{ not json"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn saves_are_atomic_and_load_back() {
        let dir = temp_dir("save");
        let mut history = History::default();
        let url = Url::parse("https://example.com/a").expect("url");
        history.record(
            &url,
            Some("A"),
            VisitKind::Typed,
            now_millis(),
            &keys::app_origin(),
        );
        save(&dir, &history).expect("save");
        assert!(!dir.join(format!("{HISTORY_FILE}.tmp")).exists());
        assert_eq!(load(&dir), history);
        // A second save replaces the file whole.
        let mut more = history.clone();
        more.record(
            &url,
            None,
            VisitKind::Reload,
            now_millis(),
            &keys::app_origin(),
        );
        save(&dir, &more).expect("save again");
        assert_eq!(load(&dir), more);
        let _ = fs::remove_dir_all(&dir);
    }
}
