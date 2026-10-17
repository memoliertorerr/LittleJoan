//! Start at login: the OS's own login entry, written by tauri-plugin-autostart — a LaunchAgent
//! plist on macOS, a registry Run value on Windows, an XDG autostart file on Linux.
//!
//! The entry is the only state: settings.json has no field for it, so an entry the user removed
//! in the OS's own settings shows as off here. The plugin is used from Rust only; no window is
//! granted any of its JS commands (capabilities/*.json). Off until the user turns it on.

use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Runtime};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::smoke;

/// Shown when macOS runs the app from a temporary copy (see [`refusal`]).
pub const MOVE_TO_APPLICATIONS: &str = "Move Little Joan to Applications first, then turn this on.";

/// The plugin. macOS uses a LaunchAgent — never AppleScript, which asks for Automation
/// permission. No launch arguments: a plain launch already starts hibernating.
pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)
}

/// Whether the login entry exists, or `None` when it can't be read.
pub fn is_enabled<R: Runtime>(app: &AppHandle<R>) -> Option<bool> {
    if let Err(err) = ensure_run_key() {
        eprintln!("little-joan: can't open the login entries: {err}");
        return None;
    }
    match app.autolaunch().is_enabled() {
        Ok(enabled) => Some(enabled),
        Err(err) => {
            eprintln!("little-joan: can't read the login entry: {err}");
            None
        }
    }
}

/// Adds (`true`) or removes (`false`) the login entry and returns what the OS reports
/// afterwards. The outcome goes to the smoke log either way.
pub fn set<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<bool, String> {
    let result = apply(app, enabled);
    match &result {
        Ok(now) => smoke::log("start-at-login", if *now { "true" } else { "false" }),
        Err(err) => {
            eprintln!("little-joan: start at login: {err}");
            smoke::log("start-at-login-failed", err);
        }
    }
    result
}

fn apply<R: Runtime>(app: &AppHandle<R>, enabled: bool) -> Result<bool, String> {
    ensure_run_key()?;
    let manager = app.autolaunch();
    // Only change what differs: Windows' disable fails when there is no value to delete.
    if manager.is_enabled().map_err(|err| err.to_string())? != enabled {
        if enabled {
            #[cfg(target_os = "macos")]
            refuse_translocated()?;
            manager.enable()
        } else {
            manager.disable()
        }
        .map_err(|err| err.to_string())?;
    }
    manager.is_enabled().map_err(|err| err.to_string())
}

/// Windows: auto-launch opens `HKCU\…\Run` without creating it, and a profile can lack the key
/// (a CI runner did), which made every read and write fail with "file not found". Create the
/// standard per-user Run key first; opening an existing one changes nothing.
#[cfg(windows)]
fn ensure_run_key() -> Result<(), String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        .map(|_| ())
        .map_err(|err| err.to_string())
}

#[cfg(not(windows))]
fn ensure_run_key() -> Result<(), String> {
    Ok(())
}

/// The plugin records the canonical path of the running executable, so check that one.
#[cfg(target_os = "macos")]
fn refuse_translocated() -> Result<(), String> {
    let exe = std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .map_err(|err| err.to_string())?;
    match refusal(&exe.to_string_lossy()) {
        Some(reason) => Err(reason.to_owned()),
        None => Ok(()),
    }
}

/// Why start at login can't be turned on for an app running from `exe`, if it can't. macOS runs
/// an app opened straight from a disk image or from Downloads from a random temporary copy
/// ("App Translocation"); a login entry made then would point at nothing after a reboot.
pub fn refusal(exe: &str) -> Option<&'static str> {
    exe.contains("/AppTranslocation/")
        .then_some(MOVE_TO_APPLICATIONS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_translocated_app_is_refused() {
        assert_eq!(
            refusal(
                "/private/var/folders/x7/abc/T/AppTranslocation/0E1D-42/d/Little Joan.app/Contents/MacOS/little-joan"
            ),
            Some(MOVE_TO_APPLICATIONS)
        );
    }

    #[test]
    fn an_installed_app_is_allowed() {
        for exe in [
            "/Applications/Little Joan.app/Contents/MacOS/little-joan",
            "/Users/someone/Applications/Little Joan.app/Contents/MacOS/little-joan",
            "/usr/bin/little-joan",
            r"C:\Users\someone\AppData\Local\Little Joan\little-joan.exe",
        ] {
            assert_eq!(refusal(exe), None, "{exe}");
        }
    }
}
