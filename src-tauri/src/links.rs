//! Links from other apps (LJ45): Little Joan as a browser the OS can hand http and https links to.
//!
//! A link is untrusted input from outside (Ground Rule 22). It arrives one of two ways:
//! - Windows and Linux start `little-joan --link <url>` (the registry's ProgId command, the desktop
//!   file's `Exec`); single-instance forwards that to the running app like every flag, and cli.rs
//!   takes exactly the argument after `--link` as data, ignoring every other flag beside it.
//! - macOS sends an open-URL Apple event: tao's `application:openURLs:`, Tauri's
//!   `RunEvent::Opened { urls }` (lib.rs's run loop). One can come before setup is done (AppKit
//!   delivers it between "will" and "did finish launching"); it waits here until `ready`.
//!
//! Both go through `accept`: an absolute http(s) URL with a host, at most 4,096 bytes, that the
//! page may load as a typed address may (navigation.rs) — never `file:`, `javascript:`, `data:`,
//! `about:`, an extension (`ljext:`, WebView2's `http://ljext.<id>.localhost`), the app's own
//! origin or Tauri's other hosts. A link is never a search (input.rs isn't used). What opens goes
//! through the toggle table as `Show(Some(url))` — Create, Summon or FocusAndNavigate — so a link
//! is one navigation of the one window, as `--show <url>` already is, and its visit records as a
//! link. The smoke log says `link-opened`, `link-refused <scheme>[ <why>]` or `link-ignored <n>`:
//! never the URL.

use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Runtime};
use url::Url;

use crate::joan;
use crate::keys;
use crate::navigation;
use crate::smoke;
use crate::summon_timing::Source;
use crate::toggle::Command;

/// The longest link that opens (history.rs records nothing longer either).
pub const MAX_LINK_BYTES: usize = 4096;
/// Tauri's own hosts on Windows (`http://<scheme>.localhost`): the app's pages and its IPC.
const TAURI_HOSTS: [&str; 2] = ["tauri.localhost", "ipc.localhost"];
/// Links that wait for setup (macOS), at most.
const MAX_WAITING: usize = 64;

/// Why a link doesn't open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Not an absolute URL at all (`--quit`, an empty argument).
    Invalid,
    /// Longer than MAX_LINK_BYTES.
    TooLong,
    /// Not a web scheme (`file:`, `javascript:`, `data:`, `about:`, `ljext:` …).
    Scheme(&'static str),
    /// http(s) without a host.
    NoHost,
    /// The app's own origin, or Tauri's IPC host.
    App,
    /// An extension's page in WebView2's http form.
    Extension,
    /// Anything else the typed-address rules refuse (navigation.rs).
    NotWeb,
    /// A page Little Joan just handed to a browser, sent straight back (LJ46's loop guard: the
    /// default it went to was Little Joan after all).
    Loop,
}

impl Refusal {
    /// The smoke log's detail: the scheme, then why — never anything else of the link.
    pub fn detail(self) -> &'static str {
        match self {
            Refusal::Invalid => "invalid",
            Refusal::TooLong => "too-long",
            Refusal::Scheme(scheme) => scheme,
            Refusal::NoHost => "http no-host",
            Refusal::App => "http app",
            Refusal::Extension => "http extension",
            Refusal::NotWeb => "http not-web",
            Refusal::Loop => "http loop",
        }
    }
}

/// A scheme as the log may name it: the usual ones by name, anything else as `other` (a scheme is
/// part of the link, and only these few words ever reach the log).
fn scheme_name(scheme: &str) -> &'static str {
    const KNOWN: [&str; 13] = [
        "file",
        "javascript",
        "data",
        "about",
        "blob",
        "ljext",
        "tauri",
        "ftp",
        "mailto",
        "vbscript",
        "chrome",
        "view-source",
        "ws",
    ];
    KNOWN
        .into_iter()
        .find(|known| known.eq_ignore_ascii_case(scheme))
        .unwrap_or("other")
}

/// Why `url` may not open from another app, or `None` when it may.
pub fn refusal(url: &Url) -> Option<Refusal> {
    if !matches!(url.scheme(), "http" | "https") {
        return Some(Refusal::Scheme(scheme_name(url.scheme())));
    }
    let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
        return Some(Refusal::NoHost);
    };
    if url.as_str().len() > MAX_LINK_BYTES {
        Some(Refusal::TooLong)
    } else if crate::extensions::urls::is_ext_like(url) {
        Some(Refusal::Extension)
    } else if keys::is_app_url(&keys::app_origin(), url) || TAURI_HOSTS.contains(&host.as_str()) {
        Some(Refusal::App)
    } else if !navigation::is_allowed_navigation(url) || !navigation::is_external_openable(url) {
        // The typed-address rules, last: everything above already covers what they refuse.
        Some(Refusal::NotWeb)
    } else {
        None
    }
}

/// The link to open for `url`, or `None` when it is refused (`refusal` says why).
pub fn accept(url: &Url) -> Option<Url> {
    refusal(url).is_none().then(|| url.clone())
}

/// A link as text (`--link <text>`): parsed as an absolute URL — never a search — then `accept`.
pub fn parse(text: &str) -> Result<Url, Refusal> {
    if text.len() > MAX_LINK_BYTES {
        return Err(Refusal::TooLong);
    }
    let url = Url::parse(text).map_err(|_| Refusal::Invalid)?;
    match refusal(&url) {
        None => Ok(url),
        Some(why) => Err(why),
    }
}

/// What a batch of opened URLs (macOS) comes to: only the first acceptable one opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    pub open: Option<Url>,
    pub refused: Vec<Refusal>,
    /// Acceptable links after the first, which don't open (`link-ignored <n>`).
    pub ignored: usize,
}

pub fn pick(urls: &[Url]) -> Batch {
    let mut batch = Batch {
        open: None,
        refused: Vec::new(),
        ignored: 0,
    };
    for url in urls {
        match (refusal(url), &batch.open) {
            (Some(why), _) => batch.refused.push(why),
            (None, None) => batch.open = Some(url.clone()),
            (None, Some(_)) => batch.ignored += 1,
        }
    }
    batch
}

/// `--link <text>` (a forwarded second launch, or the first launch's own): opens it, or logs why
/// not and does nothing else — no window, no quit, no flag.
pub fn open_text<R: Runtime>(app: &AppHandle<R>, text: &str) {
    match parse(text) {
        Ok(url) => open(app, url),
        Err(why) => refused(why),
    }
}

/// Links that came before setup was done (macOS); `None` once `ready` ran.
static WAITING: Mutex<Option<Vec<Url>>> = Mutex::new(Some(Vec::new()));

/// macOS's open-URL event (`RunEvent::Opened`): the first acceptable link opens.
pub fn opened<R: Runtime>(app: &AppHandle<R>, urls: Vec<Url>) {
    {
        let mut waiting = WAITING.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(list) = waiting.as_mut() {
            let room = MAX_WAITING.saturating_sub(list.len());
            list.extend(urls.into_iter().take(room));
            return;
        }
    }
    open_batch(app, &urls);
}

/// Setup is done: links that came before it are handled now (the end of lib.rs's setup).
pub fn ready<R: Runtime>(app: &AppHandle<R>) {
    let waiting = WAITING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
        .unwrap_or_default();
    if !waiting.is_empty() {
        open_batch(app, &waiting);
    }
}

fn open_batch<R: Runtime>(app: &AppHandle<R>, urls: &[Url]) {
    let batch = pick(urls);
    for why in batch.refused {
        refused(why);
    }
    if batch.ignored > 0 {
        smoke::log("link-ignored", &batch.ignored.to_string());
    }
    if let Some(url) = batch.open {
        open(app, url);
    }
}

fn open<R: Runtime>(app: &AppHandle<R>, url: Url) {
    // The loop guard (LJ46): a page handed on a moment ago that comes back doesn't open again.
    if crate::browsers::bounced(&url) {
        refused(Refusal::Loop);
        return;
    }
    smoke::log("link-opened", "");
    // Create, Summon or FocusAndNavigate (toggle.rs): the window comes to the front — it was asked
    // for — and the command bar closes for the page; the page before stays in the back trail.
    joan::dispatch(app, Command::Show(Some(url)), Source::Link);
}

fn refused(why: Refusal) {
    smoke::log("link-refused", why.detail());
    eprintln!(
        "little-joan: a link from another app was refused ({})",
        why.detail()
    );
}

#[cfg(test)]
mod tests;
