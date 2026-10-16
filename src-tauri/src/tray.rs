//! The tray / menu-bar icon and its menu: the only UI that exists while Little Joan hibernates.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Runtime};

use crate::settings_window;

const QUIT_ID: &str = "quit";
const SETTINGS_ID: &str = "tray-settings";

/// macOS tints a black template image to match the menu bar; Windows and Linux trays get the
/// full-colour app icon, which reads on both light and dark taskbars.
#[cfg(target_os = "macos")]
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray-template@2x.png");
#[cfg(not(target_os = "macos"))]
const TRAY_ICON: &[u8] = include_bytes!("../icons/32x32.png");

pub fn create<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, SETTINGS_ID, "Settings…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ID, "Quit Little Joan", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Little Joan")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id() == QUIT_ID {
                app.exit(0);
            } else if event.id() == SETTINGS_ID {
                settings_window::open(app);
            }
        })
        .build(app)?;
    Ok(())
}
