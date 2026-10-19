//! Little Joan's keys (LJ43): `keymap_info`, for Settings → Keyboard (LJ44) and the bar. The bar
//! and the settings window may call it (capabilities, and a test); it only reads. Also the
//! portal's summon shortcut as the bar and Settings show it (LJ17).

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::keymap::ActionInfo;
use crate::shortcut::Problem;
use crate::shortcut::backend::PortalState;
use crate::smoke;
use crate::state;
use crate::system_shortcuts::SystemShortcut;

/// Every action with its key now and its defaults, and the global shortcut beside them.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeymapInfo {
    /// `std::env::consts::OS`: "macos", "windows", "linux".
    platform: &'static str,
    /// `[{ id, title, binding, defaults }]`, in Settings' order; titles as the menu has them.
    actions: Vec<ActionInfo>,
    /// The saved global shortcut (accelerator syntax).
    global_shortcut: String,
    /// It couldn't be registered at launch (LJ44): `{ wanted, active, reason: taken | invalid }`.
    shortcut_problem: Option<Problem>,
    /// The XDG GlobalShortcuts portal holds it (LJ17): `{ trigger }`, the desktop's own words for
    /// the key; Keyboard shows that instead of the recorder.
    portal_shortcut: Option<PortalShortcut>,
    /// macOS's own shortcuts, read from its preferences (`[{ id, enabled, accelerator }]`); empty
    /// elsewhere (system_shortcuts.rs).
    system_shortcuts: Vec<SystemShortcut>,
}

#[tauri::command]
pub async fn keymap_info<R: Runtime>(app: AppHandle<R>) -> KeymapInfo {
    let (actions, global_shortcut, shortcut_problem, portal) = state::with(&app, |s| {
        (
            s.keymap.info(),
            s.settings.shortcut.clone(),
            s.shortcut_problem.clone(),
            s.portal.clone(),
        )
    });
    let system_shortcuts = crate::platform::system_shortcuts();
    if cfg!(target_os = "macos") {
        // What was read (CI compares it with the runner's export): entries, and those with keys.
        smoke::log(
            "system-shortcuts",
            &format!(
                "{} keys={}",
                system_shortcuts.len(),
                system_shortcuts
                    .iter()
                    .filter(|s| s.accelerator.is_some())
                    .count()
            ),
        );
    }
    KeymapInfo {
        platform: std::env::consts::OS,
        actions,
        global_shortcut,
        shortcut_problem,
        portal_shortcut: PortalShortcut::from_state(&portal),
        system_shortcuts,
    }
}

/// The portal's shortcut as the bar and Settings show it (LJ17).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PortalShortcut {
    /// What the desktop calls its key ("Press <ctrl><shift>space" on GNOME), or `null`.
    trigger: Option<String>,
}

impl PortalShortcut {
    /// `None` unless the portal holds the shortcut.
    pub(crate) fn from_state(portal: &PortalState) -> Option<PortalShortcut> {
        portal.registered().then(|| PortalShortcut {
            trigger: portal.trigger().map(str::to_owned),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_portals_shortcut_is_shown_only_while_it_holds_it() {
        assert_eq!(PortalShortcut::from_state(&PortalState::None), None);
        assert_eq!(PortalShortcut::from_state(&PortalState::Asking), None);
        let held = PortalShortcut::from_state(&PortalState::Registered {
            trigger: Some("Press <ctrl><shift>space".into()),
        });
        assert_eq!(
            serde_json::to_value(held).expect("json"),
            serde_json::json!({ "trigger": "Press <ctrl><shift>space" })
        );
        let keyless = PortalShortcut::from_state(&PortalState::Registered { trigger: None });
        assert_eq!(
            serde_json::to_value(keyless).expect("json"),
            serde_json::json!({ "trigger": null })
        );
    }
}
