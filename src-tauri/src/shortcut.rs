//! The global shortcut that summons and dismisses the page window.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::joan;
use crate::settings::DEFAULT_SHORTCUT;
use crate::smoke;
use crate::state;
use crate::toggle::Command;

/// The global-shortcut plugin; every registered shortcut toggles the page window.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                joan::dispatch(app, Command::Toggle);
            }
        })
        .build()
}

/// Parses an accelerator such as `CommandOrControl+Shift+Space`.
pub fn parse(accelerator: &str) -> Result<Shortcut, String> {
    accelerator
        .parse::<Shortcut>()
        .map_err(|err| format!("{accelerator}: {err}"))
}

fn register<R: Runtime>(app: &AppHandle<R>, accelerator: &str) -> Result<(), String> {
    let shortcut = parse(accelerator)?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|err| format!("{accelerator}: {err}"))
}

/// Registers the shortcut from settings; if it can't be parsed or is taken, falls back to the
/// default. If neither works the app keeps running — the tray menu still opens the window.
pub fn register_from_settings<R: Runtime>(app: &AppHandle<R>) {
    let wanted = state::with(app, |s| s.settings.shortcut.clone());
    let active = match register(app, &wanted) {
        Ok(()) => Some(wanted),
        Err(err) => {
            eprintln!("little-joan: could not register the shortcut: {err}");
            smoke::log("shortcut-failed", &err);
            if wanted == DEFAULT_SHORTCUT {
                None
            } else {
                match register(app, DEFAULT_SHORTCUT) {
                    Ok(()) => Some(DEFAULT_SHORTCUT.to_owned()),
                    Err(err) => {
                        eprintln!("little-joan: could not register the default shortcut: {err}");
                        smoke::log("shortcut-failed", &err);
                        None
                    }
                }
            }
        }
    };
    if let Some(accelerator) = &active {
        smoke::log("shortcut-registered", accelerator);
    }
    state::with(app, |s| s.active_shortcut = active);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_and_common_shortcuts_parse() {
        for accelerator in [
            DEFAULT_SHORTCUT,
            "Alt+Shift+KeyJ",
            "Control+Alt+Space",
            "Super+Digit1",
            "CommandOrControl+F12",
        ] {
            assert!(parse(accelerator).is_ok(), "{accelerator}");
        }
    }

    #[test]
    fn malformed_shortcuts_are_rejected() {
        for accelerator in ["", "Shift+", "Ctrl+Shift+Nope", "Ctrl+A+B"] {
            assert!(parse(accelerator).is_err(), "{accelerator}");
        }
    }
}
