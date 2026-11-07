//! Sign-in popups (LJ47–LJ48): which new-window requests open inside Little Joan, where a popup may
//! go, and how its window is sized, placed and titled. Pure (no Tauri types), tested in
//! popups/tests.rs; bar/popup.rs runs it.
//!
//! "Sign in with Google / Apple / Microsoft / GitHub" opens a popup and waits for it to post back,
//! so such popups open in a small window of their own whose page webview shares the page's store
//! and the opener relationship — a web page with exactly the page's trust: no IPC, no extension or
//! app URLs, no further popups, no downloads, no permissions (Ground Rule 23). Everything else still
//! goes to the main browser (LJ46), as before.
//!
//! **`decide`:** only a web page in the page webview (`http` / `https`, never an extension's panel
//! or one of the app's own pages — those keep today's behaviour) gets an in-app popup, and only for
//! a target a popup may load (`may_navigate`) that either asks for a size (`window.open(url, name,
//! "width=…,height=…")`, as sign-in libraries do) or is a known sign-in flow (`known_sign_in`). A
//! sized `about:blank` opens inside too (libraries open a blank popup, then set its location); an
//! unsized one is refused. At most `MAX_OPEN` popups at once and `OPEN_MAX` openings in any
//! `OPEN_WINDOW`; beyond either a request goes where it would have gone before (the browser, or
//! nowhere for `about:blank`). WebKitGTK says nothing about the size when it asks (wry passes none):
//! there a request that isn't a known sign-in flow is `Pending` — its window is built hidden, and
//! `settle` decides once the engine has set the popup's window properties (`ready-to-show`).
//!
//! **LJ48:** a popup belongs to the page that opened it — when the page webview commits a page of
//! another origin, its popups close (`opener_left`) —, and of Little Joan's keys only Close acts in
//! a popup, closing it (`route_key`).

use std::time::Duration;

use url::{Origin, Url};

use crate::extensions::urls::is_ext_like;
use crate::keymap::KeyAction;
use crate::keys::{self, PageAction};
use crate::layout::Rect;
use crate::navigation;

/// Every popup's label starts so: `joan-popup-<n>`. No capability may match one
/// (capability_tests.rs).
pub const LABEL_PREFIX: &str = "joan-popup-";
/// Popups open at once, at most.
pub const MAX_OPEN: usize = 2;
/// At most `OPEN_MAX` popups opened in any `OPEN_WINDOW` (a page can't open and close them in a
/// loop).
pub const OPEN_MAX: usize = 3;
pub const OPEN_WINDOW: Duration = Duration::from_secs(10);
/// A popup's inner size, logical px: what it asked for, within these bounds.
pub const MIN_SIZE: (f64, f64) = (320.0, 300.0);
pub const MAX_SIZE: (f64, f64) = (1200.0, 900.0);
/// A known sign-in flow that asked for no size.
pub const DEFAULT_SIZE: (f64, f64) = (480.0, 640.0);

/// The popup with number `n`'s label.
pub fn label(n: u64) -> String {
    format!("{LABEL_PREFIX}{n}")
}

/// Whether `label` is a popup's.
pub fn is_popup_label(label: &str) -> bool {
    label
        .strip_prefix(LABEL_PREFIX)
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Who asked for the new window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opener {
    /// A web page in the page webview: its origin (a popup belongs to it, LJ48).
    WebPage(Origin),
    /// An extension's panel, one of the app's own pages, or no page: today's behaviour.
    Other,
}

impl Opener {
    /// The opener the page webview's `url` makes (`None`: no page yet).
    pub fn of(url: Option<&Url>) -> Opener {
        match url {
            Some(url)
                if matches!(url.scheme(), "http" | "https")
                    && url.has_host()
                    && !is_ext_like(url)
                    && !navigation::intercepted_by_app(url)
                    && !keys::is_app_url(&keys::app_origin(), url) =>
            {
                Opener::WebPage(url.origin())
            }
            _ => Opener::Other,
        }
    }
}

/// What the request said about the window's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// `width=` and `height=`, logical px.
    Given(f64, f64),
    /// None asked for.
    None,
    /// The engine didn't say (WebKitGTK asks before it knows).
    Unknown,
}

/// Where a new-window request goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// A popup window inside Little Joan.
    InApp,
    /// Built hidden, decided by `settle` (WebKitGTK only).
    Pending,
    /// The main browser (LJ46's gate and checks).
    Browser,
    /// Nowhere (`new-window-refused`).
    Refuse,
}

/// Why a request that wanted a popup didn't get one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limit {
    /// `MAX_OPEN` popups are open.
    Cap,
    /// `OPEN_MAX` opened in the last `OPEN_WINDOW`.
    Rate,
}

impl Limit {
    pub fn name(self) -> &'static str {
        match self {
            Limit::Cap => "cap",
            Limit::Rate => "rate",
        }
    }
}

/// `decide`'s answer, and the limit that turned a popup away (for the smoke log).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    pub decision: Decision,
    pub limit: Option<Limit>,
}

impl Verdict {
    fn plain(decision: Decision) -> Verdict {
        Verdict {
            decision,
            limit: None,
        }
    }
}

/// Today's behaviour (LJ46): web URLs to the main browser, the rest nowhere.
fn elsewhere(target: &Url) -> Decision {
    if navigation::is_external_openable(target) {
        Decision::Browser
    } else {
        Decision::Refuse
    }
}

/// What a request for `target` with `size` wants, the opener and the limits aside.
fn wanted(target: &Url, size: Size) -> Decision {
    let blank = is_blank(target);
    if known_sign_in(target) {
        return Decision::InApp;
    }
    match size {
        Size::Given(..) => Decision::InApp,
        Size::Unknown => Decision::Pending,
        Size::None if blank => Decision::Refuse,
        Size::None => elsewhere(target),
    }
}

/// Where a new-window request goes (the module's docs). `open`: popups open now; `rate_ok` is
/// asked last, only for a request that would get a popup (it counts the opening).
pub fn decide(
    opener: &Opener,
    target: &Url,
    size: Size,
    open: usize,
    rate_ok: impl FnOnce() -> bool,
) -> Verdict {
    if *opener == Opener::Other || !may_navigate(target) {
        return Verdict::plain(elsewhere(target));
    }
    let decision = wanted(target, size);
    if !matches!(decision, Decision::InApp | Decision::Pending) {
        return Verdict::plain(decision);
    }
    let limit = if open >= MAX_OPEN {
        Some(Limit::Cap)
    } else if !rate_ok() {
        Some(Limit::Rate)
    } else {
        None
    };
    match limit {
        Some(limit) => Verdict {
            decision: elsewhere(target),
            limit: Some(limit),
        },
        None => Verdict::plain(decision),
    }
}

/// A `Pending` popup once the engine has said whether it asked for a size (`ready-to-show`): it
/// shows, goes to the browser (its window closed unseen), or goes nowhere.
pub fn settle(target: &Url, sized: bool) -> Decision {
    let size = if sized {
        Size::Given(0.0, 0.0)
    } else {
        Size::None
    };
    wanted(target, size)
}

fn is_blank(url: &Url) -> bool {
    url.scheme() == "about" && url.path() == "blank"
}

/// Whether a popup may load `url` — top level or in a frame (WKWebView and WebKitGTK ask for both):
/// `http` / `https` web pages and the engine's own `about:blank` / `about:srcdoc` (frames only), and
/// nothing the page webview refuses (`navigation::is_allowed_navigation`), never an address Windows
/// would hand to the app's protocols (`navigation::intercepted_by_app`), an extension's URL in any
/// form, or the app's own origin — and no `data:`, `blob:`, `file:`, `javascript:` or other scheme.
pub fn may_navigate(url: &Url) -> bool {
    if !navigation::is_allowed_navigation(url)
        || navigation::intercepted_by_app(url)
        || is_ext_like(url)
        || keys::is_app_url(&keys::app_origin(), url)
    {
        return false;
    }
    match url.scheme() {
        "http" | "https" => url.has_host(),
        "about" => matches!(url.path(), "blank" | "srcdoc"),
        _ => false,
    }
}

/// Whether `url` starts a known sign-in flow (always `https`, on its default port): Google, Apple,
/// Microsoft, GitHub's OAuth, Facebook's OAuth dialog, Discord, X / Twitter and Slack.
pub fn known_sign_in(url: &Url) -> bool {
    if url.scheme() != "https" || url.port().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    let segments: Vec<&str> = url
        .path_segments()
        .map(|s| s.filter(|segment| !segment.is_empty()).collect())
        .unwrap_or_default();
    match host {
        "accounts.google.com"
        | "appleid.apple.com"
        | "login.microsoftonline.com"
        | "login.live.com" => true,
        "github.com" => segments.starts_with(&["login", "oauth"]),
        "www.facebook.com" | "facebook.com" => {
            let rest = match segments.first() {
                Some(first) if is_api_version(first) => &segments[1..],
                _ => &segments[..],
            };
            rest.starts_with(&["dialog", "oauth"])
        }
        "discord.com" => segments.first() == Some(&"oauth2"),
        "x.com" | "twitter.com" => segments.starts_with(&["i", "oauth2"]),
        "slack.com" => segments.first() == Some(&"oauth"),
        _ => false,
    }
}

/// `v19.0`, `v2`: a Graph API version in a Facebook path.
fn is_api_version(segment: &str) -> bool {
    segment.strip_prefix('v').is_some_and(|rest| {
        !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit() || b == b'.')
    })
}

/// The popup's title: where it is — the host of `https` pages (with a port that isn't 443), the
/// scheme too for anything else (`http://127.0.0.1:8123`), never the page's own title.
pub fn title_for(url: &Url) -> String {
    let host = url.host_str().map(|host| match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    });
    match (url.scheme(), host) {
        ("https", Some(host)) => host,
        (scheme, Some(host)) => format!("{scheme}://{host}"),
        ("about", None) => format!("about:{}", url.path()),
        (scheme, None) => format!("{scheme}:"),
    }
}

/// The popup's inner size: what it asked for, within `MIN_SIZE`–`MAX_SIZE`; `DEFAULT_SIZE` when
/// it asked for none.
pub fn clamp_size(asked: Option<(f64, f64)>) -> (f64, f64) {
    let (width, height) = asked.unwrap_or(DEFAULT_SIZE);
    let clamp = |value: f64, min: f64, max: f64| {
        if value.is_finite() {
            value.clamp(min, max)
        } else {
            min
        }
    };
    (
        clamp(width, MIN_SIZE.0, MAX_SIZE.0),
        clamp(height, MIN_SIZE.1, MAX_SIZE.1),
    )
}

/// Where a popup of `size` goes: centred over the Little Joan window (`over`, its outer frame),
/// kept inside the monitor's work area (`screen`) when there is one — never where the page asked.
pub fn place(over: Rect, size: (f64, f64), screen: Option<Rect>) -> (f64, f64) {
    let (width, height) = size;
    let mut x = over.x + (over.width - width) / 2.0;
    let mut y = over.y + (over.height - height) / 2.0;
    if let Some(screen) = screen {
        let inside = |value: f64, start: f64, room: f64, length: f64| {
            if length >= room {
                start
            } else {
                value.clamp(start, start + room - length)
            }
        };
        x = inside(x, screen.x, screen.width, width);
        y = inside(y, screen.y, screen.height, height);
    }
    (x.round(), y.round())
}

/// Whether a page webview that committed `url` has left the popup's `opener` (LJ48): another
/// origin — a reload or a same-origin navigation keeps the popup (sign-in flows reload the opener).
pub fn opener_left(opener: &Origin, url: &Url) -> bool {
    url.origin() != *opener
}

/// What one of Little Joan's keys does (LJ48): in a popup only the close action acts — it closes
/// the popup —, every other key is the popup page's own; in the page window, the page action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRoute {
    ClosePopup,
    Nothing,
    Page(PageAction),
}

pub fn route_key(in_popup: bool, action: KeyAction) -> KeyRoute {
    match (in_popup, action) {
        (true, KeyAction::Close) => KeyRoute::ClosePopup,
        (true, _) => KeyRoute::Nothing,
        (false, action) => KeyRoute::Page(action.page_action()),
    }
}

#[cfg(test)]
mod tests;
