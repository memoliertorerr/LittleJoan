//! macOS app menu. The menu bar stays hidden (the app is an Accessory), but its key equivalents
//! still work: the Edit items are what make Cmd+C / Cmd+V / Cmd+X / Cmd+A work inside WKWebView,
//! and the page items are the macOS side of keys.rs.

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Runtime};

use crate::keys::PageAction;
use crate::page_actions;
use crate::settings_window;

const SETTINGS: &str = "menu-settings";
const OPEN_LOCATION: &str = "page-open-location";
const RELOAD: &str = "page-reload";
const BACK: &str = "page-back";
const FORWARD: &str = "page-forward";

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
            &PredefinedMenuItem::quit(app, Some("Quit Little Joan"))?,
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
        ],
    )?;
    Menu::with_items(app, &[&app_menu, &file, &edit, &view])
}

/// Menu events (the app menu and the tray menu share this stream; unknown ids are ignored).
pub fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let action = match event.id().as_ref() {
        SETTINGS => {
            settings_window::open(app);
            return;
        }
        OPEN_LOCATION => PageAction::OpenLocation,
        RELOAD => PageAction::Reload,
        BACK => PageAction::Back,
        FORWARD => PageAction::Forward,
        _ => return,
    };
    page_actions::perform(app, action);
}
