//! Settings → General's "Browser" section (LJ46): what the OS says about browsers, and the button
//! that opens the OS's own default-apps settings. Only the settings window may call these
//! (capabilities/settings.json, and a test) — never the bar or the page. Nothing here changes a
//! default: the button only opens the OS's settings (Ground Rule 21).

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::browsers;
use crate::state;

/// One browser to choose (never Little Joan). Its name comes from the OS; Settings draws it as text.
#[derive(Debug, Serialize)]
pub struct BrowserChoice {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsersInfo {
    platform: &'static str,
    /// The OS default for https: `self` (Little Joan), `other`, or `none` (couldn't be read).
    default_kind: &'static str,
    default_name: Option<String>,
    /// The browsers the OS lists, in its order — Little Joan left out.
    browsers: Vec<BrowserChoice>,
    /// The saved main browser (it may not be installed any more).
    main_browser: Option<String>,
    /// The first of `browsers`: where pages go while Little Joan is the default and no main
    /// browser is chosen.
    first_other: Option<String>,
    /// Whether the button can open the OS's settings here (Linux: GNOME's or KDE's).
    can_open_settings: bool,
}

/// What Settings shows about browsers, read from the OS now (read-only).
#[tauri::command]
pub async fn browsers_info<R: Runtime>(app: AppHandle<R>) -> BrowsersInfo {
    let main_browser = state::with(&app, |s| s.settings.main_browser.clone());
    let read = tauri::async_runtime::spawn_blocking(browsers::read)
        .await
        .unwrap_or_default();
    let can_open_settings = crate::platform::browsers::can_open_default_apps();
    let choices: Vec<BrowserChoice> = read
        .others()
        .map(|b| BrowserChoice {
            id: b.id.clone(),
            name: b.name.clone(),
        })
        .collect();
    BrowsersInfo {
        platform: std::env::consts::OS,
        default_kind: read.default_kind(),
        default_name: read.default_name().map(str::to_owned),
        first_other: choices.first().map(|c| c.name.clone()),
        browsers: choices,
        main_browser,
        can_open_settings,
    }
}

/// Opens the OS's default-apps settings: `opened`, or `none` when this desktop has none to open
/// (Settings then says where to choose).
#[tauri::command]
pub async fn open_default_apps_settings() -> Result<&'static str, String> {
    match browsers::open_default_apps_settings()? {
        Some(what) => {
            crate::smoke::log("default-apps-settings", what);
            Ok("opened")
        }
        None => Ok("none"),
    }
}
