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

/// Registers and unregisters global shortcuts. A trait so the swap logic is testable.
pub trait Registrar {
    fn register(&self, accelerator: &str) -> Result<(), String>;
    fn unregister(&self, accelerator: &str) -> Result<(), String>;
}

/// The real registrar: the global-shortcut plugin.
pub struct PluginRegistrar<'a, R: Runtime>(pub &'a AppHandle<R>);

impl<R: Runtime> Registrar for PluginRegistrar<'_, R> {
    fn register(&self, accelerator: &str) -> Result<(), String> {
        register(self.0, accelerator)
    }

    fn unregister(&self, accelerator: &str) -> Result<(), String> {
        let shortcut = parse(accelerator)?;
        self.0
            .global_shortcut()
            .unregister(shortcut)
            .map_err(|err| format!("{accelerator}: {err}"))
    }
}

pub const TAKEN: &str = "That shortcut is taken by another app or the system. Try another one.";
pub const INVALID: &str = "That shortcut can't be used. Try another one.";

/// Replaces the active shortcut `old` with `new`. On failure the old shortcut is registered
/// again and a message for the user is returned, so there is always a working shortcut if there
/// was one before.
pub fn swap(registrar: &impl Registrar, old: Option<&str>, new: &str) -> Result<(), String> {
    if parse(new).is_err() {
        return Err(INVALID.to_owned());
    }
    if old == Some(new) {
        return Ok(());
    }
    if let Some(old) = old
        && let Err(err) = registrar.unregister(old)
    {
        eprintln!("little-joan: could not unregister the old shortcut: {err}");
    }
    match registrar.register(new) {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("little-joan: could not register the new shortcut: {err}");
            if let Some(old) = old
                && let Err(err) = registrar.register(old)
            {
                eprintln!("little-joan: could not restore the old shortcut: {err}");
            }
            Err(TAKEN.to_owned())
        }
    }
}

/// Pauses the global shortcut while the settings window records a new one (so pressing the
/// current shortcut doesn't summon or dismiss the page window), and resumes it afterwards.
pub fn set_suspended<R: Runtime>(app: &AppHandle<R>, suspended: bool) {
    let (active, was_suspended) = state::with(app, |s| {
        let was = s.shortcut_suspended;
        s.shortcut_suspended = suspended;
        (s.active_shortcut.clone(), was)
    });
    let Some(active) = active else {
        return;
    };
    if suspended == was_suspended {
        return;
    }
    let registrar = PluginRegistrar(app);
    let result = if suspended {
        registrar.unregister(&active)
    } else {
        registrar.register(&active)
    };
    if let Err(err) = result {
        eprintln!("little-joan: could not change the shortcut's suspension: {err}");
    }
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
    use std::cell::RefCell;
    use std::collections::BTreeSet;

    /// A fake registrar: `taken` shortcuts belong to "another app".
    #[derive(Default)]
    struct Fake {
        registered: RefCell<BTreeSet<String>>,
        taken: BTreeSet<String>,
    }

    impl Registrar for Fake {
        fn register(&self, accelerator: &str) -> Result<(), String> {
            if self.taken.contains(accelerator) {
                return Err("taken".into());
            }
            self.registered.borrow_mut().insert(accelerator.to_owned());
            Ok(())
        }

        fn unregister(&self, accelerator: &str) -> Result<(), String> {
            self.registered.borrow_mut().remove(accelerator);
            Ok(())
        }
    }

    fn registered(fake: &Fake) -> Vec<String> {
        fake.registered.borrow().iter().cloned().collect()
    }

    #[test]
    fn swap_replaces_the_old_shortcut() {
        let fake = Fake::default();
        fake.register("Alt+KeyJ").expect("free");
        assert_eq!(swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"), Ok(()));
        assert_eq!(registered(&fake), ["Alt+KeyK"]);
    }

    #[test]
    fn swap_restores_the_old_shortcut_when_the_new_one_is_taken() {
        let fake = Fake {
            taken: BTreeSet::from(["Alt+KeyK".to_owned()]),
            ..Fake::default()
        };
        fake.register("Alt+KeyJ").expect("free");
        assert_eq!(
            swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"),
            Err(TAKEN.to_owned())
        );
        assert_eq!(registered(&fake), ["Alt+KeyJ"]);
    }

    #[test]
    fn swap_rejects_unparseable_shortcuts_without_touching_anything() {
        let fake = Fake::default();
        fake.register("Alt+KeyJ").expect("free");
        assert_eq!(
            swap(&fake, Some("Alt+KeyJ"), "Ctrl+Nope"),
            Err(INVALID.to_owned())
        );
        assert_eq!(registered(&fake), ["Alt+KeyJ"]);
    }

    #[test]
    fn swap_to_the_same_shortcut_is_a_no_op() {
        let fake = Fake::default();
        fake.register("Alt+KeyJ").expect("free");
        assert_eq!(swap(&fake, Some("Alt+KeyJ"), "Alt+KeyJ"), Ok(()));
        assert_eq!(registered(&fake), ["Alt+KeyJ"]);
    }

    #[test]
    fn swap_works_when_no_shortcut_was_active() {
        let fake = Fake::default();
        assert_eq!(swap(&fake, None, "Alt+KeyJ"), Ok(()));
        assert_eq!(registered(&fake), ["Alt+KeyJ"]);
    }

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
