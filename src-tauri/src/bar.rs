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

mod background;
mod boot;
pub mod download;
pub mod page;
mod status;
mod view;
mod watch;
mod zoom;

pub use background::{forget_painted, painted, window_background};
pub use boot::{BAR_READY, Ready, early_theme, mark_ready, set_theme, theme_script};
pub use page::{ensure_page, page_summon};
pub use view::{
    after_webkit_fullscreen, apply_layout, element_fullscreen, log_overlay, mode, overlay,
    page_in_webkit_fullscreen, set_element_fullscreen, set_overlay,
};
pub use watch::watch_url;
pub use zoom::zoom_page;

use crate::back_stack::BackStack;
use crate::extensions;
use crate::keys;
use crate::layout::{Mode, Rect, layout};
use crate::navigation;
use crate::page_address::PageAddress;
use crate::phone_mode;
use crate::platform;
use crate::smoke;
use crate::summon_timing::{self, Mark};

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

/// Adds the bar to a new page window of `width` × `height` logical pixels, with the command bar
/// open over the whole window. The page webview comes later (bar/page.rs, LJ19.4): after the bar is
/// on screen, or at once for a summon with a URL.
pub fn add_bar<R: Runtime>(
    window: &Window<R>,
    width: f64,
    height: f64,
    data_dir: Option<PathBuf>,
) -> tauri::Result<()> {
    *address() = PageAddress::new();
    *back() = BackStack::new();
    // Every summon opens the command bar: the window is built in Overlay mode.
    view::new_window();
    // The bar's script hasn't run yet in this window (panels opened now wait for it: boot.rs).
    BAR_READY.reset();
    // The theme it paints first: the preview while Settings edits one, else the saved one (what
    // start_info hands out too).
    let theme = crate::state::with(window.app_handle(), |s| s.shown_theme());
    boot::set_early_theme(theme.clone());
    *LAST_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
    // No status yet, and every summon starts at 100 % (LJ16).
    status::forget();
    zoom::new_window();
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    let bar_rect = layout(width, height, Mode::Overlay).bar.unwrap_or(whole);

    let mut bar = WebviewBuilder::new(BAR, WebviewUrl::App("bar.html".into()))
        // Its theme before its first paint (the bar only: the page webview never gets a script),
        // and that theme's background under it (LJ39, LJ19.4: computed from the theme until the
        // bar painted one; the page keeps the engine's white).
        .initialization_script(theme_script(&theme))
        .background_color(background::bar_background(window, &theme))
        // The bar is ours and never leaves the app's origin (about:blank only to unload it when
        // the window is destroyed on macOS, see joan::release_window).
        .on_navigation(|url| navigation::is_allowed_bar_navigation(url, &keys::app_origin()))
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                smoke::log("bar-loaded", "");
                summon_timing::mark(Mark::BarLoaded);
                // The bar may have missed a change while it loaded: tell it the current state.
                // (Its script may not be running yet — WKWebView can report the load before it
                // ran — so `start_info`, which the script calls, tells it again: tell_state.)
                view::tell_overlay(&webview, overlay());
                tell_back(&webview, can_go_back());
            }
        });
    // Both webviews of a window must share one data directory (WebView2 requires it).
    if let Some(dir) = data_dir {
        bar = bar.data_directory(dir);
    }
    let bar = window.add_child(
        bar,
        LogicalPosition::new(bar_rect.x, bar_rect.y),
        LogicalSize::new(bar_rect.width, bar_rect.height),
    )?;
    platform::install_bar(&bar);
    platform::set_overlay(&bar, None, true);
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
    phone_mode::republish(app);
    status::retell(app);
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
    // The toolbar's pill follows the page's site (LJ40).
    phone_mode::publish(app);
}

/// Opens the command bar and puts the keyboard in its field with the text selected
/// (Ctrl/Cmd+L, and on summon). Not while WebKit shows the page fullscreen (macOS).
pub fn focus_bar<R: Runtime>(app: &AppHandle<R>) {
    if page_in_webkit_fullscreen(app) {
        return;
    }
    set_overlay(app, true);
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.set_focus();
        let _ = bar.eval("window.littleJoan && window.littleJoan.focusField()");
    }
}

/// Opens the command bar's overlay in its history view, the keyboard in its search field
/// (Ctrl+H: page keys; Cmd+Y: the macOS menu). Not during element fullscreen, which always wins.
pub fn show_history<R: Runtime>(app: &AppHandle<R>) {
    if element_fullscreen() || page_in_webkit_fullscreen(app) {
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

/// Gives the keyboard back to the page.
pub fn focus_page<R: Runtime>(app: &AppHandle<R>) {
    if let Some(page) = app.get_webview(PAGE) {
        let _ = page.set_focus();
    }
}

/// Loads `url` in the page and tells the bar. Showing a page closes the command bar, and the
/// keyboard follows the page out of it (the toolbar has no field to keep it). The page gets its
/// site's user agent first (phone_mode.rs, which may load a known phone address's desktop one).
pub fn navigate<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    // A summon's page webview may not be built yet (LJ19.4): it is now.
    ensure_page(app)?;
    if app.get_webview(PAGE).is_some() {
        let was_open = overlay();
        set_overlay(app, false);
        load(app, phone_mode::prepare(app, url))?;
        if was_open {
            focus_page(app);
        }
    }
    Ok(())
}

/// Loads `url` in the page and tells the bar, leaving the view as it is.
pub fn load<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    ensure_page(app)?;
    if let Some(page) = app.get_webview(PAGE) {
        address().navigating(url.clone());
        page.navigate(url)?;
        publish_url(app);
    }
    Ok(())
}

/// Loads `url` in the page — an extension's panel, or a page an extension asks for — without
/// taking it as the page's address yet: the address and the toolbar change when the page's own
/// load events arrive, so a navigation the guard refuses (or the engine cancels) leaves the real
/// page named. Closes the command bar like `navigate`.
pub fn navigate_pending<R: Runtime>(app: &AppHandle<R>, url: Url) -> tauri::Result<()> {
    ensure_page(app)?;
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

/// The page webview, if the window is open and has one (it comes after the bar, LJ19.4).
pub fn page<R: Runtime>(app: &AppHandle<R>) -> Option<Webview<R>> {
    app.get_webview(PAGE)
}

/// The bar webview, if the window exists.
pub fn bar<R: Runtime>(app: &AppHandle<R>) -> Option<Webview<R>> {
    app.get_webview(BAR)
}

/// Both webviews of the window, if it is open.
pub fn webviews<R: Runtime>(app: &AppHandle<R>) -> Vec<Webview<R>> {
    [BAR, PAGE]
        .into_iter()
        .filter_map(|label| app.get_webview(label))
        .collect()
}

/// The window closed while Little Joan stays ready (LJ19.4): its page is gone, and the bar goes
/// back to how a summon opens it — the command bar over the whole window, an empty field, no rows,
/// no history view, welcome or status — so the next summon shows it at once. Nothing of the page
/// is kept: its address, its trail, its zoom, its status.
pub fn rest<R: Runtime>(app: &AppHandle<R>) {
    *address() = PageAddress::new();
    *back() = BackStack::new();
    *LAST_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
    status::forget();
    zoom::new_window();
    view::new_window();
    if let Some(bar) = app.get_webview(BAR) {
        platform::set_overlay(&bar, None, true);
        let _ = bar.eval("window.littleJoan && window.littleJoan.rest()");
    }
    if let Some(window) = app.get_window(crate::joan::LABEL) {
        apply_layout(&window);
    }
}

/// A summon from ready (LJ19.4): the bar starts over for it — the command bar's first rows, the
/// theme if it changed meanwhile, its field focused — and in smoke tests reports this summon's
/// timing (`timing`). The theme is a checked one (hex colors, a mode name, two numbers), so its
/// JSON is a safe script literal.
pub fn summoned<R: Runtime>(app: &AppHandle<R>, theme: &crate::settings::Theme, timing: bool) {
    let Some(bar) = app.get_webview(BAR) else {
        return;
    };
    let info = serde_json::json!({ "theme": theme, "timing": timing });
    let _ = bar.eval(format!(
        "window.littleJoan && window.littleJoan.summoned({info})"
    ));
}

#[cfg(test)]
mod tests;
