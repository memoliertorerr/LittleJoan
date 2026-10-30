//! The page window ("joan") and its lifecycle: built on demand, its page discarded on every close.
//!
//! The window holds two webviews (bar.rs): the trusted bar (the toolbar on top, or the command bar
//! over the whole window) and the page, built after the bar for each summon (bar/page.rs). It opens
//! with the command bar. A close always discards the page — nothing of a web page survives it — and
//! what stays at rest is the owner's "After closing" setting (LJ19.4, joan/rest.rs): the hidden
//! window and the bar ("Stay ready", the default; for 15 minutes; or nothing — hibernation).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager, Runtime, Window, WindowEvent};

use crate::bar;
use crate::extensions;
use crate::history_store;
use crate::phone_mode;
use crate::platform;
use crate::settings;
use crate::smoke;
use crate::summon_timing::{self, Kind, Request, Source};
use crate::toggle::{self, Action, Command, WindowState};
use crate::tray;

mod fullscreen;
mod rest;
mod window;

pub use rest::{after_close_changed, bar_process_ended, prewarm_after_launch};

pub const LABEL: &str = "joan";

/// Serializes lifecycle changes so two quick shortcut presses can't both create the window.
static LIFECYCLE: Mutex<()> = Mutex::new(());

/// Last fullscreen state seen, for the smoke log only.
static WAS_FULLSCREEN: AtomicBool = AtomicBool::new(false);

/// Little Joan is quitting: nothing new is built (a prewarm, a summon) on the way out.
static QUITTING: AtomicBool = AtomicBool::new(false);

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
    if QUITTING.load(Ordering::SeqCst) && !matches!(command, Command::Hide | Command::Release(_)) {
        return;
    }
    let action = toggle::decide(window_state(app), command);
    // macOS: App Nap stays off while a summon brings the window up (LJ19.1); it ends with this
    // function, once the window is up and focused. Nothing is held at rest.
    let summons = matches!(
        action,
        Action::Create(_)
            | Action::CreateExtension(_)
            | Action::Summon(_)
            | Action::SummonExtension(_)
            | Action::Rebuild(_)
            | Action::RebuildExtension(_)
    );
    let _awake = summons.then(platform::summon_activity).flatten();
    let measure = |kind: Kind, url: bool| {
        if let Some(request) = request.clone() {
            summon_timing::begin(request, kind, rest::mode(app).name(), url);
        }
    };
    let result = match action {
        Action::Create(url) => {
            measure(Kind::Cold, url.is_some());
            rest::create(app, url)
        }
        Action::Summon(url) => {
            measure(Kind::Ready, url.is_some());
            rest::summon(app, url)
        }
        Action::Rebuild(url) => {
            rest::release(app, "bar-gone");
            measure(Kind::Cold, url.is_some());
            rest::create(app, url)
        }
        Action::Focus => focus(app).map(drop),
        // The shortcut while the page is full screen (LJ19.5): out of the fullscreen, then the
        // command bar with the keyboard in its field.
        Action::LeaveFullscreen => focus(app).map(|front| {
            if front {
                bar::focus_bar(app);
            }
        }),
        Action::FocusAndNavigate(url) => focus(app).and_then(|_| bar::navigate(app, url)),
        Action::CreateExtension(id) => {
            measure(Kind::Cold, false);
            extension_window(app, &id, |app| rest::create(app, None))
        }
        Action::RebuildExtension(id) => {
            rest::release(app, "bar-gone");
            measure(Kind::Cold, false);
            extension_window(app, &id, |app| rest::create(app, None))
        }
        Action::SummonExtension(id) => {
            measure(Kind::Ready, false);
            extension_window(app, &id, |app| rest::summon(app, None))
        }
        Action::FocusAndExtension(id) => focus(app).and_then(|_| open_extension(app, &id)),
        Action::Close => rest::close(app),
        Action::Prewarm => rest::prewarm(app),
        Action::Release(why) => {
            rest::release(app, why.name());
            Ok(())
        }
        Action::Resize { width, height } => resize(app, width, height),
        Action::Nothing => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("little-joan: window action failed: {err}");
    }
}

/// `--extension <id>` while nothing is on screen: the window comes up (`show`), then the panel
/// opens once the bar's script runs (LJ38; a ready bar's ran long ago). An unknown extension
/// opens nothing.
fn extension_window<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    show: impl FnOnce(&AppHandle<R>) -> tauri::Result<()>,
) -> tauri::Result<()> {
    if extensions::registry::current(app).get(id).is_some() {
        show(app).map(|()| {
            extensions::session::open_when_bar_ready(app, id, extensions::session::Target::First);
        })
    } else {
        // Nothing is kept of the registry it read (a window reads it again when it opens).
        extensions::registry::forget();
        missing_extension(id);
        Ok(())
    }
}

/// The longest Quit waits for the window to close on macOS (a close, then the ready window's
/// release: both unload their pages first).
#[cfg(target_os = "macos")]
const QUIT_WAIT: std::time::Duration = std::time::Duration::from_millis(2500);

/// Quits Little Joan: the tray's "Quit Little Joan", `--quit` (forwarded by single-instance) and,
/// on macOS, Cmd+Q.
///
/// macOS: `app.exit` ends the process without unloading the webviews, and CI saw the next launch
/// on the same WebKit data store never load a page (LJ20). So an open window is closed first,
/// through the same path as every close, and a ready one destroyed (both unload their pages),
/// waiting at most 2.5 s. Windows and Linux kept everything without it, so they exit at once.
pub fn quit<R: Runtime>(app: &AppHandle<R>) {
    smoke::log("quit", "");
    QUITTING.store(true, Ordering::SeqCst);
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
                        handle(&closing, Command::Release(toggle::Why::Quit));
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

/// Whether Little Joan is quitting (nothing new is built then).
fn quitting() -> bool {
    QUITTING.load(Ordering::SeqCst)
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

/// Whether the window is open (on screen, or minimized) — not resting, not hibernating.
pub fn is_open<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.get_window(LABEL).is_some() && !rest::at_rest()
}

fn window_state<R: Runtime>(app: &AppHandle<R>) -> WindowState {
    match app.get_window(LABEL) {
        None => WindowState::Hibernating,
        Some(_) if rest::at_rest() => WindowState::Ready {
            usable: rest::bar_usable(),
        },
        Some(window) => WindowState::Open {
            focused: window.is_focused().unwrap_or(false),
            page_fullscreen: bar::element_fullscreen() || bar::page_in_webkit_fullscreen(app),
        },
    }
}

/// Brings the open window to the front with the keyboard; returns whether it came with the page
/// out of any fullscreen (false: there is no window, or the page didn't leave its fullscreen in
/// time — then nothing may take the keyboard from it). A page in element fullscreen leaves
/// it first (LJ19.5, joan/fullscreen.rs) — on macOS nothing is ordered until WebKit has put the
/// page back in this window and the Space with it is on screen again: ordered front during
/// WebKit's fullscreen, the window would join WebKit's full-screen Space (LJ19.2's collection
/// behaviour), and that Space closing with it inside can make the Dock refuse every app's full
/// screen for minutes (LJ19.4-fix4, the Landmine). If the page never leaves, macOS only brings the
/// app to the front (the full-screen page with the keyboard), as LJ19.4-fix4 did.
fn focus<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<bool> {
    let Some(window) = app.get_window(LABEL) else {
        return Ok(false);
    };
    let left = fullscreen::leave(app, &window);
    if left == (fullscreen::Leave::GaveUp { webkit: true }) {
        platform::activate_app(&window);
        smoke::log("summon-fullscreen", "activated");
        platform::report_front(&window);
        return Ok(false);
    }
    // (Windows and Linux: a page that kept its fullscreen keeps it in this window, which comes to
    // the front.)
    window.unminimize()?;
    window.show()?;
    platform::bring_to_front(&window)?;
    platform::report_front(&window);
    Ok(left == fullscreen::Leave::Done)
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

/// Window events for "joan": route every close through `handle` (so it is closed the same way as
/// with the shortcut), keep the bar and the page laid out, and log the window's end.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != LABEL {
        return;
    }
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            // The close button, Cmd+W / Alt+F4: close through the same path as the shortcut
            // (remember the size, discard the page; the window rests or goes).
            api.prevent_close();
            dispatch(window.app_handle(), Command::Hide, Source::App);
        }
        WindowEvent::Destroyed => {
            tray::set_page_items_enabled(window.app_handle(), false);
            // A summon still waiting for its last mark is logged as it got (LJ19.1).
            summon_timing::end();
            rest::destroyed();
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

/// A new window, or one summoned from ready: not fullscreen.
fn forget_fullscreen() {
    WAS_FULLSCREEN.store(false, Ordering::Relaxed);
}
