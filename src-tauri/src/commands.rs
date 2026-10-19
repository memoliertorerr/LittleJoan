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
use crate::extensions::{self, list::ExtensionsView, session};
use crate::history::{ClearRange, Listed, VisitKind};
use crate::history_store;
use crate::input::{self, SearchEngine};
use crate::joan;
use crate::keys::PageAction;
use crate::page_actions;
use crate::settings::{self, Change, Settings, Theme};
use crate::settings_live;
use crate::settings_window::{self, SettingsTab};
use crate::shortcut::{self, PluginRegistrar};
use crate::smoke;
use crate::state;
use crate::suggest::Suggestion;
use crate::toggle::Command;
use crate::translate::LANGUAGES;
use crate::welcome;
use keys::PortalShortcut;

/// The app itself from the command bar (LJ41): `quit_app`.
pub mod app;
/// The toolbar's "Show in folder" for a download (LJ15): `reveal_download`.
pub mod downloads;
/// Little Joan's keys for Settings (LJ43): `keymap_info`.
pub mod keys;
/// Quick commands in the command bar (LJ42): `quick_state`, `quick_command`.
pub mod quick;
/// Phone or desktop per site (LJ40): `site_mode_info`, `set_site_mode`.
pub mod site;

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
    /// A Linux Wayland session whose portal didn't take the shortcut, where the X11 shortcut
    /// can't fire: the welcome explains `--toggle`.
    wayland_no_shortcut: bool,
    /// The XDG GlobalShortcuts portal holds the shortcut (LJ17): `{ trigger }`, the desktop's own
    /// words for the key (`null` while it gave none). `shortcut` is then `null`.
    portal_shortcut: Option<PortalShortcut>,
    /// The theme, which the bar applies as it loads (ui/lib/theme.ts makes the colors).
    theme: Theme,
}

/// What the bar needs as it loads: the theme, and for the command bar's welcome on the first
/// launch the facts it shows.
#[tauri::command]
pub fn start_info<R: Runtime>(app: AppHandle<R>) -> StartInfo {
    let (shortcut, first_run, theme, portal) = state::with(&app, |s| {
        (
            s.active_shortcut.clone(),
            std::mem::take(&mut s.first_run),
            s.shown_theme(),
            s.portal.clone(),
        )
    });
    if first_run {
        smoke::log("welcome", "");
    }
    // The bar's first call: its script runs, so a panel waiting to open may (bar/boot.rs).
    bar::mark_ready();
    smoke::log("theme", &theme_summary(&theme));
    // The bar's script is running now: make sure it knows its view (bar::tell_state).
    bar::tell_state(&app);
    StartInfo {
        shortcut,
        platform: std::env::consts::OS,
        first_run,
        tray_hint: welcome::tray_hint(std::env::consts::OS),
        start_at_login: autostart::is_enabled(&app),
        wayland_no_shortcut: shortcut::backend_now(&app).toggle_hint(),
        portal_shortcut: PortalShortcut::from_state(&portal),
        theme,
    }
}

/// The smoke log's view of a theme: its mode and how many colors, never the colors.
pub(crate) fn theme_summary(theme: &Theme) -> String {
    format!("{} colors={}", theme.mode.name(), theme.colors.len())
}

/// One option in a settings list.
#[derive(Debug, Serialize)]
pub struct Choice {
    id: String,
    name: String,
}

/// The search engines, as settings lists show them: `[{ id, name }]` (Settings, quick commands).
pub(crate) fn engine_choices() -> Vec<Choice> {
    SearchEngine::ALL
        .iter()
        .map(|engine| Choice {
            id: serde_json::to_value(engine)
                .ok()
                .and_then(|v| v.as_str().map(str::to_owned))
                .unwrap_or_default(),
            name: engine.name().to_owned(),
        })
        .collect()
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
    /// The sites built in on this OS (General's "Phone or desktop per site"): `[{ host, mode }]`
    /// — YouTube on the desktop agent on macOS and Linux, nothing on Windows.
    built_in_site_modes: Vec<site::BuiltInSite>,
    /// The saved shortcut couldn't be registered at launch (LJ44): `{ wanted, active, reason }`.
    shortcut_problem: Option<shortcut::Problem>,
    /// A Linux Wayland session whose portal didn't take the shortcut, where the X11 shortcut
    /// can't fire (Keyboard explains --toggle).
    wayland_no_shortcut: bool,
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
    let (settings, folder, webview_dir, shortcut_problem) = state::with(&app, |s| {
        (
            s.settings.clone(),
            s.data_dir.clone(),
            s.webview_data_dir.clone(),
            s.shortcut_problem.clone(),
        )
    });
    smoke::log("settings-loaded", "");
    SettingsView {
        settings,
        start_at_login: autostart::is_enabled(&app),
        engines: engine_choices(),
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
        built_in_site_modes: site::built_in_sites(),
        shortcut_problem,
        wayland_no_shortcut: shortcut::backend_now(&app).toggle_hint(),
    }
}

/// The fields the settings window can change. Checked by settings::apply_change, like a quick
/// command's change (LJ42).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    shortcut: String,
    /// A search engine's id (`google`, `duckduckgo`, `bing`, `brave`).
    search_engine: String,
    translate_to: String,
    mobile_sites: bool,
    remember_history: bool,
    /// The theme, valid as it stands (settings/theme.rs `parse_strict`). Left out, the saved one
    /// stays.
    #[serde(default)]
    theme: Option<Value>,
    /// Settings → Extensions: the switched-off extensions (checked one by one) and Developer
    /// mode. Left out, the saved ones stay.
    #[serde(default)]
    disabled_extensions: Option<Value>,
    #[serde(default)]
    developer_mode: Option<bool>,
    /// Settings → General's "Phone or desktop per site": the whole table, checked entry by entry
    /// (settings/site_modes.rs). Left out, the saved one stays (the toolbar's pill may have
    /// changed it while Settings was open).
    #[serde(default)]
    site_modes: Option<Value>,
    /// Settings → General's tray / menu-bar icon switch. Left out, the saved one stays.
    #[serde(default)]
    show_tray_icon: Option<bool>,
    /// The user's own keys (LJ43; Settings → Keyboard, LJ44): the whole table, checked by
    /// keymap::validate. Left out, the saved one stays.
    #[serde(default)]
    keys: Option<Value>,
}

/// Validates and applies new settings: every part is checked first (settings::apply_change), the
/// shortcut is swapped live (the old one comes back if the new one is taken) — unless the portal
/// holds it (LJ17): the desktop owns that key, so the saved accelerator is only the trigger
/// Little Joan prefers next time —, then the rest takes effect and is saved through the path the
/// quick commands use too (settings_live.rs).
#[tauri::command]
pub async fn save_settings<R: Runtime>(
    app: AppHandle<R>,
    settings: SettingsInput,
) -> Result<Settings, String> {
    let wanted = settings.shortcut.trim().to_owned();
    let change = Change {
        shortcut: Some(wanted.clone()),
        search_engine: Some(settings.search_engine),
        translate_to: Some(settings.translate_to),
        mobile_sites: Some(settings.mobile_sites),
        remember_history: Some(settings.remember_history),
        theme: settings.theme,
        theme_mode: None,
        disabled_extensions: settings.disabled_extensions,
        developer_mode: settings.developer_mode,
        site_modes: settings.site_modes,
        show_tray_icon: settings.show_tray_icon,
        keys: settings.keys,
    };
    // A refused part changes nothing — not even the shortcut.
    let current = state::with(&app, |s| s.settings.clone());
    settings::apply_change(&current, &change)?;
    if !shortcut::portal_holds(&app) {
        swap_shortcut(&app, wanted)?;
    }
    let updated = settings_live::apply(&app, &change)?;
    settings_window::set_theme(&app, &updated.theme);
    Ok(updated)
}

/// Save's live swap of the global-hotkey shortcut (`shortcut::swap`).
fn swap_shortcut<R: Runtime>(app: &AppHandle<R>, wanted: String) -> Result<(), String> {
    // Recording may have paused the active shortcut; resume it so the swap starts from a known
    // state.
    shortcut::set_suspended(app, false);
    let old = state::with(app, |s| s.active_shortcut.clone());
    if let Err(err) = shortcut::swap(&PluginRegistrar(app), old.as_deref(), &wanted) {
        if !err.old_kept {
            // Neither the new shortcut nor the old one works now: the icon comes back.
            state::with(app, |s| s.active_shortcut = None);
            crate::tray::apply_visibility(app);
        }
        return Err(err.message);
    }
    smoke::log("shortcut-registered", &wanted);
    state::with(app, |s| {
        s.active_shortcut = Some(wanted);
        // The saved shortcut works now: the launch's problem is over.
        s.shortcut_problem = None;
    });
    Ok(())
}

/// Settings' Appearance tab previews `theme` in the open page window while it is edited (not
/// saved; checked like a saved one). `null` puts the saved theme back.
#[tauri::command]
pub async fn preview_theme<R: Runtime>(app: AppHandle<R>, theme: Option<Value>) {
    let shown = state::with(&app, |s| {
        s.theme_preview = theme.as_ref().map(|v| settings::theme::parse(Some(v)));
        s.shown_theme()
    });
    bar::forget_painted(&app);
    smoke::log("theme-preview", &theme_summary(&shown));
    bar::set_theme(&app, &shown);
}

/// The bar applied a theme: `{ scheme, vars }`, the `--lj-*` variables it computed
/// (ui/lib/theme.ts). Kept in memory for extension panels (littleJoan.theme); a report that
/// fails the checks is dropped (extensions/theme.rs). `early`: the bar painted the theme of its
/// initialization script, before start_info (LJ38) — the smoke log gets `theme-early`.
#[tauri::command]
pub async fn set_theme_tokens<R: Runtime>(app: AppHandle<R>, tokens: Value, early: Option<bool>) {
    match crate::extensions::theme::parse(&tokens) {
        Some(tokens) => {
            // The bar webview's own background follows (bar/background.rs, LJ39).
            bar::painted(&app, &tokens);
            crate::extensions::theme::store(tokens);
            if early == Some(true)
                && let Some(theme) = bar::early_theme()
            {
                smoke::log("theme-early", &theme_summary(&theme));
            }
        }
        None => eprintln!("little-joan: the bar's theme report didn't pass the checks"),
    }
}

/// Every extension that loaded — with its commands, page actions, permissions and whether it is
/// switched on — and what was skipped (the command bar's extension rows; Settings → Extensions).
#[tauri::command]
pub async fn extensions_list<R: Runtime>(app: AppHandle<R>) -> ExtensionsView {
    extensions_view(&app)
}

fn extensions_view<R: Runtime>(app: &AppHandle<R>) -> ExtensionsView {
    let registry = extensions::registry::current(app);
    let (developer_mode, folder, disabled) = state::with(app, |s| {
        (
            s.settings.developer_mode,
            s.data_dir.join(extensions::registry::EXTENSIONS_DIR),
            s.settings.disabled_extensions.clone(),
        )
    });
    let view = extensions::list::view(
        &registry,
        developer_mode,
        folder.display().to_string(),
        &disabled,
    );
    smoke::log(
        "extensions-list",
        &format!("{} skipped={}", view.extensions.len(), view.skipped.len()),
    );
    view
}

/// Settings → Extensions → Reload extensions: reads them again (a developer's edits, new
/// folders) and returns the new list.
#[tauri::command]
pub async fn reload_extensions<R: Runtime>(app: AppHandle<R>) -> ExtensionsView {
    extensions::registry::forget();
    extensions_view(&app)
}

/// Settings → Extensions → Open extensions folder: creates it if needed and shows it in the
/// file manager.
#[tauri::command]
pub async fn open_extensions_folder<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let folder = state::with(&app, |s| {
        s.data_dir.join(extensions::registry::EXTENSIONS_DIR)
    });
    std::fs::create_dir_all(&folder).map_err(|err| format!("Couldn't create the folder: {err}"))?;
    tauri_plugin_opener::reveal_item_in_dir(&folder)
        .map_err(|err| format!("Couldn't show the folder: {err}"))
}

/// The command bar picked an extension's command, or a page action (`page_action`) for the web
/// page that is open — whose address and title its panel gets when its manifest asks for
/// `pageInfo`.
#[tauri::command]
pub async fn open_extension<R: Runtime>(
    app: AppHandle<R>,
    id: String,
    command: String,
    page_action: bool,
) -> Result<(), String> {
    let target = if page_action {
        let page = web_page(&app).ok_or_else(|| "Page actions work on a web page.".to_owned())?;
        session::Target::PageAction(command, page)
    } else {
        session::Target::Command(command)
    };
    session::open(&app, &id, target)
}

/// The web page that is open — its address and the window's title (the page's) —, or `None` on
/// the start page, an extension's panel or with no window (page actions; the quick commands).
pub(crate) fn web_page<R: Runtime>(app: &AppHandle<R>) -> Option<session::PageInfo> {
    let url = bar::page_url(app).filter(|url| {
        crate::navigation::is_external_openable(url)
            && !crate::keys::is_app_url(&crate::keys::app_origin(), url)
    })?;
    let title = app
        .get_window(joan::LABEL)
        .and_then(|window| window.title().ok())
        .unwrap_or_default();
    Some(session::PageInfo {
        url: url.to_string(),
        title,
    })
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
