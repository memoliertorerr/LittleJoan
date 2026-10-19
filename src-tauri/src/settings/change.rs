//! One change to the settings (LJ42): what Settings' Save sends, or one quick command's from the
//! command bar. `apply_change` checks every part and returns the settings it makes — or refuses
//! the whole change with a message for the user, so the settings stay as they were. Save and the
//! quick commands both go through it (settings_live.rs puts the result to work), so the two can't
//! drift. Pure: no Tauri types.

use serde_json::Value;

use super::{Settings, ThemeMode, disabled_extensions, site_modes, theme};
use crate::input::SearchEngine;
use crate::translate::is_valid_lang;

/// The parts a change may set; `None` leaves that part alone.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Change {
    /// The global shortcut (accelerator syntax). The caller registers it first (shortcut.rs).
    pub shortcut: Option<String>,
    /// A search engine's id: `google`, `duckduckgo`, `bing`, `brave`.
    pub search_engine: Option<String>,
    pub translate_to: Option<String>,
    pub mobile_sites: Option<bool>,
    pub remember_history: Option<bool>,
    /// A whole theme, valid as it stands (settings/theme.rs `parse_strict`).
    pub theme: Option<Value>,
    /// The theme's mode alone: `system`, `light` or `dark`.
    pub theme_mode: Option<String>,
    /// Settings → Extensions' switches (checked one by one, like settings.json's).
    pub disabled_extensions: Option<Value>,
    pub developer_mode: Option<bool>,
    /// General's per-site list (checked entry by entry).
    pub site_modes: Option<Value>,
    pub show_tray_icon: Option<bool>,
}

impl Change {
    /// It sets the theme or its mode.
    pub fn touches_theme(&self) -> bool {
        self.theme.is_some() || self.theme_mode.is_some()
    }

    /// It switches extensions or Developer mode.
    pub fn touches_extensions(&self) -> bool {
        self.disabled_extensions.is_some() || self.developer_mode.is_some()
    }
}

/// The settings `change` makes of `current`, or why it is refused (then nothing changes).
pub fn apply_change(current: &Settings, change: &Change) -> Result<Settings, String> {
    let mut next = current.clone();
    if let Some(shortcut) = &change.shortcut {
        let shortcut = shortcut.trim();
        if shortcut.is_empty() {
            return Err("Record a shortcut first.".to_owned());
        }
        next.shortcut = shortcut.to_owned();
    }
    if let Some(id) = &change.search_engine {
        next.search_engine = serde_json::from_value::<SearchEngine>(Value::String(id.clone()))
            .map_err(|_| "Pick a search engine from the list.".to_owned())?;
    }
    if let Some(lang) = &change.translate_to {
        if !is_valid_lang(lang) {
            return Err("Pick a language from the list.".to_owned());
        }
        lang.clone_into(&mut next.translate_to);
    }
    if let Some(on) = change.mobile_sites {
        next.mobile_sites = on;
    }
    if let Some(on) = change.remember_history {
        next.remember_history = on;
    }
    if let Some(value) = &change.theme {
        next.theme =
            theme::parse_strict(value).ok_or_else(|| "That theme can't be used.".to_owned())?;
    }
    if let Some(mode) = &change.theme_mode {
        next.theme.mode =
            ThemeMode::from_name(mode).ok_or_else(|| "Pick System, Light or Dark.".to_owned())?;
    }
    if let Some(value) = &change.disabled_extensions {
        next.disabled_extensions = disabled_extensions(Some(value));
    }
    if let Some(on) = change.developer_mode {
        next.developer_mode = on;
    }
    if let Some(value) = &change.site_modes {
        next.site_modes = site_modes::parse(Some(value));
    }
    if let Some(on) = change.show_tray_icon {
        next.show_tray_icon = on;
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Theme;
    use serde_json::json;

    #[test]
    fn a_change_sets_only_its_parts() {
        let current = Settings::default();
        let next = apply_change(
            &current,
            &Change {
                search_engine: Some("duckduckgo".into()),
                mobile_sites: Some(false),
                show_tray_icon: Some(false),
                ..Change::default()
            },
        )
        .expect("valid");
        assert_eq!(next.search_engine, SearchEngine::DuckDuckGo);
        assert!(!next.mobile_sites);
        assert!(!next.show_tray_icon);
        assert_eq!(
            Settings {
                search_engine: current.search_engine,
                mobile_sites: current.mobile_sites,
                show_tray_icon: current.show_tray_icon,
                ..next
            },
            current,
            "nothing else moved"
        );
        assert_eq!(apply_change(&current, &Change::default()), Ok(current));
    }

    #[test]
    fn a_bad_part_refuses_the_whole_change() {
        let current = Settings::default();
        for bad in [
            Change {
                search_engine: Some("yahoo".into()),
                mobile_sites: Some(false),
                ..Change::default()
            },
            Change {
                search_engine: Some("DuckDuckGo".into()),
                ..Change::default()
            },
            Change {
                theme_mode: Some("sepia".into()),
                remember_history: Some(false),
                ..Change::default()
            },
            Change {
                theme: Some(
                    json!({ "mode": "dark", "colors": ["#123456", "teal"], "intensity": 0.5, "grain": 0.2 }),
                ),
                ..Change::default()
            },
            Change {
                translate_to: Some("klingon".into()),
                ..Change::default()
            },
            Change {
                shortcut: Some("  ".into()),
                ..Change::default()
            },
        ] {
            assert!(apply_change(&current, &bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_theme_and_its_mode_are_checked() {
        let current = Settings::default();
        let dark = apply_change(
            &current,
            &Change {
                theme_mode: Some("dark".into()),
                ..Change::default()
            },
        )
        .expect("valid");
        assert_eq!(dark.theme.mode, ThemeMode::Dark);
        assert_eq!(
            dark.theme.colors, current.theme.colors,
            "only the mode changed"
        );
        let preset = apply_change(
            &dark,
            &Change {
                theme: Some(
                    json!({ "mode": "dark", "colors": ["#C9A27E", "#d8c49a"], "intensity": 0.55, "grain": 0.3 }),
                ),
                ..Change::default()
            },
        )
        .expect("valid");
        assert_eq!(
            preset.theme,
            Theme {
                mode: ThemeMode::Dark,
                colors: vec!["#c9a27e".into(), "#d8c49a".into()],
                intensity: 0.55,
                grain: 0.3,
            }
        );
        assert!(
            Change {
                theme_mode: Some("light".into()),
                ..Change::default()
            }
            .touches_theme()
        );
        assert!(!Change::default().touches_theme());
    }

    #[test]
    fn lists_are_checked_entry_by_entry() {
        let next = apply_change(
            &Settings::default(),
            &Change {
                disabled_extensions: Some(json!(["hello", "Bad!", "builtin:calc"])),
                site_modes: Some(json!({ "Example.com": "desktop", "bad host": "phone" })),
                developer_mode: Some(true),
                ..Change::default()
            },
        )
        .expect("valid");
        assert_eq!(next.disabled_extensions, vec!["hello", "builtin:calc"]);
        assert_eq!(
            next.site_modes,
            std::collections::BTreeMap::from([(
                "example.com".to_owned(),
                crate::settings::SiteMode::Desktop
            )])
        );
        assert!(next.developer_mode);
    }
}
