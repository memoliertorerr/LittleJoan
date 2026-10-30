//! How the page window is laid out right now (bar.rs's view state): the 40 px toolbar above the
//! page, the command bar's overlay covering the window (the page hidden), or element fullscreen
//! (the page alone; it always wins). layout.rs does the math; this applies it to the webviews.
//!
//! macOS: while WebKit shows the page in its own fullscreen window (element fullscreen), the page
//! is left alone — no bounds, no show or hide, no command bar: wry's `set_bounds` sets a
//! WKWebView's frame in its current superview, which is then WebKit's fullscreen window, so the
//! video would shrink (LJ36). A layout skipped meanwhile is applied once that fullscreen ends.
//!
//! Only what changed is applied (LJ39, `layout::changes`): a resize sets the bounds that changed
//! and shows or hides nothing; a mode change shows or hides what it must. On macOS each mode also
//! sets the webviews' autoresizing masks (`layout::mask_for`), so AppKit resizes them in its own
//! layout pass while the window is dragged — tao's `Resized` comes a frame late; the layout it
//! triggers then sets the same frames AppKit already gave them, so a late or off-thread layout
//! can never leave a view stale. Never Tauri's `auto_resize`: it scales child webviews
//! proportionally, so the 40 px bar would grow.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, Window};

use super::{BAR, PAGE};
use crate::layout::{self, Layout, Mode, Rect, ViewChange};
use crate::platform;
use crate::smoke;

/// True while the page shows an element fullscreen (the bar is hidden meanwhile).
static ELEMENT_FULLSCREEN: AtomicBool = AtomicBool::new(false);
/// True while the bar covers the window (the command bar) and the page is hidden. A new window
/// starts with it.
static OVERLAY: AtomicBool = AtomicBool::new(false);
/// True while a layout skipped during WebKit's fullscreen waits for it to end (macOS).
static WAITING_FOR_WEBKIT: AtomicBool = AtomicBool::new(false);
/// What was last applied to this window's webviews, diffed against each new layout. `None`: a
/// new window, or not known any more (WebKit's fullscreen had the page).
static APPLIED: Mutex<Option<Applied>> = Mutex::new(None);

/// A layout as it was applied: at which scale (a scale change applies everything again — the
/// engines turn logical bounds into pixels at the scale of the moment) and in which mode (a mode
/// change sets the macOS masks again).
#[derive(Debug, Clone, Copy)]
struct Applied {
    layout: Layout,
    scale: f64,
    mode: Mode,
}

fn applied() -> std::sync::MutexGuard<'static, Option<Applied>> {
    APPLIED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A new window: no fullscreen, and the command bar open (every summon opens it). Nothing is
/// known about its webviews yet, so the first layout applies everything.
pub(super) fn new_window() {
    ELEMENT_FULLSCREEN.store(false, Ordering::Relaxed);
    OVERLAY.store(true, Ordering::Relaxed);
    *applied() = None;
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

/// Positions the bar and the page for the window's current size and mode — only what changed
/// since the last layout (`layout::changes`). The smoke log gets `bounds-set <n>`: how many
/// webviews' bounds this layout set (CI: at most 2 per resize).
pub fn apply_layout<R: Runtime>(window: &Window<R>) {
    let app = window.app_handle();
    let Some(bar) = app.get_webview(BAR) else {
        return;
    };
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let mode = mode();
    // inner_size is physical; the bounds below are logical (layout.rs).
    let wanted = layout::for_physical_size(size.width, size.height, scale, mode);
    // Read, then released before any engine call: the fullscreen check below may wait for the
    // main thread, which may be in here too.
    let previous = applied().filter(|known| known.scale == scale);
    let change = layout::changes(previous.as_ref().map(|known| &known.layout), &wanted);
    let masks = layout::mask_for(mode).filter(|_| previous.is_none_or(|known| known.mode != mode));
    apply(&bar, change.bar);
    if let Some(masks) = masks {
        platform::set_autoresizing(&bar, masks.bar);
    }
    let Some(page) = app.get_webview(PAGE) else {
        // No page webview yet (it comes after the bar, LJ19.4): its part counts as hidden, so the
        // page added later gets its bounds and shows when a layout says so (`page_added`).
        *applied() = Some(Applied {
            layout: Layout {
                page: None,
                ..wanted
            },
            scale,
            mode,
        });
        smoke::log(
            "bounds-set",
            &usize::from(change.bar.bounds.is_some()).to_string(),
        );
        return;
    };
    if platform::page_in_webkit_fullscreen(&page) {
        // WebKit has the page: it is left alone, and laid out again in full afterwards.
        *applied() = None;
        smoke::log(
            "bounds-set",
            &usize::from(change.bar.bounds.is_some()).to_string(),
        );
        after_webkit_fullscreen(app);
        return;
    }
    apply(&page, change.page);
    if let Some(masks) = masks {
        platform::set_autoresizing(&page, masks.page);
    }
    *applied() = Some(Applied {
        layout: wanted,
        scale,
        mode,
    });
    smoke::log("bounds-set", &change.bounds_set().to_string());
    if let Some(rect) = wanted.page {
        log_page_screen(window, rect, scale);
    }
}

/// Applies one webview's part of a layout change: its new bounds, then showing or hiding it. For
/// the page the smoke log gets `page-bounds <x,y,w,h>` (logical, in the window) and `page-visible
/// <bool>` for each: CI checks that nothing touches the page during WebKit's fullscreen on macOS
/// (LJ36), and that a resize shows nothing (LJ39).
fn apply<R: Runtime>(webview: &Webview<R>, change: ViewChange) {
    let page = webview.label() == PAGE;
    if let Some(rect) = change.bounds {
        let _ = webview.set_bounds(to_tauri(rect));
        if page {
            smoke::log(
                "page-bounds",
                &format!("{},{},{},{}", rect.x, rect.y, rect.width, rect.height),
            );
        }
    }
    if let Some(visible) = change.visible {
        let _ = if visible {
            webview.show()
        } else {
            webview.hide()
        };
        if page {
            smoke::log("page-visible", if visible { "true" } else { "false" });
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
    if let Some(bar) = app.get_webview(BAR) {
        platform::set_overlay(&bar, app.get_webview(PAGE).as_ref(), on);
        tell_overlay(&bar, on);
    }
    log_overlay();
    true
}

/// A page webview was just added to the window (bar/page.rs, LJ19.4), hidden: it is packed for the
/// current view (Linux), gets its autoresizing mask (macOS), and the layout shows and places it if
/// the view has a page (the toolbar view; the command bar's overlay keeps it hidden).
pub(super) fn page_added<R: Runtime>(window: &Window<R>, page: &Webview<R>) {
    if let Some(known) = applied().as_mut() {
        known.layout.page = None;
    }
    if let Some(bar) = window.app_handle().get_webview(BAR) {
        platform::set_overlay(&bar, Some(page), overlay());
    }
    if let Some(masks) = layout::mask_for(mode()) {
        platform::set_autoresizing(page, masks.page);
    }
    apply_layout(window);
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
                    // WebKit moved the page and gave it back: everything again, masks included.
                    *applied() = None;
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
    // Logged first: a summon waiting for the change (joan/fullscreen.rs) logs after it.
    smoke::log("element-fullscreen", if on { "true" } else { "false" });
    ELEMENT_FULLSCREEN.store(on, Ordering::Relaxed);
    if on {
        change_overlay(app, false);
    }
    if let Some(window) = app.get_window(crate::joan::LABEL) {
        apply_layout(&window);
    }
}
