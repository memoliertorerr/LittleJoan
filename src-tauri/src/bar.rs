//! The page window's two webviews: the URL bar (`joan-bar`, a trusted local page pinned to
//! the top) and the page (`joan-page`, where web content loads, with no IPC). Built with Tauri's
//! multi-webview API (`unstable` feature), laid out by layout.rs.
//!
//! The bar is told about the page's address whenever it may have changed: on every page load
//! (start and finish, so redirects land too), on every title change, after a navigation the
//! app starts, and — for in-page changes that raise no event (history.pushState, #anchors) — by
//! a once-a-second check that runs only while the window exists.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::webview::{NewWindowResponse, PageLoadEvent};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewBuilder, WebviewUrl,
    Window,
};
use url::Url;

use crate::history_store;
use crate::keys;
use crate::layout::{self, layout};
use crate::navigation;
use crate::platform;
use crate::smoke;
use crate::tray;

pub const BAR: &str = "joan-bar";
pub const PAGE: &str = "joan-page";

/// True while the page shows an element fullscreen (the bar is hidden meanwhile).
static ELEMENT_FULLSCREEN: AtomicBool = AtomicBool::new(false);
/// True once the page webview of this window reported a page load. Until then its address may
/// be unset, and reading an unset WKWebView address panics inside wry 0.57 (an abort in
/// release builds: CI saw it when WebKit was slow to start, LJ20), so `page_url` waits for it.
static PAGE_STARTED: AtomicBool = AtomicBool::new(false);
/// What the bar was last told (`Some(None)` = the start page); reset with each new window.
static LAST_SHOWN: Mutex<Option<Option<String>>> = Mutex::new(None);
/// Bumped for every new window, so an old address watcher stops.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// What the bar shows for `url`: nothing for the app's own pages and `about:` pages, else the
/// address itself.
pub fn bar_text(url: &Url, app_origin: &Url) -> Option<String> {
    if keys::is_app_url(app_origin, url) || url.scheme() == "about" {
        None
    } else {
        Some(url.as_str().to_owned())
    }
}

/// Adds the bar and the page to a new page window of `width` × `height` logical pixels.
pub fn add_webviews<R: Runtime>(
    window: &Window<R>,
    width: f64,
    height: f64,
    data_dir: Option<PathBuf>,
) -> tauri::Result<()> {
    ELEMENT_FULLSCREEN.store(false, Ordering::Relaxed);
    PAGE_STARTED.store(false, Ordering::Relaxed);
    *LAST_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
    let initial = layout(width, height, true);
    let bar_rect = initial.bar.unwrap_or(initial.page);

    let mut bar = WebviewBuilder::new(BAR, WebviewUrl::App("bar.html".into()))
        // The bar is ours and never leaves the app's origin (about:blank only to unload it when
        // the window closes on macOS, see joan::release_page).
        .on_navigation(|url| {
            keys::is_app_url(&keys::app_origin(), url) || url.as_str() == "about:blank"
        })
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_page_load(|_, payload| {
            if payload.event() == PageLoadEvent::Finished {
                smoke::log("bar-loaded", "");
            }
        });
    let mut page = WebviewBuilder::new(PAGE, WebviewUrl::App("index.html".into()))
        // Web pages only: no file:, javascript:, or app-launching schemes (navigation.rs).
        .on_navigation(navigation::is_allowed_navigation)
        .on_document_title_changed(|webview, title| {
            let title = title.trim();
            let title = if title.is_empty() {
                "Little Joan"
            } else {
                title
            };
            let _ = webview.window().set_title(title);
            smoke::log("title", title);
            let app = webview.app_handle();
            publish_url(app);
            if let Some(url) = page_url(app) {
                history_store::title_changed(app, &url, title);
            }
        })
        .on_page_load(|webview, payload| {
            PAGE_STARTED.store(true, Ordering::Relaxed);
            let app = webview.app_handle();
            publish_url(app);
            if payload.event() == PageLoadEvent::Finished {
                let url = payload.url();
                let web_page = !keys::is_app_url(&keys::app_origin(), url)
                    && navigation::is_external_openable(url);
                tray::set_page_items_enabled(app, web_page);
                smoke::log("page-loaded", url.as_str());
                // History records visits natively: the page itself has no IPC.
                history_store::page_loaded(app, url);
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
    // Both webviews of a window must share one data directory (WebView2 requires it).
    if let Some(dir) = data_dir {
        bar = bar.data_directory(dir.clone());
        page = page.data_directory(dir);
    }

    let bar = window.add_child(
        bar,
        LogicalPosition::new(bar_rect.x, bar_rect.y),
        LogicalSize::new(bar_rect.width, bar_rect.height),
    )?;
    let page = window.add_child(
        page,
        LogicalPosition::new(initial.page.x, initial.page.y),
        LogicalSize::new(initial.page.width, initial.page.height),
    )?;
    platform::install_bar(&bar);
    platform::install_page(&page);
    Ok(())
}

/// Positions the bar and the page for the window's current size and fullscreen state.
pub fn apply_layout<R: Runtime>(window: &Window<R>) {
    let app = window.app_handle();
    let (Some(bar), Some(page)) = (app.get_webview(BAR), app.get_webview(PAGE)) else {
        return;
    };
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    // inner_size is physical; the bounds below are logical (layout.rs).
    let wanted = layout::for_physical_size(
        size.width,
        size.height,
        scale,
        !ELEMENT_FULLSCREEN.load(Ordering::Relaxed),
    );
    match wanted.bar {
        Some(rect) => {
            let _ = bar.set_bounds(to_tauri(rect));
            let _ = bar.show();
        }
        None => {
            let _ = bar.hide();
        }
    }
    let _ = page.set_bounds(to_tauri(wanted.page));
}

fn to_tauri(rect: layout::Rect) -> tauri::Rect {
    tauri::Rect {
        position: LogicalPosition::new(rect.x, rect.y).into(),
        size: LogicalSize::new(rect.width, rect.height).into(),
    }
}

/// Whether the page shows an element fullscreen right now.
pub fn element_fullscreen() -> bool {
    ELEMENT_FULLSCREEN.load(Ordering::Relaxed)
}

/// The page entered or left element fullscreen: hide or bring back the bar.
pub fn set_element_fullscreen<R: Runtime>(app: &AppHandle<R>, on: bool) {
    ELEMENT_FULLSCREEN.store(on, Ordering::Relaxed);
    smoke::log("element-fullscreen", if on { "true" } else { "false" });
    if let Some(window) = app.get_window(crate::joan::LABEL) {
        apply_layout(&window);
    }
}

/// The page's address: `None` without a window, and until the page has started loading (see
/// `PAGE_STARTED`). Use this rather than `Webview::url` on the page.
pub fn page_url<R: Runtime>(app: &AppHandle<R>) -> Option<Url> {
    if !PAGE_STARTED.load(Ordering::Relaxed) {
        return None;
    }
    app.get_webview(PAGE)?.url().ok()
}

/// The page's address as the bar should show it (`None` = start page or no page).
pub fn current_text<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    bar_text(&page_url(app)?, &keys::app_origin())
}

/// Tells the bar the page's address if it changed since the last time.
pub fn publish_url<R: Runtime>(app: &AppHandle<R>) {
    let Some(url) = page_url(app) else {
        return;
    };
    let text = bar_text(&url, &keys::app_origin());
    {
        let mut last = LAST_SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
        if last.as_ref() == Some(&text) {
            return;
        }
        *last = Some(text.clone());
    }
    if let Some(bar) = app.get_webview(BAR) {
        let json = serde_json::to_string(&text).unwrap_or_else(|_| "null".to_owned());
        let _ = bar.eval(format!(
            "window.littleJoan && window.littleJoan.setUrl({json})"
        ));
    }
    smoke::log("bar-url", text.as_deref().unwrap_or("start"));
}

/// Checks the page's address once a second while this window exists, for in-page changes that
/// raise no load or title event. Stops when the window goes or a newer one replaces it.
pub fn watch_url<R: Runtime>(app: &AppHandle<R>) {
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-url-watch".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                if GENERATION.load(Ordering::Relaxed) != generation
                    || app.get_window(crate::joan::LABEL).is_none()
                {
                    break;
                }
                publish_url(&app);
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not start the address watcher: {err}");
    }
}

/// Puts the keyboard in the bar with its text selected (Ctrl/Cmd+L, and on summon).
pub fn focus_bar<R: Runtime>(app: &AppHandle<R>) {
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.set_focus();
        let _ = bar.eval("window.littleJoan && window.littleJoan.focusField()");
    }
}

/// Gives the keyboard back to the page.
pub fn focus_page<R: Runtime>(app: &AppHandle<R>) {
    if let Some(page) = app.get_webview(PAGE) {
        let _ = page.set_focus();
    }
}

/// Loads `url` in the page and tells the bar.
pub fn navigate<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    if let Some(page) = app.get_webview(PAGE) {
        page.navigate(url)?;
        publish_url(app);
    }
    Ok(())
}

/// The page webview, if the window is open.
pub fn page<R: Runtime>(app: &AppHandle<R>) -> Option<Webview<R>> {
    app.get_webview(PAGE)
}

/// Both webviews of the window, if it is open.
pub fn webviews<R: Runtime>(app: &AppHandle<R>) -> Vec<Webview<R>> {
    [BAR, PAGE]
        .into_iter()
        .filter_map(|label| app.get_webview(label))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn the_start_page_shows_an_empty_bar() {
        for (origin, start) in [
            ("tauri://localhost/", "tauri://localhost/index.html"),
            (
                "http://tauri.localhost/",
                "http://tauri.localhost/index.html",
            ),
        ] {
            assert_eq!(bar_text(&url(start), &url(origin)), None);
        }
        assert_eq!(
            bar_text(&url("about:blank"), &url("tauri://localhost/")),
            None
        );
    }

    #[test]
    fn web_pages_show_their_full_address() {
        let origin = url("http://tauri.localhost/");
        assert_eq!(
            bar_text(&url("https://example.com/a?b=1#c"), &origin).as_deref(),
            Some("https://example.com/a?b=1#c")
        );
        assert_eq!(
            bar_text(&url("http://127.0.0.1:8123/smoke.html"), &origin).as_deref(),
            Some("http://127.0.0.1:8123/smoke.html")
        );
    }
}
