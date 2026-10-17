//! User settings, stored as `settings.json` in the app's config folder.
//!
//! Loading is tolerant per field: a missing, wrong-typed, or invalid field falls back to its
//! default without discarding the others, and an unreadable file means "all defaults". Saving
//! is atomic (write a temp file, then rename over the old one).

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::input::SearchEngine;
use crate::translate::is_valid_lang;

pub const SETTINGS_FILE: &str = "settings.json";
pub const SETTINGS_VERSION: u32 = 1;
/// Cmd+Shift+Space on macOS, Ctrl+Shift+Space on Windows and Linux.
pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+Space";
pub const MIN_WIDTH: u32 = 320;
pub const MIN_HEIGHT: u32 = 300;
pub const MAX_SIZE: u32 = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    /// Global shortcut in Tauri accelerator syntax. Validated where it is registered.
    pub shortcut: String,
    pub search_engine: SearchEngine,
    /// Target language for "Translate Page".
    pub translate_to: String,
    /// Last size of the page window, in logical pixels.
    pub window_width: u32,
    pub window_height: u32,
    /// Phone mode: in a narrow window, pages get a phone user agent (phone.rs).
    pub mobile_sites: bool,
    /// Record visited pages in history.json (history_store.rs). Off: nothing new is recorded;
    /// what is there stays until it is cleared.
    pub remember_history: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            version: SETTINGS_VERSION,
            shortcut: DEFAULT_SHORTCUT.to_owned(),
            search_engine: SearchEngine::Google,
            translate_to: "en".to_owned(),
            window_width: 480,
            window_height: 640,
            mobile_sites: true,
            remember_history: true,
        }
    }
}

impl Settings {
    /// Returns a copy with the window size clamped into the allowed range.
    pub fn with_window_size(mut self, width: f64, height: f64) -> Self {
        self.window_width = clamp_size(width, MIN_WIDTH);
        self.window_height = clamp_size(height, MIN_HEIGHT);
        self
    }
}

fn clamp_size(value: f64, min: u32) -> u32 {
    if value.is_finite() {
        // Clamped into 0..=MAX_SIZE first, so the cast cannot truncate.
        (value.round().clamp(0.0, f64::from(MAX_SIZE)) as u32).max(min)
    } else {
        min
    }
}

/// Parses settings JSON, falling back to the default for each field that is missing or invalid.
pub fn parse(json: &str) -> Settings {
    let defaults = Settings::default();
    let Ok(Value::Object(map)) = serde_json::from_str::<Value>(json) else {
        return defaults;
    };
    let text = |key: &str| map.get(key).and_then(Value::as_str).map(str::trim);
    let size = |key: &str| map.get(key).and_then(Value::as_f64);

    let shortcut = text("shortcut")
        .filter(|s| !s.is_empty())
        .map_or(defaults.shortcut, str::to_owned);
    let search_engine = map
        .get("searchEngine")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(defaults.search_engine);
    let translate_to = text("translateTo")
        .filter(|lang| is_valid_lang(lang))
        .map_or(defaults.translate_to, str::to_owned);
    let window_width =
        size("windowWidth").map_or(defaults.window_width, |w| clamp_size(w, MIN_WIDTH));
    let window_height =
        size("windowHeight").map_or(defaults.window_height, |h| clamp_size(h, MIN_HEIGHT));
    let mobile_sites = map
        .get("mobileSites")
        .and_then(Value::as_bool)
        .unwrap_or(defaults.mobile_sites);
    let remember_history = map
        .get("rememberHistory")
        .and_then(Value::as_bool)
        .unwrap_or(defaults.remember_history);

    Settings {
        version: SETTINGS_VERSION,
        shortcut,
        search_engine,
        translate_to,
        window_width,
        window_height,
        mobile_sites,
        remember_history,
    }
}

/// Loads `dir/settings.json`; a missing or unreadable file yields the defaults.
pub fn load(dir: &Path) -> Settings {
    fs::read_to_string(dir.join(SETTINGS_FILE))
        .map_or_else(|_| Settings::default(), |json| parse(&json))
}

/// Writes `dir/settings.json` atomically, creating `dir` if needed.
pub fn save(dir: &Path, settings: &Settings) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    let tmp = dir.join(format!("{SETTINGS_FILE}.tmp"));
    fs::write(&tmp, json)?;
    fs::rename(&tmp, dir.join(SETTINGS_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("little-joan-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn garbage_yields_defaults() {
        assert_eq!(parse(""), Settings::default());
        assert_eq!(parse("not json"), Settings::default());
        assert_eq!(parse("[1, 2]"), Settings::default());
    }

    #[test]
    fn each_invalid_field_falls_back_alone() {
        let s = parse(
            r#"{"shortcut": 42, "searchEngine": "duckduckgo", "translateTo": "klingon!",
                "windowWidth": "wide", "windowHeight": 700}"#,
        );
        assert_eq!(s.shortcut, DEFAULT_SHORTCUT);
        assert_eq!(s.search_engine, SearchEngine::DuckDuckGo);
        assert_eq!(s.translate_to, "en");
        assert_eq!(s.window_width, 480);
        assert_eq!(s.window_height, 700);
    }

    #[test]
    fn partial_settings_keep_what_they_have() {
        let s = parse(r#"{"shortcut": "Alt+Shift+KeyJ", "searchEngine": "yahoo"}"#);
        assert_eq!(s.shortcut, "Alt+Shift+KeyJ");
        assert_eq!(s.search_engine, SearchEngine::Google);
        assert_eq!(s.translate_to, "en");
    }

    #[test]
    fn window_sizes_are_clamped() {
        let s = parse(r#"{"windowWidth": 10, "windowHeight": 99999}"#);
        assert_eq!((s.window_width, s.window_height), (MIN_WIDTH, MAX_SIZE));
        let s = Settings::default().with_window_size(f64::NAN, -5.0);
        assert_eq!((s.window_width, s.window_height), (MIN_WIDTH, MIN_HEIGHT));
        let s = Settings::default().with_window_size(612.4, 777.6);
        assert_eq!((s.window_width, s.window_height), (612, 778));
    }

    #[test]
    fn empty_shortcut_falls_back() {
        assert_eq!(parse(r#"{"shortcut": "   "}"#).shortcut, DEFAULT_SHORTCUT);
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = temp_dir("roundtrip");
        let settings = Settings {
            shortcut: "CommandOrControl+Alt+KeyK".into(),
            search_engine: SearchEngine::Brave,
            translate_to: "zh-CN".into(),
            window_width: 900,
            window_height: 500,
            mobile_sites: false,
            remember_history: false,
            ..Settings::default()
        };
        save(&dir, &settings).expect("save");
        assert_eq!(load(&dir), settings);
        assert!(!dir.join("settings.json.tmp").exists());
        let json = fs::read_to_string(dir.join(SETTINGS_FILE)).expect("written");
        assert!(json.contains("\"searchEngine\": \"brave\""), "{json}");
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn phone_mode_is_on_unless_switched_off() {
        assert!(Settings::default().mobile_sites);
        assert!(
            parse("{}").mobile_sites,
            "settings from before the option existed"
        );
        assert!(!parse(r#"{"mobileSites": false}"#).mobile_sites);
        assert!(
            parse(r#"{"mobileSites": "no"}"#).mobile_sites,
            "a wrong type falls back"
        );
    }

    #[test]
    fn history_is_remembered_unless_switched_off() {
        assert!(Settings::default().remember_history);
        assert!(
            parse("{}").remember_history,
            "settings from before the option existed"
        );
        assert!(!parse(r#"{"rememberHistory": false}"#).remember_history);
        assert!(
            parse(r#"{"rememberHistory": 0}"#).remember_history,
            "a wrong type falls back"
        );
        let json = serde_json::to_string(&Settings::default()).expect("json");
        assert!(json.contains(r#""rememberHistory":true"#), "{json}");
    }

    #[test]
    fn missing_file_yields_defaults() {
        assert_eq!(load(&temp_dir("missing")), Settings::default());
    }
}
