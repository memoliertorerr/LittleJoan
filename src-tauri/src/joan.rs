//! The page window ("joan") and its lifecycle: created on demand, destroyed on close.
//!
//! Nothing here keeps a webview alive while hibernating: `Destroy` (and the user closing the
//! window) drops the window and its webview, so the engine's renderer processes exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use tauri::webview::{NewWindowResponse, PageLoadEvent};
use tauri::{
    AppHandle, Manager, PhysicalPosition, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    Window, WindowEvent,
};
use url::Url;

use crate::navigation;
use crate::platform;
use crate::settings::{self, MIN_HEIGHT, MIN_WIDTH};
use crate::smoke;
use crate::state;
use crate::toggle::{self, Action, Command, WindowState};

pub const LABEL: &str = "joan";

/// Serializes lifecycle changes so two quick shortcut presses can't both create the window.
static LIFECYCLE: Mutex<()> = Mutex::new(());

/// Last fullscreen state seen, for the smoke log only.
static WAS_FULLSCREEN: AtomicBool = AtomicBool::new(false);

/// Handles `command` off the main thread. Building a webview window from the main thread can
/// deadlock on Windows, and shortcut / single-instance / menu callbacks may run there.
pub fn dispatch<R: Runtime>(app: &AppHandle<R>, command: Command) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { handle(&app, command) });
}

/// Applies `command` to the current window state. Must not run on the main thread.
pub fn handle<R: Runtime>(app: &AppHandle<R>, command: Command) {
    let _lifecycle = LIFECYCLE.lock().unwrap_or_else(PoisonError::into_inner);
    let action = toggle::decide(window_state(app), command);
    let result = match action {
        Action::Create(url) => create(app, url),
        Action::Focus => focus(app),
        Action::FocusAndNavigate(url) => focus(app).and_then(|()| navigate(app, url)),
        Action::Destroy => destroy(app),
        Action::Nothing => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("little-joan: window action failed: {err}");
    }
}

fn window_state<R: Runtime>(app: &AppHandle<R>) -> WindowState {
    match app.get_webview_window(LABEL) {
        None => WindowState::Hibernating,
        Some(window) => WindowState::Open {
            focused: window.is_focused().unwrap_or(false),
        },
    }
}

fn create<R: Runtime>(app: &AppHandle<R>, url: Option<Url>) -> tauri::Result<()> {
    let (width, height, webview_data_dir) = state::with(app, |s| {
        (
            f64::from(s.settings.window_width),
            f64::from(s.settings.window_height),
            s.webview_data_dir.clone(),
        )
    });
    let mut builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("Little Joan")
        .inner_size(width, height)
        .min_inner_size(f64::from(MIN_WIDTH), f64::from(MIN_HEIGHT))
        .resizable(true)
        .visible(false)
        // Web pages only: no file:, javascript:, or app-launching schemes (navigation.rs).
        .on_navigation(navigation::is_allowed_navigation)
        .on_document_title_changed(|window, title| {
            let title = title.trim();
            let title = if title.is_empty() {
                "Little Joan"
            } else {
                title
            };
            let _ = window.set_title(title);
            smoke::log("title", title);
        })
        .on_page_load(|_window, payload| {
            if payload.event() == PageLoadEvent::Finished {
                smoke::log("page-loaded", payload.url().as_str());
            }
        })
        // target=_blank and window.open go to the default browser (web URLs only).
        .on_new_window(|url, _features| {
            if navigation::is_external_openable(&url)
                && let Err(err) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>)
            {
                eprintln!("little-joan: could not open the link in the default browser: {err}");
            }
            NewWindowResponse::Deny
        });
    if let Some(dir) = webview_data_dir {
        builder = builder.data_directory(dir);
    }
    // Element fullscreen (video): WebView2 needs platform::install's handler; WKWebView (wry
    // enables elementFullscreenEnabled) and WebKitGTK (fullscreens the toplevel by default)
    // handle it themselves. See fullscreen.rs.
    let window = builder.build()?;
    WAS_FULLSCREEN.store(false, Ordering::Relaxed);
    platform::install(&window);
    place(app, &window, width, height);
    window.show()?;
    window.set_focus()?;
    smoke::log("window-created", "");
    if let Some(url) = url {
        window.navigate(url)?;
    }
    Ok(())
}

fn focus<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

fn navigate<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    match app.get_webview_window(LABEL) {
        Some(window) => window.navigate(url),
        None => Ok(()),
    }
}

fn destroy<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    match app.get_webview_window(LABEL) {
        Some(window) => {
            remember_size(app, &window);
            window.destroy()
        }
        None => Ok(()),
    }
}

/// Window events for "joan": remember the size when the user closes it, and log hibernation.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != LABEL {
        return;
    }
    match event {
        WindowEvent::CloseRequested { .. } => {
            let app = window.app_handle();
            if let Some(webview_window) = app.get_webview_window(LABEL) {
                remember_size(app, &webview_window);
            }
        }
        WindowEvent::Destroyed => smoke::log("hibernated", ""),
        WindowEvent::Resized(_) if smoke::enabled() => {
            let fullscreen = window.is_fullscreen().unwrap_or(false);
            if WAS_FULLSCREEN.swap(fullscreen, Ordering::Relaxed) != fullscreen {
                smoke::log("fullscreen", if fullscreen { "true" } else { "false" });
            }
        }
        _ => {}
    }
}

/// Saves the window's logical size as the size for the next summon. A fullscreen, maximized,
/// or minimized window says nothing about the size the user wants, so it is skipped.
fn remember_size<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let unusual = window.is_fullscreen().unwrap_or(true)
        || window.is_maximized().unwrap_or(true)
        || window.is_minimized().unwrap_or(true);
    if unusual {
        return;
    }
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let logical = size.to_logical::<f64>(scale);
    let changed = state::with(app, |s| {
        let updated = s
            .settings
            .clone()
            .with_window_size(logical.width, logical.height);
        if updated == s.settings {
            return None;
        }
        s.settings = updated.clone();
        Some((s.data_dir.clone(), updated))
    });
    if let Some((dir, updated)) = changed
        && let Err(err) = settings::save(&dir, &updated)
    {
        eprintln!("little-joan: could not save the window size: {err}");
    }
}

/// Centres the new window in the work area of the monitor under the mouse cursor (falling back
/// to the primary monitor, then to the window manager's centre).
fn place<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>, width: f64, height: f64) {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        let _ = window.center();
        return;
    };
    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let area = Rect {
        x: work.position.x,
        y: work.position.y,
        width: work.size.width,
        height: work.size.height,
    };
    let (x, y) = centered_in(
        area,
        (to_physical(width, scale), to_physical(height, scale)),
    );
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn to_physical(logical: f64, scale: f64) -> u32 {
    // Window sizes are clamped to MAX_SIZE in settings, so this never exceeds u32.
    (logical * scale).round().clamp(0.0, f64::from(u32::MAX)) as u32
}

/// A monitor work area in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Top-left position that centres a window of `size` in `area`; a window larger than the area
/// is aligned to its top-left corner so the title bar stays reachable.
pub fn centered_in(area: Rect, size: (u32, u32)) -> (i32, i32) {
    let offset = |available: u32, length: u32| {
        i32::try_from(available.saturating_sub(length) / 2).unwrap_or(0)
    };
    (
        area.x.saturating_add(offset(area.width, size.0)),
        area.y.saturating_add(offset(area.height, size.1)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centres_inside_the_work_area() {
        let area = Rect {
            x: 0,
            y: 25,
            width: 1440,
            height: 875,
        };
        assert_eq!(centered_in(area, (480, 640)), (480, 142));
    }

    #[test]
    fn works_on_secondary_monitors_left_of_and_above_the_primary() {
        let area = Rect {
            x: -1920,
            y: -200,
            width: 1920,
            height: 1040,
        };
        assert_eq!(centered_in(area, (960, 640)), (-1440, 0));
    }

    #[test]
    fn oversized_windows_align_to_the_top_left() {
        let area = Rect {
            x: 100,
            y: 50,
            width: 800,
            height: 600,
        };
        assert_eq!(centered_in(area, (1000, 700)), (100, 50));
        assert_eq!(centered_in(area, (500, 900)), (250, 50));
    }

    #[test]
    fn logical_to_physical_rounds() {
        assert_eq!(to_physical(480.0, 1.25), 600);
        assert_eq!(to_physical(333.0, 1.5), 500);
        assert_eq!(to_physical(-3.0, 2.0), 0);
    }
}
