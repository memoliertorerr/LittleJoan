//! Per-OS native hooks installed on the page window's two webviews (bar.rs).

#[cfg(target_os = "linux")]
mod linux;
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
