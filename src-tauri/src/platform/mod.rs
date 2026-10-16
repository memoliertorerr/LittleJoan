//! Per-OS native hooks installed on the page window.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod win;

use tauri::{Runtime, WebviewWindow, Window};

/// Installs the page-key handler (see keys.rs) on a newly created page window.
#[cfg(windows)]
pub fn install<R: Runtime>(window: &WebviewWindow<R>) {
    win::install(window);
}

/// Installs the page-key handler (see keys.rs) on a newly created page window.
#[cfg(target_os = "linux")]
pub fn install<R: Runtime>(window: &WebviewWindow<R>) {
    linux::install(window);
}

/// On macOS the app menu's key equivalents handle page keys (menu.rs).
#[cfg(not(any(windows, target_os = "linux")))]
pub fn install<R: Runtime>(_window: &WebviewWindow<R>) {}

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
