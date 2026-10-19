//! The toolbar's status area (LJ15), from Rust's side: what just happened — a download (LJ15),
//! the page's zoom level (LJ16) — pushed to the bar as `window.littleJoan.setStatus(<json>)`. The
//! bar turns it into text and times it (ui/toolbar/logic.ts). The last status of the window is told
//! again when the bar's script runs (it may have missed it while loading).

use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};

use super::BAR;

/// What the status area says (ui/toolbar/logic.ts `statusMessage` checks it).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Status {
    /// `state`: downloading, saved, failed or blocked; `name`: the file's (empty for a refusal).
    Download { state: &'static str, name: String },
    /// The page's zoom level, in percent.
    Zoom { percent: u16 },
}

/// The status the toolbar was last told in this window.
static LAST: Mutex<Option<String>> = Mutex::new(None);

fn last() -> std::sync::MutexGuard<'static, Option<String>> {
    LAST.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Tells the bar `status`.
pub fn tell<R: Runtime>(app: &AppHandle<R>, status: &Status) {
    let Ok(json) = serde_json::to_string(status) else {
        return;
    };
    *last() = Some(json.clone());
    eval(app, &json);
}

/// Tells the bar its last status again (its script runs now: `start_info`).
pub fn retell<R: Runtime>(app: &AppHandle<R>) {
    let json = last().clone();
    if let Some(json) = json {
        eval(app, &json);
    }
}

/// A new window, or a closed one: no status.
pub fn forget() {
    *last() = None;
}

fn eval<R: Runtime>(app: &AppHandle<R>, json: &str) {
    if let Some(bar) = app.get_webview(BAR) {
        let _ = bar.eval(format!(
            "window.littleJoan && window.littleJoan.setStatus({json})"
        ));
    }
}
