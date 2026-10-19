//! macOS app menu. The menu bar stays hidden (the app is an Accessory), but its key equivalents
//! still work: the Edit items are what make Cmd+C / Cmd+V / Cmd+X / Cmd+A work inside WKWebView,
//! and the page items are the macOS side of the keymap (keymap.rs, LJ43) — their key equivalents
//! come from it, an action without a key gets none, and a settings save that changes the keys sets
//! the menu again (`refresh`). Fixed here: the Edit menu, Settings… (Cmd+,) and Quit (Cmd+Q).

use tauri::menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Manager, Runtime};

use crate::joan;
use crate::keymap::{KeyAction, Keymap};
use crate::page_actions;
use crate::settings_window;
use crate::smoke;
use crate::state;

const SETTINGS: &str = "menu-settings";
/// Cmd+Q: our own item rather than the standard one, so it quits the way the tray's Quit does
/// (joan::quit closes the window first). Not "quit": that id is the tray's.
const QUIT: &str = "menu-quit";

/// A page item's menu id.
fn item_id(action: KeyAction) -> String {
    format!("page-{}", action.id())
}

/// The page item for `action`, its key equivalent from `keymap`.
fn page_item<R: Runtime>(
    app: &AppHandle<R>,
    keymap: &Keymap,
    action: KeyAction,
) -> tauri::Result<MenuItem<R>> {
    let accelerator = keymap.binding(action).map(|binding| binding.accelerator());
    MenuItem::with_id(
        app,
        item_id(action),
        action.title(),
        true,
        accelerator.as_deref(),
    )
}

pub fn create<R: Runtime>(app: &AppHandle<R>, keymap: &Keymap) -> tauri::Result<Menu<R>> {
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
    let item = |action| page_item(app, keymap, action);
    let file = Submenu::with_items(
        app,
        "File",
        true,
        &[
            &item(KeyAction::OpenLocation)?,
            &item(KeyAction::Translate)?,
            &item(KeyAction::OpenInBrowser)?,
            &PredefinedMenuItem::separator(app)?,
            // Our own Close Window (not the standard one): its key is the keymap's too.
            &item(KeyAction::Close)?,
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
            &item(KeyAction::Reload)?,
            &item(KeyAction::Back)?,
            &item(KeyAction::Forward)?,
            &PredefinedMenuItem::separator(app)?,
            &item(KeyAction::History)?,
        ],
    )?;
    let menu = Menu::with_items(app, &[&app_menu, &file, &edit, &view])?;
    if smoke::enabled() {
        // What the menu was built with (CI checks a custom key reached it).
        for action in KeyAction::ALL {
            let shown = keymap
                .binding(action)
                .map_or_else(|| "none".to_owned(), |b| b.display(keymap.platform()));
            smoke::log("menu-keys", &format!("{}={shown}", action.id()));
        }
    }
    Ok(menu)
}

/// Sets the menu again from the keymap of this moment (after the keys changed), on the main
/// thread.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    let result = app.run_on_main_thread(move || {
        let keymap = state::with(&handle, |s| s.keymap.clone());
        match create(&handle, &keymap).and_then(|menu| handle.set_menu(menu)) {
            Ok(_) => {}
            Err(err) => eprintln!("little-joan: could not set the menu's keys: {err}"),
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the main thread for the menu: {err}");
    }
}

/// Menu events (the app menu and the tray menu share this stream; unknown ids are ignored).
pub fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
    let id = event.id().as_ref();
    match id {
        SETTINGS => settings_window::open(app, None),
        QUIT => joan::quit(app),
        _ => {
            let Some(action) = KeyAction::ALL
                .into_iter()
                .find(|action| item_id(*action) == id)
            else {
                return;
            };
            if action == KeyAction::Close && close_settings(app) {
                return;
            }
            page_actions::perform(app, action.page_action());
        }
    }
}

/// Close Window with Settings in front closes Settings (the standard item's job before LJ43).
fn close_settings<R: Runtime>(app: &AppHandle<R>) -> bool {
    let Some(window) = app.get_webview_window(settings_window::LABEL) else {
        return false;
    };
    if !window.is_focused().unwrap_or(false) {
        return false;
    }
    if let Err(err) = window.close() {
        eprintln!("little-joan: could not close Settings: {err}");
    }
    true
}
