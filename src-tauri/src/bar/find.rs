//! Find in page at run time (LJ49; the state is find.rs's). Ctrl/Cmd+F — a keymap action, caught
//! natively like every page key — opens the find strip in the toolbar (the trusted bar), with a web
//! page or an extension's panel open: the command bar closes first, and the strip's field gets the
//! keyboard. The bar sends what is typed through the `find` command; the page's engine finds
//! natively (platform/find.rs) and reports here, and the result goes back to the bar as
//! `window.littleJoan.setFindResult(<json>)`. The page itself gets no script and no IPC.
//!
//! The find ends — the strip closes (`window.littleJoan.closeFind()`) — when the bar closes it
//! (Esc, ×: the page gets the keyboard), when the page starts loading another document, when the
//! command bar opens, and with the window.
//!
//! Smoke log: `find-open`, `find-action <kind> [length=<n>]` (never the query), `find-result
//! matches=3 active=1` / `found=true` / `unavailable`, `find-closed <user|page|overlay>`, and
//! `find-engine …` once (platform/find.rs).

use std::sync::{Mutex, MutexGuard, PoisonError};

use tauri::{AppHandle, Manager, Runtime};

use super::{BAR, page, page_url};
use crate::extensions;
use crate::find::{Engine, EngineOp, FindAction, FindResult, FindState, Report};
use crate::keys;
use crate::navigation;
use crate::platform;
use crate::smoke;

/// The page window's find.
static STATE: Mutex<FindState> = Mutex::new(FindState::new(Engine::current()));
/// What the bar was last told for the current query (`None`: nothing yet).
static TOLD: Mutex<Option<FindResult>> = Mutex::new(None);

fn state() -> MutexGuard<'static, FindState> {
    STATE.lock().unwrap_or_else(PoisonError::into_inner)
}

fn told() -> MutexGuard<'static, Option<FindResult>> {
    TOLD.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether the page shows something to find in: a web page or an extension's panel (never the
/// start page, nor a window without a page yet).
fn findable<R: Runtime>(app: &AppHandle<R>) -> bool {
    page_url(app).is_some_and(|url| {
        extensions::urls::is_ext_like(&url)
            || (!keys::is_app_url(&keys::app_origin(), &url)
                && navigation::is_external_openable(&url))
    })
}

/// Ctrl/Cmd+F: the find strip opens in the toolbar, its field with the keyboard and its text
/// selected (open already: the same). The command bar closes first. Nothing on the start page,
/// nor during element fullscreen.
pub fn open<R: Runtime>(app: &AppHandle<R>) {
    if super::element_fullscreen() || super::page_in_webkit_fullscreen(app) || !findable(app) {
        return;
    }
    if super::overlay() {
        super::set_overlay(app, false);
    }
    state().open();
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.set_focus();
        let _ = bar.eval("window.littleJoan && window.littleJoan.openFind()");
    }
    smoke::log("find-open", "");
}

/// The bar's `find` command: a new query, the next or previous match, or the strip closed (the
/// page then gets the keyboard).
pub fn act<R: Runtime>(app: &AppHandle<R>, action: FindAction) {
    smoke::log("find-action", &action.log_detail());
    let closing = action == FindAction::Stop {};
    let (was_open, op, generation) = {
        let mut state = state();
        let was_open = state.is_open();
        let op = state.act(action);
        (was_open, op, state.generation())
    };
    if matches!(op, Some(EngineOp::Start(_))) || closing {
        // A new query: whatever the bar was told is old now.
        *told() = None;
    }
    if let (Some(op), Some(page)) = (op, page(app)) {
        platform::find(&page, generation, op);
    }
    if closing {
        if was_open {
            smoke::log("find-closed", "user");
        }
        super::focus_page(app);
    } else {
        // WebKitGTK's next / previous move the active match here (find.rs).
        publish(app);
    }
}

/// An engine's report (platform/find.rs), from any thread: `generation` says which find it is for
/// where the engine can say (`None`: the current one).
pub fn report<R: Runtime>(app: &AppHandle<R>, generation: Option<u64>, report: Report) {
    if state().report(generation, report).is_none() {
        return;
    }
    // Off the engine's callback: the bar is told from a task of its own (each reads the latest).
    let app = app.clone();
    tauri::async_runtime::spawn(async move { publish(&app) });
}

/// Tells the bar the find's result, if the engine has said something new about the current query.
fn publish<R: Runtime>(app: &AppHandle<R>) {
    let result = {
        let state = state();
        if !state.is_open() || !state.known() {
            return;
        }
        state.result()
    };
    {
        let mut told = told();
        if *told == Some(result) {
            return;
        }
        *told = Some(result);
    }
    smoke::log("find-result", &result.log_line());
    let Ok(json) = serde_json::to_string(&result) else {
        return;
    };
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.eval(format!(
            "window.littleJoan && window.littleJoan.setFindResult({json})"
        ));
    }
}

/// The find ends without the bar asking (`why`: `page` — another document started loading —,
/// `overlay` — the command bar opened): the engine stops, the strip closes.
pub fn end<R: Runtime>(app: &AppHandle<R>, why: &str) {
    let (ended, generation) = {
        let mut state = state();
        (state.end(), state.generation())
    };
    *told() = None;
    if ended.stop_engine
        && let Some(page) = page(app)
    {
        platform::find(&page, generation, EngineOp::Stop);
    }
    if ended.was_open {
        if let Some(bar) = app.get_webview(BAR) {
            let _ = bar.eval("window.littleJoan && window.littleJoan.closeFind()");
        }
        smoke::log("find-closed", why);
    }
}

/// The page started loading a document: a find in the old one ends (off the engine's callback).
pub fn page_started<R: Runtime>(app: &AppHandle<R>) {
    let idle = {
        let state = state();
        !state.is_open() && state.query().is_none()
    };
    if idle {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move { end(&app, "page") });
}

/// A page webview built or closed (the bar rests itself): nothing of a find is kept.
pub fn forget() {
    state().end();
    *told() = None;
}
