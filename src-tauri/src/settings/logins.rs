//! Saved logins on Windows (LJ51): `savePasswords` — WebView2's own password saving, off by
//! default — and `autofillForms` — its general autofill (addresses and the like), on by default.
//! Both are the engine's own defaults, so nothing changes for anyone until they choose. Tolerant: a
//! missing or wrong-typed value is the default. These are switches only (Ground Rule 26): what the
//! engine saves stays in the engine (logins.rs, platform/win/logins.rs). Pure: no Tauri types.

use serde_json::Value;

/// WebView2's own default: no new passwords saved, no offer to save them.
pub const SAVE_PASSWORDS_DEFAULT: bool = false;
/// WebView2's own default: forms are filled in (what Windows users have had since v0.1).
pub const AUTOFILL_FORMS_DEFAULT: bool = true;

/// A switch from settings.json: a boolean, else `default`.
pub fn flag(value: Option<&Value>, default: bool) -> bool {
    value.and_then(Value::as_bool).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{self, Change, Settings, apply_change};

    #[test]
    fn password_saving_is_off_and_autofill_on_by_default() {
        let defaults = Settings::default();
        assert!(!defaults.save_passwords && !SAVE_PASSWORDS_DEFAULT);
        assert!(defaults.autofill_forms && AUTOFILL_FORMS_DEFAULT);
        assert!(flag(Some(&Value::Bool(true)), false));
        assert!(flag(Some(&Value::from("false")), true));
        assert!(!flag(None, false));
        let old = settings::parse(r#"{"shortcut": "Alt+KeyJ"}"#);
        assert!(!old.save_passwords, "settings from before LJ51");
        assert!(old.autofill_forms, "settings from before LJ51");
        let json = serde_json::to_string(&defaults).expect("json");
        assert!(json.contains(r#""savePasswords":false"#), "{json}");
        assert!(json.contains(r#""autofillForms":true"#), "{json}");
    }

    #[test]
    fn the_switches_are_read_on_their_own() {
        let s = settings::parse(r#"{"savePasswords": true, "autofillForms": false}"#);
        assert!(s.save_passwords);
        assert!(!s.autofill_forms);
        // A wrong type falls back to the engine's default, and leaves the rest alone.
        for json in [
            r#"{"savePasswords": "yes", "autofillForms": 0, "mobileSites": false}"#,
            r#"{"savePasswords": 1, "autofillForms": null, "mobileSites": false}"#,
        ] {
            let s = settings::parse(json);
            assert!(!s.save_passwords, "{json}");
            assert!(s.autofill_forms, "{json}");
            assert!(!s.mobile_sites, "{json}");
        }
    }

    #[test]
    fn save_then_load_keeps_both() {
        let dir =
            std::env::temp_dir().join(format!("little-joan-test-logins-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let saved = Settings {
            save_passwords: true,
            autofill_forms: false,
            ..Settings::default()
        };
        settings::save(&dir, &saved).expect("save");
        assert_eq!(settings::load(&dir), saved);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn only_a_change_that_names_them_switches_them() {
        let current = Settings::default();
        let next = apply_change(
            &current,
            &Change {
                save_passwords: Some(true),
                autofill_forms: Some(false),
                ..Change::default()
            },
        )
        .expect("valid");
        assert!(next.save_passwords && !next.autofill_forms);
        assert_eq!(
            Settings {
                save_passwords: current.save_passwords,
                autofill_forms: current.autofill_forms,
                ..next.clone()
            },
            current,
            "nothing else moved"
        );
        // Settings' other fields (and every quick command) leave them as they are.
        let other = apply_change(
            &next,
            &Change {
                mobile_sites: Some(false),
                remember_history: Some(false),
                ..Change::default()
            },
        )
        .expect("valid");
        assert!(other.save_passwords && !other.autofill_forms);
        assert!(!Change::default().touches_logins());
        assert!(
            Change {
                autofill_forms: Some(true),
                ..Change::default()
            }
            .touches_logins()
        );
    }
}
