//! Shared app state: the current settings, where they live, and the active shortcut.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use tauri::{AppHandle, Manager, Runtime};

use crate::cli::Args;
use crate::cookies::SessionCookies;
use crate::extensions::theme::Tokens;
use crate::history_store::OpenHistory;
use crate::settings::{self, SETTINGS_FILE, Settings, Theme};

/// Development builds point this at `.cache/dev-data` (see scripts/env.sh). Shipped builds never
/// set it, so settings live in the OS config folder and browsing data where Tauri puts it (see
/// `AppState::webview_data_dir`).
pub const DATA_DIR_ENV: &str = "LITTLEJOAN_DATA_DIR";

pub struct AppState {
    pub settings: Settings,
    /// Folder holding settings.json and history.json.
    pub data_dir: PathBuf,
    /// Webview data folder override: `<LITTLEJOAN_DATA_DIR>/webview`. `None` (shipped builds)
    /// leaves it to Tauri, which on Windows and Linux forces the app's local data folder
    /// (`%LOCALAPPDATA%\<identifier>`, `~/.local/share/<identifier>`; tauri 2.12's
    /// manager/webview.rs). macOS ignores both: WebKit's default data store, keyed by the bundle
    /// id, holds cookies and site data there (docs/architecture.md, "Where data lives").
    pub webview_data_dir: Option<PathBuf>,
    /// The accelerator currently registered as the global shortcut, if any.
    pub active_shortcut: Option<String>,
    /// True while the settings window is recording a new shortcut (the active one is paused).
    pub shortcut_suspended: bool,
    /// A first run (no settings.json at launch) whose welcome the start page hasn't shown yet.
    /// Setup clears it when the first launch had flags; `start_info` clears it once read.
    pub first_run: bool,
    /// Session cookies kept from the last window for the next one, in memory only (cookies.rs;
    /// only where the engine drops them with the window).
    pub session_cookies: SessionCookies,
    /// Browsing history, only while the page window exists (history_store.rs); `None` at rest.
    pub history: Option<OpenHistory>,
    /// The theme Settings' Appearance tab is previewing (not saved), while that window is open:
    /// a page window summoned meanwhile shows it too. `None` = the saved theme.
    pub theme_preview: Option<Theme>,
    /// The scheme and `--lj-bg-0` the bar painted last, kept across windows for the next bar's
    /// webview background (bar/background.rs); `None` until a bar painted, or after the theme
    /// changed.
    pub painted_background: Option<Tokens>,
}

impl AppState {
    /// The theme the bar shows: the preview while Settings edits one, else the saved theme.
    pub fn shown_theme(&self) -> Theme {
        self.theme_preview
            .clone()
            .unwrap_or_else(|| self.settings.theme.clone())
    }
}

pub type SharedState = Mutex<AppState>;

/// Loads settings and registers the shared state. Call once, in setup.
pub fn init<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let (data_dir, webview_data_dir) =
        match std::env::var_os(DATA_DIR_ENV).filter(|v| !v.is_empty()) {
            Some(dir) => {
                let dir = PathBuf::from(dir);
                let webview = dir.join("webview");
                (dir, Some(webview))
            }
            None => (app.path().app_config_dir()?, None),
        };
    let first_run = is_first_run(&data_dir);
    let settings = settings::load(&data_dir);
    app.manage(Mutex::new(AppState {
        settings,
        data_dir,
        webview_data_dir,
        active_shortcut: None,
        shortcut_suspended: false,
        first_run,
        session_cookies: SessionCookies::default(),
        history: None,
        theme_preview: None,
        painted_background: None,
    }));
    Ok(())
}

/// A first run: `dir` has no settings.json. A file that exists but can't be read or parsed is
/// not one (Little Joan ran here before), and neither is a folder that can't be looked into.
pub fn is_first_run(dir: &Path) -> bool {
    matches!(dir.join(SETTINGS_FILE).try_exists(), Ok(false))
}

/// A first run's bookkeeping: saves `settings` (the defaults) at once, so the welcome appears
/// only once even if the user quits straight away. Returns whether to show the welcome: only
/// when the first launch had no flags — with flags it acts on them instead.
pub fn finish_first_run(dir: &Path, settings: &Settings, args: &Args) -> bool {
    if let Err(err) = settings::save(dir, settings) {
        eprintln!("little-joan: could not save the first settings: {err}");
    }
    args.command.is_none() && args.start_at_login.is_none() && args.settings.is_none() && !args.quit
}

/// Runs `f` with the state locked. Never call anything that waits on the main thread (window
/// getters, builders) inside `f`: the main thread may be waiting for this lock.
pub fn with<R: Runtime, T>(app: &AppHandle<R>, f: impl FnOnce(&mut AppState) -> T) -> T {
    let state = app.state::<SharedState>();
    let mut guard = state.lock().unwrap_or_else(PoisonError::into_inner);
    f(&mut guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toggle::Command;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "little-joan-test-state-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn the_bar_shows_the_preview_while_settings_edits_one() {
        let mut state = AppState {
            settings: Settings::default(),
            data_dir: temp_dir("preview"),
            webview_data_dir: None,
            active_shortcut: None,
            shortcut_suspended: false,
            first_run: false,
            session_cookies: SessionCookies::default(),
            history: None,
            theme_preview: None,
            painted_background: None,
        };
        assert_eq!(state.shown_theme(), Theme::default());
        let preview = Theme {
            colors: vec!["#9d8fd0".into()],
            ..Theme::default()
        };
        state.theme_preview = Some(preview.clone());
        assert_eq!(state.shown_theme(), preview);
        assert_eq!(
            state.settings.theme,
            Theme::default(),
            "a preview is never saved"
        );
    }

    #[test]
    fn a_first_run_is_a_missing_settings_file() {
        let dir = temp_dir("missing");
        assert!(is_first_run(&dir));
        fs::create_dir_all(&dir).expect("create");
        assert!(is_first_run(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_invalid_settings_file_is_not_a_first_run() {
        let dir = temp_dir("invalid");
        fs::create_dir_all(&dir).expect("create");
        for contents in ["not json", "", "[1, 2]"] {
            fs::write(dir.join(SETTINGS_FILE), contents).expect("write");
            assert!(!is_first_run(&dir), "{contents:?}");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_defaults_are_saved_when_the_welcome_shows() {
        let dir = temp_dir("welcome");
        assert!(finish_first_run(
            &dir,
            &Settings::default(),
            &Args::default()
        ));
        assert!(!is_first_run(&dir));
        assert_eq!(settings::load(&dir), Settings::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_first_launch_with_flags_saves_but_shows_no_welcome() {
        for args in [
            Args {
                command: Some(Command::Toggle),
                ..Args::default()
            },
            Args {
                command: Some(Command::Show(None)),
                ..Args::default()
            },
            Args {
                start_at_login: Some(true),
                ..Args::default()
            },
            Args {
                quit: true,
                ..Args::default()
            },
            Args {
                settings: Some(crate::settings_window::SettingsTab::Privacy),
                ..Args::default()
            },
        ] {
            let dir = temp_dir("flags");
            assert!(
                !finish_first_run(&dir, &Settings::default(), &args),
                "{args:?}"
            );
            assert!(!is_first_run(&dir), "{args:?}");
            let _ = fs::remove_dir_all(&dir);
        }
    }
}
