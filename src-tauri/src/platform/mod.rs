//! Per-OS native hooks installed on the page window's two webviews (bar.rs).

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(windows)]
mod win;

use tauri::{AppHandle, Runtime, Webview, Window};

/// Windows: page keys on the bar's WebView2 controller.
#[cfg(windows)]
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    win::install_bar(bar);
}

/// Windows: page keys and element fullscreen on the page's WebView2 controller.
#[cfg(windows)]
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    win::install_page(page);
}

/// Linux: pack the bar at a fixed height in the window's GtkBox.
#[cfg(target_os = "linux")]
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    linux::install_bar(bar);
}

/// Linux: page keys on the GTK window, element-fullscreen signals on the page.
#[cfg(target_os = "linux")]
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    linux::install_page(page);
}

/// Linux: the command bar's overlay. The bar fills the window's GtkBox and the page widget is
/// hidden, or both go back (see linux.rs).
#[cfg(target_os = "linux")]
pub fn set_overlay<R: Runtime>(bar: &Webview<R>, page: &Webview<R>, on: bool) {
    linux::set_overlay(bar, page, on);
}

/// Windows and macOS place child webviews by bounds, so bar.rs's layout does it all.
#[cfg(not(target_os = "linux"))]
pub fn set_overlay<R: Runtime>(_bar: &Webview<R>, _page: &Webview<R>, _on: bool) {}

/// macOS: the webview's autoresizing mask (`layout::mask_for`, for AppKit's unflipped
/// coordinates), so AppKit resizes it in its own layout pass while the window is dragged.
#[cfg(target_os = "macos")]
pub fn set_autoresizing<R: Runtime>(webview: &Webview<R>, bits: usize) {
    mac::set_autoresizing(webview, bits);
}

/// Windows and Linux: no autoresizing masks (Windows places by bounds, Linux packs a GtkBox).
#[cfg(not(target_os = "macos"))]
pub fn set_autoresizing<R: Runtime>(_webview: &Webview<R>, _bits: usize) {}

/// macOS: the app menu's key equivalents handle page keys (menu.rs), and WebKit shows element
/// fullscreen in its own window, so the bar needs no hiding.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn install_bar<R: Runtime>(_bar: &Webview<R>) {}

/// macOS: nothing to install (see `install_bar`).
#[cfg(not(any(windows, target_os = "linux")))]
pub fn install_page<R: Runtime>(_page: &Webview<R>) {}

/// Whether the window is fullscreen right now, however it got there. On Linux WebKitGTK
/// fullscreens the GTK window itself for a page's fullscreen element, which tao's own state
/// doesn't see, so the GDK window state is read instead. Main thread only.
#[cfg(target_os = "linux")]
pub fn is_fullscreen<R: Runtime>(window: &Window<R>) -> bool {
    linux::is_fullscreen(window)
}

/// Whether the window is fullscreen right now (tao tracks it on Windows and macOS).
#[cfg(not(target_os = "linux"))]
pub fn is_fullscreen<R: Runtime>(window: &Window<R>) -> bool {
    window.is_fullscreen().unwrap_or(false)
}

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
/// Takes effect with the next request; phone_mode.rs reloads the page when it must.
#[cfg(windows)]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    win::set_mobile(page, mobile);
}

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
#[cfg(target_os = "linux")]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    linux::set_mobile(page, mobile);
}

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
#[cfg(target_os = "macos")]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    mac::set_mobile(page, mobile);
}

/// macOS, smoke mode only (CI): logs WebKit's element-fullscreen state (`wk-fullscreen …`) while
/// the page window exists. Nothing elsewhere: Windows and Linux report fullscreen through their
/// hooks.
#[cfg(target_os = "macos")]
pub fn watch_webkit_fullscreen<R: Runtime>(app: &AppHandle<R>) {
    mac::watch_webkit_fullscreen(app);
}

/// Windows and Linux: nothing to watch (their hooks report element fullscreen).
#[cfg(not(target_os = "macos"))]
pub fn watch_webkit_fullscreen<R: Runtime>(_app: &AppHandle<R>) {}

/// macOS: whether the page must be left alone because WebKit shows it in its own fullscreen
/// window — `fullscreenState` isn't "not in fullscreen" (macOS 13+), or the page's WKWebView no
/// longer sits in the Little Joan window (macOS 12's only sign) (fullscreen::page_layout_allowed).
/// Asks the main thread; an unanswered question counts as no.
#[cfg(target_os = "macos")]
pub fn page_in_webkit_fullscreen<R: Runtime>(page: &Webview<R>) -> bool {
    mac::webkit_fullscreen(page)
        .is_some_and(|(state, ours)| !crate::fullscreen::page_layout_allowed(state, ours))
}

/// Windows and Linux: element fullscreen goes through their hooks (`bar::element_fullscreen`).
#[cfg(not(target_os = "macos"))]
pub fn page_in_webkit_fullscreen<R: Runtime>(_page: &Webview<R>) -> bool {
    false
}

/// macOS, smoke mode only: whether the last `wk-fullscreen` line logged wasn't `not`.
#[cfg(target_os = "macos")]
pub fn webkit_fullscreen_logged() -> bool {
    mac::webkit_fullscreen_logged()
}

/// Windows and Linux: WebKit's fullscreen window doesn't exist there.
#[cfg(not(target_os = "macos"))]
pub fn webkit_fullscreen_logged() -> bool {
    false
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(windows)]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    win::write_clipboard(text)
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(target_os = "linux")]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    linux::write_clipboard(text)
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(target_os = "macos")]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    mac::write_clipboard(text)
}
