//! The `theme` field of settings.json: light / dark / the system's, 1–3 colors, an intensity
//! and a grain. Rust only checks and keeps it; ui/lib/theme.ts turns it into colors. Parsing is
//! tolerant per field, like the rest of the settings: a bad color is dropped, numbers are
//! clamped into 0–1, anything unreadable falls back to the default (the neutral "Mist" preset,
//! the same values as ui/lib/presets.ts — a TypeScript test compares them).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// At most this many colors are kept.
pub const MAX_COLORS: usize = 3;
/// The default theme's color, intensity and grain (ui/lib/presets.ts, "Mist").
pub const DEFAULT_COLOR: &str = "#8c9bb5";
pub const DEFAULT_INTENSITY: f64 = 0.45;
pub const DEFAULT_GRAIN: f64 = 0.15;

/// Light, dark, or whatever the system uses (followed as it changes).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub fn name(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    /// A mode by name, in any case, spaces around allowed; anything else is none.
    pub fn from_name(name: &str) -> Option<ThemeMode> {
        match name.trim().to_ascii_lowercase().as_str() {
            "system" => Some(ThemeMode::System),
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub mode: ThemeMode,
    /// 1–3 colors, `#rrggbb` in lower case.
    pub colors: Vec<String>,
    /// 0–1: how much color the surfaces get.
    pub intensity: f64,
    /// 0–1: how strong the grain is.
    pub grain: f64,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            mode: ThemeMode::default(),
            colors: vec![DEFAULT_COLOR.to_owned()],
            intensity: DEFAULT_INTENSITY,
            grain: DEFAULT_GRAIN,
        }
    }
}

/// `#rgb` or `#rrggbb` (any case, spaces around allowed) as `#rrggbb` in lower case.
pub fn color(text: &str) -> Option<String> {
    let digits = text.trim().strip_prefix('#')?;
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let digits = digits.to_ascii_lowercase();
    match digits.len() {
        6 => Some(format!("#{digits}")),
        3 => Some(digits.chars().fold(String::from("#"), |mut hex, c| {
            hex.push(c);
            hex.push(c);
            hex
        })),
        _ => None,
    }
}

/// A number in 0–1: clamped when it is a finite number, else `default`.
fn unit(value: Option<&Value>, default: f64) -> f64 {
    value
        .and_then(Value::as_f64)
        .filter(|x| x.is_finite())
        .map_or(default, |x| x.clamp(0.0, 1.0))
}

/// The theme in `value` (settings.json's `theme`, or what Settings saves); each field falls back
/// to the default on its own.
pub fn parse(value: Option<&Value>) -> Theme {
    let defaults = Theme::default();
    let Some(Value::Object(map)) = value else {
        return defaults;
    };
    let mode = map
        .get("mode")
        .and_then(Value::as_str)
        .and_then(ThemeMode::from_name)
        .unwrap_or(defaults.mode);
    let colors: Vec<String> = map
        .get("colors")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .filter_map(Value::as_str)
                .filter_map(color)
                .take(MAX_COLORS)
                .collect()
        })
        .unwrap_or_default();
    Theme {
        mode,
        colors: if colors.is_empty() {
            defaults.colors
        } else {
            colors
        },
        intensity: unit(map.get("intensity"), defaults.intensity),
        grain: unit(map.get("grain"), defaults.grain),
    }
}

/// A theme that is valid as it stands — what Settings saves, a preset the command bar sends
/// (LJ42): an object with a known mode, one to three colors that all parse, and an intensity and a
/// grain in 0–1. Anything else is refused (`None`), so a bad theme changes nothing.
pub fn parse_strict(value: &Value) -> Option<Theme> {
    let Value::Object(map) = value else {
        return None;
    };
    let mode = ThemeMode::from_name(map.get("mode")?.as_str()?)?;
    let list = map.get("colors")?.as_array()?;
    if list.is_empty() || list.len() > MAX_COLORS {
        return None;
    }
    let colors = list
        .iter()
        .map(|c| c.as_str().and_then(color))
        .collect::<Option<Vec<String>>>()?;
    let number = |key: &str| {
        map.get(key)?
            .as_f64()
            .filter(|x| x.is_finite() && (0.0..=1.0).contains(x))
    };
    Some(Theme {
        mode,
        colors,
        intensity: number("intensity")?,
        grain: number("grain")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_strict_theme_is_refused_whole_for_one_bad_part() {
        let good = json!({ "mode": "dark", "colors": ["#9D8FD0", "#e0a2a0"], "intensity": 0.6, "grain": 0.3 });
        assert_eq!(
            parse_strict(&good),
            Some(Theme {
                mode: ThemeMode::Dark,
                colors: vec!["#9d8fd0".into(), "#e0a2a0".into()],
                intensity: 0.6,
                grain: 0.3,
            })
        );
        for bad in [
            json!({ "mode": "sepia", "colors": ["#9d8fd0"], "intensity": 0.6, "grain": 0.3 }),
            json!({ "mode": "dark", "colors": ["#9d8fd0", "red"], "intensity": 0.6, "grain": 0.3 }),
            json!({ "mode": "dark", "colors": [], "intensity": 0.6, "grain": 0.3 }),
            json!({ "mode": "dark", "colors": ["#111111", "#222222", "#333333", "#444444"], "intensity": 0.6, "grain": 0.3 }),
            json!({ "mode": "dark", "colors": ["#9d8fd0"], "intensity": 1.5, "grain": 0.3 }),
            json!({ "mode": "dark", "colors": ["#9d8fd0"], "intensity": 0.6 }),
            json!({ "colors": ["#9d8fd0"], "intensity": 0.6, "grain": 0.3 }),
            json!("dark"),
            json!(null),
        ] {
            assert_eq!(parse_strict(&bad), None, "{bad}");
        }
    }

    fn theme(value: Value) -> Theme {
        parse(Some(&value))
    }

    #[test]
    fn the_default_is_the_neutral_preset() {
        let default = Theme::default();
        assert_eq!(default.mode, ThemeMode::System);
        assert_eq!(default.colors, vec![DEFAULT_COLOR.to_owned()]);
        assert_eq!((default.intensity, default.grain), (0.45, 0.15));
        for missing in [
            None,
            Some(&json!(null)),
            Some(&json!("dark")),
            Some(&json!([1])),
        ] {
            assert_eq!(parse(missing), default);
        }
    }

    #[test]
    fn colors_are_checked_and_one_to_three_kept() {
        let t = theme(
            json!({ "colors": ["#ABCDEF", "red", 7, "#abc", "#12345", " #00ff00 ", "#111111"] }),
        );
        assert_eq!(t.colors, vec!["#abcdef", "#aabbcc", "#00ff00"]);
        let t = theme(json!({ "colors": ["#a1b2c3", "#C0FFEE"] }));
        assert_eq!(t.colors, vec!["#a1b2c3", "#c0ffee"]);
        for bad in [
            json!([]),
            json!(["nope", "#ggg", "##123456"]),
            json!("#ffffff"),
        ] {
            assert_eq!(theme(json!({ "colors": bad })).colors, vec![DEFAULT_COLOR]);
        }
        assert_eq!(color("#FfF"), Some("#ffffff".into()));
        assert_eq!(color("#+12345"), None);
        assert_eq!(color("#é12345"), None);
    }

    #[test]
    fn numbers_are_clamped() {
        let t = theme(json!({ "intensity": 1.7, "grain": -0.2 }));
        assert_eq!((t.intensity, t.grain), (1.0, 0.0));
        let t = theme(json!({ "intensity": 0.25, "grain": 0.4 }));
        assert_eq!((t.intensity, t.grain), (0.25, 0.4));
        let t = theme(json!({ "intensity": "high", "grain": null }));
        assert_eq!((t.intensity, t.grain), (DEFAULT_INTENSITY, DEFAULT_GRAIN));
    }

    #[test]
    fn mode_is_system_light_or_dark() {
        assert_eq!(theme(json!({ "mode": "dark" })).mode, ThemeMode::Dark);
        assert_eq!(theme(json!({ "mode": " Light " })).mode, ThemeMode::Light);
        assert_eq!(theme(json!({ "mode": "system" })).mode, ThemeMode::System);
        for bad in [json!("sepia"), json!(1), json!(true)] {
            assert_eq!(theme(json!({ "mode": bad })).mode, ThemeMode::System);
        }
        assert_eq!(ThemeMode::Dark.name(), "dark");
    }

    #[test]
    fn fields_fall_back_alone_and_round_trip() {
        let t = theme(json!({ "mode": "light", "colors": "teal", "intensity": 0.8 }));
        assert_eq!(t.mode, ThemeMode::Light);
        assert_eq!(t.colors, vec![DEFAULT_COLOR]);
        assert_eq!((t.intensity, t.grain), (0.8, DEFAULT_GRAIN));
        let json = serde_json::to_value(&t).expect("json");
        assert_eq!(
            json,
            json!({ "mode": "light", "colors": [DEFAULT_COLOR], "intensity": 0.8, "grain": DEFAULT_GRAIN })
        );
        assert_eq!(parse(Some(&json)), t);
    }
}
