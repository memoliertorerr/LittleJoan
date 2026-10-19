//! Page zoom at run time (LJ16; the levels are zoom.rs's). The zoom keys are keymap actions
//! (`zoom-in`, `zoom-out`, `zoom-reset`), caught natively like every page key, and set the page
//! webview's zoom with `Webview::set_zoom` — WebView2's controller zoom factor, WKWebView's
//! `pageZoom`, WebKitGTK's zoom level, each kept by the engine across the window's navigations.
//! Per window: every summon starts at 100 %, and nothing is saved. The bar's status area says
//! "Zoom 110 %" for a moment (bar/status.rs). Phone mode is unaffected: its 600 / 640 px are the
//! window's width, not the page's CSS width.

use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager, Runtime};

use super::PAGE;
use super::status::{self, Status};
use crate::smoke;
use crate::zoom::{self, Step};

/// The open window's zoom level, in percent.
static LEVEL: Mutex<u16> = Mutex::new(zoom::DEFAULT);

/// A new window: 100 %.
pub fn new_window() {
    *LEVEL.lock().unwrap_or_else(PoisonError::into_inner) = zoom::DEFAULT;
}

/// Zooms the page in, out, or back to 100 %, and says so in the toolbar's status area.
pub fn zoom_page<R: Runtime>(app: &AppHandle<R>, step: Step) -> tauri::Result<()> {
    let Some(page) = app.get_webview(PAGE) else {
        return Ok(());
    };
    let percent = {
        let mut level = LEVEL.lock().unwrap_or_else(PoisonError::into_inner);
        *level = zoom::step(*level, step);
        *level
    };
    page.set_zoom(zoom::factor(percent))?;
    smoke::log("zoom", &percent.to_string());
    status::tell(app, &Status::Zoom { percent });
    Ok(())
}
