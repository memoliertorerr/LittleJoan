//! Extensions (experimental): small web apps that open as panels in the page window.
//!
//! The security model (Ground Rules 18–19; docs/architecture.md, "Extensions"):
//! - Extension code runs only as top-level `ljext:` documents in the page webview (`joan-page`),
//!   which has no IPC at all — never in the bar or Settings, never from the app's origin.
//! - Each extension has its own origin (urls.rs) and a Content-Security-Policy from its manifest
//!   (csp.rs): its own files only, network only to declared https hosts.
//! - It reaches the app only through the `ljext` API, served by Rust on its own origin and checked
//!   on every request (api.rs): the webview, the page it shows and that the panel committed, a
//!   POST with `X-LJ-Api: <this opening's secret token>`, the `Origin` and Referer, the method
//!   allowlist, the manifest permission, and the limits (limits.rs).
//! - The page webview may navigate to an extension URL only when it is the exact URL the app is
//!   opening, single-use nonce included (gate.rs, session.rs, nonce.rs) — a web page or a panel
//!   can't produce it; opening from a web page leaves that page first, so it can't race the
//!   panel's load.
//! - Paths never leave the extension's folder (paths.rs); manifests are checked strictly
//!   (manifest.rs); a bad extension is skipped, never a crash (registry.rs).
//! - What an extension supplies (names, titles, keywords) is drawn by the bar as text only.

pub mod api;
pub mod clipboard;
pub mod csp;
pub mod gate;
pub mod limits;
pub mod list;
pub mod manifest;
pub mod nonce;
pub mod paths;
pub mod protocol;
pub mod registry;
pub mod session;
pub mod storage;
pub mod theme;
pub mod urls;
pub mod web_address;

/// The page window closed: nothing about extensions is kept while hibernating (the next window
/// reads the registry again, so a developer's edits show up).
pub fn forget_window() {
    session::reset();
    registry::forget();
    theme::forget();
}
