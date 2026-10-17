//! The tray / menu-bar icon and its menu: the only UI that exists while Little Joan hibernates.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};

use crate::joan;
use crate::keys::PageAction;
use crate::page_actions;
use crate::settings_window;
use crate::toggle::Command;

const OPEN_ID: &str = "tray-open";
const TRANSLATE_ID: &str = "tray-translate";
const OPEN_IN_BROWSER_ID: &str = "tray-open-in-browser";
const SETTINGS_ID: &str = "tray-settings";
const QUIT_ID: &str = "quit";

/// macOS tints a black template image to match the menu bar; Windows and Linux trays get the
/// full-colour app icon, which reads on both light and dark taskbars.
#[cfg(target_os = "macos")]
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-template@2x.png");
#[cfg(not(target_os = "macos"))]
const TRAY_ICON: &[u8] = include_bytes!("../icons/32x32.png");

/// The menu items that only make sense while a web page is open.
struct PageItems<R: Runtime> {
    translate: MenuItem<R>,
    open_in_browser: MenuItem<R>,
}

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, OPEN_ID, "Open Little Joan", true, None::<&str>)?;
    let translate = MenuItem::with_id(app, TRANSLATE_ID, "Translate Page", false, None::<&str>)?;
    let open_in_browser = MenuItem::with_id(
        app,
        OPEN_IN_BROWSER_ID,
        "Open Page in Browser",
        false,
        None::<&str>,
    )?;
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit Little Joan", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &translate,
            &open_in_browser,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &quit,
        ],
    )?;
    app.manage(PageItems {
        translate,
        open_in_browser,
    });

    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Little Joan")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => joan::dispatch(app, Command::Show(None)),
            TRANSLATE_ID => page_actions::perform(app, PageAction::Translate),
            OPEN_IN_BROWSER_ID => page_actions::perform(app, PageAction::OpenInBrowser),
            SETTINGS_ID => settings_window::open(app),
            QUIT_ID => joan::quit(app),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

/// Enables "Translate Page" and "Open Page in Browser" while a web page (not the start page) is
/// open, and disables them otherwise.
pub fn set_page_items_enabled<R: Runtime>(app: &AppHandle<R>, enabled: bool) {
    if let Some(items) = app.try_state::<PageItems<R>>() {
        for item in [&items.translate, &items.open_in_browser] {
            if let Err(err) = item.set_enabled(enabled) {
                eprintln!("little-joan: could not update the tray menu: {err}");
            }
        }
    }
}
