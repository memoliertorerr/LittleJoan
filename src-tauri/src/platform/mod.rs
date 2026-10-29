//! Per-OS native hooks installed on the page window's two webviews (bar.rs), the Linux portal
//! that holds the summon shortcut in a Wayland session (portal.rs, LJ17), where a summoned
//! window shows up — the current Space, in front, over another app's full-screen window (LJ19.2,
//! spaces.rs) —, and the ready window at rest (LJ19.4): the bar's engine process ending, Windows'
//! memory target while hidden, macOS giving the keyboard back when the window hides.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac;
#[cfg(target_os = "linux")]
mod portal;
#[cfg(windows)]
mod win;

use std::path::PathBuf;
use std::sync::OnceLock;
use std::thread::ThreadId;
use std::time::Duration;

use tauri::{AppHandle, Runtime, Webview, Window};

use crate::smoke;
use crate::spaces::Front;

/// Linux: asks the XDG GlobalShortcuts portal for the summon shortcut (a Wayland session, LJ17);
/// it answers later, through shortcut::portal_answered.
#[cfg(target_os = "linux")]
pub fn ask_portal<R: Runtime>(app: &AppHandle<R>) -> bool {
    portal::start(app);
    true
}

/// Windows and macOS have no portal to ask: global-hotkey holds the shortcut.
#[cfg(not(target_os = "linux"))]
pub fn ask_portal<R: Runtime>(_app: &AppHandle<R>) -> bool {
    false
}

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

/// Linux: the command bar's overlay. The bar fills the window's GtkBox and the page widget (if
/// the window has one yet) is hidden, or both go back (see linux.rs).
#[cfg(target_os = "linux")]
pub fn set_overlay<R: Runtime>(bar: &Webview<R>, page: Option<&Webview<R>>, on: bool) {
    linux::set_overlay(bar, page, on);
}

/// Windows and macOS place child webviews by bounds, so bar.rs's layout does it all.
#[cfg(not(target_os = "linux"))]
pub fn set_overlay<R: Runtime>(_bar: &Webview<R>, _page: Option<&Webview<R>>, _on: bool) {}

/// macOS: the webview's autoresizing mask (`layout::mask_for`, for AppKit's unflipped
/// coordinates), so AppKit resizes it in its own layout pass while the window is dragged.
#[cfg(target_os = "macos")]
pub fn set_autoresizing<R: Runtime>(webview: &Webview<R>, bits: usize) {
    mac::set_autoresizing(webview, bits);
}

/// Windows and Linux: no autoresizing masks (Windows places by bounds, Linux packs a GtkBox).
#[cfg(not(target_os = "macos"))]
pub fn set_autoresizing<R: Runtime>(_webview: &Webview<R>, _bits: usize) {}

/// macOS: the app menu's key equivalents handle page keys (menu.rs), and WebKit shows element
/// fullscreen in its own window, so the bar needs no hiding.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn install_bar<R: Runtime>(_bar: &Webview<R>) {}

/// macOS: the app menu handles page keys and WebKit element fullscreen (see `install_bar`); the
/// page's downloads are followed so closing the window can cancel them.
#[cfg(not(any(windows, target_os = "linux")))]
pub fn install_page<R: Runtime>(_page: &Webview<R>) {
    mac::follow_downloads();
}

/// Linux: cancels the page's downloads under way (the window is closing). WebKitGTK keeps its
/// web context — and so its downloads — for the app's lifetime; it removes a cancelled
/// download's partial file itself.
#[cfg(target_os = "linux")]
pub fn cancel_downloads<R: Runtime>(app: &AppHandle<R>, _partial: Vec<PathBuf>) {
    linux::cancel_downloads(app);
}

/// macOS: cancels the WKDownloads (they would go on in WebKit's network process after the window
/// closed, CI showed) and removes the partial files of those under way (`partial`).
#[cfg(target_os = "macos")]
pub fn cancel_downloads<R: Runtime>(app: &AppHandle<R>, partial: Vec<PathBuf>) {
    mac::cancel_downloads(app, partial);
}

/// Windows: cancels the page's noted downloads (LJ19.4: the window's WebView2 environment, which
/// used to take them along when it closed, now outlives the page while Little Joan stays ready).
#[cfg(windows)]
pub fn cancel_downloads<R: Runtime>(app: &AppHandle<R>, _partial: Vec<PathBuf>) {
    win::cancel_downloads(app);
}

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

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
/// Takes effect with the next request; phone_mode.rs reloads the page when it must. Windows: with
/// client hints, through the DevTools protocol (LJ40); it waits for the engine's answer.
#[cfg(windows)]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    win::set_mobile(page, mobile);
}

/// Windows: whether the page already fits its viewport (CDP's `Page.getLayoutMetrics`), so a
/// reload for the phone agent can be skipped; `None` when the engine didn't say. Off the main
/// thread only (it waits for the engine).
#[cfg(windows)]
pub fn page_fits<R: Runtime>(page: &Webview<R>) -> Option<bool> {
    win::page_fits(page)
}

/// macOS and Linux: measuring the page would need a script in it (Ground Rule 10): never.
#[cfg(not(windows))]
pub fn page_fits<R: Runtime>(_page: &Webview<R>) -> Option<bool> {
    None
}

/// The main thread (set up in lib.rs).
static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

/// Remembers the main thread (setup), where nothing may wait for the engine's answers or for a
/// task queued on the main thread (Windows: WebView2's).
pub fn note_main_thread() {
    let _ = MAIN_THREAD.set(std::thread::current().id());
    #[cfg(windows)]
    win::note_main_thread();
}

/// Whether this is the main thread (false before setup noted it).
pub fn on_main_thread() -> bool {
    MAIN_THREAD.get() == Some(&std::thread::current().id())
}

/// Windows: the bar's WebView2 at rest (LJ19.4) — `rest`: the window hides, the bar's controller is
/// made invisible and its memory target low (`ICoreWebView2_19`), so WebView2 can trim what it
/// holds; or back to normal and visible before the window shows. macOS and Linux: nothing (no
/// public API for it).
#[cfg(windows)]
pub fn bar_at_rest<R: Runtime>(bar: &Webview<R>, rest: bool) {
    win::bar_at_rest(bar, rest);
}

#[cfg(not(windows))]
pub fn bar_at_rest<R: Runtime>(_bar: &Webview<R>, _rest: bool) {}

/// macOS: the window hid while Little Joan stays ready (LJ19.4): an accessory app with no window on
/// screen gives up the keyboard (`NSApp.hide`), so the app the user came from has it again, as a
/// destroyed window does. Not while another Little Joan window (Settings) is on screen. Windows and
/// Linux hand the focus on themselves when a window hides.
#[cfg(target_os = "macos")]
pub fn give_back_focus<R: Runtime>(app: &AppHandle<R>, other_window_shown: bool) {
    if !other_window_shown {
        let _ = app.hide();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn give_back_focus<R: Runtime>(_app: &AppHandle<R>, _other_window_shown: bool) {}

/// macOS: before a ready window shows again, the app is unhidden (`give_back_focus` hid it).
#[cfg(target_os = "macos")]
pub fn take_focus_back<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.show();
}

#[cfg(not(target_os = "macos"))]
pub fn take_focus_back<R: Runtime>(_app: &AppHandle<R>) {}

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
#[cfg(target_os = "linux")]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    linux::set_mobile(page, mobile);
}

/// Gives the page webview a phone user agent (`mobile`) or the engine's own one (phone.rs).
#[cfg(target_os = "macos")]
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    mac::set_mobile(page, mobile);
}

/// macOS's own shortcuts (LJ44, read-only; system_shortcuts.rs): the parsed
/// `com.apple.symbolichotkeys` domain. Nothing on Windows and Linux (the curated table alone).
#[cfg(target_os = "macos")]
pub fn system_shortcuts() -> Vec<crate::system_shortcuts::SystemShortcut> {
    mac::symbolic_hotkeys()
        .map(|domain| crate::system_shortcuts::parse(&domain))
        .unwrap_or_default()
}

#[cfg(not(target_os = "macos"))]
pub fn system_shortcuts() -> Vec<crate::system_shortcuts::SystemShortcut> {
    Vec::new()
}

/// macOS, smoke mode only (CI): logs WebKit's element-fullscreen state (`wk-fullscreen …`) while
/// the page window exists. Nothing elsewhere: Windows and Linux report fullscreen through their
/// hooks.
#[cfg(target_os = "macos")]
pub fn watch_webkit_fullscreen<R: Runtime>(app: &AppHandle<R>) {
    mac::watch_webkit_fullscreen(app);
}

/// Windows and Linux: nothing to watch (their hooks report element fullscreen).
#[cfg(not(target_os = "macos"))]
pub fn watch_webkit_fullscreen<R: Runtime>(_app: &AppHandle<R>) {}

/// macOS: whether the page must be left alone because WebKit shows it in its own fullscreen
/// window — `fullscreenState` isn't "not in fullscreen" (macOS 13+), or the page's WKWebView no
/// longer sits in the Little Joan window (macOS 12's only sign) (fullscreen::page_layout_allowed).
/// Asks the main thread; an unanswered question counts as no.
#[cfg(target_os = "macos")]
pub fn page_in_webkit_fullscreen<R: Runtime>(page: &Webview<R>) -> bool {
    mac::webkit_fullscreen(page)
        .is_some_and(|(state, ours)| !crate::fullscreen::page_layout_allowed(state, ours))
}

/// Windows and Linux: element fullscreen goes through their hooks (`bar::element_fullscreen`).
#[cfg(not(target_os = "macos"))]
pub fn page_in_webkit_fullscreen<R: Runtime>(_page: &Webview<R>) -> bool {
    false
}

/// macOS, smoke mode only: whether the last `wk-fullscreen` line logged wasn't `not`.
#[cfg(target_os = "macos")]
pub fn webkit_fullscreen_logged() -> bool {
    mac::webkit_fullscreen_logged()
}

/// Windows and Linux: WebKit's fullscreen window doesn't exist there.
#[cfg(not(target_os = "macos"))]
pub fn webkit_fullscreen_logged() -> bool {
    false
}

/// macOS: a summon's NSProcessInfo activity — App Nap stays off from the request until the
/// window is up (LJ19.1); dropping what this returns ends it. Never held at rest.
#[cfg(target_os = "macos")]
pub fn summon_activity() -> Option<mac::SummonActivity> {
    mac::SummonActivity::begin()
}

/// Windows and Linux have no App Nap: nothing to hold.
#[cfg(not(target_os = "macos"))]
pub fn summon_activity() -> Option<()> {
    None
}

/// macOS, for the smoke log: App Nap's state of the process now (LJ19.1).
#[cfg(target_os = "macos")]
pub fn nap_state() -> Option<String> {
    mac::nap_state()
}

/// Windows and Linux: no App Nap.
#[cfg(not(target_os = "macos"))]
pub fn nap_state() -> Option<String> {
    None
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(windows)]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    win::write_clipboard(text)
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(target_os = "linux")]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    linux::write_clipboard(text)
}

/// Puts `text` on the system clipboard (littleJoan.clipboard.writeText). Main thread only.
#[cfg(target_os = "macos")]
pub fn write_clipboard(text: &str) -> Result<(), String> {
    mac::write_clipboard(text)
}

/// macOS: before a window is first shown, its collection behaviour joins it to the Space the user
/// is on and lets it show over another app's full-screen window (LJ19.2, mac/spaces.rs; smoke log
/// `window-spaces …`). Queued on the main thread ahead of the caller's `show()`.
#[cfg(target_os = "macos")]
pub fn join_active_space<R: Runtime>(window: &Window<R>) {
    mac::join_active_space(window);
}

/// Windows and Linux open a new window on the current virtual desktop / workspace, where the
/// summon raises it.
#[cfg(not(target_os = "macos"))]
pub fn join_active_space<R: Runtime>(_window: &Window<R>) {}

/// Linux: raises and focuses a window being shown, with a fresh X server timestamp on X11, so the
/// window manager doesn't refuse it as focus stealing (linux/front.rs, LJ19.2).
#[cfg(target_os = "linux")]
pub fn bring_to_front<R: Runtime>(window: &Window<R>) -> tauri::Result<()> {
    linux::bring_to_front(window)
}

/// Windows and macOS: tao's `set_focus` — `SetForegroundWindow`, with its Alt-key fallback when
/// the foreground lock refuses (win/front.rs); `makeKeyAndOrderFront` and
/// `activateIgnoringOtherApps` on macOS.
#[cfg(not(target_os = "linux"))]
pub fn bring_to_front<R: Runtime>(window: &Window<R>) -> tauri::Result<()> {
    window.set_focus()
}

/// How long after a summon `report_front` looks: the window manager's (or AppKit's) answer to
/// the raise has come by then.
const FRONT_DELAY: Duration = Duration::from_millis(1200);

/// Smoke mode only (CI's over-full-screen check, LJ19.2): about a second after a summon brought
/// `window` to the front, logs `window-front <label> focused=<bool> top=<bool>[ space=<bool>]`
/// (spaces::Front), or `window-front <label> unknown` when the window is gone. Nothing runs
/// without the smoke log.
pub fn report_front<R: Runtime>(window: &Window<R>) {
    if !smoke::enabled() {
        return;
    }
    let window = window.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-front".into())
        .spawn(move || {
            std::thread::sleep(FRONT_DELAY);
            let detail = match front(&window) {
                Some(front) => front.line(window.label()),
                None => format!("{} unknown", window.label()),
            };
            smoke::log("window-front", &detail);
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not check the window's place: {err}");
    }
}

#[cfg(target_os = "macos")]
fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    mac::front(window)
}

#[cfg(windows)]
fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    win::front(window)
}

#[cfg(target_os = "linux")]
fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    linux::front(window)
}
