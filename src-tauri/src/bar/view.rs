//! How the page window is laid out right now (bar.rs's view state): the 40 px toolbar above the
//! page, the command bar's overlay covering the window (the page hidden), or element fullscreen
//! (the page alone; it always wins). layout.rs does the math; this applies it to the webviews.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, Window};

use super::{BAR, PAGE};
use crate::layout::{self, Mode, Rect};
use crate::platform;
use crate::smoke;

/// True while the page shows an element fullscreen (the bar is hidden meanwhile).
static ELEMENT_FULLSCREEN: AtomicBool = AtomicBool::new(false);
/// True while the bar covers the window (the command bar) and the page is hidden. A new window
/// starts with it.
static OVERLAY: AtomicBool = AtomicBool::new(false);

/// A new window: no fullscreen, and the command bar open (every summon opens it).
pub(super) fn new_window() {
    ELEMENT_FULLSCREEN.store(false, Ordering::Relaxed);
    OVERLAY.store(true, Ordering::Relaxed);
}

/// What the window shows now: the toolbar, the overlay, or the page alone (element fullscreen,
/// which always wins).
pub fn mode() -> Mode {
    Mode::from_state(
        OVERLAY.load(Ordering::Relaxed),
        ELEMENT_FULLSCREEN.load(Ordering::Relaxed),
    )
}

/// Whether the command bar's overlay is open.
pub fn overlay() -> bool {
    OVERLAY.load(Ordering::Relaxed)
}

/// Positions the bar and the page for the window's current size and mode.
pub fn apply_layout<R: Runtime>(window: &Window<R>) {
    let app = window.app_handle();
    let (Some(bar), Some(page)) = (app.get_webview(BAR), app.get_webview(PAGE)) else {
        return;
    };
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    // inner_size is physical; the bounds below are logical (layout.rs).
    let wanted = layout::for_physical_size(size.width, size.height, scale, mode());
    place(&bar, wanted.bar);
    place(&page, wanted.page);
}

/// Shows `webview` at `rect`, or hides it.
fn place<R: Runtime>(webview: &Webview<R>, rect: Option<Rect>) {
    match rect {
        Some(rect) => {
            let _ = webview.set_bounds(to_tauri(rect));
            let _ = webview.show();
        }
        None => {
            let _ = webview.hide();
        }
    }
}

fn to_tauri(rect: Rect) -> tauri::Rect {
    tauri::Rect {
        position: LogicalPosition::new(rect.x, rect.y).into(),
        size: LogicalSize::new(rect.width, rect.height).into(),
    }
}

/// Opens (`on`) or closes the command bar's overlay: the bar covers the whole window and the
/// page is hidden, or the bar goes back to the 40 px toolbar above the page. Element fullscreen
/// wins: the overlay doesn't open meanwhile.
pub fn set_overlay<R: Runtime>(app: &AppHandle<R>, on: bool) {
    if on && element_fullscreen() {
        return;
    }
    if change_overlay(app, on)
        && let Some(window) = app.get_window(crate::joan::LABEL)
    {
        apply_layout(&window);
    }
}

/// Records the overlay state; on a change, re-packs the bar (Linux), tells the bar and logs it.
/// Returns whether anything changed.
fn change_overlay<R: Runtime>(app: &AppHandle<R>, on: bool) -> bool {
    if OVERLAY.swap(on, Ordering::Relaxed) == on {
        return false;
    }
    if let (Some(bar), Some(page)) = (app.get_webview(BAR), app.get_webview(PAGE)) {
        platform::set_overlay(&bar, &page, on);
        tell_overlay(&bar, on);
    }
    log_overlay();
    true
}

/// Tells the bar's page which view it is in.
pub(super) fn tell_overlay<R: Runtime>(bar: &Webview<R>, on: bool) {
    let _ = bar.eval(format!(
        "window.littleJoan && window.littleJoan.setOverlay({on})"
    ));
}

/// Writes the overlay state to the smoke log (`overlay true|false`).
pub fn log_overlay() {
    smoke::log("overlay", if overlay() { "true" } else { "false" });
}

/// Whether the page shows an element fullscreen right now.
pub fn element_fullscreen() -> bool {
    ELEMENT_FULLSCREEN.load(Ordering::Relaxed)
}

/// The page entered or left element fullscreen: hide or bring back the bar. Starting it closes
/// the overlay (fullscreen always wins).
pub fn set_element_fullscreen<R: Runtime>(app: &AppHandle<R>, on: bool) {
    ELEMENT_FULLSCREEN.store(on, Ordering::Relaxed);
    smoke::log("element-fullscreen", if on { "true" } else { "false" });
    if on {
        change_overlay(app, false);
    }
    if let Some(window) = app.get_window(crate::joan::LABEL) {
        apply_layout(&window);
    }
}
