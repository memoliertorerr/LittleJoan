//! A sign-in popup's window (LJ47–LJ48; bar/popup.rs builds it): where it goes and whose it is,
//! WebKitGTK's late decision on it, Little Joan's keys in it, and the keyboard going back to the
//! page when one the user was in closes.

use tauri::{AppHandle, Manager, Runtime, WebviewWindowBuilder, Window, WindowEvent};

use super::closing::{close, mark_closing};
use super::tracked;
use crate::joan;
use crate::keymap::KeyAction;
use crate::layout::Rect;
use crate::platform;
use crate::popups::{self, KeyRoute};
use crate::smoke;

/// The Little Joan window's frame and its screen's work area, in logical px.
pub(super) fn frames<R: Runtime>(window: &Window<R>) -> Option<(Rect, Option<Rect>)> {
    let scale = window.scale_factor().ok()?;
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let over = Rect {
        x: f64::from(position.x) / scale,
        y: f64::from(position.y) / scale,
        width: f64::from(size.width) / scale,
        height: f64::from(size.height) / scale,
    };
    let screen = window.current_monitor().ok().flatten().map(|monitor| {
        let work = monitor.work_area();
        Rect {
            x: f64::from(work.position.x) / scale,
            y: f64::from(work.position.y) / scale,
            width: f64::from(work.size.width) / scale,
            height: f64::from(work.size.height) / scale,
        }
    });
    Some((over, screen))
}

/// The popup stays above the Little Joan window: its owner (Windows), its child (macOS), transient
/// for it (GTK).
#[cfg(windows)]
pub(super) fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.owner_raw(joan.hwnd()?))
}

#[cfg(target_os = "macos")]
pub(super) fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.parent_raw(joan.ns_window()?))
}

#[cfg(target_os = "linux")]
pub(super) fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.transient_for_raw(&joan.gtk_window()?))
}

/// WebKitGTK's `ready-to-show` (main thread): `asked`, the size the request asked for, if any
/// (`popups::gtk_asked`) — so it shows, goes to the browser, or goes nowhere.
#[cfg(target_os = "linux")]
pub fn settle<R: Runtime>(app: &AppHandle<R>, label: &str, asked: Option<(f64, f64)>) {
    use popups::Decision;
    let Some((target, early)) = tracked()
        .list
        .iter_mut()
        .find(|p| p.label == label && p.pending && p.closing.is_none())
        .map(|p| {
            p.pending = false;
            (p.target.clone(), p.early.take())
        })
    else {
        return;
    };
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    match popups::settle(&target, asked.is_some()) {
        Decision::InApp | Decision::Pending => {
            let (width, height) = popups::clamp_size(asked);
            let _ = window.set_size(tauri::LogicalSize::new(width, height));
            match app.get_window(joan::LABEL).and_then(|joan| frames(&joan)) {
                Some((over, screen)) => {
                    let (x, y) = popups::place(over, (width, height), screen);
                    let _ = window.set_position(tauri::LogicalPosition::new(x, y));
                }
                None => {
                    let _ = window.center();
                }
            }
            let _ = window.show();
            let _ = window.set_focus();
            smoke::log("popup-opened", &popups::title_for(&target));
            if let Some(url) = early {
                let _ = window.navigate(url);
            }
        }
        decision => {
            smoke::log("popup-settled", "unsized");
            mark_closing(label, "unshown", false);
            super::closing::destroy_later(app, label);
            match decision {
                Decision::Browser => super::elsewhere(app, target),
                _ => smoke::log("new-window-refused", target.as_str()),
            }
        }
    }
}

/// WebKitGTK: a popup still undecided `popups::GTK_SETTLE_WAIT` after it was built (no
/// `ready-to-show`) is settled as a request that asked for no size.
#[cfg(target_os = "linux")]
pub fn settle_late<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let undecided = tracked()
        .list
        .iter()
        .any(|p| p.label == label && p.pending && p.closing.is_none());
    if undecided {
        smoke::log("popup-settled", "timeout");
        settle(app, label, None);
    }
}

/// One of Little Joan's keys in the popup `label` (LJ48; platform/: WebView2's
/// `AcceleratorKeyPressed`, the popup's GTK key hook): Close closes the popup, nothing else acts.
/// Returns whether the key was Little Joan's to take (the page doesn't see it then).
pub fn key_pressed<R: Runtime>(app: &AppHandle<R>, label: &str, action: KeyAction) -> bool {
    match popups::route_key(true, action) {
        KeyRoute::ClosePopup => {
            close(app, label, "user");
            true
        }
        KeyRoute::Nothing | KeyRoute::Page(_) => false,
    }
}

/// Whether the popup `label` has the keyboard (Windows: it is the foreground window — tao's own
/// focus goes false while its webview has the keyboard, LJ19.5).
pub(super) fn has_keyboard<R: Runtime>(app: &AppHandle<R>, label: &str) -> bool {
    app.get_webview_window(label)
        .is_some_and(|window| platform::has_keyboard(&window.as_ref().window()))
}

/// The popup that has the keyboard, if one does (macOS's menu: its Close Window closes the key
/// window; the shortcut with a popup in front, LJ48).
pub fn focused<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let labels: Vec<String> = tracked()
        .list
        .iter()
        .filter(|p| p.closing.is_none() && !p.pending)
        .map(|p| p.label.clone())
        .collect();
    labels.into_iter().find(|label| has_keyboard(app, label))
}

/// Window events of popups (lib.rs): the close button marks it the user's; once it is gone it
/// leaves the list, and — if it had the keyboard — the keyboard goes back to the page.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let label = window.label();
    if !popups::is_popup_label(label) {
        return;
    }
    match event {
        WindowEvent::CloseRequested { .. } => {
            mark_closing(label, "user", platform::has_keyboard(window));
        }
        WindowEvent::Destroyed => {
            platform::forget_popup(label);
            let gone = {
                let mut state = tracked();
                let at = state.list.iter().position(|p| p.label == label);
                at.map(|at| state.list.remove(at))
            };
            let Some(gone) = gone else {
                return;
            };
            if gone.closing.is_none() {
                smoke::log("popup-closed", "user");
            }
            let why = gone.closing.unwrap_or("user");
            if matches!(why, "user" | "by-page") && gone.had_keyboard {
                give_back_keyboard(window.app_handle());
            }
        }
        _ => {}
    }
}

/// A popup with the keyboard closed while the Little Joan window is open: the page gets it again.
fn give_back_keyboard<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if !joan::is_open(&app) {
            return;
        }
        if let Some(window) = app.get_window(joan::LABEL) {
            let _ = window.set_focus();
        }
        crate::bar::focus_page(&app);
    });
}
