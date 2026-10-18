//! How the page window is laid out right now (bar.rs's view state): the 40 px toolbar above the
//! page, the command bar's overlay covering the window (the page hidden), or element fullscreen
//! (the page alone; it always wins). layout.rs does the math; this applies it to the webviews.
//!
//! macOS: while WebKit shows the page in its own fullscreen window (element fullscreen), the page
//! is left alone — no bounds, no show or hide, no command bar: wry's `set_bounds` sets a
//! WKWebView's frame in its current superview, which is then WebKit's fullscreen window, so the
//! video would shrink (LJ36). A layout skipped meanwhile is applied once that fullscreen ends.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
/// True while a layout skipped during WebKit's fullscreen waits for it to end (macOS).
static WAITING_FOR_WEBKIT: AtomicBool = AtomicBool::new(false);

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
    if platform::page_in_webkit_fullscreen(&page) {
        after_webkit_fullscreen(app);
        return;
    }
    place(&page, wanted.page);
    if let Some(rect) = wanted.page {
        log_page_screen(window, rect, scale);
    }
}

/// Shows `webview` at `rect`, or hides it. For the page the smoke log gets `page-bounds
/// <x,y,w,h>` (logical, in the window) and `page-visible <bool>`: CI checks that nothing touches
/// the page during WebKit's fullscreen on macOS (LJ36).
fn place<R: Runtime>(webview: &Webview<R>, rect: Option<Rect>) {
    let page = webview.label() == PAGE;
    match rect {
        Some(rect) => {
            let _ = webview.set_bounds(to_tauri(rect));
            let _ = webview.show();
            if page {
                smoke::log(
                    "page-bounds",
                    &format!("{},{},{},{}", rect.x, rect.y, rect.width, rect.height),
                );
                smoke::log("page-visible", "true");
            }
        }
        None => {
            let _ = webview.hide();
            if page {
                smoke::log("page-visible", "false");
            }
        }
    }
}

/// Smoke mode only: where the page is on the screen — `page-screen <x>,<y>,<w>,<h> scale=<s>`,
/// physical pixels from the window's inner position — so CI can click the smoke page's buttons.
fn log_page_screen<R: Runtime>(window: &Window<R>, rect: Rect, scale: f64) {
    if !smoke::enabled() {
        return;
    }
    let Ok(origin) = window.inner_position() else {
        return;
    };
    let physical = |logical: f64| (logical * scale).round();
    smoke::log(
        "page-screen",
        &format!(
            "{},{},{},{} scale={scale}",
            f64::from(origin.x) + physical(rect.x),
            f64::from(origin.y) + physical(rect.y),
            physical(rect.width),
            physical(rect.height),
        ),
    );
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
    if on && (element_fullscreen() || page_in_webkit_fullscreen(app)) {
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

/// macOS: whether WebKit shows the page in its own fullscreen window right now (entering, in or
/// leaving it). Always false on Windows and Linux, whose hooks set `element_fullscreen`.
pub fn page_in_webkit_fullscreen<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_webview(PAGE)
        .is_some_and(|page| platform::page_in_webkit_fullscreen(&page))
}

/// A layout (or a phone-mode switch) was skipped during WebKit's fullscreen: checks every 100 ms
/// until that fullscreen has ended (or the window closed), then applies both once. Only one such
/// check runs, and only while something was skipped.
pub fn after_webkit_fullscreen<R: Runtime>(app: &AppHandle<R>) {
    if WAITING_FOR_WEBKIT.swap(true, Ordering::SeqCst) {
        return;
    }
    smoke::log("layout-deferred", "");
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-after-fullscreen".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(100));
                let (Some(window), Some(page)) =
                    (app.get_window(crate::joan::LABEL), app.get_webview(PAGE))
                else {
                    break;
                };
                if !platform::page_in_webkit_fullscreen(&page) {
                    WAITING_FOR_WEBKIT.store(false, Ordering::SeqCst);
                    smoke::log("layout-resumed", "");
                    apply_layout(&window);
                    crate::phone_mode::apply(&app, true);
                    return;
                }
            }
            WAITING_FOR_WEBKIT.store(false, Ordering::SeqCst);
        });
    if spawned.is_err() {
        WAITING_FOR_WEBKIT.store(false, Ordering::SeqCst);
    }
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
