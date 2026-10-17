//! What the page keys (keys.rs) and the macOS menu do to the page window.

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};

use crate::joan;
use crate::keys::{self, PageAction};
use crate::toggle::Command;

/// Performs `action` on the page window, off the main thread (callers are native key and menu
/// handlers running on it).
pub fn perform<R: Runtime>(app: &AppHandle<R>, action: PageAction) {
    if action == PageAction::Close {
        joan::dispatch(app, Command::Hide);
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(window) = app.get_webview_window(joan::LABEL) else {
            return;
        };
        let result = match action {
            PageAction::OpenLocation => open_location(&window),
            PageAction::Reload => window.reload(),
            PageAction::Back => window.eval("history.back()"),
            PageAction::Forward => window.eval("history.forward()"),
            PageAction::Close => Ok(()),
        };
        if let Err(err) = result {
            eprintln!("little-joan: page action {action:?} failed: {err}");
        }
    });
}

/// Shows the start page with the current address pre-filled and selected. On the start page
/// itself there is nothing to do.
fn open_location<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let current = window.url()?;
    match keys::start_page_url(&keys::app_origin(), &current) {
        Some(url) => window.navigate(url),
        None => Ok(()),
    }
}
