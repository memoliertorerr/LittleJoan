//! Saved logins (LJ51; Ground Rule 26: saved credentials stay in the engine). Windows' WebView2 has
//! its own password saving and form autofill; Little Joan only switches them — settings
//! `savePasswords` (off by default) and `autofillForms` (on by default), the engine's own defaults —
//! and asks the engine to remove what it saved (commands/logins.rs). It never reads, exports, logs
//! or syncs a password or a form entry. macOS and Linux: nothing here (Settings → Privacy says why).
//!
//! Every webview on the page's profile shares the engine's two switches (WebView2 keeps them per
//! profile), and wry sets general autofill on each webview it creates (its default: on), so every
//! builder on that profile — the bar, the page, a sign-in popup, the settings window — passes the
//! saved `autofillForms` (`autofill_forms`); the page and each popup then get both switches right
//! after they are built, read back into the smoke log; a Save that changes them sets them at once on
//! every webview of the profile there is (`apply_everywhere`). The profile is one folder: the
//! settings window uses the page's webview data folder (settings_window.rs), and CI compares the two
//! profiles' paths (`webview-profile`).

use tauri::{AppHandle, Manager, Runtime, Webview};

use crate::bar;
use crate::platform;
use crate::popups;
use crate::settings_window;
use crate::state;

/// Whether this OS's engine has switches Little Joan can set (Windows' WebView2 only).
pub const AVAILABLE: bool = cfg!(windows);

/// A webview on the page's profile, for the smoke log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Page,
    Popup,
    Settings,
}

impl Place {
    pub fn name(self) -> &'static str {
        match self {
            Place::Page => "page",
            Place::Popup => "popup",
            Place::Settings => "settings",
        }
    }
}

/// The smoke log's detail for the switches as the engine read them back:
/// `password-autosave=<bool> autofill=<bool> webview=<place>` (`logins …`, or `logins-seen …` when
/// only read).
pub fn line(save: bool, autofill: bool, place: Place) -> String {
    format!(
        "password-autosave={save} autofill={autofill} webview={}",
        place.name()
    )
}

/// The saved `autofillForms`, for a builder on the page's profile.
pub fn autofill_forms<R: Runtime>(app: &AppHandle<R>) -> bool {
    state::with(app, |s| s.settings.autofill_forms)
}

/// A page webview or a sign-in popup was built: the saved switches go to its engine (Windows),
/// read back into the smoke log.
pub fn apply<R: Runtime>(app: &AppHandle<R>, webview: &Webview<R>, place: Place) {
    let (save, autofill) = state::with(app, |s| {
        (s.settings.save_passwords, s.settings.autofill_forms)
    });
    platform::apply_logins(webview, save, autofill, place);
}

/// A Save changed the switches: the page, its popups and the settings window — every webview of the
/// profile there is (none while hibernating but the settings window) — get them now.
pub fn apply_everywhere<R: Runtime>(app: &AppHandle<R>) {
    for (label, webview) in app.webviews() {
        let place = if label == bar::PAGE {
            Place::Page
        } else if popups::is_popup_label(&label) {
            Place::Popup
        } else if label == settings_window::LABEL {
            Place::Settings
        } else {
            continue;
        };
        apply(app, &webview, place);
    }
}

/// Smoke log (Windows): the webview's WebView2 profile folder, `webview-profile <place>=<path>`, and
/// what its engine's switches say now without setting them (`logins-seen …`).
pub fn report<R: Runtime>(webview: &Webview<R>, place: Place) {
    platform::report_logins(webview, place);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_line_names_the_switches_and_the_webview() {
        assert_eq!(
            line(false, true, Place::Page),
            "password-autosave=false autofill=true webview=page"
        );
        assert_eq!(
            line(true, false, Place::Popup),
            "password-autosave=true autofill=false webview=popup"
        );
        assert!(line(true, true, Place::Settings).ends_with(" webview=settings"));
    }

    #[test]
    fn only_windows_has_the_switches() {
        assert_eq!(AVAILABLE, cfg!(windows));
    }

    /// The source of every Rust file under src/, outside its tests.
    fn sources() -> Vec<(String, String)> {
        let mut found = Vec::new();
        let mut stack = vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read src") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    let code = text
                        .split("#[cfg(test)]")
                        .next()
                        .unwrap_or(&text)
                        .to_owned();
                    found.push((path.display().to_string(), code));
                }
            }
        }
        found
    }

    #[test]
    fn every_webview_on_the_profile_shares_the_engines_environment() {
        // WebView2 refuses a second environment on a user data folder in use when its options
        // differ (browser arguments, language, extensions, scroll bars…). The bar, the page, the
        // popups and the settings window share one folder (LJ51), so none sets any of them: all keep
        // wry's defaults — and none gets a private (incognito) profile of its own.
        for (path, code) in sources() {
            for option in [
                ".additional_browser_args(",
                ".browser_extensions_enabled(",
                ".scroll_bar_style(",
                ".proxy_url(",
                ".incognito(",
                ".data_store_identifier(",
            ] {
                assert!(!code.contains(option), "{path}: {option}");
            }
        }
    }

    #[test]
    fn every_builder_on_the_profile_passes_the_saved_autofill() {
        // wry sets general autofill on every webview it creates (default on), and the engine shares
        // it across the profile: a builder without the saved value would switch it back on.
        let builders = [
            ("bar.rs", "WebviewBuilder::new(BAR"),
            ("bar/page.rs", "WebviewBuilder::new(PAGE"),
            ("bar/popup.rs", "WebviewWindowBuilder::new(app, &label"),
            (
                "settings_window.rs",
                "WebviewWindowBuilder::new(&app, LABEL",
            ),
        ];
        let all = sources();
        for (file, builder) in builders {
            let (path, code) = all
                .iter()
                .find(|(path, _)| path.replace('\\', "/").ends_with(&format!("src/{file}")))
                .expect("the builder's file");
            let at = code
                .find(builder)
                .unwrap_or_else(|| panic!("{path}: {builder}"));
            let rest = &code[at..];
            let end = rest.find("\n}\n").unwrap_or(rest.len());
            assert!(
                rest[..end].contains(".general_autofill_enabled("),
                "{path}: {builder} passes the saved autofillForms"
            );
        }
    }
}
