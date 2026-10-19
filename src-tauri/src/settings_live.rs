//! Settings changes at runtime (LJ42): the one path Settings' Save and the command bar's quick
//! commands share. `settings::apply_change` decides what the settings become (pure, tested); this
//! puts the result to work — AppState, a new theme in the bar, the tray icon's rule (LJ41), phone
//! mode, the extensions' registry — and saves settings.json (atomically).

use tauri::{AppHandle, Runtime};

use crate::bar;
use crate::extensions;
use crate::keymap::Keymap;
use crate::phone_mode;
use crate::settings::{self, Change, Settings};
use crate::smoke;
use crate::state;

/// Applies `change` (checked again against the settings of this moment, under the state lock) and
/// saves it. Refused: nothing changed, and the message says why. A theme change ends a preview
/// Settings may be showing; the open window's bar gets the new theme (smoke log `theme <mode>
/// colors=<n>`). Callers tell the settings window themselves (Save closes it; a quick command
/// rebases its draft, settings_window::saved_changed).
pub fn apply<R: Runtime>(app: &AppHandle<R>, change: &Change) -> Result<Settings, String> {
    let (dir, before, updated, theme_changed) = state::with(app, |s| {
        let next = settings::apply_change(&s.settings, change)?;
        let shown = s.shown_theme();
        if change.touches_theme() {
            s.theme_preview = None;
        }
        let before = std::mem::replace(&mut s.settings, next);
        // The keys follow the settings (LJ43): the native hooks read this keymap.
        s.keymap = Keymap::new(s.keymap.platform(), &s.settings.keys);
        Ok::<_, String>((
            s.data_dir.clone(),
            before,
            s.settings.clone(),
            s.shown_theme() != shown,
        ))
    })?;
    if theme_changed {
        bar::forget_painted(app);
        bar::set_theme(app, &updated.theme);
        smoke::log("theme", &crate::commands::theme_summary(&updated.theme));
    }
    // The icon shown or hidden by the switch and whether the shortcut works (LJ41).
    crate::tray::apply_visibility(app);
    if change.touches_extensions() {
        // Switched-off extensions and Developer mode take effect with the next read.
        extensions::registry::forget();
    }
    // macOS: the menu's key equivalents come from the keymap.
    #[cfg(target_os = "macos")]
    if before.keys != updated.keys {
        crate::menu::refresh(app);
    }
    if before.mobile_sites != updated.mobile_sites || before.site_modes != updated.site_modes {
        // Phone mode on or off, or a site's mode: at once on the open page.
        phone_mode::apply(app, true);
    }
    settings::save(&dir, &updated)
        .map_err(|err| format!("The new settings work now, but couldn't be saved: {err}"))?;
    Ok(updated)
}
