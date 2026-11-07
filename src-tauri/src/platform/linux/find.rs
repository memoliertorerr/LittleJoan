//! WebKitGTK's find (LJ49): the page view's `WebKitFindController` — `count_matches`, then `search`
//! (case-insensitive, wrapping, at most `find::MAX_MATCHES` highlighted), then
//! `search_next`, `search_previous` and `search_finish`. Its `found-text` / `failed-to-find-text`
//! say whether a match was found and `counted-matches` how many; it never says which match is
//! active, so find.rs tracks that. The signals are connected once per page view (`install`).

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Runtime, Webview};
use webkit2gtk::{FindControllerExt, FindOptions, WebViewExt};

use crate::bar;
use crate::find::{self, EngineOp, Report};
use crate::smoke;

/// The engine's line went to the smoke log (once per process).
static LOGGED: AtomicBool = AtomicBool::new(false);

/// Case-insensitive and wrapping, as every find (find.rs).
fn options() -> u32 {
    let mut options = FindOptions::empty();
    if !find::CASE_SENSITIVE {
        options |= FindOptions::CASE_INSENSITIVE;
    }
    if find::WRAPS {
        options |= FindOptions::WRAP_AROUND;
    }
    options.bits()
}

/// The page view's find signals, reported for the current find. GTK main thread.
pub(super) fn install<R: Runtime>(app: &AppHandle<R>, view: &webkit2gtk::WebView) {
    let Some(controller) = view.find_controller() else {
        return;
    };
    if smoke::enabled() && !LOGGED.swap(true, Ordering::SeqCst) {
        let (major, minor, micro) = super::webkit_version();
        smoke::log("find-engine", &format!("webkitgtk {major}.{minor}.{micro}"));
    }
    let found = app.clone();
    controller.connect_found_text(move |_, _| {
        bar::find::report(&found, None, Report::Found(true));
    });
    let failed = app.clone();
    controller.connect_failed_to_find_text(move |_| {
        bar::find::report(&failed, None, Report::Found(false));
    });
    let counted = app.clone();
    controller.connect_counted_matches(move |_, count| {
        bar::find::report(&counted, None, Report::Count(count));
    });
}

/// Runs `op` on the page view's find controller, on the GTK main thread.
pub fn run<R: Runtime>(page: &Webview<R>, _generation: u64, op: EngineOp) {
    let result = page.with_webview(move |webview| {
        let Some(controller) = webview.inner().find_controller() else {
            return;
        };
        match op {
            EngineOp::Start(query) => {
                // Count first: WebKit's count unmarks every match once it has counted, so after
                // the search it took the search's highlights away (CI's find.png: only the active
                // match showed). GNOME Web counts first too.
                controller.count_matches(&query, options(), find::MAX_MATCHES);
                controller.search(&query, options(), find::MAX_MATCHES);
            }
            EngineOp::Next(_) => controller.search_next(),
            EngineOp::Previous(_) => controller.search_previous(),
            EngineOp::Stop => controller.search_finish(),
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the page for find: {err}");
    }
}
