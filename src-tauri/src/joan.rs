//! The page window ("joan") and its lifecycle: created on demand, destroyed on close.
//!
//! The window holds two webviews (bar.rs): the bar (the toolbar on top, or the command bar over
//! the whole window) and the page. It opens with the command bar. Nothing here keeps a webview
//! alive while hibernating: `Destroy` (and the user closing the window) drops the window and
//! both webviews, so the engine's renderer processes exit.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use tauri::window::WindowBuilder;
use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};
use url::Url;

use crate::bar;
use crate::cookies::{self, SessionCookies};
use crate::extensions;
use crate::history_store;
use crate::phone_mode;
use crate::placement;
use crate::platform;
use crate::settings::{self, MIN_HEIGHT, MIN_WIDTH};
use crate::smoke;
use crate::state;
use crate::summon_timing::{self, Mark, Request, Source};
use crate::toggle::{self, Action, Command, WindowState};
use crate::tray;

pub const LABEL: &str = "joan";

/// Serializes lifecycle changes so two quick shortcut presses can't both create the window.
static LIFECYCLE: Mutex<()> = Mutex::new(());

/// Last fullscreen state seen, for the smoke log only.
static WAS_FULLSCREEN: AtomicBool = AtomicBool::new(false);

/// Handles `command`, which came from `source`, off the main thread. Building webviews from the
/// main thread can deadlock on Windows, and shortcut / single-instance / menu callbacks may run
/// there. The request's time is taken first, for the smoke log's summon timing (LJ19.1).
pub fn dispatch<R: Runtime>(app: &AppHandle<R>, command: Command, source: Source) {
    let request = Request::now(source);
    let app = app.clone();
    tauri::async_runtime::spawn(async move { run(&app, command, Some(request)) });
}

/// Applies `command` to the current window state. Must not run on the main thread.
pub fn handle<R: Runtime>(app: &AppHandle<R>, command: Command) {
    run(app, command, None);
}

fn run<R: Runtime>(app: &AppHandle<R>, command: Command, request: Option<Request>) {
    let _lifecycle = LIFECYCLE.lock().unwrap_or_else(PoisonError::into_inner);
    let action = toggle::decide(window_state(app), command);
    // macOS: App Nap stays off while a summon builds the window (LJ19.1); it ends with this
    // function, once the window is up and focused. Nothing is held at rest.
    let _awake = matches!(action, Action::Create(_) | Action::CreateExtension(_))
        .then(platform::summon_activity)
        .flatten();
    let result = match action {
        Action::Create(url) => {
            if let Some(request) = request {
                summon_timing::begin(request, url.is_some());
            }
            create(app, url)
        }
        Action::Focus => focus(app),
        Action::FocusAndNavigate(url) => focus(app).and_then(|()| bar::navigate(app, url)),
        Action::CreateExtension(id) => {
            if extensions::registry::current(app).get(&id).is_some() {
                // A new window: the panel opens once the bar's script runs (LJ38).
                create(app, None).map(|()| {
                    extensions::session::open_when_bar_ready(
                        app,
                        &id,
                        extensions::session::Target::First,
                    );
                })
            } else {
                // An unknown extension opens nothing (and nothing is kept while hibernating).
                extensions::registry::forget();
                missing_extension(&id);
                Ok(())
            }
        }
        Action::FocusAndExtension(id) => focus(app).and_then(|()| open_extension(app, &id)),
        Action::Destroy => destroy(app),
        Action::Resize { width, height } => resize(app, width, height),
        Action::Nothing => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("little-joan: window action failed: {err}");
    }
}

/// The longest Quit waits for the window to close on macOS.
#[cfg(target_os = "macos")]
const QUIT_WAIT: std::time::Duration = std::time::Duration::from_millis(1500);

/// Quits Little Joan: the tray's "Quit Little Joan", `--quit` (forwarded by single-instance) and,
/// on macOS, Cmd+Q.
///
/// macOS: `app.exit` ends the process without unloading the webviews, and CI saw the next launch
/// on the same WebKit data store never load a page (LJ20). So an open window is closed first,
/// through the same path as every close (`destroy`: the page is unloaded), waiting at most
/// 1.5 s. Windows and Linux kept everything without it, so they exit at once.
pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    smoke::log("quit", "");
    // Visits not saved yet would be lost with the process (the window may still be open).
    history_store::close(app);
    #[cfg(target_os = "macos")]
    {
        let app = app.clone();
        let spawned = std::thread::Builder::new()
            .name("little-joan-quit".into())
            .spawn(move || {
                let (done, closed) = std::sync::mpsc::channel();
                let closing = app.clone();
                // `handle` must not run on the main thread, and may take a while: its own thread.
                let worker = std::thread::Builder::new()
                    .name("little-joan-quit-close".into())
                    .spawn(move || {
                        handle(&closing, Command::Hide);
                        let _ = done.send(());
                    });
                if worker.is_ok() && closed.recv_timeout(QUIT_WAIT).is_err() {
                    eprintln!("little-joan: the window didn't close in time; quitting anyway");
                }
                app.exit(0);
            });
        if spawned.is_ok() {
            return;
        }
    }
    app.exit(0);
}

/// Opens the extension `id`'s first command in the open window (`--extension`).
fn open_extension<R: Runtime>(app: &AppHandle<R>, id: &str) -> tauri::Result<()> {
    if let Err(err) = extensions::session::open(app, id, extensions::session::Target::First) {
        eprintln!("little-joan: could not open the extension: {err}");
        missing_extension(id);
    }
    Ok(())
}

fn missing_extension(id: &str) {
    smoke::log("extension-missing", id);
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
    let window = window_builder(app)?
        .title("Little Joan")
        .inner_size(width, height)
        .min_inner_size(f64::from(MIN_WIDTH), f64::from(MIN_HEIGHT))
        .resizable(true)
        .visible(false)
        .build()?;
    summon_timing::mark(Mark::WindowBuilt);
    WAS_FULLSCREEN.store(false, Ordering::Relaxed);
    // A new window has no extension open yet.
    extensions::session::reset();
    // Element fullscreen (video): WebView2 needs platform::install_page's handler; WKWebView
    // (wry enables elementFullscreenEnabled) and WebKitGTK (fullscreens the toplevel by
    // default) handle it themselves. See fullscreen.rs; the bar hides meanwhile (bar.rs).
    // History loads on its own thread meanwhile (LJ19.1); the command bar's first suggestions,
    // and every other use of it, wait for that.
    history_store::open(app);
    // The page's user agent starts unknown in a new window (set below, before any web page).
    phone_mode::reset();
    if let Err(err) = bar::add_webviews(&window, width, height, webview_data_dir) {
        history_store::close(app);
        let _ = window.destroy();
        return Err(err);
    }
    summon_timing::mark(Mark::WebviewsAdded);
    placement::place(app, &window, width, height);
    window.show()?;
    window.set_focus()?;
    summon_timing::mark(Mark::Shown);
    bar::apply_layout(&window);
    // The window was built with the command bar open (bar.rs); this puts the keyboard in it.
    bar::focus_bar(app);
    smoke::log("window-created", "");
    bar::log_overlay();
    // Once the window is on screen, and still before any web page loads (the start page under
    // the command bar needs neither): the kept session cookies (Windows) and the page's user
    // agent for its width (phone_mode.rs). On Windows both wait for WebView2, which kept the
    // window hidden some 50 ms longer before LJ19.1.
    restore_session_cookies(app);
    summon_timing::mark(Mark::CookiesRestored);
    phone_mode::apply(app, false);
    summon_timing::mark(Mark::AgentSet);
    if let Some(url) = url {
        bar::navigate(app, url)?;
    }
    summon_timing::mark(Mark::Created);
    bar::watch_url(app);
    // CI only (smoke log): WebKit's fullscreen state on macOS (LJ36).
    platform::watch_webkit_fullscreen(app);
    Ok(())
}

/// Where the traffic lights sit on macOS (logical px: the close button's left edge, and the
/// title bar area's height below the buttons — tao's inset): inside the 40 px toolbar, which
/// leaves them about 78 px on its left.
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHTS: (f64, f64) = (14.0, 20.0);

/// macOS: the traffic lights sit inside the toolbar. `TitleBarStyle::Overlay` with a hidden
/// title extends the content view under a transparent title bar, so the bar webview starts at
/// y = 0 beneath the buttons (LJ10.1 used `Transparent` to keep content below the title bar;
/// here that is the point). Tauri's `window::WindowBuilder` has no setter for the buttons'
/// position, but its config does, so the builder starts from a `WindowConfig`.
#[cfg(target_os = "macos")]
fn window_builder<R: Runtime>(
    app: &AppHandle<R>,
) -> tauri::Result<WindowBuilder<'_, R, AppHandle<R>>> {
    let mut config = tauri::utils::config::WindowConfig::default();
    config.label = LABEL.to_owned();
    config.title_bar_style = tauri::TitleBarStyle::Overlay;
    config.hidden_title = true;
    config.traffic_light_position = Some(tauri::utils::config::LogicalPosition {
        x: TRAFFIC_LIGHTS.0,
        y: TRAFFIC_LIGHTS.1,
    });
    WindowBuilder::from_config(app, &config)
}

/// Windows and Linux keep their native title bar (custom chrome is Later).
#[cfg(not(target_os = "macos"))]
fn window_builder<R: Runtime>(
    app: &AppHandle<R>,
) -> tauri::Result<WindowBuilder<'_, R, AppHandle<R>>> {
    Ok(WindowBuilder::new(app, LABEL))
}

fn focus<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_window(LABEL) {
        window.unminimize()?;
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

/// `--size`: the open window becomes width × height logical pixels (within the allowed range).
fn resize<R: Runtime>(app: &AppHandle<R>, width: u32, height: u32) -> tauri::Result<()> {
    if let Some(window) = app.get_window(LABEL) {
        let wanted =
            settings::Settings::default().with_window_size(f64::from(width), f64::from(height));
        window.set_size(tauri::LogicalSize::new(
            f64::from(wanted.window_width),
            f64::from(wanted.window_height),
        ))?;
    }
    Ok(())
}

fn destroy<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    match app.get_window(LABEL) {
        Some(window) => {
            remember_size(app, &window);
            history_store::close(app);
            keep_session_cookies(app);
            // A download under way ends with the window (Ground Rule 9).
            bar::download::window_closed(app);
            release_page(app, &window);
            let destroyed = window.destroy();
            extensions::forget_window();
            destroyed
        }
        None => Ok(()),
    }
}

/// Before the page is released: keeps its session cookies in memory for the next window, where
/// the engine would drop them with this one (WebView2; cookies.rs). A failed read keeps nothing.
fn keep_session_cookies<R: Runtime>(app: &AppHandle<R>) {
    if cookies::ENGINE_KEEPS_SESSION_COOKIES {
        return;
    }
    let Some(page) = bar::page(app) else {
        return;
    };
    // Off the main thread (inside `handle`), as WebView2's cookie read requires.
    let kept = match page.cookies() {
        Ok(all) => SessionCookies::from_all(all),
        Err(err) => {
            eprintln!("little-joan: could not read the page's cookies: {err}");
            SessionCookies::default()
        }
    };
    smoke::log("session-cookies-kept", &kept.len().to_string());
    state::with(app, |s| s.session_cookies = kept);
}

/// A new window: sets the kept session cookies again before any web page loads.
fn restore_session_cookies<R: Runtime>(app: &AppHandle<R>) {
    if cookies::ENGINE_KEEPS_SESSION_COOKIES {
        return;
    }
    let kept = state::with(app, |s| s.session_cookies.clone());
    let Some(page) = bar::page(app) else {
        return;
    };
    if kept.is_empty() {
        return;
    }
    for cookie in kept.cookies() {
        if let Err(err) = page.set_cookie(cookie.clone()) {
            eprintln!("little-joan: could not set a kept cookie again: {err}");
        }
    }
    // WebView2's cookie manager answers in order, so once this read returns the cookies above
    // are in place for the first navigation.
    let _ = page.cookies();
    smoke::log("session-cookies-restored", &kept.len().to_string());
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
            dispatch(window.app_handle(), Command::Hide, Source::App);
        }
        WindowEvent::Destroyed => {
            tray::set_page_items_enabled(window.app_handle(), false);
            // A summon still waiting for its last mark is logged as it got (LJ19.1).
            summon_timing::end();
            smoke::log("hibernated", "");
        }
        WindowEvent::Resized(size) => {
            log_during_webkit_fullscreen("resized", || format!("{}x{}", size.width, size.height));
            bar::apply_layout(window);
            phone_mode::on_resized(window.app_handle());
            if smoke::enabled() {
                let fullscreen = platform::is_fullscreen(window);
                if WAS_FULLSCREEN.swap(fullscreen, Ordering::Relaxed) != fullscreen {
                    smoke::log("fullscreen", if fullscreen { "true" } else { "false" });
                }
            }
        }
        WindowEvent::ScaleFactorChanged { .. } => bar::apply_layout(window),
        WindowEvent::Focused(focused) => {
            log_during_webkit_fullscreen("focused", || focused.to_string());
        }
        WindowEvent::Moved(position) => {
            log_during_webkit_fullscreen("moved", || format!("{},{}", position.x, position.y));
        }
        _ => {}
    }
}

/// Smoke mode, macOS: what happens to the Little Joan window while WebKit shows the page in its
/// own fullscreen window (`wk-window <event> <detail>`; CI reads it next to `wk-fullscreen`).
fn log_during_webkit_fullscreen(event: &str, detail: impl FnOnce() -> String) {
    if platform::webkit_fullscreen_logged() {
        smoke::log("wk-window", &format!("{event} {}", detail()));
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
