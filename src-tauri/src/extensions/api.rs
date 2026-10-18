//! Which `ljext` requests are answered (pure, tested): extension files, the SDK
//! (`/__lj/sdk/v1.js`) and the API (`/__lj/api/v1/<method>`).
//!
//! The scheme is registered for every webview, so every request is checked from scratch:
//!
//! - **the webview** that asked is `joan-page` (never the bar or Settings);
//! - **the extension** in the URL exists;
//! - **the page** that webview shows now is a page of that very extension, which the app opened
//!   (so a web page — or its frames — can't load an extension's files or call its API). "Shows"
//!   is what the navigation guard last let through (session.rs `NavGate::showing`), not the
//!   page's address from load events, which can arrive late;
//! - files and the SDK: a `GET`, not for a frame (`Sec-Fetch-Dest`), a plain path (paths.rs) of a
//!   type we serve;
//! - the API: a `POST` with `X-LJ-Api: 1` (on Chromium this also forces a CORS preflight, which
//!   is refused), the extension's own origin (`origin_ok`), `Sec-Fetch-Site: same-origin` when the
//!   engine sends it, a method on the allowlist, and the manifest permission that method needs.
//!
//! **The Origin rule differs per engine** (CI, LJ29). WebView2 (Windows) sends `Origin` with
//! every POST, so it must be there and be exactly the extension's origin. WebKit (macOS, Linux)
//! sends **no** `Origin` (nor a Referer) with a same-origin POST to a custom scheme — only the
//! CORS path adds one, and every cross-origin request takes it (a web page's request arrived with
//! `Origin: http://127.0.0.1:…`, without a preflight). So there an Origin that is sent must be
//! exactly the extension's (`null` is refused too), a missing one means the request came from the
//! extension's own origin, and a Referer, when sent, must be from that origin as well.

use std::sync::Arc;

use serde_json::{Map, Value};
use url::Url;

use super::manifest::Permission;
use super::paths::{self, RelPath};
use super::registry::{Extension, Registry};
use super::storage;
use super::urls;
use crate::bar::PAGE;
use crate::keys::Platform;

pub const SDK_PATH: &str = "/__lj/sdk/v1.js";
pub const API_PREFIX: &str = "/__lj/api/v1/";
/// The header every API call carries.
pub const API_HEADER: &str = "x-lj-api";
/// The largest API request body: a full store's worth of value, plus room for the key.
pub const MAX_BODY_BYTES: usize = storage::QUOTA_BYTES + 64 * 1024;

/// The API's methods — the allowlist. Anything else is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    StorageGet,
    StorageSet,
    StorageRemove,
    StorageKeys,
    Theme,
    PageInfo,
    OpenUrl,
    ClipboardWriteText,
    Close,
}

impl Method {
    pub const ALL: [Method; 9] = [
        Method::StorageGet,
        Method::StorageSet,
        Method::StorageRemove,
        Method::StorageKeys,
        Method::Theme,
        Method::PageInfo,
        Method::OpenUrl,
        Method::ClipboardWriteText,
        Method::Close,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Method::StorageGet => "storage.get",
            Method::StorageSet => "storage.set",
            Method::StorageRemove => "storage.remove",
            Method::StorageKeys => "storage.keys",
            Method::Theme => "theme",
            Method::PageInfo => "pageInfo",
            Method::OpenUrl => "openUrl",
            Method::ClipboardWriteText => "clipboard.writeText",
            Method::Close => "close",
        }
    }

    pub fn from_name(name: &str) -> Option<Method> {
        Method::ALL.into_iter().find(|m| m.name() == name)
    }

    /// The manifest permission the method needs, if any.
    pub fn permission(self) -> Option<Permission> {
        match self {
            Method::StorageGet
            | Method::StorageSet
            | Method::StorageRemove
            | Method::StorageKeys => Some(Permission::Storage),
            Method::PageInfo => Some(Permission::PageInfo),
            Method::ClipboardWriteText => Some(Permission::Clipboard),
            Method::Theme | Method::OpenUrl | Method::Close => None,
        }
    }
}

/// A request as the handler got it.
#[derive(Debug, Clone, Default)]
pub struct Incoming {
    /// The label of the webview that sent it.
    pub webview: String,
    /// The HTTP method.
    pub method: String,
    pub uri: String,
    /// Header names in lower case.
    pub headers: Vec<(String, String)>,
}

impl Incoming {
    /// The header's value — `None` when it is missing or sent more than once.
    pub fn header(&self, name: &str) -> Option<&str> {
        let mut values = self
            .headers
            .iter()
            .filter(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str());
        let first = values.next()?;
        values.next().is_none().then_some(first)
    }

    fn has_header(&self, name: &str) -> bool {
        self.headers
            .iter()
            .any(|(n, _)| n.eq_ignore_ascii_case(name))
    }
}

/// What else a decision needs: the platform, the extension whose panel the page webview shows
/// (as the navigation guard saw it), the registry, and the extension the app opened last
/// (session.rs).
pub struct Context<'a> {
    pub platform: Platform,
    pub showing: Option<&'a str>,
    pub registry: &'a Registry,
    pub opened: Option<&'a str>,
}

/// An answered request.
#[derive(Debug)]
pub enum Route {
    File(Arc<Extension>, RelPath),
    Sdk(Arc<Extension>),
    Api(Arc<Extension>, Method),
}

/// Why a request was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// Not the page webview.
    Webview,
    /// No such extension (or not an extension URL).
    Unknown,
    /// The page webview isn't showing this extension, opened by the app.
    Page,
    /// Not GET (files) or POST (the API).
    HttpMethod,
    /// A frame, embed or object asked for a file.
    Frame,
    /// No such file, or a path or type that is never served.
    NotFound,
    /// The API call lacks `X-LJ-Api: 1`.
    Header,
    /// The `Origin` is not the extension's.
    Origin,
    /// `Sec-Fetch-Site` says another site sent it.
    Site,
    /// Not a method on the allowlist.
    NoMethod,
    /// The manifest doesn't declare the permission the method needs.
    Permission,
}

impl Refusal {
    pub fn status(self) -> u16 {
        match self {
            Refusal::Unknown | Refusal::NotFound | Refusal::NoMethod => 404,
            Refusal::HttpMethod => 405,
            _ => 403,
        }
    }

    /// A short reason, for the smoke log and the API's error.
    pub fn code(self) -> &'static str {
        match self {
            Refusal::Webview => "webview",
            Refusal::Unknown => "unknown",
            Refusal::Page => "page",
            Refusal::HttpMethod => "http-method",
            Refusal::Frame => "frame",
            Refusal::NotFound => "not-found",
            Refusal::Header => "header",
            Refusal::Origin => "origin",
            Refusal::Site => "site",
            Refusal::NoMethod => "no-method",
            Refusal::Permission => "permission",
        }
    }
}

/// Whether the API call comes from the extension's own origin (see the module docs): its
/// `Origin`, exactly, once — or, on WebKit (macOS, Linux), no `Origin` at all (what a same-origin
/// request to a custom scheme carries there) and no Referer from another origin.
pub fn origin_ok(incoming: &Incoming, id: &str, platform: Platform) -> bool {
    let own = urls::origin(id, platform);
    match incoming.header("origin") {
        Some(origin) => origin == own,
        None if incoming.has_header("origin") => false,
        None => {
            platform != Platform::Windows
                && match incoming.header("referer") {
                    Some(referer) => referer
                        .strip_prefix(own.as_str())
                        .is_some_and(|rest| rest.starts_with('/')),
                    None => !incoming.has_header("referer"),
                }
        }
    }
}

/// Request destinations that are frames or plugins.
const FRAME_DESTS: [&str; 5] = ["iframe", "frame", "embed", "object", "fencedframe"];

/// Decides `incoming` (see the module docs for the checks, in order).
pub fn route(incoming: &Incoming, ctx: &Context<'_>) -> Result<Route, Refusal> {
    if incoming.webview != PAGE {
        return Err(Refusal::Webview);
    }
    let target = urls::parse_request(&incoming.uri).ok_or(Refusal::Unknown)?;
    let extension = ctx.registry.get(&target.id).ok_or(Refusal::Unknown)?;
    let showing = ctx.showing == Some(target.id.as_str()) && ctx.opened == Some(target.id.as_str());

    if let Some(name) = target.path.strip_prefix(API_PREFIX) {
        if incoming.method != "POST" {
            return Err(Refusal::HttpMethod);
        }
        if incoming.header(API_HEADER) != Some("1") {
            return Err(Refusal::Header);
        }
        if !origin_ok(incoming, &target.id, ctx.platform) {
            return Err(Refusal::Origin);
        }
        if incoming.has_header("sec-fetch-site")
            && incoming.header("sec-fetch-site") != Some("same-origin")
        {
            return Err(Refusal::Site);
        }
        if !showing {
            return Err(Refusal::Page);
        }
        let method = Method::from_name(name).ok_or(Refusal::NoMethod)?;
        if let Some(permission) = method.permission()
            && !extension.manifest.has(&permission)
        {
            return Err(Refusal::Permission);
        }
        return Ok(Route::Api(extension, method));
    }

    if incoming.method != "GET" {
        return Err(Refusal::HttpMethod);
    }
    if incoming
        .header("sec-fetch-dest")
        .is_some_and(|dest| FRAME_DESTS.contains(&dest))
    {
        return Err(Refusal::Frame);
    }
    if !showing {
        return Err(Refusal::Page);
    }
    if target.path == SDK_PATH {
        return Ok(Route::Sdk(extension));
    }
    let rel = paths::clean(&target.path).map_err(|_| Refusal::NotFound)?;
    if paths::mime_type(&rel).is_none() {
        return Err(Refusal::NotFound);
    }
    Ok(Route::File(extension, rel))
}

/// An API call's parameters: the body must be a JSON object (an empty body is `{}`).
pub fn params(body: &[u8]) -> Result<Map<String, Value>, (u16, &'static str)> {
    if body.len() > MAX_BODY_BYTES {
        return Err((413, "too-large"));
    }
    if body.iter().all(u8::is_ascii_whitespace) {
        return Ok(Map::new());
    }
    match serde_json::from_slice(body) {
        Ok(Value::Object(map)) => Ok(map),
        _ => Err((400, "bad-request")),
    }
}

/// A web address an extension may open: an absolute http(s) URL with a host, never an extension
/// URL in any form.
pub fn openable(text: &str) -> Option<Url> {
    let url = Url::parse(text).ok()?;
    (matches!(url.scheme(), "http" | "https") && url.has_host() && !urls::is_ext_like(&url))
        .then_some(url)
}

#[cfg(test)]
mod tests;
