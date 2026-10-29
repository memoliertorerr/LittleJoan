//! The page webview (`joan-page`) at run time (LJ19.4). It has no IPC at all (Ground Rule 18) and is
//! built fresh for every summon — after the bar is on screen, so the bar's web process starts alone
//! (or at once when a summon carries a URL, or when something needs the page first: Enter in the
//! command bar, an extension's panel) — and closed with every close, whatever "After closing" says:
//! nothing of a web page survives a close (Ground Rule 9). Everything that needs the page goes
//! through `ensure_page`, so a window without one yet copes.
//!
//! **No flash:** a webview added to a window on screen would show for a frame before it is laid out
//! (Linux packs it into the window's box beside the command bar; Windows and macOS place it over
//! the bar). On macOS and Linux it is added, hidden and packed in one task on the main thread, so
//! nothing paints in between; on Windows (where building a webview on the main thread can deadlock)
//! it is added zero-sized and hidden, and laid out afterwards. It never takes the keyboard from the
//! bar's field (`focused(false)`).
//!
//! **Closing:** the session cookies are kept first (Windows, cookies.rs), downloads under way end
//! (bar/download.rs), on macOS the page is unloaded (`about:blank`) before it goes — WKWebView's
//! processes otherwise outlive it, and a video's audio with them (LJ9) —, then `Webview::close`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use tauri::webview::{NewWindowResponse, PageLoadEvent};
use tauri::{
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, Webview, WebviewBuilder, WebviewUrl,
    Window,
};

use super::{
    BAR, BAR_READY, PAGE, Ready, address, back, can_go_back, page_url, publish_url, tell_back,
};
use crate::back_stack::BackStack;
use crate::cookies::{self, SessionCookies};
use crate::extensions;
use crate::history_store;
use crate::joan;
use crate::keys;
use crate::layout::{Mode, Rect, layout};
use crate::navigation;
use crate::page_address::PageAddress;
use crate::phone_mode;
use crate::platform;
use crate::smoke;
use crate::state;
use crate::summon_timing::{self, Mark};
use crate::tray;

/// Held while the page webview is built or closed, so a summon's deferred build, a navigation that
/// needs the page now and a close never overlap.
static BUILDING: Mutex<()> = Mutex::new(());
/// Bumped with every summon and every close: a build planned for one summon never runs in a later
/// one (or after its window closed).
static SUMMON: AtomicU64 = AtomicU64::new(0);
/// macOS: the page's `about:blank` (its unload before closing) finished loading.
static BLANK: Ready = Ready::new();

/// How long a cold summon's page waits for the bar's script before it is built anyway.
const BAR_WAIT: Duration = Duration::from_secs(3);
/// How long after the bar is on screen the page is built: the bar's first frame goes first.
const AFTER_BAR: Duration = Duration::from_millis(50);
/// The longest a close waits for the page's `about:blank` (macOS).
#[cfg(target_os = "macos")]
const BLANK_WAIT: Duration = Duration::from_millis(400);

fn building() -> std::sync::MutexGuard<'static, ()> {
    BUILDING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A summon starts (or a close): builds planned before it are off. Returns this summon's number.
pub fn page_summon() -> u64 {
    SUMMON.fetch_add(1, Ordering::SeqCst) + 1
}

/// Builds the page webview if the window exists and has none yet. Never on the main thread — it
/// waits for it (and WebView2 can deadlock there) —: from there it does nothing, which only the
/// page's own callbacks could ask, and they have a page.
pub fn ensure_page<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if app.get_webview(PAGE).is_some() {
        return Ok(());
    }
    let Some(window) = app.get_window(joan::LABEL) else {
        return Ok(());
    };
    if platform::on_main_thread() {
        eprintln!("little-joan: the page webview can't be built on the main thread");
        return Ok(());
    }
    let _building = building();
    if app.get_webview(PAGE).is_some() {
        return Ok(());
    }
    build(app, &window)
}

/// Builds the page in the background once the bar is on screen: a cold summon waits for the bar's
/// script first (its first frame follows it), a summon from ready only lets the bar's frame go
/// first. Nothing is built if the window closed (or another summon came) meanwhile.
pub fn build_after_bar<R: Runtime>(app: &AppHandle<R>, cold: bool) {
    let summon = SUMMON.load(Ordering::SeqCst);
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-page".into())
        .spawn(move || {
            if cold && !BAR_READY.wait(BAR_WAIT) {
                smoke::log("page-wait", "bar-timeout");
            }
            std::thread::sleep(AFTER_BAR);
            if SUMMON.load(Ordering::SeqCst) != summon {
                return;
            }
            if let Err(err) = ensure_page(&app) {
                eprintln!("little-joan: could not build the page webview: {err}");
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not start the page's builder: {err}");
    }
}

fn build<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) -> tauri::Result<()> {
    // A new page: its address, its trail, its zoom and its agent start fresh.
    *address() = PageAddress::new();
    *back() = BackStack::new();
    super::zoom::new_window();
    phone_mode::reset();
    let data_dir = state::with(app, |s| s.webview_data_dir.clone());
    let (size, scale) = (window.inner_size()?, window.scale_factor()?);
    let logical = size.to_logical::<f64>(scale);
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        width: logical.width,
        height: logical.height,
    };
    let rect = layout(logical.width, logical.height, Mode::Toolbar)
        .page
        .unwrap_or(whole);
    let page = add(window, data_dir, rect)?;
    platform::install_page(&page);
    super::view::page_added(window, &page);
    summon_timing::mark(Mark::PageBuilt);
    smoke::log("page-built", "");
    // Still before any web page loads (the start page under the command bar needs neither): the
    // kept session cookies (Windows) and the page's user agent for its width (phone_mode.rs).
    restore_session_cookies(app, &page);
    summon_timing::mark(Mark::CookiesRestored);
    phone_mode::apply(app, false);
    summon_timing::mark(Mark::AgentSet);
    super::watch_url(app);
    // CI only (smoke log): WebKit's fullscreen state on macOS (LJ36).
    platform::watch_webkit_fullscreen(app);
    Ok(())
}

/// Windows: added from this thread (WebView2 builds its controller asynchronously; the main thread
/// must stay free), zero-sized and hidden at once, so it never covers the command bar.
#[cfg(windows)]
fn add<R: Runtime>(
    window: &Window<R>,
    data_dir: Option<PathBuf>,
    _rect: Rect,
) -> tauri::Result<Webview<R>> {
    let page = window.add_child(
        page_builder(data_dir),
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(0.0, 0.0),
    )?;
    let _ = page.hide();
    Ok(page)
}

/// macOS and Linux: added, hidden and (Linux) taken out of the window's box in one task on the
/// main thread, so nothing paints in between.
#[cfg(not(windows))]
fn add<R: Runtime>(
    window: &Window<R>,
    data_dir: Option<PathBuf>,
    rect: Rect,
) -> tauri::Result<Webview<R>> {
    let (sent, done) = std::sync::mpsc::channel();
    let target = window.clone();
    window.run_on_main_thread(move || {
        let added = target.add_child(
            page_builder(data_dir),
            LogicalPosition::new(rect.x, rect.y),
            LogicalSize::new(rect.width, rect.height),
        );
        if let Ok(page) = &added {
            let _ = page.hide();
            if let Some(bar) = target.app_handle().get_webview(BAR) {
                platform::set_overlay(&bar, Some(page), super::overlay());
            }
        }
        let _ = sent.send(added);
    })?;
    done.recv_timeout(Duration::from_secs(10))
        .map_err(|_| tauri::Error::FailedToReceiveMessage)?
}

/// The page's builder: web pages only, its title and load events, new windows to the default
/// browser, downloads to the Downloads folder.
fn page_builder<R: Runtime>(data_dir: Option<PathBuf>) -> WebviewBuilder<R> {
    let mut page = WebviewBuilder::new(PAGE, WebviewUrl::App("index.html".into()))
        // Never the keyboard: the bar's field has it (keys typed right after a summon land there).
        .focused(false)
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
                    // A summon with a URL: its web page is on its way (LJ19.1; not the start page).
                    if !keys::is_app_url(&keys::app_origin(), payload.url())
                        && navigation::is_external_openable(payload.url())
                    {
                        summon_timing::mark(Mark::PageStarted);
                    }
                    // What the toolbar shows, and what the extension handler serves beyond a
                    // panel's document: an extension only once its panel committed here.
                    extensions::session::page_committed(webview.app_handle(), payload.url());
                    // A page the page itself opened gets its site's user agent (phone_mode.rs).
                    phone_mode::page_started(webview.app_handle(), payload.url());
                }
                PageLoadEvent::Finished => {
                    address().finished(payload.url().clone());
                    if payload.url().scheme() == "about" {
                        BLANK.set();
                    }
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
    // Downloads go to the Downloads folder, the toolbar says how they went (bar/download.rs).
    // WebKitGTK's come through platform/linux.rs instead: wry's handler there is the web
    // context's, and the first webview's decides for all of them (Landmines).
    #[cfg(not(target_os = "linux"))]
    {
        page = page.on_download(super::download::on_download);
    }
    // Both webviews of a window must share one data directory (WebView2 requires it).
    if let Some(dir) = data_dir {
        page = page.data_directory(dir);
    }
    page
}

/// Closes the page webview, if there is one: its session cookies are kept (Windows), downloads
/// under way end, macOS unloads it first. Nothing of it is kept: its address and trail go too.
pub fn close_page<R: Runtime>(app: &AppHandle<R>) {
    page_summon();
    let _building = building();
    let Some(page) = app.get_webview(PAGE) else {
        return;
    };
    keep_session_cookies(app, &page);
    // A download under way ends with the page (Ground Rule 9).
    super::download::window_closed(app);
    unload(&page);
    if let Err(err) = page.close() {
        eprintln!("little-joan: could not close the page webview: {err}");
    }
    *address() = PageAddress::new();
    *back() = BackStack::new();
    tray::set_page_items_enabled(app, false);
    smoke::log("page-closed", "");
}

/// macOS: the page goes to `about:blank` and gets a moment to unload (its scripts, audio and
/// network end there) before it is closed — at most `BLANK_WAIT`, usually much less.
#[cfg(target_os = "macos")]
fn unload<R: Runtime>(page: &Webview<R>) {
    BLANK.reset();
    if let Ok(blank) = url::Url::parse("about:blank") {
        let _ = page.navigate(blank);
        if !BLANK.wait(BLANK_WAIT) {
            smoke::log("page-unload", "timeout");
        }
    }
}

/// WebView2 and WebKitGTK end a page's processes when its webview goes (measured in CI).
#[cfg(not(target_os = "macos"))]
fn unload<R: Runtime>(_page: &Webview<R>) {}

/// Before the page is closed: keeps its session cookies in memory for the next page, where the
/// engine might drop them with this one (WebView2; cookies.rs). A failed read keeps nothing.
fn keep_session_cookies<R: Runtime>(app: &AppHandle<R>, page: &Webview<R>) {
    if cookies::ENGINE_KEEPS_SESSION_COOKIES {
        return;
    }
    // Off the main thread (inside joan's lifecycle), as WebView2's cookie read requires.
    let kept = match page.cookies() {
        Ok(all) => SessionCookies::from_all(all),
        Err(err) => {
            eprintln!("little-joan: could not read the page's cookies: {err}");
            SessionCookies::default()
        }
    };
    smoke::log("session-cookies-kept", &kept.len().to_string());
    state::with(app, |s| s.session_cookies = kept);
}

/// A new page: sets the kept session cookies again before any web page loads.
fn restore_session_cookies<R: Runtime>(app: &AppHandle<R>, page: &Webview<R>) {
    if cookies::ENGINE_KEEPS_SESSION_COOKIES {
        return;
    }
    let kept = state::with(app, |s| s.session_cookies.clone());
    if kept.is_empty() {
        return;
    }
    for cookie in kept.cookies() {
        if let Err(err) = page.set_cookie(cookie.clone()) {
            eprintln!("little-joan: could not set a kept cookie again: {err}");
        }
    }
    // WebView2's cookie manager answers in order, so once this read returns the cookies above
    // are in place for the first navigation.
    let _ = page.cookies();
    smoke::log("session-cookies-restored", &kept.len().to_string());
}
