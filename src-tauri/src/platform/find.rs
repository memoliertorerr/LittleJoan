//! Find in page per engine (LJ49; the state is find.rs's, the run-time part bar/find.rs): each
//! engine's own find, driven from Rust — WebView2's `ICoreWebView2Find` (win/find.rs), WKWebView's
//! `findString:withConfiguration:completionHandler:` (mac/find.rs), WebKitGTK's
//! `WebKitFindController` (linux/find.rs). Nothing goes into the page: no script, no IPC. The
//! engines report through `bar::find::report`.

use tauri::{Runtime, Webview};

use crate::find::EngineOp;

/// Asks the page's engine to do `op` for find `generation`. Queued on the main thread.
#[cfg(windows)]
pub fn find<R: Runtime>(page: &Webview<R>, generation: u64, op: EngineOp) {
    super::win::find::run(page, generation, op);
}

#[cfg(target_os = "macos")]
pub fn find<R: Runtime>(page: &Webview<R>, generation: u64, op: EngineOp) {
    super::mac::find::run(page, generation, op);
}

#[cfg(target_os = "linux")]
pub fn find<R: Runtime>(page: &Webview<R>, generation: u64, op: EngineOp) {
    super::linux::find::run(page, generation, op);
}
