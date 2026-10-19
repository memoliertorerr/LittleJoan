//! Downloads from the toolbar (LJ15): its "Show in folder" button. Only the bar may call it
//! (capabilities/bar.json, and a test). Which file is Rust's — the last one saved in this window —
//! never the bar's.

use crate::bar;

/// Shows the last file saved in this window in the file manager, selected.
#[tauri::command]
pub async fn reveal_download() -> Result<(), String> {
    let path = bar::download::last_saved().ok_or("No download to show yet")?;
    tauri_plugin_opener::reveal_item_in_dir(&path)
        .map_err(|err| format!("Couldn't show the download: {err}"))
}
