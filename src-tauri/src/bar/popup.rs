//! The page's sign-in popups at run time (LJ47; popups.rs decides). A popup is a small window of
//! its own (`joan-popup-<n>`) with one web page webview built from the opener's — its WebKit
//! configuration (macOS), WebView2 environment (Windows) or related view (Linux) through
//! `window_features` —, so it shares the page's store and `window.opener` works (the page can be
//! posted to, and it can close itself). It is page trust and less: no capability names it
//! (capability_tests.rs), no script of ours, no extension or app URL (`popups::may_navigate`), no
//! further popups (its new windows go to the main browser), no downloads, no permissions. Its title
//! is where it is (`popups::title_for`), set here on every load, never the page's own.
//!
//! **The window:** the size it asked for within bounds, centred over the Little Joan window and
//! kept on its screen (never where the page asked), owned by it (Windows' owner, macOS' child
//! window, GTK's transient parent) so it stays above it; resizable, not always on top.
//!
//! **Its end:** every close of the Little Joan window closes its popups first (`close_all`,
//! `popup-closed hibernate`); the popup page's `window.close()` closes it (`popup-closed by-page`:
//! WebView2's `WindowCloseRequested`, WebKit's `webViewDidClose:`, WebKitGTK's `close` — wired in
//! platform/, as wry only destroys the webview); its close button too (`popup-closed user`). A popup
//! that closes gives the keyboard back to the page. Nothing of a popup outlives the window.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use tauri::webview::{NewWindowFeatures, NewWindowResponse, PageLoadEvent, PermissionResponse};
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window,
    WindowEvent,
};
use url::Url;

use crate::browsers::{self, NewWindowFrom};
use crate::extensions;
use crate::extensions::limits::RateLimit;
use crate::joan;
use crate::layout::Rect;
use crate::navigation;
use crate::platform;
use crate::popups::{self, Decision, Opener, Size};
use crate::smoke;
use crate::state;

/// An open popup (or one being built, or closing).
struct Popup {
    label: String,
    /// What it was opened for.
    target: Url,
    /// WebKitGTK: built hidden until `settle`.
    pending: bool,
    /// WebKitGTK: a navigation refused while pending, made once it settles in.
    early: Option<Url>,
    /// Why it is closing (set once; `None` while open).
    closing: Option<&'static str>,
    /// macOS: its `about:blank` loaded (the unload before it goes).
    unloaded: bool,
}

static POPUPS: Mutex<Vec<Popup>> = Mutex::new(Vec::new());
static NUMBER: AtomicU64 = AtomicU64::new(0);
/// Openings in the last `popups::OPEN_WINDOW`.
static OPENINGS: Mutex<Option<RateLimit>> = Mutex::new(None);

/// How long a closing waits for a popup's `about:blank` (macOS), and for its window to go.
#[cfg(target_os = "macos")]
const UNLOAD_WAIT: Duration = Duration::from_millis(400);
const GONE_WAIT: Duration = Duration::from_secs(2);

fn popups() -> MutexGuard<'static, Vec<Popup>> {
    POPUPS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Popups open now (not closing).
fn open_count() -> usize {
    popups().iter().filter(|p| p.closing.is_none()).count()
}

fn rate_ok() -> bool {
    OPENINGS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert_with(|| RateLimit::new(popups::OPEN_WINDOW, popups::OPEN_MAX))
        .allow("popup", Instant::now())
}

/// Who asked: a web page in the page webview, or an extension's panel / the app's own page.
fn opener<R: Runtime>(app: &AppHandle<R>) -> Opener {
    if extensions::session::showing()
        .or_else(extensions::session::committed)
        .is_some()
    {
        return Opener::Other;
    }
    Opener::of(super::page_url(app).as_ref())
}

/// What the request said about its size: WebKitGTK says nothing when it asks (`Unknown`).
fn size_of(features: &NewWindowFeatures) -> Size {
    if cfg!(target_os = "linux") {
        return Size::Unknown;
    }
    features
        .size()
        .map_or(Size::None, |size| Size::Given(size.width, size.height))
}

/// The page webview's new-window request (`target=_blank`, `window.open`): a popup inside Little
/// Joan, the main browser through LJ46's gate and checks, or nowhere (popups.rs decides).
pub fn new_window<R: Runtime>(
    app: &AppHandle<R>,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<R> {
    let opener = opener(app);
    let size = size_of(&features);
    let verdict = popups::decide(&opener, &url, size, open_count(), rate_ok);
    if let Some(limit) = verdict.limit {
        smoke::log("popup-limited", limit.name());
    }
    match (verdict.decision, opener) {
        (Decision::InApp | Decision::Pending, Opener::WebPage(_)) => {
            let pending = verdict.decision == Decision::Pending;
            match build(app, &url, size, features, pending) {
                Ok(window) => NewWindowResponse::Create { window },
                Err(err) => {
                    eprintln!("little-joan: could not open the popup: {err}");
                    smoke::log("popup-failed", "");
                    elsewhere(app, url);
                    NewWindowResponse::Deny
                }
            }
        }
        _ if verdict.decision == Decision::Refuse => {
            smoke::log("new-window-refused", url.as_str());
            NewWindowResponse::Deny
        }
        _ => {
            elsewhere(app, url);
            NewWindowResponse::Deny
        }
    }
}

/// Today's behaviour: a web URL to the main browser (the gate; a panel's with `openUrl`'s checks),
/// anything else nowhere.
fn elsewhere<R: Runtime>(app: &AppHandle<R>, url: Url) {
    if navigation::is_external_openable(&url) {
        browsers::hand_off_new_window(app, url, NewWindowFrom::Page);
    } else {
        smoke::log("new-window-refused", url.as_str());
    }
}

/// The Little Joan window's frame and its screen's work area, in logical px.
fn frames<R: Runtime>(window: &Window<R>) -> Option<(Rect, Option<Rect>)> {
    let scale = window.scale_factor().ok()?;
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let over = Rect {
        x: f64::from(position.x) / scale,
        y: f64::from(position.y) / scale,
        width: f64::from(size.width) / scale,
        height: f64::from(size.height) / scale,
    };
    let screen = window.current_monitor().ok().flatten().map(|monitor| {
        let work = monitor.work_area();
        Rect {
            x: f64::from(work.position.x) / scale,
            y: f64::from(work.position.y) / scale,
            width: f64::from(work.size.width) / scale,
            height: f64::from(work.size.height) / scale,
        }
    });
    Some((over, screen))
}

/// Builds the popup window (inside the engine's new-window callback, on the main thread — as
/// tauri's own example does; WebView2 runs it after its event, so building there can't deadlock).
fn build<R: Runtime>(
    app: &AppHandle<R>,
    target: &Url,
    size: Size,
    features: NewWindowFeatures,
    pending: bool,
) -> tauri::Result<WebviewWindow<R>> {
    let joan = app
        .get_window(joan::LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    let label = popups::label(NUMBER.fetch_add(1, Ordering::SeqCst) + 1);
    let asked = match size {
        Size::Given(width, height) => Some((width, height)),
        _ => None,
    };
    let (width, height) = popups::clamp_size(asked);
    let title = popups::title_for(target);
    // macOS: `window.close()` reaches Little Joan only through the delegate's webViewDidClose:,
    // which WebKit looks for when the popup's delegate is set (platform/mac/popups.rs).
    platform::prepare_popups(app, &features);
    // Built blank, as tauri's own example does: the engine then loads the request in it.
    let blank = Url::parse("about:blank").expect("about:blank is a valid URL");
    let data_dir = state::with(app, |s| s.webview_data_dir.clone());
    // The opener's configuration / environment / related view first (and the size and place it
    // asked for, which the next calls replace).
    let mut builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(blank))
        .window_features(features)
        .title(&title)
        .inner_size(width, height)
        .min_inner_size(popups::MIN_SIZE.0, popups::MIN_SIZE.1)
        .resizable(true)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(false)
        .visible(!pending)
        .focused(!pending)
        .on_navigation({
            let label = label.clone();
            move |url| may_navigate(&label, url)
        })
        .on_new_window({
            // No popup opens another: its new windows go to the main browser (the gate), or
            // nowhere.
            let app = app.clone();
            move |url, _features| {
                smoke::log("popup-new-window", "");
                if navigation::is_external_openable(&url) {
                    browsers::hand_off_new_window(&app, url, NewWindowFrom::Popup);
                } else {
                    smoke::log("new-window-refused", url.as_str());
                }
                NewWindowResponse::Deny
            }
        })
        .on_page_load(|window, payload| loaded(&window, payload.event(), payload.url()))
        .on_permission_request(|_, kind| {
            smoke::log("permission-refused", &format!("popup {kind}"));
            PermissionResponse::Deny
        });
    if let Some((over, screen)) = frames(&joan) {
        let (x, y) = popups::place(over, (width, height), screen);
        builder = builder.position(x, y);
    }
    // WebKitGTK's downloads are decided in platform/linux/downloads.rs, which refuses a popup's.
    #[cfg(not(target_os = "linux"))]
    {
        builder = builder.on_download(|_, event| {
            if matches!(event, tauri::webview::DownloadEvent::Requested { .. }) {
                smoke::log("download-refused", "popup");
            }
            false
        });
    }
    // The page's data directory (WebView2 takes the opener's environment, which already has it).
    if let Some(dir) = data_dir {
        builder = builder.data_directory(dir);
    }
    builder = owned_by(builder, &joan)?;
    popups().push(Popup {
        label: label.clone(),
        target: target.clone(),
        pending,
        early: None,
        closing: None,
        unloaded: false,
    });
    let window = match builder.build() {
        Ok(window) => window,
        Err(err) => {
            popups().retain(|p| p.label != label);
            return Err(err);
        }
    };
    // Keys, `window.close()`, and (WebKitGTK) the size it asked for and its downloads.
    platform::install_popup(app, &window, pending);
    if pending {
        smoke::log("popup-pending", "");
    } else {
        smoke::log("popup-opened", &title);
    }
    Ok(window)
}

/// The popup stays above the Little Joan window: its owner (Windows), its child (macOS), transient
/// for it (GTK).
#[cfg(windows)]
fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.owner_raw(joan.hwnd()?))
}

#[cfg(target_os = "macos")]
fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.parent_raw(joan.ns_window()?))
}

#[cfg(target_os = "linux")]
fn owned_by<'a, R: Runtime>(
    builder: WebviewWindowBuilder<'a, R, AppHandle<R>>,
    joan: &Window<R>,
) -> tauri::Result<WebviewWindowBuilder<'a, R, AppHandle<R>>> {
    Ok(builder.transient_for_raw(&joan.gtk_window()?))
}

/// The popup's navigation guard: what a popup may load (popups.rs). WebKitGTK: a popup not settled
/// yet loads nothing but `about:blank` — the navigation is made once it shows.
fn may_navigate(label: &str, url: &Url) -> bool {
    if !popups::may_navigate(url) {
        smoke::log("popup-navigation-refused", url.as_str());
        return false;
    }
    if url.scheme() != "about"
        && let Some(popup) = popups().iter_mut().find(|p| p.label == label && p.pending)
    {
        popup.early = Some(url.clone());
        smoke::log("popup-early", "");
        return false;
    }
    true
}

/// A popup's load events: its title is where it is, set on every one.
fn loaded<R: Runtime>(window: &WebviewWindow<R>, event: PageLoadEvent, url: &Url) {
    let title = popups::title_for(url);
    let _ = window.set_title(&title);
    if event == PageLoadEvent::Finished {
        if url.scheme() == "about"
            && let Some(popup) = popups()
                .iter_mut()
                .find(|p| p.label == window.label() && p.closing.is_some())
        {
            popup.unloaded = true;
        }
        smoke::log("popup-loaded", &title);
    }
}

/// WebKitGTK's `ready-to-show` (main thread): the engine has set the popup's window properties —
/// `asked`, the size it asked for, if any —, so it shows, goes to the browser, or goes nowhere.
pub fn settle<R: Runtime>(app: &AppHandle<R>, label: &str, asked: Option<(f64, f64)>) {
    let Some((target, early)) = popups()
        .iter_mut()
        .find(|p| p.label == label && p.pending && p.closing.is_none())
        .map(|p| {
            p.pending = false;
            (p.target.clone(), p.early.take())
        })
    else {
        return;
    };
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    match popups::settle(&target, asked.is_some()) {
        Decision::InApp | Decision::Pending => {
            let (width, height) = popups::clamp_size(asked);
            let _ = window.set_size(tauri::LogicalSize::new(width, height));
            if let Some(joan) = app.get_window(joan::LABEL)
                && let Some((over, screen)) = frames(&joan)
            {
                let (x, y) = popups::place(over, (width, height), screen);
                let _ = window.set_position(tauri::LogicalPosition::new(x, y));
            }
            let _ = window.show();
            let _ = window.set_focus();
            smoke::log("popup-opened", &popups::title_for(&target));
            if let Some(url) = early {
                let _ = window.navigate(url);
            }
        }
        decision => {
            smoke::log("popup-settled", "unsized");
            mark_closing(label, "unshown");
            destroy_later(app, label);
            match decision {
                Decision::Browser => elsewhere(app, target),
                _ => smoke::log("new-window-refused", target.as_str()),
            }
        }
    }
}

/// Marks a popup closing for `why` (once) and logs `popup-closed <why>` (but for one never shown).
/// Returns whether this call marked it.
fn mark_closing(label: &str, why: &'static str) -> bool {
    let mut all = popups();
    let Some(popup) = all
        .iter_mut()
        .find(|p| p.label == label && p.closing.is_none())
    else {
        return false;
    };
    popup.closing = Some(why);
    if why != "unshown" {
        smoke::log("popup-closed", why);
    }
    true
}

/// Closes a popup: the page called `window.close()` (`by-page`), the user's key (`user`), its
/// opener left (`opener-left`). Never inside the engine's callback: the window goes from a thread
/// of its own.
pub fn close<R: Runtime>(app: &AppHandle<R>, label: &str, why: &'static str) {
    if mark_closing(label, why) {
        destroy_later(app, label);
    }
}

fn destroy_later<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let app = app.clone();
    let label = label.to_owned();
    let spawned = std::thread::Builder::new()
        .name("little-joan-popup-close".into())
        .spawn(move || {
            if let Some(window) = app.get_webview_window(&label) {
                unload(std::slice::from_ref(&window));
                let _ = window.destroy();
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not close the popup: {err}");
    }
}

/// macOS: popups go to `about:blank` first, as the page does — WebKit's processes and a page's
/// audio otherwise outlive the web view (LJ9) — at most `UNLOAD_WAIT`.
#[cfg(target_os = "macos")]
fn unload<R: Runtime>(windows: &[WebviewWindow<R>]) {
    let Ok(blank) = Url::parse("about:blank") else {
        return;
    };
    for window in windows {
        let _ = window.navigate(blank.clone());
    }
    let labels: Vec<&str> = windows.iter().map(WebviewWindow::label).collect();
    let deadline = Instant::now() + UNLOAD_WAIT;
    while Instant::now() < deadline {
        let unloaded = popups()
            .iter()
            .filter(|p| labels.contains(&p.label.as_str()))
            .all(|p| p.unloaded);
        if unloaded {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    smoke::log("popup-unload", "timeout");
}

/// WebView2 and WebKitGTK end a popup's page with its webview (as the page's, measured in CI).
#[cfg(not(target_os = "macos"))]
fn unload<R: Runtime>(_windows: &[WebviewWindow<R>]) {}

/// Every popup closes (`why`: `hibernate` — the Little Joan window is closing; bar/page.rs calls
/// this before the page is released). Off the main thread; returns once they are gone (at most
/// `GONE_WAIT`), so nothing of them outlives the window.
pub fn close_all<R: Runtime>(app: &AppHandle<R>, why: &'static str) {
    let labels: Vec<String> = popups()
        .iter()
        .filter(|p| p.closing.is_none())
        .map(|p| p.label.clone())
        .collect();
    let labels: Vec<String> = labels
        .into_iter()
        .filter(|label| mark_closing(label, why))
        .collect();
    let all: Vec<String> = popups().iter().map(|p| p.label.clone()).collect();
    let windows: Vec<WebviewWindow<R>> = all
        .iter()
        .filter_map(|label| app.get_webview_window(label))
        .collect();
    if windows.is_empty() {
        return;
    }
    unload(&windows);
    for window in &windows {
        let _ = window.destroy();
    }
    let deadline = Instant::now() + GONE_WAIT;
    while Instant::now() < deadline
        && windows
            .iter()
            .any(|w| app.get_webview_window(w.label()).is_some())
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !labels.is_empty() {
        smoke::log("popups-closed", &labels.len().to_string());
    }
}

/// The popup page called `window.close()` (platform/: WebView2's `WindowCloseRequested`,
/// WebKit's `webViewDidClose:`, the WebKitGTK view's end).
pub fn closed_by_page<R: Runtime>(app: &AppHandle<R>, label: &str) {
    close(app, label, "by-page");
}

/// WebKitGTK: the popup's web view went. wry destroys it when the page calls `window.close()`,
/// leaving its window; a view that goes while its popup isn't closing went so.
#[cfg(target_os = "linux")]
pub fn view_gone<R: Runtime>(app: &AppHandle<R>, label: &str) {
    closed_by_page(app, label);
}

/// Window events of popups (lib.rs): the close button marks it the user's; once it is gone it
/// leaves the list, and the keyboard goes back to the page.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    let label = window.label();
    if !popups::is_popup_label(label) {
        return;
    }
    match event {
        WindowEvent::CloseRequested { .. } => {
            mark_closing(label, "user");
        }
        WindowEvent::Destroyed => {
            platform::forget_popup(label);
            let gone = {
                let mut all = popups();
                let at = all.iter().position(|p| p.label == label);
                at.map(|at| all.remove(at))
            };
            let why = match gone {
                Some(Popup {
                    closing: Some(why), ..
                }) => why,
                Some(_) => {
                    smoke::log("popup-closed", "user");
                    "user"
                }
                None => return,
            };
            if matches!(why, "user" | "by-page") {
                give_back_keyboard(window.app_handle());
            }
        }
        _ => {}
    }
}

/// A popup closed while the Little Joan window is open: the page gets the keyboard again.
fn give_back_keyboard<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if !joan::is_open(&app) {
            return;
        }
        if let Some(window) = app.get_window(joan::LABEL) {
            let _ = window.set_focus();
        }
        super::focus_page(&app);
    });
}
