//! The tray / menu-bar icon and its menu: the only UI that exists while Little Joan hibernates.
//!
//! The icon can be hidden (settings' `showTrayIcon`, LJ41), but never so that Little Joan can't be
//! reached: it comes back by itself whenever the global shortcut can't work (`icon_visible`). A
//! second launch still opens the window, and the command bar has Settings and Quit.

use std::sync::atomic::{AtomicU8, Ordering};

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};

use crate::joan;
use crate::keys::PageAction;
use crate::page_actions;
use crate::settings_window;
use crate::smoke;
use crate::state;
use crate::toggle::Command;
use crate::welcome;

/// The tray icon's id (`AppHandle::tray_by_id`).
const TRAY_ID: &str = "main";

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

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Little Joan")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            OPEN_ID => joan::dispatch(app, Command::Show(None)),
            TRANSLATE_ID => page_actions::perform(app, PageAction::Translate),
            OPEN_IN_BROWSER_ID => page_actions::perform(app, PageAction::OpenInBrowser),
            SETTINGS_ID => settings_window::open(app, None),
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

/// Whether the icon shows: unless the setting is off **and** the global shortcut works — with no
/// working shortcut and no icon, nothing would reach Little Joan but a second launch.
pub fn icon_visible(show_icon: bool, shortcut_works: bool) -> bool {
    show_icon || !shortcut_works
}

/// Whether the global shortcut can summon Little Joan: one is registered (`active_shortcut`: the
/// saved one or the default; `None` when both failed), and this isn't a Linux Wayland session,
/// where the X11 grab never fires (welcome::wayland_no_shortcut). A shortcut paused while
/// Settings records a new one stays the active one, so it still counts — the icon doesn't blink.
pub fn shortcut_works(active_shortcut: Option<&str>, wayland_no_shortcut: bool) -> bool {
    active_shortcut.is_some() && !wayland_no_shortcut
}

/// What the icon was last set to: 0 nothing yet (it is built visible), 1 shown, 2 hidden.
static APPLIED: AtomicU8 = AtomicU8::new(0);

/// Shows or hides the icon by the rule above, from the settings and the active shortcut. Called
/// after the shortcut is registered at startup and after every settings save; logs
/// `tray-visible <bool>` the first time and on each change. Takes the state lock only to read;
/// the tray's own call runs on the main thread (from another thread it waits for it).
pub fn apply_visibility<R: Runtime>(app: &AppHandle<R>) {
    let (show, active) = state::with(app, |s| {
        (s.settings.show_tray_icon, s.active_shortcut.clone())
    });
    let visible = icon_visible(
        show,
        shortcut_works(active.as_deref(), welcome::wayland_no_shortcut_here()),
    );
    let wanted = if visible { 1 } else { 2 };
    if APPLIED.load(Ordering::SeqCst) == wanted {
        return;
    }
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    if let Err(err) = tray.set_visible(visible) {
        eprintln!("little-joan: could not show or hide the tray icon: {err}");
        return;
    }
    APPLIED.store(wanted, Ordering::SeqCst);
    smoke::log("tray-visible", &visible.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_hides_only_while_the_shortcut_works() {
        assert!(icon_visible(true, true));
        assert!(icon_visible(true, false));
        assert!(!icon_visible(false, true));
        assert!(
            icon_visible(false, false),
            "no icon and no shortcut would strand the user"
        );
    }

    #[test]
    fn the_shortcut_works_only_when_registered_outside_wayland() {
        assert!(shortcut_works(Some("CommandOrControl+Shift+Space"), false));
        assert!(
            !shortcut_works(None, false),
            "the saved shortcut and the default both failed"
        );
        assert!(
            !shortcut_works(Some("CommandOrControl+Shift+Space"), true),
            "on Wayland the X11 grab never fires"
        );
        assert!(!shortcut_works(None, true));
    }

    #[test]
    fn a_shortcut_paused_for_recording_still_works() {
        // shortcut::set_suspended unregisters it but keeps it as the active one, so the icon
        // stays hidden while Settings records a new shortcut.
        let mut state = crate::state::AppState::for_tests();
        state.settings.show_tray_icon = false;
        state.active_shortcut = Some("Alt+KeyJ".into());
        state.shortcut_suspended = true;
        let works = shortcut_works(state.active_shortcut.as_deref(), false);
        assert!(works);
        assert!(!icon_visible(state.settings.show_tray_icon, works));
    }
}
