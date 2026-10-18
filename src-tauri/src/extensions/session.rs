//! The extension open in the page window, and the navigation guard that keeps `ljext:` pages to
//! the ones the app opens.
//!
//! Opening an extension (the command line's `--extension`, the command bar) closes the command
//! bar and navigates the page webview to the panel's URL — after telling the guard to expect
//! exactly that URL (`NavGate`). That URL carries a fresh single-use nonce (`?lj-open=<128 random
//! bits>`, nonce.rs) that only the app knows, so no document can claim the expectation first: the
//! guard admits an extension URL only when it is the expected one, nonce included, once, within a
//! few seconds. A web page (or a panel) therefore can't navigate the page webview — or, on macOS
//! and Linux where the guard also sees frames, a frame — to an extension URL, in any form
//! (`urls::is_ext_like`). While a panel is shown
//! it can't turn into a `blob:` or `data:` document either (one would keep its origin outside
//! the files the policy allows). The bar's toolbar shows the extension's name and icon instead of
//! a domain. All of it is per window: a new window starts with nothing opened.
//!
//! **Which extension the page shows** has two answers. For serving files and the API (api.rs)
//! it is what the guard admitted (`NavGate::showing`): an admitted extension URL makes it that
//! extension, any other navigation the guard lets through makes it none — every navigation passes
//! the guard first and in order, while load events can arrive late (on Windows CI saw the start
//! page's `Finished` land after the app had started opening a panel). For what the user sees —
//! the toolbar's extension name and icon, and Reload — and for everything the handler serves but
//! the panel's document, it is what the page then **committed** (`NavGate::committed`): the
//! admitted extension's URL arriving in the page's `Started` event — the engines' commit callback
//! (WKWebView `didCommitNavigation`, WebKitGTK `LOAD_COMMITTED`, WebView2 `ContentLoading`), after
//! which the previous page is gone. `Finished` doesn't count: it can come late, and WebKit reports
//! the URL of a navigation that is still provisional. A navigation that was refused or cancelled
//! never commits, so the toolbar keeps showing the real page.
//!
//! **Opening from a web page leaves it first.** The app navigates the page to its own start page
//! (what it shows when nothing is open, whose load events CI has seen on every engine), and only
//! once that committed (or after `LEAVE_FOR`) expects and opens the panel — so normally no web
//! page is alive while a panel loads, and its frames, subresources or popups can't spoil the
//! panel's load (CI on Windows: a web page's `window.open` landing while the panel loaded left the
//! panel without its files — once unstyled with no script, once WebView2's own 404 page). It is
//! best effort, for availability: after `LEAVE_FOR` the panel opens anyway, and another navigation
//! can supersede the detour. What keeps web pages out of a panel is the nonce and the commit rule
//! (gate.rs, api.rs), not this.
//!
//! **A panel opened from hibernation waits for the bar** (LJ38): `--extension <id>` builds the
//! window and opens the panel, which could commit before the bar's script ran, and on macOS the
//! toolbar then stayed an empty field. So that opening waits — on a thread of its own, at most
//! `BAR_WAIT` — until the bar's first call (`start_info`, bar/boot.rs), then goes on as above.
//! Openings in a window whose bar already runs (the command bar, `--extension` with the window
//! open) don't wait.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

pub use super::gate::{EXPECT_FOR, LEAVE_FOR, Leaving, NavGate, page_may_navigate};
use super::nonce;
use super::registry;
use super::urls;
use crate::bar;
use crate::keys::{self, Platform};
use crate::navigation;
use crate::smoke;

/// The page a page action was opened for (`pageInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone)]
struct Session {
    id: String,
    /// What was opened, so Reload can open it again (with a new nonce).
    target: Target,
    /// The nonce in the URL the panel was opened at (`?lj-open=`): until the panel commits, the
    /// handler serves only the request that carries it.
    nonce: String,
    /// This opening's secret: the SDK served to its panel sends it with every API call.
    token: String,
    page: Option<PageInfo>,
    /// The web page that was open when the panel opened: `close` goes back to it.
    before: Option<Url>,
}

static GATE: Mutex<NavGate> = Mutex::new(NavGate::new());
static LEAVING: Mutex<Leaving> = Mutex::new(Leaving::new());
static SESSION: Mutex<Option<Session>> = Mutex::new(None);
/// What the toolbar was last told (`Some(None)` = no extension); reset per window.
static LABEL_SHOWN: Mutex<Option<Option<String>>> = Mutex::new(None);

fn session() -> std::sync::MutexGuard<'static, Option<Session>> {
    SESSION.lock().unwrap_or_else(PoisonError::into_inner)
}

fn leaving() -> std::sync::MutexGuard<'static, Leaving> {
    LEAVING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A new window, or a closed one: nothing opened, nothing expected.
pub fn reset() {
    *session() = None;
    leaving().left();
    *GATE.lock().unwrap_or_else(PoisonError::into_inner) = NavGate::new();
    *LABEL_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

/// The page webview's navigation guard (bar.rs). Refusals go to the smoke log.
pub fn allow_page_navigation(url: &Url) -> bool {
    let allowed = page_may_navigate(
        url,
        &mut GATE.lock().unwrap_or_else(PoisonError::into_inner),
        Instant::now(),
        Platform::current(),
    );
    if !allowed {
        smoke::log("navigation-refused", url.as_str());
    }
    allowed
}

/// The extension whose panel the page shows, as the guard saw it (`NavGate::showing`).
pub fn showing() -> Option<String> {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .showing()
        .map(str::to_owned)
}

/// The extension whose panel a load event committed (`NavGate::committed`): what the toolbar
/// shows.
pub fn committed() -> Option<String> {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .committed()
        .map(str::to_owned)
}

/// The page webview committed a document at `url` (bar.rs: its `Started` load event). If an
/// opening waited for the page to leave a web page and this is the app's start page, the panel
/// opens.
pub fn page_committed<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .commit(url, Platform::current());
    if keys::is_app_url(&keys::app_origin(), url)
        && let Some(panel) = leaving().left()
    {
        // Not inside the engine's load event: navigate from a thread of its own.
        let app = app.clone();
        std::thread::spawn(move || go_to_panel(&app, panel));
    }
}

/// The extension the app opened last in this window.
pub fn opened() -> Option<String> {
    session().as_ref().map(|s| s.id.clone())
}

/// The nonce of the last opening (in its panel's URL).
pub fn nonce() -> Option<String> {
    session().as_ref().map(|s| s.nonce.clone())
}

/// The secret token of the last opening (its SDK sends it as `X-LJ-Api`).
pub fn token() -> Option<String> {
    session().as_ref().map(|s| s.token.clone())
}

/// The page the open panel was opened for, if any.
pub fn page_info() -> Option<PageInfo> {
    session().as_ref().and_then(|s| s.page.clone())
}

/// What to open of an extension.
#[derive(Debug, Clone)]
pub enum Target {
    /// Its first command (`--extension <id>`).
    First,
    /// One of its commands (the command bar).
    Command(String),
    /// One of its page actions, for the web page that is open (the command bar).
    PageAction(String, PageInfo),
}

/// Opens `target` of the extension `id` in the page (switched-off extensions can't be). A page
/// action's panel gets the page (`pageInfo`) only when its manifest asks for it. The window must
/// exist.
pub fn open<R: Runtime>(app: &AppHandle<R>, id: &str, target: Target) -> Result<(), String> {
    let registry = registry::current(app);
    let extension = registry
        .get(id)
        .ok_or_else(|| format!("there is no extension `{id}` (or it is switched off)"))?;
    let manifest = &extension.manifest;
    let (contribution, page) = match &target {
        Target::First => (manifest.commands.first(), None),
        Target::Command(command) => (manifest.commands.iter().find(|c| &c.id == command), None),
        Target::PageAction(action, page) => (
            manifest.page_actions.iter().find(|c| &c.id == action),
            manifest
                .has(&super::manifest::Permission::PageInfo)
                .then(|| page.clone()),
        ),
    };
    let contribution =
        contribution.ok_or_else(|| format!("the extension `{id}` has no such command"))?;
    let nonce = nonce::random_hex();
    let url = panel_url(id, contribution.panel.as_str(), &nonce)
        .ok_or_else(|| "no URL for the panel".to_owned())?;
    let before = bar::page_url(app).filter(|current| {
        navigation::is_external_openable(current) && !keys::is_app_url(&keys::app_origin(), current)
    });
    let on_web_page = before.is_some();
    *session() = Some(Session {
        id: id.to_owned(),
        target,
        nonce,
        token: nonce::random_hex(),
        page,
        before,
    });
    smoke::log("extension-opened", id);
    if !on_web_page {
        return go_to_panel(app, url);
    }
    // Leave the web page first (the module docs say why); the start page's commit, or the timer,
    // opens the panel.
    let seq = leaving().wait(url);
    bar::navigate_pending(app, start_page()?).map_err(|err| err.to_string())?;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(LEAVE_FOR);
        let late = leaving().timed_out(seq);
        if let Some(panel) = late {
            let _ = go_to_panel(&app, panel);
        }
    });
    Ok(())
}

/// The longest a panel opened with a new window waits for the bar's script. Normally the bar runs
/// well within a second; CI's macOS runner, launching WebKit cold, took up to 2.1 s from the
/// window's creation (LJ38's notes), so the limit only bounds a bar that never runs.
pub const BAR_WAIT: Duration = Duration::from_secs(5);

/// Waits until `ready` (the bar's script runs) or `limit` passed, then calls `open` with whether
/// the bar was ready. Blocks the calling thread: never the main thread, never an engine callback.
pub fn after_bar(ready: &bar::Ready, limit: Duration, open: impl FnOnce(bool)) {
    open(ready.wait(limit));
}

/// Opens `target` of the extension `id` in a window just built (`--extension` while
/// hibernating): from a thread of its own, once the bar's script runs, or after `BAR_WAIT`
/// anyway (logging `bar-ready timeout`). Errors go to stderr and the smoke log
/// (`extension-missing`), as the opening's would.
pub fn open_when_bar_ready<R: Runtime>(app: &AppHandle<R>, id: &str, target: Target) {
    fn open_now<R: Runtime>(app: &AppHandle<R>, id: &str, target: Target) {
        if let Err(err) = open(app, id, target) {
            eprintln!("little-joan: could not open the extension: {err}");
            smoke::log("extension-missing", id);
        }
    }
    let (waiting_app, waiting_id, waiting_target) = (app.clone(), id.to_owned(), target.clone());
    let spawned = std::thread::Builder::new()
        .name("little-joan-open-extension".into())
        .spawn(move || {
            after_bar(&bar::BAR_READY, BAR_WAIT, |ready| {
                if !ready {
                    smoke::log("bar-ready", "timeout");
                }
                open_now(&waiting_app, &waiting_id, waiting_target);
            });
        });
    if spawned.is_err() {
        // No thread to wait on: open now, as before LJ38.
        open_now(app, id, target);
    }
}

/// The app's start page, what the page shows when nothing is open.
fn start_page() -> Result<Url, String> {
    keys::app_origin()
        .join("index.html")
        .map_err(|err| err.to_string())
}

/// Tells the guard to expect the panel's URL and navigates the page there. The page's address and
/// the toolbar change when the panel's own load events arrive.
fn go_to_panel<R: Runtime>(app: &AppHandle<R>, url: Url) -> Result<(), String> {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .expect(&url, Instant::now());
    bar::navigate_pending(app, url).map_err(|err| err.to_string())
}

/// The URL the app opens a panel at: the panel file in this platform's form, with the
/// single-use nonce (`?lj-open=<hex>`) the guard will expect.
pub fn panel_url(id: &str, panel: &str, nonce: &str) -> Option<Url> {
    let mut url = urls::file_url(id, panel, Platform::current())?;
    url.set_query(Some(&format!("lj-open={nonce}")));
    Some(url)
}

/// The Reload page action: on a panel that a load event committed, the app opens the same thing
/// again — a new nonce, never the old address. Returns whether it did (otherwise the page reloads
/// as a web page would; a panel that never committed isn't reloaded at all, the guard refuses it).
pub fn reopen<R: Runtime>(app: &AppHandle<R>) -> bool {
    let Some(committed) = committed() else {
        return false;
    };
    let Some(open_again) = session().as_ref().filter(|s| s.id == committed).cloned() else {
        return false;
    };
    if let Err(err) = open(app, &open_again.id, open_again.target) {
        eprintln!("little-joan: could not open the extension again: {err}");
    }
    // Close still goes back to the web page the panel was first opened from.
    if let Some(session) = session().as_mut() {
        session.before = open_again.before;
    }
    true
}

/// littleJoan.close(): back to the web page the panel was opened from, or to the command bar.
pub fn close<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    leaving().left();
    let before = session().take().and_then(|s| s.before);
    match before {
        Some(url) => bar::navigate_pending(app, url).map_err(|err| err.to_string()),
        None => {
            bar::navigate_pending(app, start_page()?).map_err(|err| err.to_string())?;
            bar::focus_bar(app);
            Ok(())
        }
    }
}

/// What the toolbar shows for an extension's panel.
#[derive(Debug, Clone, Serialize)]
struct Label {
    name: String,
    icon: Option<String>,
}

/// Tells the bar which extension the page shows — only once a load event committed it
/// (`committed`; `None`: a web page or nothing) — when that changed, or always with `force` (the
/// bar's script just started).
pub fn publish_label<R: Runtime>(app: &AppHandle<R>, force: bool) {
    let id = committed();
    {
        let mut shown = LABEL_SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
        if !force && shown.as_ref() == Some(&id) {
            return;
        }
        *shown = Some(id.clone());
    }
    smoke::log("toolbar-label", id.as_deref().unwrap_or("none"));
    let label = id
        .and_then(|id| registry::current(app).get(&id))
        .map(|extension| Label {
            name: extension.manifest.name.clone(),
            icon: extension.icon_data_uri(),
        });
    let Some(bar) = app.get_webview(bar::BAR) else {
        return;
    };
    // The name is checked text and the icon a base64 data: URI; JSON makes a safe script
    // literal of both, and the bar draws the name as text.
    let json = serde_json::to_string(&label).unwrap_or_else(|_| "null".to_owned());
    let _ = bar.eval(format!(
        "window.littleJoan && window.littleJoan.setExtension && window.littleJoan.setExtension({json})"
    ));
}

#[cfg(test)]
mod tests;
