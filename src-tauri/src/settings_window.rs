//! The settings window: created on demand, destroyed on close, like the page window. A sidebar
//! of tabs (General, Keyboard, Appearance, Extensions, Privacy, About) — the "advanced settings" that sit
//! where Little Arc has "Contact the Team" (the command bar's "Settings & extensions" row). It
//! opens on a tab from `--settings [tab]`, `open_settings(tab)`, the tray and Cmd+,; an open
//! window is focused and switched to the tab asked for. Its webview uses the page window's webview
//! data folder (LJ51): on Windows its WebView2 profile is the page's, so Privacy's "Remove saved
//! passwords and form data…" reaches what the page saved, also while Little Joan hibernates.

use std::path::PathBuf;

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};

use crate::keys;
use crate::navigation;
use crate::platform;
use crate::shortcut;
use crate::smoke;
use crate::state::{self, AppState};

pub const LABEL: &str = "settings";

/// What the window's builder takes from the state (LJ51): the page window's webview data folder
/// (`AppState::webview_data_dir`; `None` in installed builds, where Tauri gives every webview the
/// app's local data folder) — one WebView2 profile with the page, whose environment options all
/// keep wry's defaults (logins.rs) — and the saved form autofill, which that profile shares.
#[derive(Debug, Clone, PartialEq, Eq)]
struct BuilderInput {
    data_directory: Option<PathBuf>,
    autofill_forms: bool,
}

fn builder_input(state: &AppState) -> BuilderInput {
    BuilderInput {
        data_directory: state.webview_data_dir.clone(),
        autofill_forms: state.settings.autofill_forms,
    }
}

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
        // macOS: the app may be hidden — a ready page window gave the keyboard back (LJ19.4).
        platform::take_focus_back(&app);
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.unminimize();
            let _ = platform::bring_to_front(&window.as_ref().window());
            platform::report_front(&window.as_ref().window());
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
        let input = state::with(&app, |s| builder_input(s));
        let mut builder =
            WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("settings.html".into()))
                .title("Little Joan Settings")
                .inner_size(SIZE.0, SIZE.1)
                .resizable(false)
                .maximizable(false)
                .minimizable(false)
                .center()
                // Shown below, once macOS knows it may join the Space the user is on (LJ19.2).
                .visible(false)
                .focused(true)
                .general_autofill_enabled(input.autofill_forms)
                // The tab to open on, before the page's own script runs.
                .initialization_script(format!(
                    "window.__littleJoanSettingsTab = \"{}\";",
                    tab.name()
                ))
                // A trusted window stays on the app's own pages; links (About's repository link)
                // open in the main browser (web URLs only; browsers.rs).
                .on_navigation(|url| may_navigate(url, &keys::app_origin()))
                .on_new_window({
                    let app = app.clone();
                    move |url, _features| {
                        if navigation::is_external_openable(&url) {
                            crate::browsers::hand_off_new_window(
                                &app,
                                url,
                                crate::browsers::NewWindowFrom::Settings,
                            );
                        }
                        NewWindowResponse::Deny
                    }
                });
        // The page's profile (LJ51): Privacy's removal acts on what the page saved.
        if let Some(dir) = input.data_directory {
            builder = builder.data_directory(dir);
        }
        match builder.build() {
            Ok(window) => {
                // Smoke log (Windows): its profile's folder, for CI to compare with the page's.
                crate::logins::report(window.as_ref(), crate::logins::Place::Settings);
                // Like the page window: on the current Space (macOS), then in front (LJ19.2).
                let window = window.as_ref().window();
                platform::join_active_space(&window);
                if let Err(err) = window.show() {
                    eprintln!("little-joan: could not show settings: {err}");
                }
                let _ = platform::bring_to_front(&window);
                smoke::log("settings-window", tab.name());
                platform::report_front(&window);
            }
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

/// Smoke tests only (LJ19.7; `LITTLEJOAN_SMOKE_SETTINGS_QUIT=1` in the app's environment, ignored
/// without the smoke log): `SMOKE_QUIT_AFTER` after the settings page asked for its settings, its
/// "Quit Little Joan" button is clicked — the page's own handler runs and calls
/// `quit_from_settings` as a click would. CI has no driver for the window's controls (the button
/// sits below the fold of the General tab).
pub fn smoke_click_quit<R: Runtime>(app: &AppHandle<R>) {
    if !smoke::enabled()
        || std::env::var("LITTLEJOAN_SMOKE_SETTINGS_QUIT")
            .ok()
            .as_deref()
            != Some("1")
    {
        return;
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-smoke-quit".into())
        .spawn(move || {
            std::thread::sleep(SMOKE_QUIT_AFTER);
            if let Some(window) = app.get_webview_window(LABEL) {
                smoke::log("settings-quit-click", "");
                let _ = window.eval("document.getElementById(\"quit\")?.click()");
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not plan the smoke test's click: {err}");
    }
}

/// How long after Settings loaded the smoke test's click comes (its screenshot is taken first).
const SMOKE_QUIT_AFTER: std::time::Duration = std::time::Duration::from_secs(4);

/// Smoke tests only (`LITTLEJOAN_SMOKE_SETTINGS_SCROLL=<section>`, ignored without the smoke log): 2 s
/// after the settings page asked for its settings, the open tab scrolls to one of its sections below
/// the fold for the screenshot — General's Browser (`browser`, LJ46) or Privacy's saved passwords
/// (`logins`, LJ51) — and logs `settings-scrolled <section>`. With `LITTLEJOAN_SMOKE_SETTINGS_LOGINS=
/// remove` too (Windows), 4 s later Privacy's "Remove saved passwords and form data…" is clicked and
/// then its question's Remove — the page's own handlers run and call `clear_saved_logins` as clicks
/// would (`settings-logins-click`). CI has no driver for the window's controls.
pub fn smoke_scroll<R: Runtime>(app: &AppHandle<R>) {
    if !smoke::enabled() {
        return;
    }
    let Some((section, anchor)) = std::env::var("LITTLEJOAN_SMOKE_SETTINGS_SCROLL")
        .ok()
        .and_then(|wanted| SMOKE_SECTIONS.iter().find(|(name, _)| *name == wanted))
        .copied()
    else {
        return;
    };
    let remove = section == "logins"
        && std::env::var("LITTLEJOAN_SMOKE_SETTINGS_LOGINS")
            .ok()
            .as_deref()
            == Some("remove");
    let app = app.clone();
    let _ = std::thread::Builder::new()
        .name("little-joan-smoke-scroll".into())
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let Some(window) = app.get_webview_window(LABEL) else {
                return;
            };
            // Ids of our own page (SMOKE_SECTIONS): safe in the script as they are.
            let _ = window.eval(format!(
                "document.getElementById(\"{anchor}\")?.scrollIntoView({{ block: \"start\" }})"
            ));
            smoke::log("settings-scrolled", section);
            if !remove {
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(4));
            smoke::log("settings-logins-click", "remove");
            let _ = window.eval("document.getElementById(\"logins-remove\")?.click()");
            std::thread::sleep(std::time::Duration::from_millis(600));
            smoke::log("settings-logins-click", "confirm");
            let _ = window.eval("document.getElementById(\"logins-go\")?.click()");
        });
}

/// The sections `smoke_scroll` may scroll to: its name in the environment, the element's id.
const SMOKE_SECTIONS: [(&str, &str); 2] =
    [("browser", "browser-label"), ("logins", "logins-label")];

/// When the window closes: the shortcut it may have paused while recording comes back, and a
/// theme (or Liquid Glass setting) it was previewing without saving (Cancel, or the close button)
/// gives way to the saved one in the page window.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() == LABEL && matches!(event, WindowEvent::Destroyed) {
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            shortcut::set_suspended(&app, false);
            let previewed = state::with(&app, |s| {
                let theme = s.theme_preview.take().is_some();
                let glass = s.glass_preview.take().is_some();
                theme || glass
            });
            if previewed {
                crate::glass::refresh(&app);
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
    fn the_window_gets_the_pages_data_folder_and_the_saved_autofill() {
        // LJ51: the settings window's WebView2 profile is the page's, so Privacy's removal reaches
        // what the page saved — the folder the page's builder (bar/page.rs) takes from the state.
        let mut state = AppState::for_tests();
        let folder = std::env::temp_dir()
            .join("little-joan-test-data")
            .join("webview");
        state.webview_data_dir = Some(folder.clone());
        state.settings.autofill_forms = false;
        assert_eq!(
            builder_input(&state),
            BuilderInput {
                data_directory: Some(folder),
                autofill_forms: false,
            }
        );
        // Installed builds: no folder of ours — Tauri gives every webview the same one.
        state.webview_data_dir = None;
        assert_eq!(builder_input(&state).data_directory, None);
        let page = include_str!("bar/page.rs");
        assert!(page.contains("state::with(app, |s| s.webview_data_dir.clone())"));
        let source = include_str!("settings_window.rs");
        let code = source.split("#[cfg(test)]").next().unwrap_or(source);
        let input = code
            .find("let input = state::with(&app, |s| builder_input(s));")
            .expect("open() reads the builder's input");
        let folder = code
            .find("builder = builder.data_directory(dir);")
            .expect("the builder gets the folder");
        assert!(input < folder);
    }

    #[test]
    fn smoke_sections_are_ids_safe_in_a_script_string() {
        for (name, id) in SMOKE_SECTIONS {
            assert!(name.chars().all(|c| c.is_ascii_lowercase()), "{name}");
            assert!(
                id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{id}"
            );
        }
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
