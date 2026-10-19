//! The global shortcut that summons and dismisses the page window.

use serde::Serialize;
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

pub const TAKEN: &str = "Another app or the system already uses that shortcut. Try another one.";
pub const INVALID: &str = "That shortcut can't be used. Try another one.";

/// Why a swap failed: the message for the user, and whether the old shortcut still works (it
/// was put back, or never touched). `false` when there was none, or it couldn't come back — then
/// no shortcut works, and the tray icon must show (tray.rs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapError {
    pub message: String,
    pub old_kept: bool,
}

/// Replaces the active shortcut `old` with `new`. On failure the old shortcut is registered
/// again and a message for the user is returned, so there is always a working shortcut if there
/// was one before — unless the old one can't come back either (`SwapError::old_kept`).
pub fn swap(registrar: &impl Registrar, old: Option<&str>, new: &str) -> Result<(), SwapError> {
    if parse(new).is_err() {
        return Err(SwapError {
            message: INVALID.to_owned(),
            old_kept: old.is_some(),
        });
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
            let old_kept = old.is_some_and(|old| match registrar.register(old) {
                Ok(()) => true,
                Err(err) => {
                    eprintln!("little-joan: could not restore the old shortcut: {err}");
                    false
                }
            });
            Err(SwapError {
                message: TAKEN.to_owned(),
                old_kept,
            })
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

/// Why the saved shortcut didn't work at launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Failure {
    /// Another app or the system has it.
    Taken,
    /// It isn't a shortcut (settings.json says something that doesn't parse).
    Invalid,
}

/// What went wrong with the saved shortcut at launch (LJ44): Settings says so, and which one
/// Little Joan uses instead (`active`: the default, or none).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub wanted: String,
    pub active: Option<String>,
    pub reason: Failure,
}

/// Registers `wanted`, else the default: the shortcut that works (or none) and what went wrong
/// with `wanted`, if anything.
pub fn settle(registrar: &impl Registrar, wanted: &str) -> (Option<String>, Option<Problem>) {
    let reason = if parse(wanted).is_err() {
        Failure::Invalid
    } else {
        match registrar.register(wanted) {
            Ok(()) => return (Some(wanted.to_owned()), None),
            Err(err) => {
                eprintln!("little-joan: could not register the shortcut: {err}");
                smoke::log("shortcut-failed", &err);
                Failure::Taken
            }
        }
    };
    if reason == Failure::Invalid {
        smoke::log("shortcut-failed", &format!("{wanted}: invalid"));
    }
    let active = if wanted == DEFAULT_SHORTCUT {
        None
    } else {
        match registrar.register(DEFAULT_SHORTCUT) {
            Ok(()) => Some(DEFAULT_SHORTCUT.to_owned()),
            Err(err) => {
                eprintln!("little-joan: could not register the default shortcut: {err}");
                smoke::log("shortcut-failed", &err);
                None
            }
        }
    };
    let problem = Problem {
        wanted: wanted.to_owned(),
        active: active.clone(),
        reason,
    };
    (active, Some(problem))
}

/// Registers the shortcut from settings; if it can't be parsed or is taken, falls back to the
/// default. If neither works the app keeps running — the tray icon comes back (tray.rs) and a
/// second launch opens the window. What went wrong is kept for Settings (`shortcut_problem`).
pub fn register_from_settings<R: Runtime>(app: &AppHandle<R>) {
    let wanted = state::with(app, |s| s.settings.shortcut.clone());
    let (active, problem) = settle(&PluginRegistrar(app), &wanted);
    if let Some(accelerator) = &active {
        smoke::log("shortcut-registered", accelerator);
    }
    state::with(app, |s| {
        s.active_shortcut = active;
        s.shortcut_problem = problem;
    });
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
            Err(SwapError {
                message: TAKEN.to_owned(),
                old_kept: true
            })
        );
        assert_eq!(registered(&fake), ["Alt+KeyJ"]);
    }

    #[test]
    fn swap_says_when_the_old_shortcut_couldnt_come_back() {
        // Another app took the old one in the moment it was free.
        let fake = Fake {
            taken: BTreeSet::from(["Alt+KeyK".to_owned(), "Alt+KeyJ".to_owned()]),
            ..Fake::default()
        };
        fake.registered.borrow_mut().insert("Alt+KeyJ".to_owned());
        assert_eq!(
            swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"),
            Err(SwapError {
                message: TAKEN.to_owned(),
                old_kept: false
            })
        );
        assert!(registered(&fake).is_empty());
        let none = Fake {
            taken: BTreeSet::from(["Alt+KeyK".to_owned()]),
            ..Fake::default()
        };
        assert_eq!(
            swap(&none, None, "Alt+KeyK").map_err(|err| err.old_kept),
            Err(false)
        );
    }

    #[test]
    fn swap_rejects_unparseable_shortcuts_without_touching_anything() {
        let fake = Fake::default();
        fake.register("Alt+KeyJ").expect("free");
        assert_eq!(
            swap(&fake, Some("Alt+KeyJ"), "Ctrl+Nope"),
            Err(SwapError {
                message: INVALID.to_owned(),
                old_kept: true
            })
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
    fn a_shortcut_that_fails_at_launch_falls_back_and_says_why() {
        let free = Fake::default();
        assert_eq!(
            settle(&free, "Alt+KeyJ"),
            (Some("Alt+KeyJ".to_owned()), None)
        );
        let taken = Fake {
            taken: BTreeSet::from(["Alt+KeyJ".to_owned()]),
            ..Fake::default()
        };
        assert_eq!(
            settle(&taken, "Alt+KeyJ"),
            (
                Some(DEFAULT_SHORTCUT.to_owned()),
                Some(Problem {
                    wanted: "Alt+KeyJ".into(),
                    active: Some(DEFAULT_SHORTCUT.into()),
                    reason: Failure::Taken
                })
            )
        );
        assert_eq!(registered(&taken), [DEFAULT_SHORTCUT]);
        let invalid = settle(&Fake::default(), "Ctrl+Nope");
        assert_eq!(invalid.0.as_deref(), Some(DEFAULT_SHORTCUT));
        assert_eq!(invalid.1.map(|p| p.reason), Some(Failure::Invalid));
        let both = Fake {
            taken: BTreeSet::from(["Alt+KeyJ".to_owned(), DEFAULT_SHORTCUT.to_owned()]),
            ..Fake::default()
        };
        let (active, problem) = settle(&both, "Alt+KeyJ");
        assert_eq!(active, None, "no shortcut at all");
        assert_eq!(problem.and_then(|p| p.active), None);
        let json = serde_json::to_value(Problem {
            wanted: "Alt+KeyJ".into(),
            active: None,
            reason: Failure::Taken,
        })
        .expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "wanted": "Alt+KeyJ", "active": null, "reason": "taken" })
        );
        assert_eq!(
            TAKEN,
            "Another app or the system already uses that shortcut. Try another one."
        );
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
