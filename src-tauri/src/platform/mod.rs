//! Per-OS native hooks installed on the page window's two webviews (bar.rs).

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(windows)]
mod win;

use tauri::{Runtime, Webview, Window};

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
