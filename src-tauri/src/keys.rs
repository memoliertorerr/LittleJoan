//! Keys that act on the page window, handled natively before the page sees them.
//!
//! Remote pages never get IPC, so in-page keys are caught by native hooks: WebView2's
//! `AcceleratorKeyPressed` on Windows and the GTK window's `key-press-event` on Linux. On macOS
//! the app menu's key equivalents do the same job, so this table returns nothing there.
//!
//! | Keys                    | Windows | Linux | Action        |
//! |-------------------------|---------|-------|---------------|
//! | Ctrl+W                  | yes     | yes   | Close         |
//! | Ctrl+L                  | yes     | yes   | Open location |
//! | Ctrl+R, F5              | native  | yes   | Reload        |
//! | Alt+Left / Alt+Right    | native  | yes   | Back / Forward|
//!
//! "native" = WebView2 already handles it as a browser accelerator key.

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAction {
    Close,
    OpenLocation,
    Reload,
    Back,
    Forward,
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

/// A key, reduced to what the table above needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A letter, lowercase.
    Letter(char),
    F5,
    Left,
    Right,
    Other,
}

impl Key {
    /// From a Windows virtual-key code (VK_A..VK_Z are 0x41..0x5A).
    pub fn from_windows_vk(vk: u32) -> Key {
        match vk {
            0x41..=0x5A => {
                char::from_u32(vk).map_or(Key::Other, |c| Key::Letter(c.to_ascii_lowercase()))
            }
            0x74 => Key::F5,
            0x25 => Key::Left,
            0x27 => Key::Right,
            _ => Key::Other,
        }
    }

    /// From a GDK key name ("w", "W", "F5", "Left", "Right").
    pub fn from_gdk_name(name: &str) -> Key {
        match name {
            "F5" => Key::F5,
            "Left" => Key::Left,
            "Right" => Key::Right,
            _ => {
                let mut chars = name.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_ascii_alphabetic() => {
                        Key::Letter(c.to_ascii_lowercase())
                    }
                    _ => Key::Other,
                }
            }
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

/// The page action for a key press, if any. Exact modifier matches only: Ctrl+Shift+W is not
/// Ctrl+W, and Ctrl+Alt (AltGr on many layouts, used to type characters) never matches.
pub fn action_for(key: Key, mods: Mods, platform: Platform) -> Option<PageAction> {
    if platform == Platform::MacOs {
        return None;
    }
    let only_ctrl = mods
        == Mods {
            ctrl: true,
            ..Mods::default()
        };
    let only_alt = mods
        == Mods {
            alt: true,
            ..Mods::default()
        };
    let none = mods == Mods::default();
    let linux = platform == Platform::Linux;
    match key {
        Key::Letter('w') if only_ctrl => Some(PageAction::Close),
        Key::Letter('l') if only_ctrl => Some(PageAction::OpenLocation),
        Key::Letter('r') if linux && only_ctrl => Some(PageAction::Reload),
        Key::F5 if linux && none => Some(PageAction::Reload),
        Key::Left if linux && only_alt => Some(PageAction::Back),
        Key::Right if linux && only_alt => Some(PageAction::Forward),
        _ => None,
    }
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

/// The start page with `current` pre-filled (`index.html?u=…`), or `None` when `current` already
/// is a page of the app.
pub fn start_page_url(origin: &Url, current: &Url) -> Option<Url> {
    if is_app_url(origin, current) {
        return None;
    }
    let mut url = origin.join("index.html").ok()?;
    url.query_pairs_mut().append_pair("u", current.as_str());
    Some(url)
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

    const CTRL: Mods = Mods {
        ctrl: true,
        shift: false,
        alt: false,
        meta: false,
    };
    const ALT: Mods = Mods {
        ctrl: false,
        shift: false,
        alt: true,
        meta: false,
    };
    const NONE: Mods = Mods {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
    };

    fn w() -> Key {
        Key::Letter('w')
    }

    #[test]
    fn close_and_location_on_windows_and_linux() {
        for platform in [Platform::Windows, Platform::Linux] {
            assert_eq!(action_for(w(), CTRL, platform), Some(PageAction::Close));
            assert_eq!(
                action_for(Key::Letter('l'), CTRL, platform),
                Some(PageAction::OpenLocation)
            );
        }
    }

    #[test]
    fn reload_and_history_only_on_linux() {
        assert_eq!(
            action_for(Key::Letter('r'), CTRL, Platform::Linux),
            Some(PageAction::Reload)
        );
        assert_eq!(
            action_for(Key::F5, NONE, Platform::Linux),
            Some(PageAction::Reload)
        );
        assert_eq!(
            action_for(Key::Left, ALT, Platform::Linux),
            Some(PageAction::Back)
        );
        assert_eq!(
            action_for(Key::Right, ALT, Platform::Linux),
            Some(PageAction::Forward)
        );
        // WebView2 handles these itself.
        assert_eq!(action_for(Key::Letter('r'), CTRL, Platform::Windows), None);
        assert_eq!(action_for(Key::F5, NONE, Platform::Windows), None);
        assert_eq!(action_for(Key::Left, ALT, Platform::Windows), None);
    }

    #[test]
    fn near_misses_do_nothing() {
        for platform in [Platform::Windows, Platform::Linux] {
            assert_eq!(
                action_for(w(), NONE, platform),
                None,
                "plain W types a letter"
            );
            let ctrl_shift = Mods {
                shift: true,
                ..CTRL
            };
            assert_eq!(action_for(w(), ctrl_shift, platform), None);
            let altgr = Mods { alt: true, ..CTRL };
            assert_eq!(action_for(w(), altgr, platform), None, "AltGr is Ctrl+Alt");
            let ctrl_meta = Mods { meta: true, ..CTRL };
            assert_eq!(action_for(w(), ctrl_meta, platform), None);
            assert_eq!(action_for(Key::Other, CTRL, platform), None);
        }
    }

    #[test]
    fn macos_leaves_keys_to_the_menu() {
        for key in [w(), Key::Letter('l'), Key::Letter('r'), Key::F5, Key::Left] {
            for mods in [CTRL, ALT, NONE] {
                assert_eq!(action_for(key, mods, Platform::MacOs), None);
            }
        }
    }

    #[test]
    fn windows_virtual_keys() {
        assert_eq!(Key::from_windows_vk(0x57), Key::Letter('w'));
        assert_eq!(Key::from_windows_vk(0x4C), Key::Letter('l'));
        assert_eq!(Key::from_windows_vk(0x74), Key::F5);
        assert_eq!(Key::from_windows_vk(0x25), Key::Left);
        assert_eq!(Key::from_windows_vk(0x27), Key::Right);
        assert_eq!(Key::from_windows_vk(0x31), Key::Other);
    }

    #[test]
    fn gdk_key_names() {
        assert_eq!(Key::from_gdk_name("w"), Key::Letter('w'));
        assert_eq!(Key::from_gdk_name("W"), Key::Letter('w'));
        assert_eq!(Key::from_gdk_name("F5"), Key::F5);
        assert_eq!(Key::from_gdk_name("Left"), Key::Left);
        assert_eq!(Key::from_gdk_name("Right"), Key::Right);
        assert_eq!(Key::from_gdk_name("Return"), Key::Other);
        assert_eq!(Key::from_gdk_name("1"), Key::Other);
    }

    #[test]
    fn start_page_url_carries_the_current_address() {
        let origin = Url::parse("tauri://localhost/").expect("valid");
        let current = Url::parse("https://example.com/a?b=1&c=d").expect("valid");
        let got = start_page_url(&origin, &current).expect("a remote page");
        assert_eq!(
            got.as_str(),
            "tauri://localhost/index.html?u=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1%26c%3Dd"
        );
        let windows = Url::parse("http://tauri.localhost/").expect("valid");
        assert_eq!(
            start_page_url(&windows, &current)
                .expect("a remote page")
                .as_str(),
            "http://tauri.localhost/index.html?u=https%3A%2F%2Fexample.com%2Fa%3Fb%3D1%26c%3Dd"
        );
    }

    #[test]
    fn start_page_url_is_none_on_the_start_page() {
        for (origin, start) in [
            (
                "http://tauri.localhost/",
                "http://tauri.localhost/index.html?u=x",
            ),
            ("tauri://localhost/", "tauri://localhost/index.html"),
        ] {
            let origin = Url::parse(origin).expect("valid");
            let start = Url::parse(start).expect("valid");
            assert_eq!(start_page_url(&origin, &start), None, "{start}");
        }
        let origin = Url::parse("tauri://localhost/").expect("valid");
        let other = Url::parse("tauri://elsewhere/").expect("valid");
        assert!(start_page_url(&origin, &other).is_some());
    }

    #[test]
    fn app_origin_is_parseable() {
        assert!(matches!(app_origin().scheme(), "tauri" | "http"));
    }
}
