//! Phone mode at runtime: give the page webview a phone user agent while the window is narrow
//! (decisions in phone.rs) or its site says so (`siteModes`, a built-in list on WebKit), switch
//! when a resize that crosses the threshold has settled or the page goes to a site with another
//! mode, and reload the current page so the site can answer with its other version.
//!
//! - The app's own navigations (`bar::navigate`: typed addresses, picked rows, `--show <url>`)
//!   set the agent for the target's host first (`prepare`), so the request goes out right.
//! - A navigation the page starts itself (a link, a redirect) gets its host's agent when its
//!   `Started` event arrives (`page_started`), and when the agent really changes the page loads
//!   once more with it — never twice in a row for the same URL.
//! - Going to the desktop agent on a known phone host (`m.youtube.com`) loads the desktop address
//!   (`phone::desktop_url`) instead of reloading.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Runtime, Webview};
use url::Url;

use crate::bar;
use crate::history::VisitKind;
use crate::history_store;
use crate::joan;
use crate::keys::{self, Platform};
use crate::navigation;
use crate::phone::{self, Debounce};
use crate::platform;
use crate::settings::SiteMode;
use crate::smoke;
use crate::state;

/// The user agent the page webview has: 0 = not set yet (new window), 1 = desktop, 2 = phone.
static APPLIED: AtomicU8 = AtomicU8::new(0);
static DEBOUNCE: Mutex<Debounce> = Mutex::new(Debounce::new(phone::SETTLE));
static WAITING: AtomicBool = AtomicBool::new(false);
/// The page last loaded again because its agent changed on `Started` (never twice in a row).
static RELOADED: Mutex<Option<Url>> = Mutex::new(None);

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
    *RELOADED.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

/// A web page's host, for its site mode (the app's own pages and extension panels have none).
fn web_host(url: &Url) -> Option<&str> {
    let web = navigation::is_external_openable(url) && !keys::is_app_url(&keys::app_origin(), url);
    web.then(|| url.host_str()).flatten()
}

/// The mode of `url`'s site (auto for anything but a web page).
fn mode_for<R: Runtime>(app: &AppHandle<R>, url: Option<&Url>) -> SiteMode {
    let Some(host) = url.and_then(web_host) else {
        return SiteMode::Auto;
    };
    state::with(app, |s| {
        phone::site_mode(host, Platform::current(), &s.settings.site_modes)
    })
}

/// Gives the page the agent that `url` (the page's address when `None` is passed by `apply`)
/// and the window's width call for. Returns the agent it switched to, or `None` when nothing
/// changed. Never switches during fullscreen: a fullscreen video makes the window wide, and a
/// reload would end it (macOS: WebKit's own fullscreen window, platform::page_in_webkit_fullscreen).
fn switch_for<R: Runtime>(app: &AppHandle<R>, url: Option<&Url>) -> Option<(Webview<R>, bool)> {
    let (Some(window), Some(page)) = (app.get_window(joan::LABEL), bar::page(app)) else {
        return None;
    };
    if bar::element_fullscreen() || window.is_fullscreen().unwrap_or(false) {
        return None;
    }
    if platform::page_in_webkit_fullscreen(&page) {
        // Checked again (with the layout) once WebKit's fullscreen has ended.
        bar::after_webkit_fullscreen(app);
        return None;
    }
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return None;
    };
    let width = size.to_logical::<f64>(scale).width;
    let enabled = state::with(app, |s| s.settings.mobile_sites);
    let wanted = phone::wants_mobile(enabled, width, mode_for(app, url));
    let mobile = phone::switch(applied(), wanted)?;
    APPLIED.store(if mobile { 2 } else { 1 }, Ordering::Relaxed);
    platform::set_mobile(&page, mobile);
    smoke::log("user-agent", if mobile { "phone" } else { "desktop" });
    Some((page, mobile))
}

/// Loads the open web page again with its new agent — or, going to the desktop agent on a known
/// phone host, its desktop address.
fn load_again<R: Runtime>(app: &AppHandle<R>, page: &Webview<R>, url: &Url, mobile: bool) {
    match phone::desktop_url(url).filter(|_| !mobile) {
        Some(desktop) => {
            if let Err(err) = bar::load(app, desktop) {
                eprintln!("little-joan: could not open the desktop site: {err}");
            }
        }
        None => {
            history_store::set_next_kind(app, VisitKind::Reload);
            let _ = page.reload();
        }
    }
}

/// Gives the page the user agent its current width (and its site) calls for. With `reload`, a
/// web page that is already loaded is loaded again when the agent changes (the agent goes out
/// with requests).
pub fn apply<R: Runtime>(app: &AppHandle<R>, reload: bool) {
    let previous = applied();
    let url = bar::page_url(app);
    let Some((page, mobile)) = switch_for(app, url.as_ref()) else {
        return;
    };
    if reload
        && previous.is_some()
        && let Some(url) = url.filter(|url| web_host(url).is_some())
    {
        load_again(app, &page, &url, mobile);
    }
}

/// The app is about to load `url` in the page: the agent for its site first, so the request goes
/// out with it. Returns the address to load — a known phone address's desktop one when its site
/// is on the desktop agent (`m.youtube.com` on macOS and Linux).
pub fn prepare<R: Runtime>(app: &AppHandle<R>, url: Url) -> Url {
    let url = match phone::desktop_url(&url) {
        Some(desktop) if mode_for(app, Some(&url)) == SiteMode::Desktop => desktop,
        _ => url,
    };
    let _ = switch_for(app, Some(&url));
    url
}

/// The page's `Started` event for `url` (a navigation it may have started itself): the agent for
/// that site, and when it really changed, the page once more with it — off the engine's callback.
pub fn page_started<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    if web_host(url).is_none() {
        return;
    }
    let app = app.clone();
    let url = url.clone();
    tauri::async_runtime::spawn(async move {
        {
            let mut reloaded = RELOADED.lock().unwrap_or_else(PoisonError::into_inner);
            if reloaded.as_ref() != Some(&url) {
                *reloaded = None;
            }
        }
        let previous = applied();
        let Some((page, mobile)) = switch_for(&app, Some(&url)) else {
            return;
        };
        if previous.is_none() {
            return;
        }
        {
            let mut reloaded = RELOADED.lock().unwrap_or_else(PoisonError::into_inner);
            if reloaded.as_ref() == Some(&url) {
                return;
            }
            *reloaded = Some(url.clone());
        }
        load_again(&app, &page, &url, mobile);
    });
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
