//! How sign-in popups end (LJ47–LJ48; bar/popup.rs opens them). Every close of the Little Joan
//! window closes them first (`page_closing`, `popup-closed hibernate`) — and from the moment it
//! starts closing (`stop_accepting`) until a new page is built, the page gets no new window at all:
//! the page lives on for a moment while it closes (its cookies, downloads, macOS' unload), and a
//! page that opened a popup on `blur` or `visibilitychange` then would get one after the window was
//! dismissed (the security review). The popup page's `window.close()` closes it (`popup-closed
//! by-page`: WebView2's `WindowCloseRequested`, WebKit's `webViewDidClose:`, WebKitGTK's `close` —
//! wired in platform/, as wry only destroys the webview); its close button too (`popup-closed
//! user`); its opener committing another origin too (`popup-closed opener-left`, LJ48). Popups are
//! destroyed from a thread of their own, never inside the engine's callback.

use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use url::Url;

use super::tracked;
use super::window::has_keyboard;
use crate::popups as rules;
use crate::smoke;

/// How long a closing waits for a popup's `about:blank` (macOS), and for its window to go.
#[cfg(target_os = "macos")]
const UNLOAD_WAIT: Duration = Duration::from_millis(400);
const GONE_WAIT: Duration = Duration::from_secs(2);

/// Marks a popup closing for `why` (once) — `had_keyboard`: it had the keyboard then — and logs
/// `popup-closed <why>` (but for one never shown). Returns whether this call marked it.
pub(super) fn mark_closing(label: &str, why: &'static str, had_keyboard: bool) -> bool {
    let mut state = tracked();
    let Some(popup) = state
        .list
        .iter_mut()
        .find(|p| p.label == label && p.closing.is_none())
    else {
        return false;
    };
    popup.closing = Some(why);
    popup.had_keyboard = had_keyboard;
    if why != "unshown" {
        smoke::log("popup-closed", why);
    }
    true
}

/// Closes a popup: the page called `window.close()` (`by-page`), the user's key (`user`), its
/// opener left (`opener-left`). Never inside the engine's callback: the window goes from a thread
/// of its own.
pub(super) fn close<R: Runtime>(app: &AppHandle<R>, label: &str, why: &'static str) {
    let open = tracked()
        .list
        .iter()
        .any(|p| p.label == label && p.closing.is_none());
    if !open {
        return;
    }
    // Only a popup the user was in gives the keyboard back to the page.
    let had_keyboard = matches!(why, "user" | "by-page") && has_keyboard(app, label);
    if mark_closing(label, why, had_keyboard) {
        destroy_later(app, label);
    }
}

pub(super) fn destroy_later<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let app = app.clone();
    let label = label.to_owned();
    let spawned = std::thread::Builder::new()
        .name("little-joan-popup-close".into())
        .spawn(move || {
            if let Some(window) = app.get_webview_window(&label) {
                unload(std::slice::from_ref(&window));
                let _ = window.destroy();
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not close the popup: {err}");
    }
}

/// macOS: popups go to `about:blank` first, as the page does — WebKit's processes and a page's
/// audio otherwise outlive the web view (LJ9) — at most `UNLOAD_WAIT`.
#[cfg(target_os = "macos")]
fn unload<R: Runtime>(windows: &[WebviewWindow<R>]) {
    let Ok(blank) = Url::parse("about:blank") else {
        return;
    };
    for window in windows {
        let _ = window.navigate(blank.clone());
    }
    let labels: Vec<&str> = windows.iter().map(WebviewWindow::label).collect();
    let deadline = Instant::now() + UNLOAD_WAIT;
    while Instant::now() < deadline {
        let unloaded = tracked()
            .list
            .iter()
            .filter(|p| labels.contains(&p.label.as_str()))
            .all(|p| p.unloaded);
        if unloaded {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    smoke::log("popup-unload", "timeout");
}

/// WebView2 and WebKitGTK end a popup's page with its webview (as the page's, measured in CI).
#[cfg(not(target_os = "macos"))]
fn unload<R: Runtime>(_windows: &[WebviewWindow<R>]) {}

/// The page starts closing: from now until a page is built again, its new windows go nowhere
/// (bar/popup.rs's `new_window`; joan/rest.rs calls this before the window hides).
pub fn stop_accepting() {
    tracked().accepting = false;
}

/// The Little Joan window is closing (bar/page.rs, before the page is released): the page takes no
/// new window any more, and every popup closes (`popup-closed hibernate`).
pub fn page_closing<R: Runtime>(app: &AppHandle<R>) {
    stop_accepting();
    close_all(app, "hibernate");
}

/// Every popup closes (`why`: `hibernate` — the window is closing —, `opener-left` — the page
/// leaves for an extension's panel, extensions/session.rs). Off the main thread; returns once they
/// are gone (at most `GONE_WAIT`), so nothing of them outlives what closed them.
pub fn close_all<R: Runtime>(app: &AppHandle<R>, why: &'static str) {
    let (marked, all) = {
        let mut state = tracked();
        let mut marked = 0;
        for popup in state.list.iter_mut().filter(|p| p.closing.is_none()) {
            popup.closing = Some(why);
            marked += 1;
        }
        let all: Vec<String> = state.list.iter().map(|p| p.label.clone()).collect();
        (marked, all)
    };
    for _ in 0..marked {
        smoke::log("popup-closed", why);
    }
    let windows: Vec<WebviewWindow<R>> = all
        .iter()
        .filter_map(|label| app.get_webview_window(label))
        .collect();
    if windows.is_empty() {
        return;
    }
    unload(&windows);
    for window in &windows {
        let _ = window.destroy();
    }
    let deadline = Instant::now() + GONE_WAIT;
    while Instant::now() < deadline
        && windows
            .iter()
            .any(|w| app.get_webview_window(w.label()).is_some())
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    if marked > 0 {
        smoke::log("popups-closed", &marked.to_string());
    }
}

/// Whether any popup exists: open, being built, or closing with its window still there. macOS: a
/// popup's `ljext` requests arrive under the page's label (the shared URL-scheme handlers), so
/// extensions/protocol.rs refuses every one while this holds.
pub fn any<R: Runtime>(app: &AppHandle<R>) -> bool {
    let (open, closing): (Vec<_>, Vec<_>) = tracked()
        .list
        .iter()
        .map(|p| (p.label.clone(), p.closing.is_some()))
        .partition(|(_, closing)| !closing);
    !open.is_empty()
        || closing
            .iter()
            .any(|(label, _)| app.get_webview_window(label).is_some())
}

/// The page webview committed `url` (bar/page.rs's `Started`, LJ48): popups whose opener's origin
/// it left close (`popup-closed opener-left`); a reload or a same-origin page keeps them.
pub fn page_committed<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    let left: Vec<String> = tracked()
        .list
        .iter()
        .filter(|p| p.closing.is_none() && rules::opener_left(&p.opener, url))
        .map(|p| p.label.clone())
        .collect();
    for label in left {
        close(app, &label, "opener-left");
    }
}

/// The popup page called `window.close()` (platform/: WebView2's `WindowCloseRequested`,
/// WebKit's `webViewDidClose:`, the WebKitGTK view's end).
pub fn closed_by_page<R: Runtime>(app: &AppHandle<R>, label: &str) {
    close(app, label, "by-page");
}

/// WebKitGTK: the popup's web view went. wry destroys it when the page calls `window.close()`,
/// leaving its window; a view that goes while its popup isn't closing went so.
#[cfg(target_os = "linux")]
pub fn view_gone<R: Runtime>(app: &AppHandle<R>, label: &str) {
    closed_by_page(app, label);
}
