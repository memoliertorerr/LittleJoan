//! Commands the app's own pages may call. build.rs's app manifest makes each one need a
//! permission, granted per webview: capabilities/bar.json (the bar: toolbar, command bar,
//! welcome) and capabilities/settings.json (the settings window). The page webview (`joan-page`)
//! has no capability at all, so nothing loaded there — web pages, the start page — can call any
//! of these.

use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

use crate::autostart;
use crate::bar;
use crate::history::{ClearRange, Listed, VisitKind};
use crate::history_store;
use crate::input::{self, SearchEngine};
use crate::joan;
use crate::keys::PageAction;
use crate::page_actions;
use crate::phone_mode;
use crate::settings::{self, Settings, Theme};
use crate::settings_window::{self, SettingsTab};
use crate::shortcut::{self, PluginRegistrar};
use crate::smoke;
use crate::state;
use crate::suggest::Suggestion;
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

/// Opens (`on`: a click on the toolbar's site; the field gets the keyboard with the address
/// selected, as with Ctrl/Cmd+L) or closes the command bar's overlay (Esc or × with a page behind
/// it; the page gets the keyboard back).
#[tauri::command]
pub async fn set_overlay<R: Runtime>(app: AppHandle<R>, on: bool) {
    if on {
        bar::focus_bar(&app);
    } else {
        bar::set_overlay(&app, false);
        bar::focus_page(&app);
    }
}

/// The toolbar's back button: the page goes back (the same page action as Alt+Left / Cmd+[).
#[tauri::command]
pub async fn go_back<R: Runtime>(app: AppHandle<R>) {
    page_actions::perform(&app, PageAction::Back);
}

/// The toolbar's "open in default browser" button: the page moves to the default browser and
/// Little Joan hibernates (the same page action as the tray's item).
#[tauri::command]
pub async fn open_in_browser<R: Runtime>(app: AppHandle<R>) {
    page_actions::perform(&app, PageAction::OpenInBrowser);
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
    /// The first launch: the command bar shows the welcome. Reading it clears it, so only the
    /// first summon is a welcome.
    first_run: bool,
    /// Where the tray / menu-bar icon is on this OS.
    tray_hint: &'static str,
    /// Whether the OS login entry exists (`null` when it can't be read).
    start_at_login: Option<bool>,
    /// A Linux Wayland session, where the X11 global shortcut can't fire.
    wayland_no_shortcut: bool,
    /// The theme, which the bar applies as it loads (ui/lib/theme.ts makes the colors).
    theme: Theme,
}

/// What the bar needs as it loads: the theme, and for the command bar's welcome on the first
/// launch the facts it shows.
#[tauri::command]
pub fn start_info<R: Runtime>(app: AppHandle<R>) -> StartInfo {
    let (shortcut, first_run, theme) = state::with(&app, |s| {
        (
            s.active_shortcut.clone(),
            std::mem::take(&mut s.first_run),
            s.settings.theme.clone(),
        )
    });
    if first_run {
        smoke::log("welcome", "");
    }
    smoke::log("theme", &theme_summary(&theme));
    StartInfo {
        shortcut,
        platform: std::env::consts::OS,
        first_run,
        tray_hint: welcome::tray_hint(std::env::consts::OS),
        start_at_login: autostart::is_enabled(&app),
        wayland_no_shortcut: welcome::wayland_no_shortcut_here(),
        theme,
    }
}

/// The smoke log's view of a theme: its mode and how many colors, never the colors.
fn theme_summary(theme: &Theme) -> String {
    format!("{} colors={}", theme.mode.name(), theme.colors.len())
}

/// One option in a settings list.
#[derive(Debug, Serialize)]
pub struct Choice {
    id: String,
    name: String,
}

/// Where Little Joan keeps things, for Settings → Privacy.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    /// The folder of settings.json and history.json.
    folder: String,
    /// Where the web engine keeps cookies and site data (a folder, or on macOS a description).
    site_data: String,
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
    storage: Storage,
    /// Little Joan's version (About).
    version: String,
}

/// Where the web engine keeps cookies and site data (docs/architecture.md, "Where data lives"):
/// WebKit's own store on macOS whatever the app says; elsewhere the webview data folder.
fn site_data<R: Runtime>(app: &AppHandle<R>, webview_dir: Option<&Path>) -> String {
    if cfg!(target_os = "macos") {
        return "WebKit's own store for Little Joan, in ~/Library/WebKit and ~/Library/HTTPStorages"
            .to_owned();
    }
    match webview_dir {
        Some(dir) => dir.display().to_string(),
        None => app.path().app_local_data_dir().map_or_else(
            |_| "the web engine's own folder for Little Joan".to_owned(),
            |dir| dir.display().to_string(),
        ),
    }
}

#[tauri::command]
pub fn get_settings<R: Runtime>(app: AppHandle<R>) -> SettingsView {
    let (settings, folder, webview_dir) = state::with(&app, |s| {
        (
            s.settings.clone(),
            s.data_dir.clone(),
            s.webview_data_dir.clone(),
        )
    });
    smoke::log("settings-loaded", "");
    SettingsView {
        settings,
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
        storage: Storage {
            folder: folder.display().to_string(),
            site_data: site_data(&app, webview_dir.as_deref()),
        },
        version: app.package_info().version.to_string(),
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
    remember_history: bool,
    /// The theme; checked field by field (settings/theme.rs). Left out, the saved one stays.
    #[serde(default)]
    theme: Option<Value>,
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
            remember_history: settings.remember_history,
            theme: settings.theme.as_ref().map_or_else(
                || s.settings.theme.clone(),
                |v| settings::theme::parse(Some(v)),
            ),
            ..s.settings.clone()
        };
        (s.data_dir.clone(), s.settings.clone())
    });
    // Phone mode on or off takes effect on the open page at once, and so does the theme.
    phone_mode::apply(&app, true);
    bar::set_theme(&app, &updated.theme);
    settings_window::set_theme(&app, &updated.theme);
    settings::save(&dir, &updated)
        .map_err(|err| format!("The new settings work now, but couldn't be saved: {err}"))?;
    Ok(updated)
}

/// Every page in history, latest visit first: `[{ url, title, host, lastVisit, visitCount }]`
/// (the history view; it groups, searches and draws them as text). Empty while no window is
/// open — only the bar, which lives in that window, asks.
#[tauri::command]
pub async fn history_list<R: Runtime>(app: AppHandle<R>) -> Vec<Listed> {
    let pages = history_store::list(&app);
    // A count only, never what is in it.
    smoke::log("history-list", &pages.len().to_string());
    pages
}

/// The command bar's suggestions for what is typed (suggest.rs): top sites (or the starters)
/// while nothing is, history matches and a search row while something is. The site of the page
/// that is open isn't suggested.
#[tauri::command]
pub async fn suggest<R: Runtime>(app: AppHandle<R>, query: String) -> Vec<Suggestion> {
    let current = bar::page_url(&app);
    let engine = state::with(&app, |s| s.settings.search_engine);
    let rows = history_store::suggest(&app, &query, current.as_ref(), engine);
    // The smoke log gets counts and the first row's site, never the typed text.
    smoke::log(
        "suggestions",
        &format!(
            "{} first={} typed={}",
            rows.len(),
            rows.first().map_or("-", |row| row.host.as_str()),
            query.trim().chars().count()
        ),
    );
    rows
}

/// Forgets one history record (Shift+Delete on a suggestion, × in the history view) and
/// rewrites history.json at once. Returns whether there was one.
#[tauri::command]
pub async fn history_remove<R: Runtime>(app: AppHandle<R>, url: String) -> Result<bool, String> {
    history_store::remove(&app, &url).map_err(|err| format!("Couldn't save the history: {err}"))
}

/// Forgets the visits of the last hour, day or week, or everything (`range`: "hour", "day",
/// "week", "all"), and rewrites history.json at once — with no window open, the file itself (the
/// settings window's Privacy tab). Returns how many pages went.
#[tauri::command]
pub async fn history_clear<R: Runtime>(
    app: AppHandle<R>,
    range: ClearRange,
) -> Result<usize, String> {
    history_store::clear(&app, range).map_err(|err| format!("Couldn't save the history: {err}"))
}

/// Opens (or focuses) the settings window on `tab` — general, appearance, extensions, privacy,
/// about; the command bar's "Settings & extensions" row asks for extensions. Without a tab (or
/// with one it doesn't know) a new window opens on General and an open one stays where it is.
#[tauri::command]
pub async fn open_settings<R: Runtime>(app: AppHandle<R>, tab: Option<String>) {
    settings_window::open(&app, tab.as_deref().and_then(SettingsTab::from_name));
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
