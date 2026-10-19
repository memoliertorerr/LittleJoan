//! Which backend holds the shortcut that summons Little Joan, and what the XDG GlobalShortcuts
//! portal is asked and told (LJ17; pure, tested). platform/portal.rs talks to the portal.
//!
//! global-hotkey's Linux backend grabs keys on X11, so in a Wayland session it hears them only
//! while an X11 window has the focus. There the portal is asked first; the desktop then owns the
//! key — it may ask the user, and may pick another trigger than the one Little Joan prefers.

use tauri_plugin_global_shortcut::Code;

/// The portal shortcut's id (Activated names it) and the description the desktop shows.
pub const SHORTCUT_ID: &str = "toggle";
pub const DESCRIPTION: &str = "Summon or dismiss Little Joan";

/// Smoke-log events (smoke.rs): the portal took the shortcut, it didn't, its trigger changed, and
/// whether its host registry took Little Joan's app id.
pub const REGISTERED: &str = "shortcut-registered";
pub const UNAVAILABLE: &str = "shortcut-portal-unavailable";
pub const CHANGED: &str = "shortcut-portal-changed";
pub const HOST: &str = "shortcut-portal-host";

/// Who holds the summon shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// global-hotkey, as since v0.1: Windows, macOS, and Linux outside a Wayland session.
    Hotkey,
    /// The XDG GlobalShortcuts portal: a Wayland session whose portal took the shortcut.
    Portal,
    /// A Wayland session without the portal (or it refused): global-hotkey on XWayland as before
    /// — it hears keys only while an X11 window has the focus — and the `little-joan --toggle`
    /// hint (the welcome, Settings → Keyboard).
    HotkeyOnXwayland,
}

/// The portal's side of it, kept in AppState.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PortalState {
    /// Not asked (no Wayland session), or it didn't take the shortcut.
    #[default]
    None,
    /// Being asked, at launch; nothing holds the shortcut meanwhile.
    Asking,
    /// The portal holds it. `trigger`: the desktop's own words for the key ("Press <ctrl>…" on
    /// GNOME), `None` while the desktop gave it no key.
    Registered { trigger: Option<String> },
}

impl PortalState {
    pub fn registered(&self) -> bool {
        matches!(self, PortalState::Registered { .. })
    }

    /// The desktop's trigger, when the portal holds the shortcut with a key.
    pub fn trigger(&self) -> Option<&str> {
        match self {
            PortalState::Registered { trigger } => trigger.as_deref(),
            _ => None,
        }
    }
}

/// The decision. `wayland_session` is welcome::wayland_session — never true on Windows or macOS.
pub fn backend(wayland_session: bool, portal: &PortalState) -> Backend {
    match (wayland_session, portal.registered()) {
        (false, _) => Backend::Hotkey,
        (true, true) => Backend::Portal,
        (true, false) => Backend::HotkeyOnXwayland,
    }
}

impl Backend {
    /// The welcome and Settings explain `little-joan --toggle` (StartInfo.waylandNoShortcut).
    pub fn toggle_hint(self) -> bool {
        self == Backend::HotkeyOnXwayland
    }

    /// Whether the shortcut can summon Little Joan (tray.rs's icon rule, LJ41): global-hotkey
    /// with a registered shortcut outside Wayland; the portal once the desktop gave it a key.
    pub fn works(self, hotkey_registered: bool, portal: &PortalState) -> bool {
        match self {
            Backend::Hotkey => hotkey_registered,
            Backend::Portal => portal.trigger().is_some(),
            Backend::HotkeyOnXwayland => false,
        }
    }
}

/// Whether the first launch's welcome waits for the portal's answer (so it names the desktop's
/// key, or explains `--toggle` when there is none) instead of opening at once.
pub fn welcome_waits(portal: &PortalState) -> bool {
    *portal == PortalState::Asking
}

/// The preferred trigger for `accelerator` in the XDG shortcuts spec's syntax — modifiers `CTRL`,
/// `ALT`, `SHIFT`, `LOGO` and an xkb key name, joined by `+` (`CTRL+SHIFT+space`) — read as Linux
/// reads it (`CommandOrControl` is Ctrl). `None` when it doesn't parse or its key has no name
/// there; the portal is then asked without a preference.
pub fn trigger(accelerator: &str) -> Option<String> {
    // The same parser registration uses: it says whether this is a shortcut, and its key.
    let key = keysym(crate::shortcut::parse(accelerator).ok()?.key)?;
    let (mut logo, mut ctrl, mut alt, mut shift) = (false, false, false, false);
    let mut parts: Vec<&str> = accelerator.split('+').map(str::trim).collect();
    parts.pop();
    for part in parts {
        match part.to_ascii_uppercase().as_str() {
            "OPTION" | "ALT" => alt = true,
            "CONTROL" | "CTRL" | "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL"
            | "CMDORCONTROL" => ctrl = true,
            "COMMAND" | "CMD" | "SUPER" => logo = true,
            "SHIFT" => shift = true,
            _ => return None,
        }
    }
    let mods = [
        (logo, "LOGO"),
        (ctrl, "CTRL"),
        (alt, "ALT"),
        (shift, "SHIFT"),
    ];
    let mut names: Vec<&str> = mods.iter().filter(|(on, _)| *on).map(|(_, n)| *n).collect();
    names.push(key);
    Some(names.join("+"))
}

/// A key's xkb name (`xkbcommon-keysyms.h` without `XKB_KEY_`), as the spec wants it. None for
/// keys it can't name: Caps Lock and Num Lock (the spec's CAPS and NUM are modifiers), and any
/// key without a fixed symbol (Fn, the layout-dependent Intl keys).
pub fn keysym(key: Code) -> Option<&'static str> {
    const LETTERS: [&str; 26] = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];
    const DIGITS: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    const KEYPAD: [&str; 10] = [
        "KP_0", "KP_1", "KP_2", "KP_3", "KP_4", "KP_5", "KP_6", "KP_7", "KP_8", "KP_9",
    ];
    const F_KEYS: [&str; 35] = [
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14",
        "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23", "F24", "F25", "F26", "F27",
        "F28", "F29", "F30", "F31", "F32", "F33", "F34", "F35",
    ];
    const NAMED: [(&str, &str); 45] = [
        ("Space", "space"),
        ("Enter", "Return"),
        ("Tab", "Tab"),
        ("Escape", "Escape"),
        ("Backspace", "BackSpace"),
        ("Delete", "Delete"),
        ("Insert", "Insert"),
        ("Home", "Home"),
        ("End", "End"),
        ("PageUp", "Page_Up"),
        ("PageDown", "Page_Down"),
        ("ArrowUp", "Up"),
        ("ArrowDown", "Down"),
        ("ArrowLeft", "Left"),
        ("ArrowRight", "Right"),
        ("Equal", "equal"),
        ("Minus", "minus"),
        ("BracketLeft", "bracketleft"),
        ("BracketRight", "bracketright"),
        ("Comma", "comma"),
        ("Period", "period"),
        ("Slash", "slash"),
        ("Semicolon", "semicolon"),
        ("Quote", "apostrophe"),
        ("Backquote", "grave"),
        ("Backslash", "backslash"),
        ("NumpadAdd", "KP_Add"),
        ("NumpadSubtract", "KP_Subtract"),
        ("NumpadMultiply", "KP_Multiply"),
        ("NumpadDivide", "KP_Divide"),
        ("NumpadDecimal", "KP_Decimal"),
        ("NumpadEnter", "KP_Enter"),
        ("NumpadEqual", "KP_Equal"),
        ("PrintScreen", "Print"),
        ("ScrollLock", "Scroll_Lock"),
        ("Pause", "Pause"),
        ("AudioVolumeMute", "XF86AudioMute"),
        ("AudioVolumeDown", "XF86AudioLowerVolume"),
        ("AudioVolumeUp", "XF86AudioRaiseVolume"),
        ("MediaPlay", "XF86AudioPlay"),
        ("MediaPlayPause", "XF86AudioPlay"),
        ("MediaPause", "XF86AudioPause"),
        ("MediaStop", "XF86AudioStop"),
        ("MediaTrackNext", "XF86AudioNext"),
        ("MediaTrackPrevious", "XF86AudioPrev"),
    ];
    // Code's names are the W3C `KeyboardEvent.code` ones (KeyA, Digit1, Numpad0, F5, …).
    let name = key.to_string();
    let numbered = |prefix: &str, table: &'static [&'static str], first: usize| {
        let n: usize = name.strip_prefix(prefix)?.parse().ok()?;
        table.get(n.checked_sub(first)?).copied()
    };
    if let Some(letter) = name.strip_prefix("Key") {
        let mut chars = letter.chars();
        return match (chars.next(), chars.next()) {
            (Some(c), None) if c.is_ascii_uppercase() => {
                LETTERS.get(usize::from(c as u8 - b'A')).copied()
            }
            _ => None,
        };
    }
    numbered("Digit", &DIGITS, 0)
        .or_else(|| numbered("Numpad", &KEYPAD, 0))
        .or_else(|| numbered("F", &F_KEYS, 1))
        .or_else(|| {
            NAMED
                .iter()
                .find(|(code, _)| *code == name)
                .map(|(_, sym)| *sym)
        })
}

/// What the desktop reported as the trigger, for showing: trimmed, `None` when it said nothing.
pub fn shown_trigger(description: &str) -> Option<String> {
    let text = description.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

/// The smoke log's line when the portal took the shortcut: `shortcut-registered <trigger>
/// portal` (`none` when the desktop gave it no key yet).
pub fn registered_line(trigger: Option<&str>) -> (&'static str, String) {
    (REGISTERED, format!("{} portal", trigger.unwrap_or("none")))
}

/// Why the portal didn't take the shortcut (the smoke log's first word; the error follows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unavailable {
    /// No D-Bus session bus.
    NoBus,
    /// No portal, or none that offers GlobalShortcuts (GNOME before 48, a desktop without one).
    NoPortal,
    /// The portal said no (an app id it can't place, an error of the desktop's).
    Refused,
    /// The user declined the desktop's dialog.
    Cancelled,
    /// The answer, apart from the user's dialog, didn't come within the time allowed.
    Timeout,
    /// It answered without Little Joan's shortcut.
    NoShortcut,
    /// The portal's session went away later (the portal restarted, the bus closed).
    Ended,
    /// Anything else.
    Failed,
}

impl Unavailable {
    pub fn word(self) -> &'static str {
        match self {
            Unavailable::NoBus => "no-bus",
            Unavailable::NoPortal => "no-portal",
            Unavailable::Refused => "refused",
            Unavailable::Cancelled => "cancelled",
            Unavailable::Timeout => "timeout",
            Unavailable::NoShortcut => "no-shortcut",
            Unavailable::Ended => "ended",
            Unavailable::Failed => "failed",
        }
    }
}

/// The smoke log's line when the portal didn't take the shortcut (then global-hotkey does, as
/// before): `shortcut-portal-unavailable <why>[ <detail>]`.
pub fn unavailable_line(why: Unavailable, detail: &str) -> (&'static str, String) {
    let detail = detail.trim();
    let text = if detail.is_empty() {
        why.word().to_owned()
    } else {
        format!("{} {detail}", why.word())
    };
    (UNAVAILABLE, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smoke::format_line;
    use crate::welcome::wayland_session;

    #[test]
    fn accelerators_become_the_specs_triggers() {
        for (accelerator, expected) in [
            ("CommandOrControl+Shift+Space", "CTRL+SHIFT+space"),
            ("Alt+KeyJ", "ALT+j"),
            ("Super+Digit1", "LOGO+1"),
            ("Control+Alt+Space", "CTRL+ALT+space"),
            ("Super+Shift+KeyK", "LOGO+SHIFT+k"),
            ("CommandOrControl+F12", "CTRL+F12"),
            ("Ctrl+Shift+ArrowLeft", "CTRL+SHIFT+Left"),
            ("Alt+PageUp", "ALT+Page_Up"),
            ("CommandOrControl+Numpad0", "CTRL+KP_0"),
            ("Alt+Backquote", "ALT+grave"),
            ("Ctrl+Enter", "CTRL+Return"),
            ("F24", "F24"),
            ("Ctrl+F1", "CTRL+F1"),
            ("alt+shift+keyz", "ALT+SHIFT+z"),
            ("CommandOrControl+MediaPlayPause", "CTRL+XF86AudioPlay"),
        ] {
            assert_eq!(
                trigger(accelerator).as_deref(),
                Some(expected),
                "{accelerator}"
            );
        }
    }

    #[test]
    fn what_the_spec_cant_express_has_no_trigger() {
        for accelerator in ["Shift+CapsLock", "Ctrl+NumLock", "Hyper+Nope", "", "Shift+"] {
            assert_eq!(trigger(accelerator), None, "{accelerator}");
        }
        for code in [Code::Fn, Code::IntlBackslash, Code::CapsLock, Code::NumLock] {
            assert_eq!(keysym(code), None, "{code}");
        }
        assert_eq!(keysym(Code::F35), Some("F35"));
        assert_eq!(keysym(Code::KeyA), Some("a"));
        assert_eq!(keysym(Code::MediaTrackPrevious), Some("XF86AudioPrev"));
        assert_eq!(keysym(Code::Space), Some("space"));
    }

    #[test]
    fn the_backend_follows_the_session_and_the_portal() {
        let registered = PortalState::Registered {
            trigger: Some("Press <ctrl><shift>space".into()),
        };
        // Not Linux, or an X11 session: global-hotkey, whatever else.
        for (os, kind, display) in [
            ("macos", Some("wayland"), Some("wayland-0")),
            ("windows", None, Some("wayland-0")),
            ("linux", Some("x11"), None),
            ("linux", None, None),
        ] {
            let session = wayland_session(os, kind, display);
            assert_eq!(backend(session, &PortalState::None), Backend::Hotkey);
            assert_eq!(backend(session, &registered), Backend::Hotkey);
            assert!(!backend(session, &PortalState::None).toggle_hint());
        }
        let wayland = wayland_session("linux", Some("wayland"), Some("wayland-0"));
        assert_eq!(backend(wayland, &registered), Backend::Portal);
        assert!(!Backend::Portal.toggle_hint(), "the desktop's key: no hint");
        for portal in [PortalState::None, PortalState::Asking] {
            assert_eq!(backend(wayland, &portal), Backend::HotkeyOnXwayland);
        }
        assert!(Backend::HotkeyOnXwayland.toggle_hint());
    }

    #[test]
    fn a_portal_shortcut_works_once_the_desktop_gave_it_a_key() {
        let keyed = PortalState::Registered {
            trigger: Some("Meta+Shift+Space".into()),
        };
        let keyless = PortalState::Registered { trigger: None };
        assert!(Backend::Portal.works(false, &keyed));
        assert!(!Backend::Portal.works(false, &keyless));
        assert!(Backend::Hotkey.works(true, &PortalState::None));
        assert!(!Backend::Hotkey.works(false, &PortalState::None));
        assert!(
            !Backend::HotkeyOnXwayland.works(true, &PortalState::None),
            "the X11 grab can't fire in a Wayland session"
        );
        assert_eq!(keyed.trigger(), Some("Meta+Shift+Space"));
        assert_eq!(PortalState::Asking.trigger(), None);
    }

    #[test]
    fn the_welcome_waits_only_while_the_portal_is_asked() {
        assert!(welcome_waits(&PortalState::Asking));
        assert!(!welcome_waits(&PortalState::None));
        assert!(!welcome_waits(&PortalState::Registered { trigger: None }));
    }

    #[test]
    fn the_desktops_trigger_is_shown_as_it_said_it() {
        assert_eq!(
            shown_trigger("  Press <Control><Shift>space "),
            Some("Press <Control><Shift>space".to_owned())
        );
        assert_eq!(shown_trigger(" \n"), None);
    }

    #[test]
    fn the_smoke_log_says_who_holds_the_shortcut() {
        let (event, detail) = registered_line(Some("Press <ctrl><shift>space"));
        assert_eq!(
            format_line(5, event, &detail),
            "5 shortcut-registered Press <ctrl><shift>space portal\n"
        );
        let (event, detail) = registered_line(None);
        assert_eq!(
            format_line(5, event, &detail),
            "5 shortcut-registered none portal\n"
        );
        let (event, detail) = unavailable_line(Unavailable::NoPortal, "");
        assert_eq!(
            format_line(6, event, &detail),
            "6 shortcut-portal-unavailable no-portal\n"
        );
        let (event, detail) = unavailable_line(Unavailable::Refused, " An app id is required\n");
        assert_eq!(
            format_line(6, event, &detail),
            "6 shortcut-portal-unavailable refused An app id is required\n"
        );
        let words: Vec<&str> = [
            Unavailable::NoBus,
            Unavailable::NoPortal,
            Unavailable::Refused,
            Unavailable::Cancelled,
            Unavailable::Timeout,
            Unavailable::NoShortcut,
            Unavailable::Ended,
            Unavailable::Failed,
        ]
        .iter()
        .map(|why| why.word())
        .collect();
        let unique: std::collections::BTreeSet<&str> = words.iter().copied().collect();
        assert_eq!(unique.len(), words.len());
        assert!(words.iter().all(|w| !w.contains(' ')));
    }
}
