//! Per-OS native hooks installed on the page window.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(windows)]
mod win;

use tauri::{Runtime, WebviewWindow};

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
