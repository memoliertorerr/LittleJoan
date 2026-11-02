//! Liquid Glass (LJ19.9, the owner's request of 2026-10-30): macOS 26's own setting — **Clear** (more
//! transparent) or **Tinted** (more opaque, more contrast) — for the toolbar and the command bar, on
//! top of the theme, or **Off** (the default). `glass` in settings.json. Real glass only on macOS 26
//! and later; elsewhere the setting is kept but hidden, and nothing changes. With Reduce Transparency
//! or Increase Contrast on, the solid theme shows instead. Pure: no Tauri types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The Liquid Glass setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Glass {
    /// The theme as it is: its gradient and grain (the default).
    #[default]
    Off,
    /// The clear glass style with a light tint of the theme.
    Clear,
    /// The regular glass style with a stronger tint: more opaque, more contrast.
    Tinted,
}

impl Glass {
    pub const ALL: [Glass; 3] = [Glass::Off, Glass::Clear, Glass::Tinted];

    /// Its name in settings.json and in the smoke log.
    pub fn name(self) -> &'static str {
        match self {
            Glass::Off => "off",
            Glass::Clear => "clear",
            Glass::Tinted => "tinted",
        }
    }

    /// A setting by name (spaces around allowed, any case); anything else is none.
    pub fn from_name(name: &str) -> Option<Glass> {
        let name = name.trim().to_ascii_lowercase();
        Glass::ALL.into_iter().find(|glass| glass.name() == name)
    }

    pub fn is_off(&self) -> bool {
        *self == Glass::Off
    }

    /// How the native glass shows it: the style (clear or regular) and how strongly the theme's
    /// colour tints it (the tint colour's alpha). `None` for Off.
    pub fn native(self) -> Option<NativeGlass> {
        match self {
            Glass::Off => None,
            Glass::Clear => Some(NativeGlass {
                clear: true,
                tint_alpha: CLEAR_TINT,
            }),
            Glass::Tinted => Some(NativeGlass {
                clear: false,
                tint_alpha: TINTED_TINT,
            }),
        }
    }
}

/// Clear: a light tint, so what is behind shows through (macOS's Clear).
pub const CLEAR_TINT: f64 = 0.25;
/// Tinted: a strong tint over the regular style: more opaque, more contrast (macOS's Tinted).
pub const TINTED_TINT: f64 = 0.65;

/// The native glass for a setting that shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeGlass {
    /// `NSGlassEffectViewStyleClear` (else `…Regular`).
    pub clear: bool,
    /// The alpha of the theme's colour as the glass's tint.
    pub tint_alpha: f64,
}

/// `glass` in settings.json: one of the three names; anything else (missing, another type, another
/// word) is the default, Off.
pub fn parse(value: Option<&Value>) -> Glass {
    value
        .and_then(Value::as_str)
        .and_then(Glass::from_name)
        .unwrap_or_default()
}

/// Whether this Mac can show Liquid Glass now.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Support {
    /// macOS 26 or later with AppKit's NSGlassEffectView (never on Windows or Linux).
    pub available: bool,
    /// The user's accessibility settings (System Settings → Accessibility → Display).
    pub reduce_transparency: bool,
    pub increase_contrast: bool,
}

impl Support {
    /// What keeps glass from showing although the system has it: `reduceTransparency` or
    /// `increaseContrast` (Settings says so), else none.
    pub fn blocked_by(self) -> Option<&'static str> {
        if !self.available {
            None
        } else if self.reduce_transparency {
            Some("reduceTransparency")
        } else if self.increase_contrast {
            Some("increaseContrast")
        } else {
            None
        }
    }
}

/// The glass a window shows for `setting`: the setting where the system has glass and the user's
/// accessibility settings allow it, else Off (the solid theme) — as macOS hides its own Liquid Glass
/// setting under Reduce Transparency and Increase Contrast.
pub fn effective(setting: Glass, support: Support) -> Glass {
    if support.available && support.blocked_by().is_none() {
        setting
    } else {
        Glass::Off
    }
}

/// Whether a macOS major version has Liquid Glass: 26 and later — and 16, which is what macOS 26
/// reports to an app linked against an older SDK. (The class's presence is checked too.)
pub fn version_has_glass(major: i64) -> bool {
    major >= 26 || major == 16
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_default_is_off_and_names_round_trip() {
        assert_eq!(Glass::default(), Glass::Off);
        assert_eq!(parse(None), Glass::Off);
        for junk in [
            json!(null),
            json!(1),
            json!(true),
            json!("frosted"),
            json!(["clear"]),
        ] {
            assert_eq!(parse(Some(&junk)), Glass::Off, "{junk}");
        }
        for glass in Glass::ALL {
            assert_eq!(parse(Some(&json!(glass.name()))), glass);
            assert_eq!(Glass::from_name(glass.name()), Some(glass));
            assert_eq!(serde_json::to_value(glass).ok(), Some(json!(glass.name())));
        }
        assert_eq!(Glass::from_name(" Tinted "), Some(Glass::Tinted));
        assert_eq!(Glass::from_name("CLEAR"), Some(Glass::Clear));
        assert_eq!(Glass::from_name(""), None);
    }

    #[test]
    fn clear_is_the_clear_style_with_a_lighter_tint_than_tinted() {
        assert_eq!(Glass::Off.native(), None);
        let clear = Glass::Clear.native().expect("clear shows");
        let tinted = Glass::Tinted.native().expect("tinted shows");
        assert!(clear.clear && !tinted.clear);
        assert!(clear.tint_alpha < tinted.tint_alpha);
        for alpha in [clear.tint_alpha, tinted.tint_alpha] {
            assert!(alpha > 0.0 && alpha < 1.0, "a tint, never opaque: {alpha}");
        }
    }

    #[test]
    fn glass_shows_only_where_the_system_has_it_and_allows_it() {
        let mac26 = Support {
            available: true,
            ..Support::default()
        };
        for glass in Glass::ALL {
            assert_eq!(effective(glass, mac26), glass);
            // Windows, Linux, older Macs: always the solid theme.
            assert_eq!(effective(glass, Support::default()), Glass::Off);
            for blocked in [
                Support {
                    reduce_transparency: true,
                    ..mac26
                },
                Support {
                    increase_contrast: true,
                    ..mac26
                },
            ] {
                assert_eq!(effective(glass, blocked), Glass::Off);
            }
        }
        assert_eq!(mac26.blocked_by(), None);
        let both = Support {
            available: true,
            reduce_transparency: true,
            increase_contrast: true,
        };
        assert_eq!(both.blocked_by(), Some("reduceTransparency"));
        let contrast = Support {
            available: true,
            increase_contrast: true,
            ..Support::default()
        };
        assert_eq!(contrast.blocked_by(), Some("increaseContrast"));
        // Without glass at all, nothing "blocks" it: Settings hides the option instead.
        let old = Support {
            reduce_transparency: true,
            ..Support::default()
        };
        assert_eq!(old.blocked_by(), None);
    }

    #[test]
    fn macos_26_and_later_have_glass() {
        for old in [0, 10, 11, 12, 13, 14, 15, 17, 25] {
            assert!(!version_has_glass(old), "{old}");
        }
        for new in [16, 26, 27, 30] {
            assert!(version_has_glass(new), "{new}");
        }
    }
}
