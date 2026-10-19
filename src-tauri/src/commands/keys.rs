//! Little Joan's keys (LJ43): `keymap_info`, for Settings → Keyboard (LJ44) and the bar. The bar
//! and the settings window may call it (capabilities, and a test); it only reads.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::keymap::ActionInfo;
use crate::shortcut::Problem;
use crate::state;

/// Every action with its key now and its defaults, and the global shortcut beside them.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeymapInfo {
    /// `std::env::consts::OS`: "macos", "windows", "linux".
    platform: &'static str,
    /// `[{ id, title, binding, defaults }]`, in Settings' order; titles as the menu has them.
    actions: Vec<ActionInfo>,
    /// The saved global shortcut (accelerator syntax).
    global_shortcut: String,
    /// It couldn't be registered at launch (LJ44): `{ wanted, active, reason: taken | invalid }`.
    shortcut_problem: Option<Problem>,
}

#[tauri::command]
pub async fn keymap_info<R: Runtime>(app: AppHandle<R>) -> KeymapInfo {
    let (actions, global_shortcut, shortcut_problem) = state::with(&app, |s| {
        (
            s.keymap.info(),
            s.settings.shortcut.clone(),
            s.shortcut_problem.clone(),
        )
    });
    KeymapInfo {
        platform: std::env::consts::OS,
        actions,
        global_shortcut,
        shortcut_problem,
    }
}
