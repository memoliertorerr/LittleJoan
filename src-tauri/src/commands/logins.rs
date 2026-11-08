//! Settings → Privacy's "Remove saved passwords and form data…" (LJ51): `clear_saved_logins`. Only
//! the settings window may call it (capabilities/settings.json, and a test) — never the bar, never
//! the page. The page asks a question first. Windows' web engine removes the passwords and form data
//! it saved, through the settings window's own WebView2 profile: the page's (settings_window.rs uses
//! the page's webview data folder), so it works while Little Joan hibernates or rests — no page
//! webview, no extra webview (Ground Rule 9). Little Joan never sees what is removed (Ground Rule
//! 26). macOS and Linux: not available (Settings shows no button there).

use tauri::{AppHandle, Manager, Runtime};

use crate::platform;
use crate::settings_window;
use crate::smoke;

/// Asks Windows' web engine to remove the passwords and form data it saved, and waits for its
/// answer — on a blocking thread of the async runtime, never one of its workers or the main thread
/// (phase 24's security review). Smoke log `saved-logins-cleared ok`, or `saved-logins-cleared
/// failed <reason>`.
#[tauri::command]
pub async fn clear_saved_logins<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let Some(settings) = app.get_webview_window(settings_window::LABEL) else {
        return Err("Open Settings to remove saved passwords.".to_owned());
    };
    let webview: tauri::Webview<R> = settings.as_ref().clone();
    let result =
        tauri::async_runtime::spawn_blocking(move || platform::clear_saved_logins(&webview))
            .await
            .unwrap_or_else(|err| Err(format!("the removal stopped: {err}")));
    match &result {
        Ok(()) => smoke::log("saved-logins-cleared", "ok"),
        Err(reason) => {
            eprintln!("little-joan: could not remove saved passwords: {reason}");
            smoke::log("saved-logins-cleared", &format!("failed {reason}"));
        }
    }
    result.map_err(|reason| format!("Couldn't remove them: {reason}"))
}
