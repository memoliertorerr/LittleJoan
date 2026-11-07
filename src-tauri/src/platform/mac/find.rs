//! WKWebView's find (LJ49): `findString:withConfiguration:completionHandler:` (macOS 11+) with a
//! `WKFindConfiguration` — case-insensitive, wrapping, backwards for the previous match. Its
//! `WKFindResult` says only `matchFound`: no count, no active match, so the strip says "Found" or
//! "No matches". Next and previous find the same text again from the selection the last find left.
//! Stop has nothing to undo but that selection, which stays. Each call's completion handler (a
//! block, block2) carries its find's number, so an answer for an older query is dropped.

use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;
use tauri::{Manager, Runtime, Webview};

use crate::bar;
use crate::find::{self, EngineOp, Report};
use crate::smoke;

/// The engine's line went to the smoke log (once per process).
static LOGGED: AtomicBool = AtomicBool::new(false);

/// Runs `op` on the page's WKWebView, on the main thread.
pub fn run<R: Runtime>(page: &Webview<R>, generation: u64, op: EngineOp) {
    let (query, backwards) = match op {
        EngineOp::Start(query) | EngineOp::Next(query) => (query, false),
        EngineOp::Previous(query) => (query, true),
        // The selection the find left stays (as Safari's).
        EngineOp::Stop => return,
    };
    let app = page.app_handle().clone();
    let result = page.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        if view.is_null() {
            return;
        }
        let Some(class) = AnyClass::get(c"WKFindConfiguration") else {
            log_engine(false);
            bar::find::report(&app, Some(generation), Report::Unavailable);
            return;
        };
        log_engine(true);
        let text = NSString::from_str(&query);
        let answer = app.clone();
        let handler = RcBlock::new(move |result: *mut AnyObject| {
            // SAFETY: WebKit hands the block a WKFindResult (or nil); -matchFound returns a BOOL.
            let found = !result.is_null() && unsafe { msg_send![&*result, matchFound] };
            bar::find::report(&answer, Some(generation), Report::Found(found));
        });
        // SAFETY: `view` is the live WKWebView of this webview (with_webview runs on the main
        // thread while it exists). +new returns a WKFindConfiguration; its setters take BOOLs;
        // findString:withConfiguration:completionHandler: takes an NSString, the configuration and
        // a block taking a WKFindResult, which WebKit copies and calls once on the main thread.
        unsafe {
            let config: Option<Retained<AnyObject>> = msg_send![class, new];
            let Some(config) = config else {
                return;
            };
            let _: () = msg_send![&*config, setCaseSensitive: find::CASE_SENSITIVE];
            let _: () = msg_send![&*config, setWraps: find::WRAPS];
            let _: () = msg_send![&*config, setBackwards: backwards];
            let _: () = msg_send![
                &*view,
                findString: &*text,
                withConfiguration: &*config,
                completionHandler: &*handler
            ];
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the page for find: {err}");
    }
}

/// Smoke log, once: `find-engine wkwebview find=<bool>` (WKFindConfiguration exists: macOS 11+).
fn log_engine(available: bool) {
    if smoke::enabled() && !LOGGED.swap(true, Ordering::SeqCst) {
        smoke::log("find-engine", &format!("wkwebview find={available}"));
    }
}
