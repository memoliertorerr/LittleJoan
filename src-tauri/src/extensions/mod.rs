//! Extensions (experimental): small web apps that open as panels in the page window.
//!
//! The security model (Ground Rules 18–19; docs/architecture.md, "Extensions"):
//! - Extension code runs only as top-level `ljext:` documents in the page webview (`joan-page`),
//!   which has no IPC at all — never in the bar or Settings, never from the app's origin.
//! - Each extension has its own origin (urls.rs) and a Content-Security-Policy from its manifest
//!   (csp.rs): its own files only, network only to declared https hosts.
//! - It reaches the app only through the `ljext` API, served by Rust on its own origin and checked
//!   on every request (api.rs): the webview, the page it shows, POST + `X-LJ-Api: 1`, the
//!   `Origin`, the method allowlist and the manifest permission.
//! - The page webview may navigate to an extension URL only when the app opened it (session.rs);
//!   web pages can't, in any form.
//! - Paths never leave the extension's folder (paths.rs); manifests are checked strictly
//!   (manifest.rs); a bad extension is skipped, never a crash (registry.rs).
//! - What an extension supplies (names, titles, keywords) is drawn by the bar as text only.

pub mod api;
pub mod clipboard;
pub mod csp;
pub mod list;
pub mod manifest;
pub mod paths;
pub mod protocol;
pub mod registry;
pub mod session;
pub mod storage;
pub mod theme;
pub mod urls;

/// The page window closed: nothing about extensions is kept while hibernating (the next window
/// reads the registry again, so a developer's edits show up).
pub fn forget_window() {
    session::reset();
    registry::forget();
    theme::forget();
}
