//! Quick commands in the command bar (LJ42): actions and settings, each row showing its current
//! state. Only the bar may call these (capabilities/bar.json, and a test). The command set is
//! fixed — a serde enum that refuses any other kind or field —, and nothing a page supplies
//! chooses one: the bar builds them from its own rows (ui/command/quick.ts). Settings changes take
//! the path Settings' Save takes (settings_live.rs). Never from here, only from Settings: the
//! shortcut and the keys, developer mode, switching extensions, custom theme colors — they change
//! what code runs or what keys do.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Runtime};

use crate::autostart;
use crate::extensions::{self, session::PageInfo};
use crate::history::ClearRange;
use crate::history_store;
use crate::input::SearchEngine;
use crate::keys::PageAction;
use crate::page_actions;
use crate::settings::{Change, Theme};
use crate::settings_live;
use crate::settings_window::{self, SettingsTab};
use crate::smoke;
use crate::state;
use crate::tray;
use crate::welcome;

use super::{Choice, engine_choices, web_page};

/// What the quick commands' rows show (`quick_state`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickState {
    /// The open web page (its address, the window's title), or `null` — no page commands then.
    page: Option<PageInfo>,
    theme: Theme,
    search_engine: SearchEngine,
    engines: Vec<Choice>,
    mobile_sites: bool,
    remember_history: bool,
    /// The OS login entry exists (`null` when it can't be read).
    start_at_login: Option<bool>,
    show_tray_icon: bool,
    /// Shown although the setting is off: the shortcut can't work (LJ41's rule).
    icon_forced: bool,
}

/// One quick command, as the bar sends it: `{ "kind": "<kind>", …its fields }`. Values are checked
/// where they are used (settings::apply_change for the settings ones).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum QuickCommand {
    // Empty structs, not unit variants: serde's internally tagged unit variants ignore extra
    // fields despite deny_unknown_fields (a test sends one).
    Translate {},
    OpenInBrowser {},
    Reload {},
    SetThemeMode {
        mode: String,
    },
    /// A preset's whole theme (the bar builds it from ui/lib/presets.ts).
    SetTheme {
        theme: Value,
    },
    SetSearchEngine {
        id: String,
    },
    SetMobileSites {
        on: bool,
    },
    SetRememberHistory {
        on: bool,
    },
    ClearHistory {
        range: ClearRange,
    },
    SetStartAtLogin {
        on: bool,
    },
    OpenSettings {
        tab: String,
    },
    ReloadExtensions {},
    SetTrayIcon {
        on: bool,
    },
}

impl QuickCommand {
    /// The smoke log's line: the kind and its value — never a URL or a title.
    pub fn log_line(&self) -> String {
        match self {
            QuickCommand::Translate {} => "translate".to_owned(),
            QuickCommand::OpenInBrowser {} => "openInBrowser".to_owned(),
            QuickCommand::Reload {} => "reload".to_owned(),
            QuickCommand::SetThemeMode { mode } => format!("setThemeMode {}", word(mode)),
            QuickCommand::SetTheme { theme } => format!(
                "setTheme colors={}",
                theme
                    .get("colors")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len)
            ),
            QuickCommand::SetSearchEngine { id } => format!("setSearchEngine {}", word(id)),
            QuickCommand::SetMobileSites { on } => format!("setMobileSites {on}"),
            QuickCommand::SetRememberHistory { on } => format!("setRememberHistory {on}"),
            QuickCommand::ClearHistory { range } => format!("clearHistory {}", range.name()),
            QuickCommand::SetStartAtLogin { on } => format!("setStartAtLogin {on}"),
            QuickCommand::OpenSettings { tab } => format!("openSettings {}", word(tab)),
            QuickCommand::ReloadExtensions {} => "reloadExtensions".to_owned(),
            QuickCommand::SetTrayIcon { on } => format!("setTrayIcon {on}"),
        }
    }

    /// The settings change it makes, if it is a setting.
    pub fn change(&self) -> Option<Change> {
        let change = match self {
            QuickCommand::SetThemeMode { mode } => Change {
                theme_mode: Some(mode.clone()),
                ..Change::default()
            },
            QuickCommand::SetTheme { theme } => Change {
                theme: Some(theme.clone()),
                ..Change::default()
            },
            QuickCommand::SetSearchEngine { id } => Change {
                search_engine: Some(id.clone()),
                ..Change::default()
            },
            QuickCommand::SetMobileSites { on } => Change {
                mobile_sites: Some(*on),
                ..Change::default()
            },
            QuickCommand::SetRememberHistory { on } => Change {
                remember_history: Some(*on),
                ..Change::default()
            },
            QuickCommand::SetTrayIcon { on } => Change {
                show_tray_icon: Some(*on),
                ..Change::default()
            },
            _ => return None,
        };
        Some(change)
    }

    /// It acts on the open web page (refused without one).
    fn needs_page(&self) -> bool {
        matches!(
            self,
            QuickCommand::Translate {} | QuickCommand::OpenInBrowser {} | QuickCommand::Reload {}
        )
    }
}

/// A value the bar sent, fit for the log: up to 24 letters, digits and `-`, else `?`.
fn word(text: &str) -> &str {
    if !text.is_empty()
        && text.len() <= 24
        && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        text
    } else {
        "?"
    }
}

/// What the quick commands' rows show: the page, the settings they change, their state.
#[tauri::command]
pub async fn quick_state<R: Runtime>(app: AppHandle<R>) -> QuickState {
    let page = web_page(&app);
    let (settings, active) = state::with(&app, |s| (s.settings.clone(), s.active_shortcut.clone()));
    let works = tray::shortcut_works(active.as_deref(), welcome::wayland_no_shortcut_here());
    QuickState {
        page,
        theme: settings.theme,
        search_engine: settings.search_engine,
        engines: engine_choices(),
        mobile_sites: settings.mobile_sites,
        remember_history: settings.remember_history,
        start_at_login: autostart::is_enabled(&app),
        show_tray_icon: settings.show_tray_icon,
        icon_forced: !settings.show_tray_icon && tray::icon_visible(false, works),
    }
}

/// Runs a quick command. Settings ones apply and save at once (and an open settings window's
/// draft follows); `clearHistory` returns how many pages went. Logs `quick-command <kind>
/// [<value>]` once it was accepted.
#[tauri::command]
pub async fn quick_command<R: Runtime>(
    app: AppHandle<R>,
    command: QuickCommand,
) -> Result<Option<usize>, String> {
    if command.needs_page() && web_page(&app).is_none() {
        return Err("That works on a web page.".to_owned());
    }
    if let Some(change) = command.change() {
        let updated = settings_live::apply(&app, &change)?;
        smoke::log("quick-command", &command.log_line());
        settings_window::saved_changed(&app, &updated, None);
        return Ok(None);
    }
    let removed = match &command {
        QuickCommand::Translate {} => perform(&app, PageAction::Translate),
        QuickCommand::OpenInBrowser {} => perform(&app, PageAction::OpenInBrowser),
        QuickCommand::Reload {} => perform(&app, PageAction::Reload),
        QuickCommand::ClearHistory { range } => Some(
            history_store::clear(&app, *range)
                .map_err(|err| format!("Couldn't save the history: {err}"))?,
        ),
        QuickCommand::SetStartAtLogin { on } => {
            let now = autostart::set(&app, *on)?;
            let settings = state::with(&app, |s| s.settings.clone());
            settings_window::saved_changed(&app, &settings, Some(now));
            None
        }
        QuickCommand::OpenSettings { tab } => {
            let tab =
                SettingsTab::from_name(tab).ok_or_else(|| "There's no such tab.".to_owned())?;
            settings_window::open(&app, Some(tab));
            None
        }
        QuickCommand::ReloadExtensions {} => {
            extensions::registry::forget();
            None
        }
        // The settings ones, above.
        _ => None,
    };
    smoke::log("quick-command", &command.log_line());
    Ok(removed)
}

/// A page action (it runs off this thread); nothing to count.
fn perform<R: Runtime>(app: &AppHandle<R>, action: PageAction) -> Option<usize> {
    page_actions::perform(app, action);
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: Value) -> Result<QuickCommand, String> {
        serde_json::from_value(value).map_err(|err| err.to_string())
    }

    #[test]
    fn the_fixed_kinds_parse() {
        assert_eq!(
            parse(json!({ "kind": "reload" })),
            Ok(QuickCommand::Reload {})
        );
        assert_eq!(
            parse(json!({ "kind": "openInBrowser" })),
            Ok(QuickCommand::OpenInBrowser {})
        );
        assert_eq!(
            parse(json!({ "kind": "setMobileSites", "on": false })),
            Ok(QuickCommand::SetMobileSites { on: false })
        );
        assert_eq!(
            parse(json!({ "kind": "clearHistory", "range": "week" })),
            Ok(QuickCommand::ClearHistory {
                range: ClearRange::Week
            })
        );
        assert_eq!(
            parse(json!({ "kind": "setThemeMode", "mode": "dark" })),
            Ok(QuickCommand::SetThemeMode {
                mode: "dark".into()
            })
        );
        assert_eq!(
            parse(json!({ "kind": "openSettings", "tab": "privacy" })),
            Ok(QuickCommand::OpenSettings {
                tab: "privacy".into()
            })
        );
    }

    #[test]
    fn anything_else_is_refused() {
        for bad in [
            // Kinds that don't exist — the shortcut, keys, developer mode, extensions.
            json!({ "kind": "setShortcut", "accelerator": "Alt+KeyJ" }),
            json!({ "kind": "setDeveloperMode", "on": true }),
            json!({ "kind": "setExtension", "id": "hello", "on": false }),
            json!({ "kind": "setKeys", "keys": {} }),
            json!({ "kind": "Reload" }),
            json!({ "kind": "" }),
            json!({ "on": true }),
            // Fields that don't belong.
            json!({ "kind": "reload", "url": "https://example.com/" }),
            json!({ "kind": "setMobileSites", "on": true, "extra": 1 }),
            json!({ "kind": "setTrayIcon", "on": true, "kind2": "quit" }),
            // Wrong types and missing fields.
            json!({ "kind": "setMobileSites", "on": "yes" }),
            json!({ "kind": "setMobileSites" }),
            json!({ "kind": "clearHistory", "range": "year" }),
            json!({ "kind": "setSearchEngine", "id": 3 }),
            json!("reload"),
            json!(null),
        ] {
            assert!(parse(bad.clone()).is_err(), "{bad}");
        }
    }

    #[test]
    fn settings_commands_make_changes_and_actions_none() {
        assert_eq!(
            QuickCommand::SetMobileSites { on: false }.change(),
            Some(Change {
                mobile_sites: Some(false),
                ..Change::default()
            })
        );
        assert_eq!(
            QuickCommand::SetSearchEngine { id: "brave".into() }
                .change()
                .and_then(|c| c.search_engine),
            Some("brave".into())
        );
        for action in [
            QuickCommand::Reload {},
            QuickCommand::ReloadExtensions {},
            QuickCommand::ClearHistory {
                range: ClearRange::All,
            },
            QuickCommand::SetStartAtLogin { on: true },
            QuickCommand::OpenSettings {
                tab: "about".into(),
            },
        ] {
            assert_eq!(action.change(), None, "{action:?}");
        }
        assert!(QuickCommand::Translate {}.needs_page());
        assert!(!QuickCommand::SetTrayIcon { on: true }.needs_page());
    }

    #[test]
    fn the_log_says_the_kind_and_a_value_never_more() {
        assert_eq!(
            QuickCommand::SetMobileSites { on: false }.log_line(),
            "setMobileSites false"
        );
        assert_eq!(
            QuickCommand::SetThemeMode {
                mode: "dark".into()
            }
            .log_line(),
            "setThemeMode dark"
        );
        assert_eq!(
            QuickCommand::SetTheme {
                theme: json!({ "mode": "light", "colors": ["#c9a27e", "#d8c49a"] })
            }
            .log_line(),
            "setTheme colors=2"
        );
        assert_eq!(
            QuickCommand::SetSearchEngine {
                id: "https://evil.example/?q=1".into()
            }
            .log_line(),
            "setSearchEngine ?"
        );
        assert_eq!(
            QuickCommand::ClearHistory {
                range: ClearRange::Hour
            }
            .log_line(),
            "clearHistory hour"
        );
    }
}
