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
//! - Per site from the bar (LJ40): `set_site_mode` keeps the open web page's site on `phone`,
//!   `desktop` or `auto` (the host is Rust's, never the bar's), saves it and applies it at once;
//!   `info` / `publish` tell the toolbar's pill and the command bar what the site is on.
//! - Windows asks the engine whether the page already fits a narrower window before reloading it
//!   for the phone agent (`platform::page_fits`, CDP's `Page.getLayoutMetrics`); a page that fits
//!   stays as it is (`ua-reload skipped fits`). macOS and Linux would need a script in the page to
//!   measure it, so they reload.

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
use crate::phone::{self, Debounce, Reaction, Refusal, SiteInfo};
use crate::platform;
use crate::settings::{self, SiteMode};
use crate::smoke;
use crate::state;

/// The user agent the page webview has: 0 = not set yet (new window), 1 = desktop, 2 = phone.
static APPLIED: AtomicU8 = AtomicU8::new(0);
static DEBOUNCE: Mutex<Debounce> = Mutex::new(Debounce::new(phone::SETTLE));
static WAITING: AtomicBool = AtomicBool::new(false);
/// The page last loaded again because its agent changed on `Started` (never twice in a row).
static RELOADED: Mutex<Option<Url>> = Mutex::new(None);
/// What the bar was last told about the page's site (`setSiteMode`'s JSON); reset per window.
static PUBLISHED: Mutex<Option<String>> = Mutex::new(None);

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
    *PUBLISHED.lock().unwrap_or_else(PoisonError::into_inner) = None;
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
    let wanted = phone::wants_mobile(enabled, width, mode_for(app, url), applied());
    let mobile = phone::switch(applied(), wanted)?;
    APPLIED.store(if mobile { 2 } else { 1 }, Ordering::Relaxed);
    platform::set_mobile(&page, mobile);
    smoke::log("user-agent", if mobile { "phone" } else { "desktop" });
    publish(app);
    Some((page, mobile))
}

/// Loads the open web page again with its new agent — or, going to the desktop agent on a known
/// phone host, its desktop address (`phone::reaction`).
fn load_again<R: Runtime>(app: &AppHandle<R>, page: &Webview<R>, url: &Url, mobile: bool) {
    match phone::reaction(Some(mobile), url) {
        Reaction::Load(desktop) => {
            if let Err(err) = bar::load(app, desktop) {
                eprintln!("little-joan: could not open the desktop site: {err}");
            }
        }
        Reaction::Reload => {
            history_store::set_next_kind(app, VisitKind::Reload);
            let _ = page.reload();
        }
        Reaction::Stay => {}
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
        // Narrower, and the page already fits (Windows can ask the engine): it stays as it is.
        if mobile && platform::page_fits(&page) == Some(true) {
            smoke::log("ua-reload", "skipped fits");
            return;
        }
        load_again(app, &page, &url, mobile);
    }
}

/// What the toolbar and the command bar are told about the open web page's site (`SiteInfo`), or
/// `None` without one (the start page, an extension's panel).
pub fn info<R: Runtime>(app: &AppHandle<R>) -> Option<SiteInfo> {
    let url = bar::page_url(app)?;
    let host = web_host(&url)?;
    let mobile = applied() == Some(true);
    state::with(app, |s| {
        phone::site_info(host, Platform::current(), &s.settings.site_modes, mobile)
    })
}

/// Tells the bar the site info when it changed (`window.littleJoan.setSiteMode(<info | null>)`):
/// after an agent switch, and with every change of the page's address (bar::publish_url).
pub fn publish<R: Runtime>(app: &AppHandle<R>) {
    let json = serde_json::to_string(&info(app)).unwrap_or_else(|_| "null".to_owned());
    {
        let mut last = PUBLISHED.lock().unwrap_or_else(PoisonError::into_inner);
        if last.as_deref() == Some(json.as_str()) {
            return;
        }
        *last = Some(json.clone());
    }
    if let Some(bar) = app.get_webview(bar::BAR) {
        let _ = bar.eval(format!(
            "window.littleJoan && window.littleJoan.setSiteMode && window.littleJoan.setSiteMode({json})"
        ));
    }
}

/// The bar's script runs now (start_info): it is told again, whatever it was told before.
pub fn republish<R: Runtime>(app: &AppHandle<R>) {
    *PUBLISHED.lock().unwrap_or_else(PoisonError::into_inner) = None;
    publish(app);
}

/// The bar asked for `mode` on the open web page's site ("Request desktop site", the toolbar's
/// pill): keeps it in `siteModes` for that site (phone::with_site_mode), saves the settings, and
/// applies it at once — the agent changes, and the page loads again only when it really changed
/// (its desktop address, going to desktop on a known phone host). Refused without a web page.
pub fn set_site_mode<R: Runtime>(app: &AppHandle<R>, mode: SiteMode) -> Result<SiteInfo, String> {
    let url = bar::page_url(app)
        .filter(|url| web_host(url).is_some())
        .ok_or_else(|| Refusal::NoSite.message().to_owned())?;
    let host = web_host(&url).unwrap_or_default().to_owned();
    let (dir, updated) = state::with(app, |s| {
        let table =
            phone::with_site_mode(&s.settings.site_modes, &host, mode, Platform::current())?;
        s.settings.site_modes = table;
        Ok::<_, Refusal>((s.data_dir.clone(), s.settings.clone()))
    })
    .map_err(|refusal| refusal.message().to_owned())?;
    let site = phone::mode_site(&host).unwrap_or_default();
    smoke::log("site-mode", &format!("{site} {}", mode.name()));
    if let Some((page, mobile)) = switch_for(app, Some(&url)) {
        load_again(app, &page, &url, mobile);
    }
    publish(app);
    if let Err(err) = settings::save(&dir, &updated) {
        return Err(format!("It works now, but couldn't be saved: {err}"));
    }
    info(app).ok_or_else(|| Refusal::NoSite.message().to_owned())
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
