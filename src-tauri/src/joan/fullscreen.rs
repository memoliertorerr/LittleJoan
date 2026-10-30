//! A summon while the page shows an element fullscreen (LJ19.5, the owner's decision): the page
//! leaves its fullscreen first — the page stays, a video keeps playing in the window —, and only
//! then does the window come to the front (joan.rs `focus`), with the command bar for the shortcut.
//!
//! How, per engine: the page is asked to leave as its own Exit button would (`EXIT_SCRIPT`, the
//! Fullscreen API's `document.exitFullscreen()`), on every OS — on macOS that is WebKit's normal,
//! animated exit, the one CI's fullscreen runs show leaving the Dock healthy. Then:
//! - **macOS:** nothing is ordered until WebKit has put the page back in this window and the window
//!   is on the Space on screen again (`fullscreen::webkit_fullscreen_over`): during WebKit's
//!   fullscreen an ordered window would join WebKit's full-screen Space, and that Space closing with
//!   it inside can make the Dock refuse every app's full screen (LJ19.4-fix4, the Landmine). A page
//!   that doesn't start leaving within `FORCE_AFTER` is closed by WebKit itself (WKWebView's
//!   public `closeAllMediaPresentationsWithCompletionHandler:`, macOS 12+, its quick exit).
//! - **Windows** (WebView2's `ContainsFullScreenElementChanged` made the window fullscreen) and
//!   **Linux** (WebKitGTK's own `leave-fullscreen` handling): the page's fullscreen ends
//!   (`bar::element_fullscreen` false), then the window gets a moment to leave its own.
//!
//! Smoke log: `summon-fullscreen leaving`, `forcing` (macOS), and `left <ms>` or `gave-up <ms>`
//! after `LIMIT`, when the summon gives up (macOS then only brings the app to the front).

use std::time::{Duration, Instant};

use tauri::{AppHandle, Runtime, Window};

use crate::bar;
use crate::fullscreen::{webkit_fullscreen_over, webkit_still_in};
use crate::platform;
use crate::smoke;

/// The longest a summon waits for the page to leave its fullscreen (macOS: WebKit's exit
/// animation and the Space switching back, about a second in CI).
const LIMIT: Duration = Duration::from_secs(4);
/// How often it looks.
const POLL: Duration = Duration::from_millis(50);
/// macOS: WebKit still in its fullscreen this long after the page was asked to leave → WebKit is
/// asked itself.
const FORCE_AFTER: Duration = Duration::from_millis(1200);
/// Windows and Linux: once the page's fullscreen ended, how long the window gets to leave its own
/// (it may have been fullscreen before the page's, then it stays so).
const WINDOW_SETTLE: Duration = Duration::from_secs(1);

/// The page's own way out of its fullscreen, as its Exit button would take it: the Fullscreen API
/// (prefixed for old WebKit), and a video in WebKit's own video fullscreen. Evaluated in the page —
/// no IPC, nothing reported back; a page that won't leave keeps its fullscreen.
pub const EXIT_SCRIPT: &str = "(() => { const d = document; try { \
    if (d.fullscreenElement && d.exitFullscreen) { d.exitFullscreen().catch(() => {}); } \
    else if (d.webkitFullscreenElement && d.webkitExitFullscreen) { d.webkitExitFullscreen(); } \
    for (const v of d.querySelectorAll('video')) { \
    if (v.webkitDisplayingFullscreen && v.webkitExitFullscreen) { v.webkitExitFullscreen(); } } \
    } catch (e) {} })();";

/// Leaves the page's element fullscreen, if it shows one, and waits until the page is back in
/// `window` (the module's docs). Returns false when it isn't after `LIMIT`. Off the main thread
/// (it waits; the engines answer on the main thread).
pub(super) fn leave<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) -> bool {
    let webkit = bar::page_in_webkit_fullscreen(app);
    if !webkit && !bar::element_fullscreen() {
        return true;
    }
    let Some(page) = bar::page(app) else {
        return true;
    };
    let started = Instant::now();
    smoke::log("summon-fullscreen", "leaving");
    if let Err(err) = page.eval(EXIT_SCRIPT) {
        eprintln!("little-joan: could not ask the page to leave its fullscreen: {err}");
    }
    let mut forced = false;
    loop {
        std::thread::sleep(POLL);
        let waited = started.elapsed();
        if webkit {
            let exit = platform::webkit_fullscreen_exit(&page);
            if exit.is_some_and(|e| webkit_fullscreen_over(e.state, e.in_window, e.on_screen)) {
                break;
            }
            if !forced
                && waited >= FORCE_AFTER
                && exit.is_some_and(|e| webkit_still_in(e.state, e.in_window))
            {
                forced = true;
                smoke::log("summon-fullscreen", "forcing");
                platform::close_media_presentations(&page);
            }
        } else if !bar::element_fullscreen() {
            settle(window, started);
            break;
        }
        if waited >= LIMIT {
            smoke::log(
                "summon-fullscreen",
                &format!("gave-up {}", waited.as_millis()),
            );
            return false;
        }
    }
    smoke::log(
        "summon-fullscreen",
        &format!("left {}", started.elapsed().as_millis()),
    );
    true
}

/// Windows and Linux: the page's fullscreen ended; the window leaves its own (WebView2's hook,
/// WebKitGTK's handler) within `WINDOW_SETTLE`, or stays as it is.
fn settle<R: Runtime>(window: &Window<R>, started: Instant) {
    let until = Instant::now() + WINDOW_SETTLE;
    while platform::fullscreen_now(window) && Instant::now() < until {
        std::thread::sleep(POLL);
    }
    if smoke::enabled() && platform::fullscreen_now(window) {
        smoke::log(
            "summon-fullscreen",
            &format!("window-stays-fullscreen {}", started.elapsed().as_millis()),
        );
    }
}
