//! The global shortcut that summons and dismisses the page window: global-hotkey (the plugin),
//! or in a Linux Wayland session the XDG GlobalShortcuts portal when it takes it (LJ17;
//! backend.rs decides, platform/portal.rs talks to the portal).

pub mod backend;

use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::joan;
use crate::settings::DEFAULT_SHORTCUT;
use crate::smoke;
use crate::state;
use crate::summon_timing::Source;
use crate::toggle::Command;
use crate::welcome;
use backend::{Backend, PortalState};

/// The global-shortcut plugin; every registered shortcut toggles the page window.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                joan::dispatch(app, Command::Toggle, Source::Shortcut);
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

/// Registers the summon shortcut at launch. In a Linux Wayland session the portal is asked first
/// (platform/portal.rs); it answers later (`portal_answered`), and only when it can't take the
/// shortcut does global-hotkey get it, as before. Elsewhere global-hotkey at once.
pub fn register_at_launch<R: Runtime>(app: &AppHandle<R>) {
    if welcome::wayland_session_here() {
        state::with(app, |s| s.portal = PortalState::Asking);
        if crate::platform::ask_portal(app) {
            return;
        }
        state::with(app, |s| s.portal = PortalState::None);
    }
    register_from_settings(app);
}

/// The portal answered (platform/portal.rs): it holds the shortcut (`Registered`), or it doesn't
/// (`None` — at launch, or when its session went away later) and global-hotkey takes it, as
/// before. Then the icon's rule runs again, and a first launch's welcome that waited opens.
pub fn portal_answered<R: Runtime>(app: &AppHandle<R>, portal: PortalState) {
    if !portal.registered() {
        register_from_settings(app);
    }
    let welcome = state::with(app, |s| {
        s.portal = portal;
        std::mem::take(&mut s.welcome_after_portal)
    });
    crate::tray::apply_visibility(app);
    if welcome {
        joan::dispatch(app, Command::Show(None), Source::Launch);
    }
}

/// The desktop changed the portal shortcut's trigger (its settings): Settings and the welcome
/// show the new one, and the icon's rule runs again (no key any more brings the icon back).
pub fn portal_changed<R: Runtime>(app: &AppHandle<R>, trigger: Option<String>) {
    smoke::log(backend::CHANGED, trigger.as_deref().unwrap_or("none"));
    state::with(app, |s| {
        if s.portal.registered() {
            s.portal = PortalState::Registered { trigger };
        }
    });
    crate::tray::apply_visibility(app);
}

/// Who holds the summon shortcut now (backend.rs).
pub fn backend_now<R: Runtime>(app: &AppHandle<R>) -> Backend {
    let portal = state::with(app, |s| s.portal.clone());
    backend::backend(welcome::wayland_session_here(), &portal)
}

/// Whether the portal has the shortcut, or is being asked for it: then the desktop owns the key,
/// and Settings' Save swaps nothing (the saved accelerator is the trigger Little Joan prefers).
pub fn portal_holds<R: Runtime>(app: &AppHandle<R>) -> bool {
    state::with(app, |s| s.portal != PortalState::None)
}

/// Whether a first launch's welcome opens now, or waits for the portal's answer
/// (`portal_answered` opens it then, so it names the desktop's key or explains `--toggle`).
pub fn welcome_now<R: Runtime>(app: &AppHandle<R>) -> bool {
    state::with(app, |s| {
        let waits = backend::welcome_waits(&s.portal);
        s.welcome_after_portal = waits;
        !waits
    })
}

/// Registers the shortcut from settings with global-hotkey; if it can't be parsed or is taken,
/// falls back to the default. If neither works the app keeps running — the tray icon comes back
/// (tray.rs) and a second launch opens the window. What went wrong is kept for Settings
/// (`shortcut_problem`).
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
mod tests;
