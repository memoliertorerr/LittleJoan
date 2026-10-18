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
//! - **the panel committed**: between the guard admitting the panel and the panel's document
//!   replacing the web page there is a moment in which the web page still runs. In it only the
//!   panel's document itself is served — the request carrying this opening's nonce
//!   (`?lj-open=<hex>`, which no other document knows); every other file, the SDK and the API wait
//!   until a load event committed the panel (`NavGate::committed`; the handler waits up to
//!   `COMMIT_WAIT`), by which time the web page is gone. Without this a web page's frames and
//!   subresources racing the app's navigation were served there (CI, Windows);
//! - a Referer, when sent, is from the extension's own origin (sent twice: refused);
//! - never for a frame, a plugin or a service / shared worker (`Sec-Fetch-Dest`, refused when sent
//!   twice; a `Service-Worker` header) — kept for engines that send them: none of ours passes
//!   `Sec-Fetch-*` to a custom-scheme handler (CI's header log, LJ29), so the guard, the page and
//!   commit rules, the Referer and the response headers are what stop frames;
//! - files and the SDK: a `GET`, a plain path (paths.rs) of a type we serve;
//! - the API: a `POST` whose `X-LJ-Api` is this opening's secret token, compared in constant time
//!   (only the SDK served to that panel carries it; the custom header also forces a CORS preflight
//!   on Chromium, which is refused), the extension's own origin (`origin_ok`), `Sec-Fetch-Site: same-origin` when the
//!   engine sends it, a method on the allowlist, and the manifest permission that method needs.
//!
//! **The Origin rule differs per engine** (CI, LJ29). WebView2 (Windows) sends `Origin` with
//! every POST, so it must be there and be exactly the extension's origin. WebKit (macOS, Linux)
//! sends **no** `Origin` with a same-origin POST to a custom scheme (a Referer, yes) — only the
//! CORS path adds one, and every cross-origin request takes it (a web page's request arrived with
//! `Origin: http://127.0.0.1:…`, without a preflight). So there an Origin that is sent must be
//! exactly the extension's (`null` is refused too), a missing one means the request came from the
//! extension's own origin, and a Referer, when sent, must be from that origin as well.
//!
//! **What each engine passes the handler** (CI's `ljext-headers` log): WebView2 — `Accept`,
//! `Referer` (on everything but the app's own navigation), `User-Agent`, `sec-ch-ua*`, `Origin` on
//! POST, never `Sec-Fetch-Dest` / `-Site` (only `Sec-Fetch-Mode` on a CORS preflight). WebKit —
//! `Accept`, `User-Agent`, a `Referer` on POST only, no `Origin` from the panel itself, no
//! `Sec-Fetch-*`.

use std::sync::Arc;

use serde_json::{Map, Value};
use url::Url;

use super::manifest::Permission;
use super::nonce;
use super::paths::{self, RelPath};
use super::registry::{Extension, Registry};
use super::storage;
use super::urls;
pub use super::web_address::{is_private_host, openable};
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
/// (as the navigation guard saw it) and the one a load event committed, the registry, the
/// extension the app opened last, and that opening's nonce (in the panel's URL) and secret token
/// (session.rs), which only the SDK served to that panel carries.
pub struct Context<'a> {
    pub platform: Platform,
    pub showing: Option<&'a str>,
    pub committed: Option<&'a str>,
    pub registry: &'a Registry,
    pub opened: Option<&'a str>,
    pub nonce: Option<&'a str>,
    pub token: Option<&'a str>,
}

/// How long the handler holds a request that came before the panel committed (`Uncommitted`)
/// before it refuses it: the panel's own subresources can reach the handler a moment before the
/// load event that commits it is handled.
pub const COMMIT_WAIT: std::time::Duration = std::time::Duration::from_secs(2);

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
    /// The panel hasn't committed yet, and this isn't its document (the handler waits, then
    /// refuses).
    Uncommitted,
    /// A Referer from another origin (or sent twice).
    Referer,
    /// Not GET (files) or POST (the API).
    HttpMethod,
    /// A frame, embed or object asked for a file (or `Sec-Fetch-Dest` came twice).
    Frame,
    /// A service or shared worker asked (`Sec-Fetch-Dest`, the `Service-Worker` header).
    Worker,
    /// No such file, or a path or type that is never served.
    NotFound,
    /// The API call lacks `X-LJ-Api: <the session's token>` (exactly once).
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
            Refusal::Uncommitted => "uncommitted",
            Refusal::Referer => "referer",
            Refusal::HttpMethod => "http-method",
            Refusal::Frame => "frame",
            Refusal::Worker => "worker",
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
        None => platform != Platform::Windows && referer_ok(incoming, id, platform),
    }
}

/// Whether the request's Referer, if it sent one (once), is a page of the extension's own origin.
/// The panel's own requests carry its URL or none; a web page's carry the web page (or none).
pub fn referer_ok(incoming: &Incoming, id: &str, platform: Platform) -> bool {
    let own = urls::origin(id, platform);
    match incoming.header("referer") {
        Some(referer) => referer
            .strip_prefix(own.as_str())
            .is_some_and(|rest| rest.starts_with('/')),
        None => !incoming.has_header("referer"),
    }
}

/// Whether `incoming` asks for the panel's document by the URL the app opened it at: its query
/// is exactly `lj-open=<this opening's nonce>`.
fn is_panel_document(incoming: &Incoming, nonce: Option<&str>) -> bool {
    let Some(nonce) = nonce else {
        return false;
    };
    Url::parse(&incoming.uri).is_ok_and(|url| {
        url.query()
            .and_then(|query| query.strip_prefix("lj-open="))
            .is_some_and(|sent| nonce::same_secret(sent, nonce))
    })
}

/// Request destinations that are frames or plugins: they would embed a panel in another page.
const FRAME_DESTS: [&str; 5] = ["iframe", "frame", "embed", "object", "fencedframe"];
/// Service and shared workers: one would serve the extension's pages without the handler's
/// policy or checks, and outlive the panel.
const WORKER_DESTS: [&str; 2] = ["serviceworker", "sharedworker"];

/// Whether a frame, a plugin or a worker that must never get extension files asked — or a
/// `Sec-Fetch-Dest` sent twice, which can't be trusted either way.
fn embedding(incoming: &Incoming) -> Option<Refusal> {
    if incoming.has_header("service-worker") {
        return Some(Refusal::Worker);
    }
    if !incoming.has_header("sec-fetch-dest") {
        return None;
    }
    match incoming.header("sec-fetch-dest") {
        None => Some(Refusal::Frame),
        Some(dest) if FRAME_DESTS.contains(&dest) => Some(Refusal::Frame),
        Some(dest) if WORKER_DESTS.contains(&dest) => Some(Refusal::Worker),
        Some(_) => None,
    }
}

/// Decides `incoming` (see the module docs for the checks, in order).
pub fn route(incoming: &Incoming, ctx: &Context<'_>) -> Result<Route, Refusal> {
    if incoming.webview != PAGE {
        return Err(Refusal::Webview);
    }
    let target = urls::parse_request(&incoming.uri).ok_or(Refusal::Unknown)?;
    let extension = ctx.registry.get(&target.id).ok_or(Refusal::Unknown)?;
    let showing = ctx.showing == Some(target.id.as_str()) && ctx.opened == Some(target.id.as_str());
    let committed = ctx.committed == Some(target.id.as_str());
    if let Some(refusal) = embedding(incoming) {
        return Err(refusal);
    }

    if let Some(name) = target.path.strip_prefix(API_PREFIX) {
        if incoming.method != "POST" {
            return Err(Refusal::HttpMethod);
        }
        let token_ok = ctx.token.is_some_and(|token| {
            incoming
                .header(API_HEADER)
                .is_some_and(|sent| nonce::same_secret(sent, token))
        });
        if !token_ok {
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
        if !referer_ok(incoming, &target.id, ctx.platform) {
            return Err(Refusal::Referer);
        }
        if !committed {
            return Err(Refusal::Uncommitted);
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
    if !showing {
        return Err(Refusal::Page);
    }
    if !referer_ok(incoming, &target.id, ctx.platform) {
        return Err(Refusal::Referer);
    }
    if target.path == SDK_PATH {
        return if committed {
            Ok(Route::Sdk(extension))
        } else {
            Err(Refusal::Uncommitted)
        };
    }
    // Before the panel committed, only its document — asked for by the app's URL — is served.
    if !committed && !is_panel_document(incoming, ctx.nonce) {
        return Err(Refusal::Uncommitted);
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

#[cfg(test)]
mod tests;
