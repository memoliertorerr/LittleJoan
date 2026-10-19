//! macOS's own shortcuts, read-only (LJ44): the `com.apple.symbolichotkeys` preferences domain
//! (`AppleSymbolicHotKeys`: id → `{ enabled, value: { type, parameters: [character, virtual key
//! code, modifier flags] } }`), which platform/mac.rs reads through NSUserDefaults and hands over as
//! JSON. Settings → Keyboard keeps its curated table of reserved shortcuts as the base and lets this
//! refine it: an entry marked disabled drops that curated warning, an enabled one with a key adds a
//! warning (ui/settings/keyboard-logic.ts). Never written.
//!
//! Decided by a CI runner's export (run 39216324726, `macos-runner.json`): the domain lists only
//! entries that were written — there, 19: twelve with `enabled` alone (no key: a default nobody
//! changed), Mission Control's spaces enabled without a key, the two input-source entries disabled
//! with their keys (`[32, 49, 262144]` = ⌃Space, `[32, 49, 786432]` = ⌃⌥Space — exactly the
//! curated table's), and one `modifier` entry (a lone modifier, no key). Entries with a key decode
//! reliably (US positions, Apple's modifier flags); entries without one say nothing about their key.
//! Pure: no Tauri types.

use serde::Serialize;
use serde_json::Value;

/// At most this many entries are read (the domain holds a few dozen).
const MAX_ENTRIES: usize = 512;

/// One of macOS's own shortcuts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SystemShortcut {
    /// Its number in the domain (64 = Spotlight's search).
    pub id: u32,
    pub enabled: bool,
    /// Its key in Little Joan's accelerator syntax, when the entry gives one (a `standard` entry
    /// whose key Little Joan can name); `None` otherwise.
    pub accelerator: Option<String>,
}

/// Apple's modifier flags (NSEventModifierFlags) and the accelerator names they become, in the
/// keymap's order.
const FLAGS: [(u64, &str); 4] = [
    (1 << 20, "CommandOrControl"),
    (1 << 18, "Control"),
    (1 << 19, "Alt"),
    (1 << 17, "Shift"),
];

/// A macOS virtual key code (kVK_*, positions on a US keyboard) as a `KeyboardEvent.code`.
fn code_for(keycode: u64) -> Option<&'static str> {
    const LETTERS: [(u64, &str); 26] = [
        (0x00, "KeyA"),
        (0x0B, "KeyB"),
        (0x08, "KeyC"),
        (0x02, "KeyD"),
        (0x0E, "KeyE"),
        (0x03, "KeyF"),
        (0x05, "KeyG"),
        (0x04, "KeyH"),
        (0x22, "KeyI"),
        (0x26, "KeyJ"),
        (0x28, "KeyK"),
        (0x25, "KeyL"),
        (0x2E, "KeyM"),
        (0x2D, "KeyN"),
        (0x1F, "KeyO"),
        (0x23, "KeyP"),
        (0x0C, "KeyQ"),
        (0x0F, "KeyR"),
        (0x01, "KeyS"),
        (0x11, "KeyT"),
        (0x20, "KeyU"),
        (0x09, "KeyV"),
        (0x0D, "KeyW"),
        (0x07, "KeyX"),
        (0x10, "KeyY"),
        (0x06, "KeyZ"),
    ];
    const OTHERS: [(u64, &str); 44] = [
        (0x1D, "Digit0"),
        (0x12, "Digit1"),
        (0x13, "Digit2"),
        (0x14, "Digit3"),
        (0x15, "Digit4"),
        (0x17, "Digit5"),
        (0x16, "Digit6"),
        (0x1A, "Digit7"),
        (0x1C, "Digit8"),
        (0x19, "Digit9"),
        (0x18, "Equal"),
        (0x1B, "Minus"),
        (0x21, "BracketLeft"),
        (0x1E, "BracketRight"),
        (0x2B, "Comma"),
        (0x2F, "Period"),
        (0x2C, "Slash"),
        (0x29, "Semicolon"),
        (0x27, "Quote"),
        (0x32, "Backquote"),
        (0x2A, "Backslash"),
        (0x31, "Space"),
        (0x30, "Tab"),
        (0x35, "Escape"),
        (0x24, "Enter"),
        (0x33, "Backspace"),
        (0x7B, "ArrowLeft"),
        (0x7C, "ArrowRight"),
        (0x7D, "ArrowDown"),
        (0x7E, "ArrowUp"),
        (0x73, "Home"),
        (0x77, "End"),
        (0x74, "PageUp"),
        (0x79, "PageDown"),
        (0x7A, "F1"),
        (0x78, "F2"),
        (0x63, "F3"),
        (0x76, "F4"),
        (0x60, "F5"),
        (0x61, "F6"),
        (0x62, "F7"),
        (0x64, "F8"),
        (0x65, "F9"),
        (0x6D, "F10"),
    ];
    const MORE: [(u64, &str); 2] = [(0x67, "F11"), (0x6F, "F12")];
    LETTERS
        .iter()
        .chain(OTHERS.iter())
        .chain(MORE.iter())
        .find(|(code, _)| *code == keycode)
        .map(|(_, name)| *name)
}

/// A `standard` entry's key: `[character, virtual key code, modifier flags]`.
fn accelerator(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("standard") {
        return None;
    }
    let parameters = value.get("parameters")?.as_array()?;
    let keycode = parameters.get(1)?.as_u64()?;
    let flags = parameters.get(2)?.as_u64()?;
    let code = code_for(keycode)?;
    let mut parts: Vec<&str> = FLAGS
        .iter()
        .filter(|(bit, _)| flags & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    parts.push(code);
    Some(parts.join("+"))
}

/// Whether an entry is on: `true` / `false`, or a number (NSNumber read as one).
fn enabled(entry: &Value) -> Option<bool> {
    let flag = entry.get("enabled")?;
    flag.as_bool().or_else(|| flag.as_i64().map(|n| n != 0))
}

/// The domain's entries (the export's `AppleSymbolicHotKeys`), by id; entries that don't say
/// whether they are on are skipped.
pub fn parse(domain: &Value) -> Vec<SystemShortcut> {
    let Some(entries) = domain
        .get("AppleSymbolicHotKeys")
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let mut shortcuts: Vec<SystemShortcut> = entries
        .iter()
        .take(MAX_ENTRIES)
        .filter_map(|(id, entry)| {
            Some(SystemShortcut {
                id: id.parse().ok()?,
                enabled: enabled(entry)?,
                accelerator: entry.get("value").and_then(accelerator),
            })
        })
        .collect();
    shortcuts.sort_by_key(|shortcut| shortcut.id);
    shortcuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A CI runner's export (macOS 15, run 39216324726): nothing personal.
    const RUNNER: &str = include_str!("system_shortcuts/macos-runner.json");

    #[test]
    fn the_runners_export_reads_as_it_should() {
        let domain: Value = serde_json::from_str(RUNNER).expect("the export is JSON");
        let shortcuts = parse(&domain);
        assert_eq!(shortcuts.len(), 19);
        let by_id = |id: u32| shortcuts.iter().find(|s| s.id == id).expect("an entry");
        assert_eq!(
            by_id(60),
            &SystemShortcut {
                id: 60,
                enabled: false,
                accelerator: Some("Control+Space".into())
            },
            "the previous input source, switched off on the runner"
        );
        assert_eq!(by_id(61).accelerator.as_deref(), Some("Control+Alt+Space"));
        assert_eq!(
            by_id(79),
            &SystemShortcut {
                id: 79,
                enabled: true,
                accelerator: None
            }
        );
        assert_eq!(by_id(164).accelerator, None, "a lone modifier is no key");
        assert_eq!(
            shortcuts.iter().filter(|s| s.accelerator.is_some()).count(),
            2
        );
        assert!(shortcuts.windows(2).all(|w| w[0].id < w[1].id));
    }

    #[test]
    fn keys_decode_from_codes_and_flags() {
        let entry = |parameters: Value| json!({ "AppleSymbolicHotKeys": { "64": { "enabled": 1, "value": { "type": "standard", "parameters": parameters } } } });
        let spotlight = parse(&entry(json!([32, 49, 1_048_576])));
        assert_eq!(
            spotlight,
            [SystemShortcut {
                id: 64,
                enabled: true,
                accelerator: Some("CommandOrControl+Space".into())
            }]
        );
        assert_eq!(
            parse(&entry(json!([51, 20, 1_179_648])))[0]
                .accelerator
                .as_deref(),
            Some("CommandOrControl+Shift+Digit3"),
            "⇧⌘3"
        );
        assert_eq!(
            parse(&entry(json!([65535, 123, 8_650_752])))[0]
                .accelerator
                .as_deref(),
            Some("Control+ArrowLeft"),
            "⌃← (the function-key flag ignored)"
        );
        assert_eq!(
            parse(&entry(json!([65535, 999, 262_144])))[0].accelerator,
            None,
            "an unknown key code"
        );
    }

    #[test]
    fn odd_domains_read_as_nothing() {
        for bad in [
            json!(null),
            json!({}),
            json!({ "AppleSymbolicHotKeys": [] }),
            json!({ "AppleSymbolicHotKeys": { "x": { "enabled": true }, "7": { "value": {} } } }),
        ] {
            assert!(parse(&bad).is_empty(), "{bad}");
        }
    }
}
