//! The page window's two webviews: the bar (`joan-bar`, the trusted local page that holds every
//! screen of the app's own: the toolbar, the command bar, the welcome, the history view) and the
//! page (`joan-page`, where web content loads, with no IPC at all). Built with Tauri's multi-webview
//! API (`unstable` feature), laid out by layout.rs: the bar is either the 40 px toolbar above the
//! page, or an overlay covering the whole window while the command bar is open (the page is
//! hidden meanwhile). Every summon opens the overlay; showing a page closes it.
//!
//! The bar is told about the page's address whenever it may have changed: on every page load
//! (start and finish, so redirects land too), on every title change, after a navigation the
//! app starts, and — for in-page changes that raise no event (history.pushState, #anchors) — by
//! a once-a-second check that runs only while the window exists. The address itself comes from
//! those events (page_address.rs); the engine is asked only by that check, after a finished load.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use tauri::webview::{NewWindowResponse, PageLoadEvent};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewBuilder, WebviewUrl,
    Window,
};
use url::Url;

mod view;
mod watch;

pub use view::{
    apply_layout, element_fullscreen, log_overlay, mode, overlay, set_element_fullscreen,
    set_overlay,
};
pub use watch::watch_url;

use crate::back_stack::BackStack;
use crate::extensions;
use crate::history_store;
use crate::keys;
use crate::layout::{Mode, Rect, layout};
use crate::navigation;
use crate::page_address::PageAddress;
use crate::platform;
use crate::smoke;
use crate::tray;

pub const BAR: &str = "joan-bar";
pub const PAGE: &str = "joan-page";

/// The page's address from its load events and the app's own navigations (page_address.rs).
/// Reading an unset WKWebView address panics inside wry 0.57 (CI saw it when WebKit was slow to
/// start, LJ20), so the engine's address is read only by the in-page check, after a finished
/// load; everything else uses this.
static ADDRESS: Mutex<PageAddress> = Mutex::new(PageAddress::new());
/// This window's trail of web pages, for the toolbar's back button (back_stack.rs).
static BACK: Mutex<BackStack> = Mutex::new(BackStack::new());
/// What the bar was last told (`Some(None)` = the start page); reset with each new window.
static LAST_SHOWN: Mutex<Option<Option<String>>> = Mutex::new(None);

/// What the bar shows for `url`: nothing for the app's own pages, `about:` pages and extension
/// panels (the toolbar shows the extension instead, extensions/session.rs), else the address.
pub fn bar_text(url: &Url, app_origin: &Url) -> Option<String> {
    if keys::is_app_url(app_origin, url)
        || url.scheme() == "about"
        || extensions::urls::is_ext_like(url)
    {
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
    *address() = PageAddress::new();
    *back() = BackStack::new();
    // Every summon opens the command bar: the window is built in Overlay mode.
    view::new_window();
    *LAST_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    let bar_rect = layout(width, height, Mode::Overlay).bar.unwrap_or(whole);
    // The page starts hidden; these are its bounds for when the overlay closes.
    let page_rect = layout(width, height, Mode::Toolbar).page.unwrap_or(whole);

    let mut bar = WebviewBuilder::new(BAR, WebviewUrl::App("bar.html".into()))
        // The bar is ours and never leaves the app's origin (about:blank only to unload it when
        // the window closes on macOS, see joan::release_page).
        .on_navigation(|url| navigation::is_allowed_bar_navigation(url, &keys::app_origin()))
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                smoke::log("bar-loaded", "");
                // The bar may have missed a change while it loaded: tell it the current state.
                // (Its script may not be running yet — WKWebView can report the load before it
                // ran — so `start_info`, which the script calls, tells it again: tell_state.)
                view::tell_overlay(&webview, overlay());
                tell_back(&webview, can_go_back());
            }
        });
    let mut page = WebviewBuilder::new(PAGE, WebviewUrl::App("index.html".into()))
        // Web pages only: no file:, javascript:, or app-launching schemes (navigation.rs); an
        // extension's panel only when the app opened it (extensions/session.rs).
        .on_navigation(extensions::session::allow_page_navigation)
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
            match payload.event() {
                PageLoadEvent::Started => {
                    address().started(payload.url().clone());
                    smoke::log("page-started", payload.url().as_str());
                    // What the toolbar shows, and what the extension handler serves beyond a
                    // panel's document: an extension only once its panel committed here.
                    extensions::session::page_committed(webview.app_handle(), payload.url());
                }
                PageLoadEvent::Finished => {
                    address().finished(payload.url().clone());
                }
            }
            let app = webview.app_handle();
            publish_url(app);
            if payload.event() == PageLoadEvent::Finished {
                let url = payload.url();
                let web_page = !keys::is_app_url(&keys::app_origin(), url)
                    && navigation::is_external_openable(url);
                tray::set_page_items_enabled(app, web_page);
                smoke::log("page-loaded", url.as_str());
                let changed = back().loaded(url, &keys::app_origin());
                if changed && let Some(bar) = app.get_webview(BAR) {
                    tell_back(&bar, can_go_back());
                }
                // History records visits natively: the page itself has no IPC.
                history_store::page_loaded(app, url);
            }
        })
        // target=_blank and window.open go to the default browser (web URLs only).
        .on_new_window(|url, _features| {
            if !navigation::is_external_openable(&url) {
                smoke::log("new-window-refused", url.as_str());
            } else if let Err(err) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>) {
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
        LogicalPosition::new(page_rect.x, page_rect.y),
        LogicalSize::new(page_rect.width, page_rect.height),
    )?;
    platform::install_bar(&bar);
    platform::install_page(&page);
    platform::set_overlay(&bar, &page, true);
    let _ = page.hide();
    Ok(())
}

fn address() -> std::sync::MutexGuard<'static, PageAddress> {
    ADDRESS.lock().unwrap_or_else(PoisonError::into_inner)
}

fn back() -> std::sync::MutexGuard<'static, BackStack> {
    BACK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether the page has an earlier web page in this window to go back to (the toolbar's back
/// button).
pub fn can_go_back() -> bool {
    back().can_go_back()
}

/// Tells the bar its current view (toolbar or command bar) and whether the page can go back.
/// `start_info` calls it: the bar asks for that only once its script runs, so this reaches it even
/// when the load event's re-tell came too early (CI saw the macOS bar stay in the command-bar view
/// above a page that `--show <url>` opened while hibernating). Evals reach the bar in order, so a
/// later change still wins.
pub fn tell_state<R: Runtime>(app: &AppHandle<R>) {
    if let Some(bar) = app.get_webview(BAR) {
        view::tell_overlay(&bar, overlay());
        tell_back(&bar, can_go_back());
    }
    extensions::session::publish_label(app, true);
}

fn tell_back<R: Runtime>(bar: &Webview<R>, on: bool) {
    let _ = bar.eval(format!(
        "window.littleJoan && window.littleJoan.setCanGoBack({on})"
    ));
}

/// The page's address: `None` without a window, and until the page has started loading. It
/// comes from the page's load events and the app's own navigations (`ADDRESS`); use this rather
/// than `Webview::url` on the page.
pub fn page_url<R: Runtime>(app: &AppHandle<R>) -> Option<Url> {
    app.get_webview(PAGE)?;
    address().current().cloned()
}

/// The page's address as the bar should show it (`None` = start page or no page).
pub fn current_text<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    bar_text(&page_url(app)?, &keys::app_origin())
}

/// Tells the bar the page's address if it changed since the last time (and the extension the
/// toolbar shows).
pub fn publish_url<R: Runtime>(app: &AppHandle<R>) {
    let Some(url) = page_url(app) else {
        return;
    };
    extensions::session::publish_label(app, false);
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

/// Opens the command bar and puts the keyboard in its field with the text selected
/// (Ctrl/Cmd+L, and on summon).
pub fn focus_bar<R: Runtime>(app: &AppHandle<R>) {
    set_overlay(app, true);
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.set_focus();
        let _ = bar.eval("window.littleJoan && window.littleJoan.focusField()");
    }
}

/// Opens the command bar's overlay in its history view, the keyboard in its search field
/// (Ctrl/Cmd+Y: page keys and the macOS menu). Not during element fullscreen, which always wins.
pub fn show_history<R: Runtime>(app: &AppHandle<R>) {
    if element_fullscreen() {
        return;
    }
    // Opening the overlay tells the bar first, so the history view comes after (evals run in
    // order).
    set_overlay(app, true);
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.set_focus();
        let _ = bar.eval("window.littleJoan && window.littleJoan.showHistory()");
    }
    smoke::log("history-view", "");
}

/// Shows `theme` in the open window's bar at once (after Settings saves one).
pub fn set_theme<R: Runtime>(app: &AppHandle<R>, theme: &crate::settings::Theme) {
    let Some(bar) = app.get_webview(BAR) else {
        return;
    };
    // Checked hex colors, a mode name and two numbers: the JSON is a safe script literal.
    match serde_json::to_string(theme) {
        Ok(json) => {
            let _ = bar.eval(format!(
                "window.littleJoan && window.littleJoan.setTheme({json})"
            ));
        }
        Err(err) => eprintln!("little-joan: could not send the theme to the bar: {err}"),
    }
}

/// Gives the keyboard back to the page.
pub fn focus_page<R: Runtime>(app: &AppHandle<R>) {
    if let Some(page) = app.get_webview(PAGE) {
        let _ = page.set_focus();
    }
}

/// Loads `url` in the page and tells the bar. Showing a page closes the command bar, and the
/// keyboard follows the page out of it (the toolbar has no field to keep it).
pub fn navigate<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    if let Some(page) = app.get_webview(PAGE) {
        let was_open = overlay();
        set_overlay(app, false);
        address().navigating(url.clone());
        page.navigate(url)?;
        publish_url(app);
        if was_open {
            focus_page(app);
        }
    }
    Ok(())
}

/// Loads `url` in the page — an extension's panel, or a page an extension asks for — without
/// taking it as the page's address yet: the address and the toolbar change when the page's own
/// load events arrive, so a navigation the guard refuses (or the engine cancels) leaves the real
/// page named. Closes the command bar like `navigate`.
pub fn navigate_pending<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    if let Some(page) = app.get_webview(PAGE) {
        let was_open = overlay();
        set_overlay(app, false);
        address().pending();
        page.navigate(url)?;
        if was_open {
            focus_page(app);
        }
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
