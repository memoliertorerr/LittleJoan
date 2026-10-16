//! Commands the local start page may call. They are restricted to the "joan" window by
//! build.rs's app manifest and capabilities/joan.json; remote pages get no IPC at all.

use serde::Serialize;
use tauri::{AppHandle, Runtime, WebviewWindow};

use crate::input;
use crate::joan;
use crate::state;
use crate::toggle::Command;

/// Opens what the user typed: a web address, or a search with the configured engine.
#[tauri::command]
pub async fn open_input<R: Runtime>(
    app: AppHandle<R>,
    window: WebviewWindow<R>,
    input: String,
) -> Result<(), String> {
    let engine = state::with(&app, |s| s.settings.search_engine);
    let url = input::normalize(&input, engine)
        .ok_or_else(|| "Type an address or something to search for.".to_owned())?;
    window.navigate(url).map_err(|err| err.to_string())
}

/// Closes the page window: back to hibernation.
#[tauri::command]
pub async fn hibernate<R: Runtime>(app: AppHandle<R>) {
    joan::handle(&app, Command::Hide);
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartInfo {
    /// The registered global shortcut (accelerator syntax), if any.
    shortcut: Option<String>,
    /// `std::env::consts::OS`: "macos", "windows", "linux".
    platform: &'static str,
}

/// What the start page needs to show its hint line.
#[tauri::command]
pub fn start_info<R: Runtime>(app: AppHandle<R>) -> StartInfo {
    StartInfo {
        shortcut: state::with(&app, |s| s.active_shortcut.clone()),
        platform: std::env::consts::OS,
    }
}
