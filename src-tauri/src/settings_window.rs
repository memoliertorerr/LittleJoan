//! The settings window: created on demand, destroyed on close, like the page window. A sidebar
//! of tabs (General, Keyboard, Appearance, Extensions, Privacy, About) — the "advanced settings" that sit
//! where Little Arc has "Contact the Team" (the command bar's "Settings & extensions" row). It
//! opens on a tab from `--settings [tab]`, `open_settings(tab)`, the tray and Cmd+,; an open
//! window is focused and switched to the tab asked for.

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};

use crate::bar;
use crate::keys;
use crate::navigation;
use crate::shortcut;
use crate::smoke;
use crate::state;

pub const LABEL: &str = "settings";

/// The window's size in logical pixels: a sidebar on the left, the tab on the right.
const SIZE: (f64, f64) = (600.0, 480.0);

/// A tab of the settings window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SettingsTab {
    #[default]
    General,
    /// Every key of Little Joan's own and the global shortcut (LJ44).
    Keyboard,
    Appearance,
    Extensions,
    Privacy,
    About,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 6] = [
        SettingsTab::General,
        SettingsTab::Keyboard,
        SettingsTab::Appearance,
        SettingsTab::Extensions,
        SettingsTab::Privacy,
        SettingsTab::About,
    ];

    /// The tab's name on the command line and in the page (`general`, …, `about`).
    pub fn name(self) -> &'static str {
        match self {
            SettingsTab::General => "general",
            SettingsTab::Keyboard => "keyboard",
            SettingsTab::Appearance => "appearance",
            SettingsTab::Extensions => "extensions",
            SettingsTab::Privacy => "privacy",
            SettingsTab::About => "about",
        }
    }

    /// A tab by name, in any case; anything else is no tab.
    pub fn from_name(name: &str) -> Option<SettingsTab> {
        let name = name.trim();
        SettingsTab::ALL
            .into_iter()
            .find(|tab| tab.name().eq_ignore_ascii_case(name))
    }
}

/// Opens the settings window on `tab` (General when it is new and no tab is asked for), or
/// focuses it if it is already open, switching to `tab` if one is asked for. Runs off the main
/// thread (building a webview window on the main thread can deadlock on Windows).
pub fn open<R: Runtime>(app: &AppHandle<R>, tab: Option<SettingsTab>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.unminimize();
            let _ = window.set_focus();
            if let Some(tab) = tab {
                // The name is one of ours (no user text), so it can go into the script as is.
                let _ = window.eval(format!(
                    "window.littleJoanSettings && window.littleJoanSettings.showTab(\"{}\")",
                    tab.name()
                ));
            }
            smoke::log("settings-window", tab.map_or("focus", SettingsTab::name));
            return;
        }
        let tab = tab.unwrap_or_default();
        let built = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("settings.html".into()))
            .title("Little Joan Settings")
            .inner_size(SIZE.0, SIZE.1)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .center()
            .focused(true)
            // The tab to open on, before the page's own script runs.
            .initialization_script(format!(
                "window.__littleJoanSettingsTab = \"{}\";",
                tab.name()
            ))
            // A trusted window stays on the app's own pages; links (About's repository link)
            // open in the default browser (web URLs only).
            .on_navigation(|url| may_navigate(url, &keys::app_origin()))
            .on_new_window(|url, _features| {
                if navigation::is_external_openable(&url)
                    && let Err(err) = tauri_plugin_opener::open_url(url.as_str(), None::<&str>)
                {
                    eprintln!("little-joan: could not open the link in the default browser: {err}");
                }
                NewWindowResponse::Deny
            })
            .build();
        match built {
            Ok(_) => smoke::log("settings-window", tab.name()),
            Err(err) => eprintln!("little-joan: could not open settings: {err}"),
        }
    });
}

/// Shows `theme` in the open settings window at once (after a save).
pub fn set_theme<R: Runtime>(app: &AppHandle<R>, theme: &crate::settings::Theme) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    // Checked hex colors, a mode name and two numbers: the JSON is a safe script literal.
    if let Ok(json) = serde_json::to_string(theme) {
        let _ = window.eval(format!(
            "window.littleJoanSettings && window.littleJoanSettings.setTheme({json})"
        ));
    }
}

/// Tells an open settings window that settings changed from the command bar (a quick command,
/// LJ42) — and, with `start_at_login`, that the login entry did — so its draft follows
/// (settings-logic.ts `rebase`: a field it didn't edit takes the new value) and a later Save
/// doesn't undo the change.
pub fn saved_changed<R: Runtime>(
    app: &AppHandle<R>,
    settings: &crate::settings::Settings,
    start_at_login: Option<bool>,
) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    // Checked settings (settings.rs) as JSON: one value each, a safe script literal.
    if let (Ok(json), Ok(login)) = (
        serde_json::to_string(settings),
        serde_json::to_string(&start_at_login),
    ) {
        let _ = window.eval(format!(
            "window.littleJoanSettings && window.littleJoanSettings.savedChanged({json}, {login})"
        ));
    }
}

/// When the window closes: the shortcut it may have paused while recording comes back, and a
/// theme it was previewing without saving (Cancel, or the close button) gives way to the saved
/// one in the page window.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() == LABEL && matches!(event, WindowEvent::Destroyed) {
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            shortcut::set_suspended(&app, false);
            let previewed = state::with(&app, |s| {
                s.theme_preview.take().map(|_| s.settings.theme.clone())
            });
            if let Some(saved) = previewed {
                bar::set_theme(&app, &saved);
                smoke::log("theme-preview", "ended");
            }
            // The Extensions tab read the registry; with no page window, nothing stays at rest.
            if app.get_window(crate::joan::LABEL).is_none() {
                crate::extensions::registry::forget();
            }
        });
    }
}

/// The settings window's navigation guard: the app's own pages only — never a web page, never an
/// extension's panel in any form.
fn may_navigate(url: &url::Url, app_origin: &url::Url) -> bool {
    keys::is_app_url(app_origin, url) && !crate::extensions::urls::is_ext_like(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_stays_on_the_app_and_never_opens_an_extension() {
        let parse = |s: &str| url::Url::parse(s).expect("valid");
        for origin in ["tauri://localhost/", "http://tauri.localhost/"] {
            let origin = parse(origin);
            assert!(may_navigate(
                &origin.join("settings.html").expect("join"),
                &origin
            ));
            for s in [
                "ljext://hello.localhost/index.html",
                "http://ljext.hello.localhost/index.html",
                "https://github.com/memoliertorerr/LittleJoan",
                "about:blank",
            ] {
                assert!(!may_navigate(&parse(s), &origin), "{s}");
            }
        }
    }

    #[test]
    fn tabs_go_by_name_in_any_case() {
        for tab in SettingsTab::ALL {
            assert_eq!(SettingsTab::from_name(tab.name()), Some(tab));
            assert_eq!(
                SettingsTab::from_name(&tab.name().to_uppercase()),
                Some(tab)
            );
        }
        assert_eq!(
            SettingsTab::from_name(" Privacy "),
            Some(SettingsTab::Privacy)
        );
        assert_eq!(
            SettingsTab::from_name("Keyboard"),
            Some(SettingsTab::Keyboard)
        );
        assert_eq!(
            SettingsTab::ALL[1],
            SettingsTab::Keyboard,
            "between General and Appearance"
        );
        for bad in ["", "advanced", "privacy-tab", "--quit"] {
            assert_eq!(SettingsTab::from_name(bad), None, "{bad}");
        }
        assert_eq!(SettingsTab::default(), SettingsTab::General);
    }

    #[test]
    fn tab_names_are_safe_in_a_script_string() {
        for tab in SettingsTab::ALL {
            assert!(
                tab.name().chars().all(|c| c.is_ascii_lowercase()),
                "{}",
                tab.name()
            );
        }
    }
}
