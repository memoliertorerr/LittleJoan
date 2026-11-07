//! The page's sign-in popups at run time (LJ47; popups.rs decides). A popup is a small window of
//! its own (`joan-popup-<n>`) with one web page webview built from the opener's — its WebKit
//! configuration (macOS), WebView2 environment (Windows) or related view (Linux) through
//! `window_features` —, so it shares the page's store and `window.opener` works (the page can be
//! posted to, and it can close itself). It is page trust and less: no capability names it
//! (capability_tests.rs), no script of ours, no extension or app URL (`popups::may_navigate`), no
//! further popups (its new windows go to the main browser), no downloads, no permissions.
//!
//! **Its title** is where it is, never the page's own (popups/title.rs): built titled by its opener
//! — the only script that can run there before a page commits —, changed only when a page commits
//! in it (`Started`: the engines' commit), never to a host it was only asked to open (the security
//! review's spoof: `open(target)`, `stop()`, `document.write`).
//!
//! **The window:** the size it asked for within bounds, centred over the Little Joan window and
//! kept on its screen (never where the page asked; centred on the screen when the window's frame
//! can't be read), owned by it (Windows' owner, macOS' child window, GTK's transient parent) so it
//! stays above it; resizable, not always on top.
//!
//! **Its end** (bar/popup/closing.rs): every close of the Little Joan window closes its popups
//! first, and from then on until a new page is built no new window of the page opens at all (the
//! review: a page that opens one on `blur` while it closes); the popup page's `window.close()`
//! closes it, its close button too, its opener leaving its origin too (LJ48). A popup that closes
//! with the keyboard gives it back to the page. Of Little Joan's keys only Close acts in a popup —
//! it closes the popup, never the window behind it.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use tauri::webview::{NewWindowFeatures, NewWindowResponse, PageLoadEvent, PermissionResponse};
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use url::{Origin, Url};

use crate::browsers::{self, NewWindowFrom};
use crate::extensions;
use crate::extensions::limits::RateLimit;
use crate::joan;
use crate::navigation;
use crate::platform;
use crate::popups::{self, Counts, Decision, Opener, Size};
use crate::smoke;
use crate::state;

mod closing;
mod window;
use closing::destroy_later;
#[cfg(target_os = "linux")]
pub use closing::view_gone;
pub use closing::{any, close_all, closed_by_page, page_closing, page_committed, stop_accepting};
pub use window::{focused, key_pressed, on_window_event};
use window::{frames, owned_by};
#[cfg(target_os = "linux")]
pub use window::{settle, settle_late};

/// An open popup (or one being built, or closing).
struct Popup {
    label: String,
    /// The origin of the page that opened it (it closes when the page leaves it, LJ48).
    opener: Origin,
    /// What it was opened for (WebKitGTK decides on it once it settles).
    #[cfg(target_os = "linux")]
    target: Url,
    /// WebKitGTK: built hidden until `settle`.
    pending: bool,
    /// WebKitGTK: a navigation refused while pending, made once it settles in.
    early: Option<Url>,
    /// Why it is closing (set once; `None` while open).
    closing: Option<&'static str>,
    /// Whether it had the keyboard when it started closing (it gives it back to the page then).
    had_keyboard: bool,
    /// macOS: its `about:blank` loaded (the unload before it goes).
    unloaded: bool,
}

/// The popups, and whether the page takes new windows at all.
struct Popups {
    list: Vec<Popup>,
    /// A page is up: cleared as it starts closing (`stop_accepting`), set when one is built.
    accepting: bool,
    /// In-app popups opened since the page was built (`popups::LIFETIME_MAX`).
    opened: usize,
}

static POPUPS: Mutex<Popups> = Mutex::new(Popups {
    list: Vec::new(),
    accepting: false,
    opened: 0,
});
static NUMBER: AtomicU64 = AtomicU64::new(0);
/// Openings in the last `popups::OPEN_WINDOW`.
static OPENINGS: Mutex<Option<RateLimit>> = Mutex::new(None);

fn tracked() -> MutexGuard<'static, Popups> {
    POPUPS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A page was built (bar/page.rs): its new windows are taken, and its popups counted afresh.
pub fn page_built() {
    let mut state = tracked();
    state.accepting = true;
    state.opened = 0;
}

/// Popups open now (not closing) and opened in the page's life.
fn counts() -> Counts {
    let state = tracked();
    Counts {
        open: state.list.iter().filter(|p| p.closing.is_none()).count(),
        opened: state.opened,
    }
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
/// Joan, the main browser through LJ46's gate and checks, or nowhere (popups.rs decides). While the
/// page is closing, nowhere at all.
pub fn new_window<R: Runtime>(
    app: &AppHandle<R>,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<R> {
    if !tracked().accepting {
        smoke::log("new-window-dropped", "closing");
        return NewWindowResponse::Deny;
    }
    let opener = opener(app);
    let size = size_of(&features);
    let verdict = popups::decide(&opener, &url, size, counts(), rate_ok);
    if let Some(limit) = verdict.limit {
        smoke::log("popup-limited", limit.name());
    }
    match (verdict.decision, opener) {
        (Decision::InApp | Decision::Pending, Opener::WebPage(origin)) => {
            let pending = verdict.decision == Decision::Pending;
            match build(app, &url, size, features, origin, pending) {
                Ok(Some(window)) => NewWindowResponse::Create { window },
                Ok(None) => NewWindowResponse::Deny,
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

/// Sets a popup's title (and says so in the smoke log).
fn set_title<R: Runtime>(window: &WebviewWindow<R>, title: &str) {
    let _ = window.set_title(title);
    smoke::log("popup-title", title);
}

/// Builds the popup window (inside the engine's new-window callback, on the main thread — as
/// tauri's own example does; WebView2 runs it after its event, so building there can't deadlock).
/// `None`: the page started closing meanwhile, so nothing opens.
fn build<R: Runtime>(
    app: &AppHandle<R>,
    target: &Url,
    size: Size,
    features: NewWindowFeatures,
    opener: Origin,
    pending: bool,
) -> tauri::Result<Option<WebviewWindow<R>>> {
    let joan = app
        .get_window(joan::LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    let label = popups::label(NUMBER.fetch_add(1, Ordering::SeqCst) + 1);
    let asked = match size {
        Size::Given(width, height) => Some((width, height)),
        _ => None,
    };
    let (width, height) = popups::clamp_size(asked);
    // Its opener's place until a page commits in it (never the target's: it may never load).
    let title = popups::opener_title(&opener);
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
    builder = match frames(&joan) {
        Some((over, screen)) => {
            let (x, y) = popups::place(over, (width, height), screen);
            builder.position(x, y)
        }
        // Never where the page asked.
        None => builder.center(),
    };
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
    {
        // Under the lock `stop_accepting` takes: a page that started closing gets nothing, and
        // one that starts closing from here on finds this popup in the list.
        let mut state = tracked();
        if !state.accepting {
            smoke::log("new-window-dropped", "closing");
            return Ok(None);
        }
        state.opened += 1;
        state.list.push(Popup {
            label: label.clone(),
            opener,
            #[cfg(target_os = "linux")]
            target: target.clone(),
            pending,
            early: None,
            closing: None,
            had_keyboard: false,
            unloaded: false,
        });
    }
    let window = match builder.build() {
        Ok(window) => window,
        Err(err) => {
            tracked().list.retain(|p| p.label != label);
            return Err(err);
        }
    };
    // The page started closing while it was built (its close found no window yet): it goes now.
    if tracked()
        .list
        .iter()
        .any(|p| p.label == label && p.closing.is_some())
    {
        destroy_later(app, &label);
        return Ok(None);
    }
    // Keys, `window.close()`, and (WebKitGTK) the size it asked for and its downloads.
    platform::install_popup(app, &window, pending);
    smoke::log("popup-title", &title);
    if pending {
        smoke::log("popup-pending", "");
    } else {
        // What it was opened for (its title is its opener's until a page commits).
        smoke::log("popup-opened", &popups::title_for(target));
    }
    Ok(Some(window))
}

/// The popup's navigation guard: what a popup may load (popups.rs). WebKitGTK: a popup not settled
/// yet loads nothing but `about:blank` — the navigation is made once it shows. A closing popup
/// loads nothing but `about:blank` (its unload).
fn may_navigate(label: &str, url: &Url) -> bool {
    if !popups::may_navigate(url) {
        smoke::log("popup-navigation-refused", url.as_str());
        return false;
    }
    if url.scheme() == "about" {
        return true;
    }
    let mut state = tracked();
    let Some(popup) = state.list.iter_mut().find(|p| p.label == label) else {
        return true;
    };
    if popup.closing.is_some() {
        smoke::log("popup-navigation-refused", "closing");
        return false;
    }
    if popup.pending {
        popup.early = Some(url.clone());
        smoke::log("popup-early", "");
        return false;
    }
    true
}

/// A popup's load events: a page committed (`Started`) sets its title — the opener's place for a
/// blank document —; a finished one (macOS) may be its unload.
fn loaded<R: Runtime>(window: &WebviewWindow<R>, event: PageLoadEvent, url: &Url) {
    match event {
        PageLoadEvent::Started => {
            let opener = tracked()
                .list
                .iter()
                .find(|p| p.label == window.label())
                .map(|p| p.opener.clone());
            if let Some(opener) = opener {
                set_title(window, &popups::title_at(url, &opener));
            }
        }
        PageLoadEvent::Finished => {
            if url.scheme() == "about"
                && let Some(popup) = tracked()
                    .list
                    .iter_mut()
                    .find(|p| p.label == window.label() && p.closing.is_some())
            {
                popup.unloaded = true;
            }
            smoke::log("popup-loaded", &popups::title_for(url));
        }
    }
}
