//! The app itself, from the command bar (LJ41): with the tray / menu-bar icon hidden, the command
//! bar's "Quit Little Joan" is how to quit without a terminal. Only the bar may call it
//! (capabilities/bar.json, and a test). Settings → General has its own "Quit Little Joan" (LJ19.7):
//! `quit_from_settings`, which only the settings window may call (capabilities/settings.json, and a
//! test) — never the bar's or the page's.

use tauri::{AppHandle, Runtime};

use crate::joan;
use crate::smoke;

/// Quits Little Joan — the tray's Quit, `--quit` and Cmd+Q path (joan::quit: history saved, on
/// macOS the window closed first).
#[tauri::command]
pub async fn quit_app<R: Runtime>(app: AppHandle<R>) {
    joan::quit(&app);
}

/// Settings → General's "Quit Little Joan" (LJ19.7): the same quit (joan::quit). Whatever Settings
/// hasn't saved is dropped with its window, as with Cancel (the note under the button says so).
#[tauri::command]
pub async fn quit_from_settings<R: Runtime>(app: AppHandle<R>) {
    smoke::log("settings-quit", "");
    joan::quit(&app);
}
