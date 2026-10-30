//! A summon leaving the page's element fullscreen (LJ19.5, joan/fullscreen.rs): what WebKit says
//! while it waits (macOS), WebKit's own way out, whether the window is still fullscreen, and
//! whether it has the keyboard (the toggle's "focused").

use tauri::{Runtime, Webview, Window};

#[cfg(target_os = "linux")]
use super::linux;
#[cfg(target_os = "macos")]
use super::mac;

/// macOS: what WebKit says while a summon waits for the page's element fullscreen to end
/// (LJ19.5, joan/fullscreen.rs): its state, whether the page is back in the Little Joan window and
/// that window on the Space on screen. Asks the main thread; `None` when it didn't answer.
#[cfg(target_os = "macos")]
pub fn webkit_fullscreen_exit<R: Runtime>(
    page: &Webview<R>,
) -> Option<crate::fullscreen::WebKitExit> {
    mac::webkit_fullscreen_exit(page)
}

/// Windows and Linux: WebKit's fullscreen window doesn't exist there.
#[cfg(not(target_os = "macos"))]
pub fn webkit_fullscreen_exit<R: Runtime>(
    _page: &Webview<R>,
) -> Option<crate::fullscreen::WebKitExit> {
    None
}

/// macOS: WebKit closes the page's element fullscreen itself (WKWebView's
/// `closeAllMediaPresentationsWithCompletionHandler:`) — a summon's fallback when the page didn't
/// leave as asked (LJ19.5).
#[cfg(target_os = "macos")]
pub fn close_media_presentations<R: Runtime>(page: &Webview<R>) {
    mac::close_media_presentations(page);
}

/// Windows and Linux: the page's own exit is all there is.
#[cfg(not(target_os = "macos"))]
pub fn close_media_presentations<R: Runtime>(_page: &Webview<R>) {}

/// Whether the window is fullscreen now, asked from off the main thread (LJ19.5: a summon waiting
/// for the window to leave the fullscreen a page's element gave it). Linux reads the GDK state on
/// the main thread (`is_fullscreen`), waiting at most 500 ms; Windows and macOS ask tao.
#[cfg(target_os = "linux")]
pub fn fullscreen_now<R: Runtime>(window: &Window<R>) -> bool {
    let (sent, answer) = std::sync::mpsc::channel();
    let target = window.clone();
    if window
        .run_on_main_thread(move || {
            let _ = sent.send(linux::is_fullscreen(&target));
        })
        .is_err()
    {
        return false;
    }
    answer
        .recv_timeout(std::time::Duration::from_millis(500))
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
pub fn fullscreen_now<R: Runtime>(window: &Window<R>) -> bool {
    window.is_fullscreen().unwrap_or(false)
}

/// Whether the open window has the keyboard (the toggle's "focused": a press then closes it).
/// Windows: it is the foreground window (`win::foreground`; tao's `is_focused` turns false as soon
/// as a webview inside took the keyboard — CI's LJ19.5 run saw the second press only focus).
/// macOS (the key window) and Linux (GTK's toplevel focus): tao's `is_focused`.
#[cfg(windows)]
pub fn has_keyboard<R: Runtime>(window: &Window<R>) -> bool {
    super::win::foreground(window).unwrap_or_else(|| window.is_focused().unwrap_or(false))
}

#[cfg(not(windows))]
pub fn has_keyboard<R: Runtime>(window: &Window<R>) -> bool {
    window.is_focused().unwrap_or(false)
}
