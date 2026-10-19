//! Keys and key presses, reduced to what Little Joan's keymap needs (keymap.rs), plus the app's
//! origin.
//!
//! Remote pages never get IPC, so in-page keys are caught by native hooks: WebView2's
//! `AcceleratorKeyPressed` on Windows and the GTK window's `key-press-event` on Linux turn a press
//! into a `Key` and `Mods` here and ask the keymap; on macOS the app menu's key equivalents (built
//! from the same keymap, menu.rs) do the same job. Keys are named like `KeyboardEvent.code`
//! (`KeyL`, `Digit1`, `BracketLeft`, `F5`, `ArrowLeft`) and matched by what the key types on a US
//! layout: a letter key is the key that types that letter in the user's layout (Windows' virtual
//! keys, GDK's key names), as the macOS menu's key equivalents are characters.

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAction {
    Close,
    /// Put the keyboard in the URL bar with the address selected.
    OpenLocation,
    Reload,
    Back,
    Forward,
    /// Show the page through Google Translate (translate.rs).
    Translate,
    /// Hand the page to the default browser and hibernate (tray and macOS menu only, no key).
    OpenInBrowser,
    /// Open the command bar's overlay in its history view.
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    Linux,
    MacOs,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(windows) {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else {
            Platform::Linux
        }
    }
}

/// A key Little Joan can bind: what the Settings recorder can record (LJ43).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A letter, lowercase.
    Letter(char),
    /// A digit on the main row, 0–9.
    Digit(u8),
    /// F1–F24.
    F(u8),
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Equal,
    Minus,
    BracketLeft,
    BracketRight,
    Comma,
    Period,
    Slash,
    Semicolon,
    Quote,
    Backquote,
    Backslash,
    /// A digit on the numeric keypad, 0–9.
    Numpad(u8),
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    Other,
}

/// The named keys and their `KeyboardEvent.code` names (letters, digits, F-keys and the keypad's
/// digits are built from their number).
const NAMED: [(Key, &str); 24] = [
    (Key::Left, "ArrowLeft"),
    (Key::Right, "ArrowRight"),
    (Key::Up, "ArrowUp"),
    (Key::Down, "ArrowDown"),
    (Key::Home, "Home"),
    (Key::End, "End"),
    (Key::PageUp, "PageUp"),
    (Key::PageDown, "PageDown"),
    (Key::Equal, "Equal"),
    (Key::Minus, "Minus"),
    (Key::BracketLeft, "BracketLeft"),
    (Key::BracketRight, "BracketRight"),
    (Key::Comma, "Comma"),
    (Key::Period, "Period"),
    (Key::Slash, "Slash"),
    (Key::Semicolon, "Semicolon"),
    (Key::Quote, "Quote"),
    (Key::Backquote, "Backquote"),
    (Key::Backslash, "Backslash"),
    (Key::NumpadAdd, "NumpadAdd"),
    (Key::NumpadSubtract, "NumpadSubtract"),
    (Key::NumpadMultiply, "NumpadMultiply"),
    (Key::NumpadDivide, "NumpadDivide"),
    (Key::NumpadDecimal, "NumpadDecimal"),
];

/// `text` as a number up to `max`: ASCII digits only, no sign, no leading zero (but "0").
fn small_number(text: &str, max: u8) -> Option<u8> {
    let plain = !text.is_empty()
        && text.bytes().all(|b| b.is_ascii_digit())
        && (text == "0" || !text.starts_with('0'));
    text.parse::<u8>().ok().filter(|n| plain && *n <= max)
}

impl Key {
    /// From a `KeyboardEvent.code` name (`KeyL`, `Digit1`, `F5`, `ArrowLeft`, `Numpad0`, …);
    /// anything else is no key.
    pub fn from_code(code: &str) -> Option<Key> {
        let number = |rest: &str, max: u8| small_number(rest, max);
        if let Some(rest) = code.strip_prefix("Key") {
            let mut chars = rest.chars();
            return match (chars.next(), chars.next()) {
                (Some(c), None) if c.is_ascii_uppercase() => {
                    Some(Key::Letter(c.to_ascii_lowercase()))
                }
                _ => None,
            };
        }
        if let Some(rest) = code.strip_prefix("Digit") {
            return (rest.len() == 1)
                .then(|| number(rest, 9))
                .flatten()
                .map(Key::Digit);
        }
        if let Some(rest) = code.strip_prefix("Numpad") {
            if rest.len() == 1 {
                return number(rest, 9).map(Key::Numpad);
            }
        }
        if let Some(rest) = code.strip_prefix('F') {
            return number(rest, 24).filter(|n| *n >= 1).map(Key::F);
        }
        NAMED
            .iter()
            .find(|(_, name)| *name == code)
            .map(|(key, _)| *key)
    }

    /// Its `KeyboardEvent.code` name (`None` for `Other`).
    pub fn code(self) -> Option<String> {
        match self {
            Key::Letter(c) => Some(format!("Key{}", c.to_ascii_uppercase())),
            Key::Digit(n) => Some(format!("Digit{n}")),
            Key::F(n) => Some(format!("F{n}")),
            Key::Numpad(n) => Some(format!("Numpad{n}")),
            Key::Other => None,
            named => NAMED
                .iter()
                .find(|(key, _)| *key == named)
                .map(|(_, name)| (*name).to_owned()),
        }
    }

    /// A key that types a character (a letter, a digit, punctuation, the keypad's): bound only with
    /// a modifier other than Shift (keymap.rs).
    pub fn types_text(self) -> bool {
        !matches!(
            self,
            Key::F(_)
                | Key::Left
                | Key::Right
                | Key::Up
                | Key::Down
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown
                | Key::Other
        )
    }

    /// From a Windows virtual-key code (VK_A..VK_Z are 0x41..0x5A, the US layout's keys for the
    /// punctuation codes).
    pub fn from_windows_vk(vk: u32) -> Key {
        match vk {
            0x41..=0x5A => {
                char::from_u32(vk).map_or(Key::Other, |c| Key::Letter(c.to_ascii_lowercase()))
            }
            0x30..=0x39 => Key::Digit((vk - 0x30) as u8),
            0x60..=0x69 => Key::Numpad((vk - 0x60) as u8),
            0x70..=0x87 => Key::F((vk - 0x6F) as u8),
            0x25 => Key::Left,
            0x26 => Key::Up,
            0x27 => Key::Right,
            0x28 => Key::Down,
            0x24 => Key::Home,
            0x23 => Key::End,
            0x21 => Key::PageUp,
            0x22 => Key::PageDown,
            0x6A => Key::NumpadMultiply,
            0x6B => Key::NumpadAdd,
            0x6D => Key::NumpadSubtract,
            0x6E => Key::NumpadDecimal,
            0x6F => Key::NumpadDivide,
            0xBA => Key::Semicolon,
            0xBB => Key::Equal,
            0xBC => Key::Comma,
            0xBD => Key::Minus,
            0xBE => Key::Period,
            0xBF => Key::Slash,
            0xC0 => Key::Backquote,
            0xDB => Key::BracketLeft,
            0xDC => Key::Backslash,
            0xDD => Key::BracketRight,
            0xDE => Key::Quote,
            _ => Key::Other,
        }
    }

    /// From a GDK key name ("w", "W", "F5", "Left", "bracketleft", "KP_1"; with Shift held, GDK
    /// names the shifted character — "braceleft", "exclam" — which counts as its key on a US
    /// layout).
    pub fn from_gdk_name(name: &str) -> Key {
        let named = match name {
            "Left" => Some(Key::Left),
            "Right" => Some(Key::Right),
            "Up" => Some(Key::Up),
            "Down" => Some(Key::Down),
            "Home" => Some(Key::Home),
            "End" => Some(Key::End),
            "Page_Up" | "Prior" => Some(Key::PageUp),
            "Page_Down" | "Next" => Some(Key::PageDown),
            "equal" | "plus" => Some(Key::Equal),
            "minus" | "underscore" => Some(Key::Minus),
            "bracketleft" | "braceleft" => Some(Key::BracketLeft),
            "bracketright" | "braceright" => Some(Key::BracketRight),
            "comma" | "less" => Some(Key::Comma),
            "period" | "greater" => Some(Key::Period),
            "slash" | "question" => Some(Key::Slash),
            "semicolon" | "colon" => Some(Key::Semicolon),
            "apostrophe" | "quotedbl" => Some(Key::Quote),
            "grave" | "asciitilde" => Some(Key::Backquote),
            "backslash" | "bar" => Some(Key::Backslash),
            "parenright" => Some(Key::Digit(0)),
            "exclam" => Some(Key::Digit(1)),
            "at" => Some(Key::Digit(2)),
            "numbersign" => Some(Key::Digit(3)),
            "dollar" => Some(Key::Digit(4)),
            "percent" => Some(Key::Digit(5)),
            "asciicircum" => Some(Key::Digit(6)),
            "ampersand" => Some(Key::Digit(7)),
            "asterisk" => Some(Key::Digit(8)),
            "parenleft" => Some(Key::Digit(9)),
            "KP_Add" => Some(Key::NumpadAdd),
            "KP_Subtract" => Some(Key::NumpadSubtract),
            "KP_Multiply" => Some(Key::NumpadMultiply),
            "KP_Divide" => Some(Key::NumpadDivide),
            "KP_Decimal" => Some(Key::NumpadDecimal),
            _ => None,
        };
        if let Some(key) = named {
            return key;
        }
        if let Some(digit) = name.strip_prefix("KP_") {
            return small_number(digit, 9)
                .filter(|_| digit.len() == 1)
                .map_or(Key::Other, Key::Numpad);
        }
        if let Some(n) = name
            .strip_prefix('F')
            .and_then(|rest| small_number(rest, 24))
        {
            return if n >= 1 { Key::F(n) } else { Key::Other };
        }
        let mut chars = name.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_ascii_alphabetic() => Key::Letter(c.to_ascii_lowercase()),
            (Some(c), None) if c.is_ascii_digit() => Key::Digit(c as u8 - b'0'),
            _ => Key::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mods {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    /// The Windows / Super / Meta key.
    pub meta: bool,
}

/// The app's own origin, where the start page lives.
pub fn app_origin() -> Url {
    let origin = if cfg!(dev) {
        "http://localhost:1420/"
    } else if cfg!(windows) {
        "http://tauri.localhost/"
    } else {
        "tauri://localhost/"
    };
    // Constant, valid URLs.
    Url::parse(origin).expect("app origin is a valid URL")
}

/// Whether `url` is on the app's origin. Compared by parts: `tauri://` is not a special scheme,
/// so `Url::origin()` would be opaque and never equal.
pub fn is_app_url(origin: &Url, url: &Url) -> bool {
    url.scheme() == origin.scheme()
        && url.host_str() == origin.host_str()
        && url.port_or_known_default() == origin.port_or_known_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        let keys = [
            Key::Letter('a'),
            Key::Letter('z'),
            Key::Digit(0),
            Key::Digit(9),
            Key::F(1),
            Key::F(12),
            Key::F(24),
            Key::Numpad(0),
            Key::Numpad(9),
        ]
        .into_iter()
        .chain(NAMED.iter().map(|(key, _)| *key));
        for key in keys {
            let code = key.code().expect("a named key");
            assert_eq!(Key::from_code(&code), Some(key), "{code}");
        }
        assert_eq!(Key::from_code("KeyL"), Some(Key::Letter('l')));
        assert_eq!(Key::from_code("BracketLeft"), Some(Key::BracketLeft));
        assert_eq!(Key::Other.code(), None);
    }

    #[test]
    fn odd_codes_are_no_key() {
        for code in [
            "",
            "Key",
            "Keyl",
            "KeyAB",
            "Digit",
            "Digit10",
            "Digit+1",
            "F0",
            "F25",
            "F+5",
            "F05",
            "Numpad10",
            "Space",
            "Enter",
            "Escape",
            "Tab",
            "ShiftLeft",
            "kEYL",
        ] {
            assert_eq!(Key::from_code(code), None, "{code}");
        }
    }

    #[test]
    fn only_keys_that_type_need_a_real_modifier() {
        for key in [
            Key::Letter('k'),
            Key::Digit(1),
            Key::Equal,
            Key::BracketLeft,
            Key::Numpad(5),
            Key::NumpadAdd,
        ] {
            assert!(key.types_text(), "{key:?}");
        }
        for key in [Key::F(5), Key::Left, Key::Home, Key::PageDown] {
            assert!(!key.types_text(), "{key:?}");
        }
    }

    #[test]
    fn windows_virtual_keys() {
        assert_eq!(Key::from_windows_vk(0x57), Key::Letter('w'));
        assert_eq!(Key::from_windows_vk(0x4C), Key::Letter('l'));
        assert_eq!(Key::from_windows_vk(0x48), Key::Letter('h'));
        assert_eq!(Key::from_windows_vk(0x59), Key::Letter('y'));
        assert_eq!(Key::from_windows_vk(0x31), Key::Digit(1));
        assert_eq!(Key::from_windows_vk(0x74), Key::F(5));
        assert_eq!(Key::from_windows_vk(0x70), Key::F(1));
        assert_eq!(Key::from_windows_vk(0x7B), Key::F(12));
        assert_eq!(Key::from_windows_vk(0x25), Key::Left);
        assert_eq!(Key::from_windows_vk(0x26), Key::Up);
        assert_eq!(Key::from_windows_vk(0x27), Key::Right);
        assert_eq!(Key::from_windows_vk(0x28), Key::Down);
        assert_eq!(Key::from_windows_vk(0x24), Key::Home);
        assert_eq!(Key::from_windows_vk(0x23), Key::End);
        assert_eq!(Key::from_windows_vk(0x21), Key::PageUp);
        assert_eq!(Key::from_windows_vk(0x22), Key::PageDown);
        assert_eq!(Key::from_windows_vk(0x60), Key::Numpad(0));
        assert_eq!(Key::from_windows_vk(0x6B), Key::NumpadAdd);
        assert_eq!(Key::from_windows_vk(0xBB), Key::Equal);
        assert_eq!(Key::from_windows_vk(0xBD), Key::Minus);
        assert_eq!(Key::from_windows_vk(0xDB), Key::BracketLeft);
        assert_eq!(Key::from_windows_vk(0xDD), Key::BracketRight);
        assert_eq!(Key::from_windows_vk(0xBC), Key::Comma);
        assert_eq!(Key::from_windows_vk(0xBE), Key::Period);
        assert_eq!(Key::from_windows_vk(0xBF), Key::Slash);
        assert_eq!(Key::from_windows_vk(0xBA), Key::Semicolon);
        assert_eq!(Key::from_windows_vk(0xDE), Key::Quote);
        assert_eq!(Key::from_windows_vk(0xC0), Key::Backquote);
        assert_eq!(Key::from_windows_vk(0xDC), Key::Backslash);
        assert_eq!(Key::from_windows_vk(0x0D), Key::Other);
    }

    #[test]
    fn gdk_key_names() {
        assert_eq!(Key::from_gdk_name("w"), Key::Letter('w'));
        assert_eq!(Key::from_gdk_name("W"), Key::Letter('w'));
        assert_eq!(Key::from_gdk_name("h"), Key::Letter('h'));
        assert_eq!(Key::from_gdk_name("1"), Key::Digit(1));
        assert_eq!(Key::from_gdk_name("exclam"), Key::Digit(1), "Shift+1, US");
        assert_eq!(Key::from_gdk_name("F5"), Key::F(5));
        assert_eq!(Key::from_gdk_name("F12"), Key::F(12));
        assert_eq!(Key::from_gdk_name("Left"), Key::Left);
        assert_eq!(Key::from_gdk_name("Right"), Key::Right);
        assert_eq!(Key::from_gdk_name("Page_Up"), Key::PageUp);
        assert_eq!(Key::from_gdk_name("Next"), Key::PageDown);
        assert_eq!(Key::from_gdk_name("bracketleft"), Key::BracketLeft);
        assert_eq!(Key::from_gdk_name("braceleft"), Key::BracketLeft);
        assert_eq!(Key::from_gdk_name("equal"), Key::Equal);
        assert_eq!(Key::from_gdk_name("minus"), Key::Minus);
        assert_eq!(Key::from_gdk_name("apostrophe"), Key::Quote);
        assert_eq!(Key::from_gdk_name("grave"), Key::Backquote);
        assert_eq!(Key::from_gdk_name("KP_7"), Key::Numpad(7));
        assert_eq!(Key::from_gdk_name("KP_Add"), Key::NumpadAdd);
        assert_eq!(Key::from_gdk_name("Return"), Key::Other);
        assert_eq!(Key::from_gdk_name("F0"), Key::Other);
        assert_eq!(Key::from_gdk_name("KP_Enter"), Key::Other);
    }

    #[test]
    fn app_urls_are_recognised_on_both_origins() {
        for (origin, inside, outside) in [
            (
                "tauri://localhost/",
                "tauri://localhost/bar.html",
                "tauri://elsewhere/",
            ),
            (
                "http://tauri.localhost/",
                "http://tauri.localhost/index.html",
                "http://example.com/",
            ),
        ] {
            let origin = Url::parse(origin).expect("valid");
            assert!(is_app_url(&origin, &Url::parse(inside).expect("valid")));
            assert!(!is_app_url(&origin, &Url::parse(outside).expect("valid")));
        }
    }

    #[test]
    fn app_origin_is_parseable() {
        assert!(matches!(app_origin().scheme(), "tauri" | "http"));
    }
}
