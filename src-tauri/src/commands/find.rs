//! Find in page from the toolbar's find strip (LJ49): `find`. Only the bar may call it
//! (capabilities/bar.json, and a test). It takes one of a fixed set of actions — a query to find,
//! the next or previous match, or closing the strip — and the page's engine finds natively; the
//! page itself gets no script and no IPC.

use tauri::{AppHandle, Runtime};

use crate::bar;
use crate::find::FindAction;

/// `{ kind: "start", query }` (at most 1,000 characters; an empty one clears the find), `{ kind:
/// "next" }`, `{ kind: "previous" }` or `{ kind: "stop" }` (the strip closed: the page gets the
/// keyboard back). The result reaches the bar as `window.littleJoan.setFindResult(<json>)`.
#[tauri::command]
pub async fn find<R: Runtime>(app: AppHandle<R>, action: FindAction) {
    bar::find::act(&app, action);
}
