//! Commands the local pages may call. build.rs's app manifest makes each one need a permission,
//! granted per webview: capabilities/bar.json (the URL bar), capabilities/joan.json (the local
//! start page in the page webview), capabilities/settings.json (the settings window). Remote
//! pages get no IPC at all.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use crate::autostart;
use crate::bar;
use crate::history::{Row, VisitKind};
use crate::history_store;
use crate::input::{self, SearchEngine};
use crate::joan;
use crate::phone_mode;
use crate::settings::{self, Settings};
use crate::shortcut::{self, PluginRegistrar};
use crate::smoke;
use crate::state;
use crate::toggle::Command;
use crate::translate::{LANGUAGES, is_valid_lang};
use crate::welcome;

/// Opens what the user typed in the URL bar in the page: a web address, or a search with the
/// configured engine. The keyboard then moves to the page.
#[tauri::command]
pub async fn open_input<R: Runtime>(app: AppHandle<R>, input: String) -> Result<(), String> {
    let engine = state::with(&app, |s| s.settings.search_engine);
    let url = input::normalize(&input, engine)
        .ok_or_else(|| "Type an address or something to search for.".to_owned())?;
    // The user typed it: the visit counts double for frecency (history.rs).
    history_store::set_next_kind(&app, VisitKind::Typed);
    bar::navigate(&app, url).map_err(|err| err.to_string())?;
    bar::focus_page(&app);
    Ok(())
}

/// What the URL bar should show right now (`null` on the start page). The bar asks once when
/// it loads; after that the app pushes changes to it.
#[tauri::command]
pub async fn page_url<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    bar::current_text(&app)
}

/// Gives the keyboard back to the page (Esc in the URL bar).
#[tauri::command]
pub async fn focus_page<R: Runtime>(app: AppHandle<R>) {
    bar::focus_page(&app);
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
    /// The first launch: this start page is the welcome. Reading it clears it, so only the
    /// first summon is a welcome.
    first_run: bool,
    /// Where the tray / menu-bar icon is on this OS.
    tray_hint: &'static str,
    /// Whether the OS login entry exists (`null` when it can't be read).
    start_at_login: Option<bool>,
    /// A Linux Wayland session, where the X11 global shortcut can't fire.
    wayland_no_shortcut: bool,
}

/// What the start page needs for its hint line and, on the first launch, the welcome.
#[tauri::command]
pub fn start_info<R: Runtime>(app: AppHandle<R>) -> StartInfo {
    let (shortcut, first_run) = state::with(&app, |s| {
        (s.active_shortcut.clone(), std::mem::take(&mut s.first_run))
    });
    if first_run {
        smoke::log("welcome", "");
    }
    StartInfo {
        shortcut,
        platform: std::env::consts::OS,
        first_run,
        tray_hint: welcome::tray_hint(std::env::consts::OS),
        start_at_login: autostart::is_enabled(&app),
        wayland_no_shortcut: welcome::wayland_no_shortcut_here(),
    }
}

/// One option in a settings list.
#[derive(Debug, Serialize)]
pub struct Choice {
    id: String,
    name: String,
}

/// Everything the settings window shows.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    settings: Settings,
    /// Whether the OS login entry exists (`null` when it can't be read). Not in settings.json:
    /// the entry is the only state.
    start_at_login: Option<bool>,
    engines: Vec<Choice>,
    languages: Vec<Choice>,
    platform: &'static str,
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> SettingsView {
    SettingsView {
        settings: state::with(&app, |s| s.settings.clone()),
        start_at_login: autostart::is_enabled(&app),
        engines: SearchEngine::ALL
            .iter()
            .map(|engine| Choice {
                id: serde_json::to_value(engine)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default(),
                name: engine.name().to_owned(),
            })
            .collect(),
        languages: LANGUAGES
            .iter()
            .map(|(code, name)| Choice {
                id: (*code).to_owned(),
                name: (*name).to_owned(),
            })
            .collect(),
        platform: std::env::consts::OS,
    }
}

/// The fields the settings window can change.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    shortcut: String,
    search_engine: SearchEngine,
    translate_to: String,
    mobile_sites: bool,
}

/// Validates and applies new settings: the shortcut is swapped live (the old one comes back if
/// the new one is taken), then everything is saved.
#[tauri::command]
pub async fn save_settings<R: Runtime>(
    app: AppHandle<R>,
    settings: SettingsInput,
) -> Result<Settings, String> {
    if !is_valid_lang(&settings.translate_to) {
        return Err("Pick a language from the list.".to_owned());
    }
    let wanted = settings.shortcut.trim().to_owned();
    // Recording may have paused the active shortcut; resume it so the swap starts from a known
    // state.
    shortcut::set_suspended(&app, false);
    let old = state::with(&app, |s| s.active_shortcut.clone());
    shortcut::swap(&PluginRegistrar(&app), old.as_deref(), &wanted)?;
    smoke::log("shortcut-registered", &wanted);
    let (dir, updated) = state::with(&app, |s| {
        s.active_shortcut = Some(wanted.clone());
        s.settings = Settings {
            shortcut: wanted,
            search_engine: settings.search_engine,
            translate_to: settings.translate_to,
            mobile_sites: settings.mobile_sites,
            ..s.settings.clone()
        };
        (s.data_dir.clone(), s.settings.clone())
    });
    // Phone mode on or off takes effect on the open page at once.
    phone_mode::apply(&app, true);
    settings::save(&dir, &updated)
        .map_err(|err| format!("The new settings work now, but couldn't be saved: {err}"))?;
    Ok(updated)
}

/// The most rows a history command returns.
const MAX_ROWS: usize = 50;

/// Up to `n` top sites from history, by frecency (the bar's empty command bar, LJ23).
/// Read-only; rows are `{ url, title, host, score }`.
#[tauri::command]
pub async fn history_top<R: Runtime>(app: AppHandle<R>, n: usize) -> Vec<Row> {
    history_store::top(&app, n.min(MAX_ROWS))
}

/// Up to `limit` history records matching every word of `query` (the bar, LJ23). Read-only.
#[tauri::command]
pub async fn history_search<R: Runtime>(
    app: AppHandle<R>,
    query: String,
    limit: usize,
) -> Vec<Row> {
    history_store::search(&app, &query, limit.min(MAX_ROWS))
}

/// Pauses (true) or resumes (false) the global shortcut while a new one is being recorded.
#[tauri::command]
pub async fn suspend_shortcut<R: Runtime>(app: AppHandle<R>, suspended: bool) {
    shortcut::set_suspended(&app, suspended);
}

/// Adds or removes the OS login entry; returns whether it exists afterwards.
#[tauri::command]
pub async fn set_start_at_login<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<bool, String> {
    autostart::set(&app, enabled)
}
