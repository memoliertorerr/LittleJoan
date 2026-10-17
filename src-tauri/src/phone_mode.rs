//! Phone mode at runtime: give the page webview a phone user agent while the window is narrow
//! (decisions in phone.rs), switch when a resize that crosses the threshold has settled, and
//! reload the current page so the site can answer with its other version.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Runtime};

use crate::bar;
use crate::joan;
use crate::phone::{self, Debounce};
use crate::platform;
use crate::smoke;
use crate::state;

/// The user agent the page webview has: 0 = not set yet (new window), 1 = desktop, 2 = phone.
static APPLIED: AtomicU8 = AtomicU8::new(0);
static DEBOUNCE: Mutex<Debounce> = Mutex::new(Debounce::new(phone::SETTLE));
static WAITING: AtomicBool = AtomicBool::new(false);

fn applied() -> Option<bool> {
    match APPLIED.load(Ordering::Relaxed) {
        1 => Some(false),
        2 => Some(true),
        _ => None,
    }
}

/// A new page window: its webview starts with the engine's own user agent.
pub fn reset() {
    APPLIED.store(0, Ordering::Relaxed);
}

/// Gives the page the user agent its current width calls for. With `reload`, a web page that
/// is already loaded is reloaded when the agent changes (the agent goes out with requests).
/// Never switches during fullscreen: a fullscreen video makes the window wide, and a reload
/// would end it.
pub fn apply<R: Runtime>(app: &AppHandle<R>, reload: bool) {
    let (Some(window), Some(page)) = (app.get_window(joan::LABEL), bar::page(app)) else {
        return;
    };
    if bar::element_fullscreen() || window.is_fullscreen().unwrap_or(false) {
        return;
    }
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let width = size.to_logical::<f64>(scale).width;
    let enabled = state::with(app, |s| s.settings.mobile_sites);
    let previous = applied();
    let Some(mobile) = phone::switch(previous, phone::wants_mobile(enabled, width)) else {
        return;
    };
    APPLIED.store(if mobile { 2 } else { 1 }, Ordering::Relaxed);
    platform::set_mobile(&page, mobile);
    smoke::log("user-agent", if mobile { "phone" } else { "desktop" });
    if reload && previous.is_some() && bar::current_text(app).is_some() {
        let _ = page.reload();
    }
}

/// The window was resized: re-check once the resize has settled (not on every drag frame).
pub fn on_resized<R: Runtime>(app: &AppHandle<R>) {
    DEBOUNCE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .touch(Instant::now());
    if WAITING.swap(true, Ordering::Relaxed) {
        return;
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-phone-mode".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(50));
                let ready = DEBOUNCE
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .ready(Instant::now());
                if ready || app.get_window(joan::LABEL).is_none() {
                    WAITING.store(false, Ordering::Relaxed);
                    if ready {
                        apply(&app, true);
                    }
                    break;
                }
            }
        });
    if spawned.is_err() {
        WAITING.store(false, Ordering::Relaxed);
    }
}
