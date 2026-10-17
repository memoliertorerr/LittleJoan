//! macOS app menu. The menu bar stays hidden (the app is an Accessory), but its key equivalents
//! still work: the Edit items are what make Cmd+C / Cmd+V / Cmd+X / Cmd+A work inside WKWebView,
//! and the page items are the macOS side of keys.rs.

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};

use crate::joan;
use crate::keys::PageAction;
use crate::page_actions;
use crate::settings_window;

const SETTINGS: &str = "menu-settings";
/// Cmd+Q: our own item rather than the standard one, so it quits the way the tray's Quit does
/// (joan::quit closes the window first). Not "quit": that id is the tray's.
const QUIT: &str = "menu-quit";
const OPEN_LOCATION: &str = "page-open-location";
const TRANSLATE: &str = "page-translate";
const OPEN_IN_BROWSER: &str = "page-open-in-browser";
const RELOAD: &str = "page-reload";
const BACK: &str = "page-back";
const FORWARD: &str = "page-forward";
const HISTORY: &str = "page-history";

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let about = AboutMetadata {
        name: Some("Little Joan".into()),
        ..AboutMetadata::default()
    };
    let app_menu = Submenu::with_items(
        app,
        "Little Joan",
        true,
        &[
            &PredefinedMenuItem::about(app, Some("About Little Joan"), Some(about))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, SETTINGS, "Settings…", true, Some("CmdOrCtrl+,"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, QUIT, "Quit Little Joan", true, Some("CmdOrCtrl+Q"))?,
        ],
    )?;
    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &MenuItem::with_id(
                app,
                OPEN_LOCATION,
                "Open Location…",
                true,
                Some("CmdOrCtrl+L"),
            )?,
            &MenuItem::with_id(
                app,
                TRANSLATE,
                "Translate Page",
                true,
                Some("CmdOrCtrl+Shift+T"),
            )?,
            &MenuItem::with_id(
                app,
                OPEN_IN_BROWSER,
                "Open Page in Browser",
                true,
                Some("CmdOrCtrl+Shift+O"),
            )?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::close_window(app, Some("Close Window"))?,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        "Edit",
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let view = Submenu::with_items(
        app,
        "View",
        true,
        &[
            &MenuItem::with_id(app, RELOAD, "Reload", true, Some("CmdOrCtrl+R"))?,
            &MenuItem::with_id(app, BACK, "Back", true, Some("CmdOrCtrl+["))?,
            &MenuItem::with_id(app, FORWARD, "Forward", true, Some("CmdOrCtrl+]"))?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, HISTORY, "History", true, Some("CmdOrCtrl+Y"))?,
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &file, &edit, &view])
}

/// Menu events (the app menu and the tray menu share this stream; unknown ids are ignored).
pub fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let action = match event.id().as_ref() {
        SETTINGS => {
            settings_window::open(app, None);
            return;
        }
        QUIT => {
            joan::quit(app);
            return;
        }
        OPEN_LOCATION => PageAction::OpenLocation,
        TRANSLATE => PageAction::Translate,
        OPEN_IN_BROWSER => PageAction::OpenInBrowser,
        RELOAD => PageAction::Reload,
        BACK => PageAction::Back,
        FORWARD => PageAction::Forward,
        HISTORY => PageAction::History,
        _ => return,
    };
    page_actions::perform(app, action);
}
