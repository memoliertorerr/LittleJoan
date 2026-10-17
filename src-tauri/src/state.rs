//! Shared app state: the current settings, where they live, and the active shortcut.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager, Runtime};

use crate::settings::{self, Settings};

/// Development builds point this at `.cache/dev-data` (see scripts/env.sh). Shipped builds never
/// set it, so settings live in the OS config folder and browsing data in the engine's default.
pub const DATA_DIR_ENV: &str = "LITTLEJOAN_DATA_DIR";

pub struct AppState {
    pub settings: Settings,
    /// Folder holding settings.json.
    pub data_dir: PathBuf,
    /// Webview data folder override (Windows and Linux honor it; macOS WebKit uses its own
    /// store). `None` keeps the engine's default location.
    pub webview_data_dir: Option<PathBuf>,
    /// The accelerator currently registered as the global shortcut, if any.
    pub active_shortcut: Option<String>,
    /// True while the settings window is recording a new shortcut (the active one is paused).
    pub shortcut_suspended: bool,
}

pub type SharedState = Mutex<AppState>;

/// Loads settings and registers the shared state. Call once, in setup.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let (data_dir, webview_data_dir) =
        match std::env::var_os(DATA_DIR_ENV).filter(|v| !v.is_empty()) {
            Some(dir) => {
                let dir = PathBuf::from(dir);
                let webview = dir.join("webview");
                (dir, Some(webview))
            }
            None => (app.path().app_config_dir()?, None),
        };
    let settings = settings::load(&data_dir);
    app.manage(Mutex::new(AppState {
        settings,
        data_dir,
        webview_data_dir,
        active_shortcut: None,
        shortcut_suspended: false,
    }));
    Ok(())
}

/// Runs `f` with the state locked. Never call anything that waits on the main thread (window
/// getters, builders) inside `f`: the main thread may be waiting for this lock.
pub fn with<R: Runtime, T>(app: &AppHandle<R>, f: impl FnOnce(&mut AppState) -> T) -> T {
    let state = app.state::<SharedState>();
    let mut guard = state.lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard)
}
