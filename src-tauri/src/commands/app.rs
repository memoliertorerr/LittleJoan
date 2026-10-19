//! The app itself, from the command bar (LJ41): with the tray / menu-bar icon hidden, the command
//! bar's "Quit Little Joan" is how to quit without a terminal. Only the bar may call it
//! (capabilities/bar.json, and a test).

use tauri::{AppHandle, Runtime};

use crate::joan;

/// Quits Little Joan — the tray's Quit, `--quit` and Cmd+Q path (joan::quit: history saved, on
/// macOS the window closed first).
#[tauri::command]
pub async fn quit_app<R: Runtime>(app: AppHandle<R>) {
    joan::quit(&app);
}
