//! The page window ("joan") and its lifecycle: created on demand, destroyed on close.
//!
//! The window holds two webviews (bar.rs): the URL bar on top and the page below. Nothing here
//! keeps a webview alive while hibernating: `Destroy` (and the user closing the window) drops
//! the window and both webviews, so the engine's renderer processes exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use tauri::window::WindowBuilder;
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};
use url::Url;

use crate::bar;
use crate::placement;
use crate::platform;
use crate::settings::{self, MIN_HEIGHT, MIN_WIDTH};
use crate::smoke;
use crate::state;
use crate::toggle::{self, Action, Command, WindowState};
use crate::tray;

pub const LABEL: &str = "joan";

/// Serializes lifecycle changes so two quick shortcut presses can't both create the window.
static LIFECYCLE: Mutex<()> = Mutex::new(());

/// Last fullscreen state seen, for the smoke log only.
static WAS_FULLSCREEN: AtomicBool = AtomicBool::new(false);

/// Handles `command` off the main thread. Building webviews from the main thread can deadlock
/// on Windows, and shortcut / single-instance / menu callbacks may run there.
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
        Action::FocusAndNavigate(url) => focus(app).and_then(|()| bar::navigate(app, url)),
        Action::Destroy => destroy(app),
        Action::Nothing => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("little-joan: window action failed: {err}");
    }
}

fn window_state<R: Runtime>(app: &AppHandle<R>) -> WindowState {
    match app.get_window(LABEL) {
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
    let window = WindowBuilder::new(app, LABEL)
        .title("Little Joan")
        .inner_size(width, height)
        .min_inner_size(f64::from(MIN_WIDTH), f64::from(MIN_HEIGHT))
        .resizable(true)
        .visible(false)
        .build()?;
    WAS_FULLSCREEN.store(false, Ordering::Relaxed);
    // Element fullscreen (video): WebView2 needs platform::install_page's handler; WKWebView
    // (wry enables elementFullscreenEnabled) and WebKitGTK (fullscreens the toplevel by
    // default) handle it themselves. See fullscreen.rs; the bar hides meanwhile (bar.rs).
    if let Err(err) = bar::add_webviews(&window, width, height, webview_data_dir) {
        let _ = window.destroy();
        return Err(err);
    }
    placement::place(app, &window, width, height);
    window.show()?;
    window.set_focus()?;
    bar::apply_layout(&window);
    bar::focus_bar(app);
    smoke::log("window-created", "");
    if let Some(url) = url {
        bar::navigate(app, url)?;
    }
    bar::watch_url(app);
    Ok(())
}

fn focus<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_window(LABEL) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

fn destroy<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    match app.get_window(LABEL) {
        Some(window) => {
            remember_size(app, &window);
            release_page(app, &window);
            window.destroy()
        }
        None => Ok(()),
    }
}

/// macOS: the CI smoke test (LJ9) saw WKWebView's processes outlive a destroyed window by
/// minutes unless their page was unloaded first, which would also let a video's audio keep
/// playing. So the window is hidden at once, both webviews (page and URL bar) are unloaded, and
/// WebKit gets a moment before the window is destroyed. Runs off the main thread (inside
/// `handle`), so the short sleep blocks nothing the user sees.
#[cfg(target_os = "macos")]
fn release_page<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
    let _ = window.hide();
    if let Ok(blank) = Url::parse("about:blank") {
        for webview in bar::webviews(app) {
            let _ = webview.navigate(blank.clone());
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
}

/// WebView2 and WebKitGTK release a page's processes when the window is destroyed (measured in
/// CI), so nothing extra is needed.
#[cfg(not(target_os = "macos"))]
fn release_page<R: Runtime>(_app: &AppHandle<R>, _window: &Window<R>) {}

/// Window events for "joan": route every close through `handle` (so it is hibernated the same
/// way as with the shortcut), keep the bar and the page laid out, and log hibernation.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != LABEL {
        return;
    }
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            // The close button, Cmd+W / Alt+F4: hibernate through the same path as the
            // shortcut (remember the size, release the page, destroy).
            api.prevent_close();
            dispatch(window.app_handle(), Command::Hide);
        }
        WindowEvent::Destroyed => {
            tray::set_page_items_enabled(window.app_handle(), false);
            smoke::log("hibernated", "");
        }
        WindowEvent::Resized(_) => {
            bar::apply_layout(window);
            if smoke::enabled() {
                let fullscreen = platform::is_fullscreen(window);
                if WAS_FULLSCREEN.swap(fullscreen, Ordering::Relaxed) != fullscreen {
                    smoke::log("fullscreen", if fullscreen { "true" } else { "false" });
                }
            }
        }
        WindowEvent::ScaleFactorChanged { .. } => bar::apply_layout(window),
        _ => {}
    }
}

/// Saves the window's logical size as the size for the next summon. A fullscreen, maximized,
/// or minimized window says nothing about the size the user wants, so it is skipped.
fn remember_size<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
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
