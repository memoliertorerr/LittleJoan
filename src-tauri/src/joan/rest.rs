//! What a summon builds and what a close leaves (LJ19.4, the owner's "After closing" setting):
//!
//! - **Cold** (`create`): the hidden window and the bar, placed, shown on the current Space, in
//!   front with the keyboard in the command bar's field; the page webview after the bar's script
//!   runs (bar/page.rs), or at once for a URL.
//! - **Close** (`close`): the size is remembered, history saved, and the page discarded — its
//!   session cookies kept (Windows), downloads under way ended, macOS unloading it first —, always.
//!   Then, as the setting says: **Stay ready** hides the window and keeps only it and the bar, the
//!   bar back at its summon state (`bar::rest`); **Stay ready for 15 minutes** does that and
//!   destroys both after 15 idle minutes; **Hibernate** destroys the window (nothing at rest).
//! - **Summon from ready** (`summon`): the hidden window is placed by the same rules, shown on the
//!   current Space, in front, the bar's field focused; the bar starts over for the summon; the page
//!   is built once the bar is on screen.
//! - **Prewarm** (`prewarm`): a few seconds after the launch (never during the first run's welcome,
//!   never while quitting), and when the setting switches to a ready mode, the hidden window and
//!   the bar are built ahead of the first summon. Nothing shows, nothing takes the keyboard.
//! - **A gone bar:** if the engine says the bar's web process ended while the window rests
//!   (`bar_process_ended`), the next summon builds everything anew (`toggle`'s Rebuild) — never a
//!   blank window. On screen, the bar is reloaded instead.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, Window};
use url::Url;

use super::window::{normal_size, remember_size, size_of, window_builder};
use super::{LABEL, forget_fullscreen, quitting};
use crate::bar;
use crate::extensions;
use crate::history_store;
use crate::placement;
use crate::platform;
use crate::settings::{AfterClose, MIN_HEIGHT, MIN_WIDTH};
use crate::smoke;
use crate::state;
use crate::summon_timing::{self, Mark, Source};
use crate::toggle::{Command, Why};
use crate::tray;

/// The window exists but rests: hidden, the bar alive, no page (toggle's `Ready`).
static AT_REST: AtomicBool = AtomicBool::new(false);
/// The engine said the resting bar's web process ended: the next summon builds anew.
static BAR_GONE: AtomicBool = AtomicBool::new(false);
/// Bumped by every summon, close and setting change: a "Stay ready for 15 minutes" timer started
/// before it does nothing.
static REST: AtomicU64 = AtomicU64::new(0);
/// Why the window is being destroyed (the smoke log's line once it is gone).
static DESTROYING: Mutex<Option<String>> = Mutex::new(None);

/// How long after the launch the ready window is built (after the tray and the shortcut).
const PREWARM_AFTER: Duration = Duration::from_secs(3);

/// Whether the window rests (it exists, hidden, with the bar and no page).
pub(super) fn at_rest() -> bool {
    AT_REST.load(Ordering::SeqCst)
}

/// Whether a resting window's bar can be shown (its web process didn't end).
pub(super) fn bar_usable() -> bool {
    !BAR_GONE.load(Ordering::SeqCst)
}

/// The "After closing" setting.
pub(super) fn mode<R: Runtime>(app: &AppHandle<R>) -> AfterClose {
    state::with(app, |s| s.settings.after_close)
}

/// A summon that builds the window: window + bar, shown with the command bar; the page after.
pub(super) fn create<R: Runtime>(app: &AppHandle<R>, url: Option<Url>) -> tauri::Result<()> {
    let window = build(app, true)?;
    let (width, height) = size_of(app);
    show(app, &window, width, height, "cold")?;
    open_page(app, url, true)
}

/// A summon from ready: the window is there with its bar; shown, then the page.
pub(super) fn summon<R: Runtime>(app: &AppHandle<R>, url: Option<Url>) -> tauri::Result<()> {
    let Some(window) = app.get_window(LABEL) else {
        return create(app, url);
    };
    AT_REST.store(false, Ordering::SeqCst);
    REST.fetch_add(1, Ordering::SeqCst);
    bar::page_summon();
    forget_fullscreen();
    // A new summon: no extension open, the registry read again (a developer's edits show up),
    // history loaded on its own thread (LJ19.1).
    extensions::session::reset();
    extensions::registry::forget();
    history_store::open(app);
    if let Some(bar) = bar::bar(app) {
        platform::bar_at_rest(&bar, false);
    }
    let theme = state::with(app, |s| s.shown_theme());
    smoke::log("theme-summon", &crate::commands::theme_summary(&theme));
    bar::summoned(app, &theme, smoke::enabled());
    platform::take_focus_back(app);
    let (width, height) = size_of(app);
    show(app, &window, width, height, "ready")?;
    open_page(app, url, false)
}

/// Places, shows and focuses the window with the command bar open; logs `window-created <kind>`.
fn show<R: Runtime>(
    app: &AppHandle<R>,
    window: &Window<R>,
    width: f64,
    height: f64,
    kind: &str,
) -> tauri::Result<()> {
    placement::place(app, window, width, height);
    // macOS: on the Space the user is on, a full-screen app's too, before it is shown (LJ19.2).
    platform::join_active_space(window);
    window.show()?;
    // In front with the keyboard (on Linux with a fresh X server timestamp; LJ19.2).
    platform::bring_to_front(window)?;
    summon_timing::mark(Mark::Shown);
    bar::apply_layout(window);
    // The window opens with the command bar (bar.rs); this puts the keyboard in its field.
    bar::focus_bar(app);
    smoke::log("window-created", kind);
    bar::log_overlay();
    platform::report_front(window);
    Ok(())
}

/// The page webview: at once with a URL (then the navigation), else once the bar is on screen.
fn open_page<R: Runtime>(app: &AppHandle<R>, url: Option<Url>, cold: bool) -> tauri::Result<()> {
    match url {
        Some(url) => bar::navigate(app, url)?,
        None => bar::page::build_after_bar(app, cold),
    }
    summon_timing::mark(Mark::Created);
    Ok(())
}

/// Builds the hidden window with the bar (the command bar open) and, for a summon (`history`),
/// history loading on its own thread. Used by a cold summon and a prewarm.
fn build<R: Runtime>(app: &AppHandle<R>, history: bool) -> tauri::Result<Window<R>> {
    let (width, height) = size_of(app);
    let (webview_data_dir, theme) =
        state::with(app, |s| (s.webview_data_dir.clone(), s.shown_theme()));
    let window = window_builder(app)?
        .title("Little Joan")
        .inner_size(width, height)
        .min_inner_size(f64::from(MIN_WIDTH), f64::from(MIN_HEIGHT))
        .resizable(true)
        .visible(false)
        .build()?;
    summon_timing::mark(Mark::WindowBuilt);
    // The theme's own colour under everything before the bar paints (LJ19.4: no flash of the
    // default theme, or of white, while the window opens).
    let _ = window.set_background_color(Some(bar::window_background(&window, &theme)));
    AT_REST.store(false, Ordering::SeqCst);
    BAR_GONE.store(false, Ordering::SeqCst);
    REST.fetch_add(1, Ordering::SeqCst);
    bar::page_summon();
    forget_fullscreen();
    // A new window has no extension open yet.
    extensions::session::reset();
    // Element fullscreen (video): WebView2 needs platform::install_page's handler; WKWebView
    // (wry enables elementFullscreenEnabled) and WebKitGTK (fullscreens the toplevel by
    // default) handle it themselves. See fullscreen.rs; the bar hides meanwhile (bar.rs).
    // History loads on its own thread meanwhile (LJ19.1); the command bar's first suggestions,
    // and every other use of it, wait for that. A prewarm holds none (nothing extra at rest).
    if history {
        history_store::open(app);
    }
    if let Err(err) = bar::add_bar(&window, width, height, webview_data_dir) {
        if history {
            history_store::close(app);
        }
        let _ = window.destroy();
        return Err(err);
    }
    summon_timing::mark(Mark::WebviewsAdded);
    Ok(window)
}

/// Builds the ready window ahead of a summon (Stay ready): hidden, the bar loaded, no page, no
/// history held. Nothing shows.
pub(super) fn prewarm<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if quitting() || !mode(app).stays_ready() {
        return Ok(());
    }
    let window = build(app, false)?;
    rest_now(app, &window);
    smoke::log("prewarmed", mode(app).name());
    Ok(())
}

/// The window is at rest now: hidden, the bar at its summon state, its memory target low
/// (Windows), and Stay ready for 15 minutes' timer started.
fn rest_now<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
    AT_REST.store(true, Ordering::SeqCst);
    let rest = REST.fetch_add(1, Ordering::SeqCst) + 1;
    // Nothing of the page stays: not even its title on the hidden window.
    let _ = window.set_title("Little Joan");
    bar::rest(app);
    if let Some(bar) = bar::bar(app) {
        platform::bar_at_rest(&bar, true);
    }
    let _ = window.hide();
    if let Some(after) = rest_for(mode(app)) {
        expire_after(app, rest, after);
    }
}

/// How long a close (or a prewarm) stays ready: `None` = until Little Joan quits. CI may shorten
/// Stay ready for 15 minutes with `LITTLEJOAN_SMOKE_READY_MS` (smoke tests only).
fn rest_for(mode: AfterClose) -> Option<Duration> {
    let after = mode.ready_for()?;
    let smoke_ms = smoke::enabled()
        .then(|| std::env::var("LITTLEJOAN_SMOKE_READY_MS").ok())
        .flatten()
        .and_then(|ms| ms.parse::<u64>().ok());
    Some(smoke_ms.map_or(after, Duration::from_millis))
}

/// Stay ready for 15 minutes: `after` the rest numbered `rest` began, if nothing happened since,
/// the window goes (hibernation).
fn expire_after<R: Runtime>(app: &AppHandle<R>, rest: u64, after: Duration) {
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-ready-timer".into())
        .spawn(move || {
            std::thread::sleep(after);
            if REST.load(Ordering::SeqCst) == rest {
                smoke::log("release", "timer");
                super::dispatch(&app, Command::Release(Why::Timer), Source::App);
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not start the ready timer: {err}");
    }
}

/// Closes the window: the page is always discarded; the window rests or goes as "After closing"
/// says (the module's docs).
pub(super) fn close<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(window) = app.get_window(LABEL) else {
        return Ok(());
    };
    let mode = mode(app);
    // CI's "nothing of the page survives" check times the page's last ping from here (LJ19.4).
    smoke::log("closing", mode.name());
    remember_size(app, &window);
    history_store::close(app);
    if !mode.stays_ready() {
        release_window(app, &window, "close");
        return Ok(());
    }
    // Out of sight at once; then the page goes. A window a video's fullscreen (or the user)
    // left fullscreen or maximized rests at its size, as a new one would open.
    let _ = window.hide();
    normal_size(app, &window);
    bar::page::close_page(app);
    // Nothing of the page's extensions is kept; the bar's painted theme is (panels read it).
    extensions::session::reset();
    extensions::registry::forget();
    forget_fullscreen();
    rest_now(app, &window);
    let settings_shown = app
        .get_window(crate::settings_window::LABEL)
        .is_some_and(|w| w.is_visible().unwrap_or(false));
    platform::give_back_focus(app, settings_shown);
    tray::set_page_items_enabled(app, false);
    // A summon still waiting for its last mark is logged as it got (LJ19.1).
    summon_timing::end();
    smoke::log("hibernated", mode.name());
    Ok(())
}

/// Destroys the ready window (Stay ready for 15 minutes' timer, the setting switching to
/// Hibernate, a gone bar, Quit): back to hibernation.
pub(super) fn release<R: Runtime>(app: &AppHandle<R>, why: &str) {
    if let Some(window) = app.get_window(LABEL) {
        release_window(app, &window, why);
    }
}

/// Destroys the window and both webviews (`why` for the smoke log): a download under way ends, the
/// page's session cookies are kept (Windows), and macOS unloads both webviews first.
fn release_window<R: Runtime>(app: &AppHandle<R>, window: &Window<R>, why: &str) {
    *DESTROYING.lock().unwrap_or_else(PoisonError::into_inner) = Some(why.to_owned());
    REST.fetch_add(1, Ordering::SeqCst);
    bar::page_summon();
    let _ = window.hide();
    bar::page::close_page(app);
    unload_window(app);
    let _ = window.destroy();
    extensions::forget_window();
    // The window goes on the main thread: a summon right after (a rebuild, a quick press) must
    // find it gone, or it would build a second "joan" (at most 2 s).
    for _ in 0..200 {
        if app.get_window(LABEL).is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// macOS: the CI smoke test (LJ9) saw WKWebView's processes outlive a destroyed window by minutes
/// unless their page was unloaded first, which would also let a video's audio keep playing. So
/// (the window hidden and the page closed, which unloads it) the bar is unloaded too, and WebKit
/// gets a moment before the window is destroyed. Off the main thread (inside `handle`), so the
/// short sleep blocks nothing the user sees.
#[cfg(target_os = "macos")]
fn unload_window<R: Runtime>(app: &AppHandle<R>) {
    if let (Ok(blank), Some(bar)) = (Url::parse("about:blank"), bar::bar(app)) {
        let _ = bar.navigate(blank);
    }
    std::thread::sleep(Duration::from_millis(400));
}

/// WebView2 and WebKitGTK release a window's processes when it is destroyed (measured in CI).
#[cfg(not(target_os = "macos"))]
fn unload_window<R: Runtime>(_app: &AppHandle<R>) {}

/// The window is gone (its `Destroyed` event): nothing rests now. A close logs `hibernated` (as
/// before LJ19.4), anything else `released <why>`.
pub(super) fn destroyed() {
    AT_REST.store(false, Ordering::SeqCst);
    BAR_GONE.store(false, Ordering::SeqCst);
    let why = DESTROYING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take();
    match why.as_deref() {
        None | Some("close") => smoke::log("hibernated", ""),
        Some(why) => smoke::log("released", why),
    }
}

/// The engine says the bar's web process ended (crashed, killed, out of memory; `why` for the log).
/// Resting, the next summon builds everything anew; on screen, the bar loads again. Main thread
/// (the engines' callbacks): nothing here waits.
pub fn bar_process_ended<R: Runtime>(app: &AppHandle<R>, why: &str) {
    if at_rest() {
        BAR_GONE.store(true, Ordering::SeqCst);
        smoke::log("bar-gone", why);
    } else if let Some(bar) = bar::bar(app) {
        smoke::log("bar-gone", &format!("{why} reload"));
        let _ = bar.reload();
    }
}

/// After the launch (lib.rs): with Stay ready, the ready window is built `PREWARM_AFTER` later —
/// unless this launch shows the first run's welcome, or Little Joan quits meanwhile.
pub fn prewarm_after_launch<R: Runtime>(app: &AppHandle<R>, welcome: bool) {
    if welcome || !mode(app).stays_ready() {
        return;
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-prewarm".into())
        .spawn(move || {
            std::thread::sleep(PREWARM_AFTER);
            super::dispatch(&app, Command::Prewarm, Source::App);
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not plan the ready window: {err}");
    }
}

/// "After closing" changed (Settings' Save; settings_live.rs): at once — to Hibernate, a resting
/// window goes; to a ready mode, a hibernating Little Joan prewarms; Stay ready ↔ for 15 minutes
/// restarts or stops the timer of a resting window. An open window is left alone: its close
/// follows the new setting.
pub fn after_close_changed<R: Runtime>(app: &AppHandle<R>, before: AfterClose, now: AfterClose) {
    if before == now {
        return;
    }
    smoke::log("after-close", now.name());
    let rest = REST.fetch_add(1, Ordering::SeqCst) + 1;
    if !now.stays_ready() {
        smoke::log("release", "setting");
        super::dispatch(app, Command::Release(Why::Setting), Source::App);
    } else if !before.stays_ready() {
        super::dispatch(app, Command::Prewarm, Source::App);
    } else if at_rest()
        && let Some(after) = rest_for(now)
    {
        expire_after(app, rest, after);
    }
}
