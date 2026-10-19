//! One keymap (LJ43, Ground Rule 25): every key of Little Joan's own — the page keys Windows' and
//! Linux's native hooks catch (platform/win.rs, platform/linux.rs), the macOS menu's key
//! equivalents (menu.rs), the keys Settings shows (`keymap_info`) — comes from here and the user's
//! overrides (settings' `keys`); no other file hard-codes one. Not in it (fixed): Esc / Enter /
//! arrows / Tab in the bar, the Edit menu, Cmd+Q and Cmd+, on macOS; and on Windows WebView2's own
//! browser keys (Ctrl+R, F5, Alt+← / Alt+→), which keep working beside a custom binding (turning
//! them off would take other keys with them).
//!
//! Bindings use the global shortcut's accelerator syntax with `KeyboardEvent.code` key names
//! (`CommandOrControl+Shift+KeyT`, `Alt+ArrowLeft`, `F5`), `CommandOrControl` resolved per OS when
//! a key is matched — never stored resolved. Each action has an ordered list of default bindings
//! per OS: the first is the one Settings shows and changes, the rest are fixed aliases. Pure: no
//! Tauri types; tests in keymap/tests.rs.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::keys::{Key, Mods, PageAction, Platform};

/// What a Little Joan key does. Ids are these names in settings' `keys`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyAction {
    OpenLocation,
    History,
    Translate,
    Reload,
    Back,
    Forward,
    OpenInBrowser,
    Close,
}

impl KeyAction {
    /// In the order Settings lists them.
    pub const ALL: [KeyAction; 8] = [
        KeyAction::OpenLocation,
        KeyAction::History,
        KeyAction::Translate,
        KeyAction::Reload,
        KeyAction::Back,
        KeyAction::Forward,
        KeyAction::OpenInBrowser,
        KeyAction::Close,
    ];

    pub fn id(self) -> &'static str {
        match self {
            KeyAction::OpenLocation => "open-location",
            KeyAction::History => "history",
            KeyAction::Translate => "translate",
            KeyAction::Reload => "reload",
            KeyAction::Back => "back",
            KeyAction::Forward => "forward",
            KeyAction::OpenInBrowser => "open-in-browser",
            KeyAction::Close => "close",
        }
    }

    pub fn from_id(id: &str) -> Option<KeyAction> {
        KeyAction::ALL.into_iter().find(|action| action.id() == id)
    }

    /// Its title as the macOS menu has it.
    pub fn title(self) -> &'static str {
        match self {
            KeyAction::OpenLocation => "Open Location…",
            KeyAction::History => "History",
            KeyAction::Translate => "Translate Page",
            KeyAction::Reload => "Reload",
            KeyAction::Back => "Back",
            KeyAction::Forward => "Forward",
            KeyAction::OpenInBrowser => "Open Page in Browser",
            KeyAction::Close => "Close Window",
        }
    }

    /// What it does to the page window (page_actions.rs) — the one place they are paired.
    pub fn page_action(self) -> PageAction {
        match self {
            KeyAction::OpenLocation => PageAction::OpenLocation,
            KeyAction::History => PageAction::History,
            KeyAction::Translate => PageAction::Translate,
            KeyAction::Reload => PageAction::Reload,
            KeyAction::Back => PageAction::Back,
            KeyAction::Forward => PageAction::Forward,
            KeyAction::OpenInBrowser => PageAction::OpenInBrowser,
            KeyAction::Close => PageAction::Close,
        }
    }
}

/// A key and its modifiers, as an accelerator says them (unresolved).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Binding {
    /// `CommandOrControl`: Cmd on macOS, Ctrl elsewhere.
    pub primary: bool,
    /// `Control`: ⌃ on macOS; elsewhere the same key as `primary`.
    pub control: bool,
    /// `Super`: the Windows / Super key; ⌘ on macOS.
    pub meta: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: Key,
}

impl Binding {
    /// A key with `CommandOrControl` (and Shift).
    const fn primary(shift: bool, key: Key) -> Binding {
        Binding {
            primary: true,
            control: false,
            meta: false,
            alt: false,
            shift,
            key,
        }
    }

    /// An accelerator: modifiers (any case: `CommandOrControl` / `CmdOrCtrl`, `Control` / `Ctrl`,
    /// `Super` / `Cmd` / `Command` / `Meta`, `Alt` / `Option`, `Shift`) and a `KeyboardEvent.code`
    /// key last. Anything else is none.
    pub fn parse(text: &str) -> Option<Binding> {
        let mut parts: Vec<&str> = text.split('+').map(str::trim).collect();
        let key = Key::from_code(parts.pop()?)?;
        let mut binding = Binding {
            primary: false,
            control: false,
            meta: false,
            alt: false,
            shift: false,
            key,
        };
        for part in parts {
            let flag = match part.to_ascii_lowercase().as_str() {
                "commandorcontrol" | "cmdorctrl" | "commandorctrl" | "cmdorcontrol" => {
                    &mut binding.primary
                }
                "control" | "ctrl" => &mut binding.control,
                "super" | "cmd" | "command" | "meta" => &mut binding.meta,
                "alt" | "option" => &mut binding.alt,
                "shift" => &mut binding.shift,
                _ => return None,
            };
            if *flag {
                return None;
            }
            *flag = true;
        }
        Some(binding)
    }

    /// The accelerator to store and to give the macOS menu: `CommandOrControl`, `Control`,
    /// `Super`, `Alt`, `Shift` in that order, then the key's code. Both Tauri's menu parser and the
    /// global shortcut's take these names.
    pub fn accelerator(&self) -> String {
        let mut parts: Vec<String> = [
            (self.primary, "CommandOrControl"),
            (self.control, "Control"),
            (self.meta, "Super"),
            (self.alt, "Alt"),
            (self.shift, "Shift"),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, name)| name.to_owned())
        .collect();
        parts.push(self.key.code().unwrap_or_default());
        parts.join("+")
    }

    /// The modifiers a press must have, on `platform`.
    pub fn mods(&self, platform: Platform) -> Mods {
        if platform == Platform::MacOs {
            Mods {
                ctrl: self.control,
                shift: self.shift,
                alt: self.alt,
                meta: self.primary || self.meta,
            }
        } else {
            Mods {
                ctrl: self.primary || self.control,
                shift: self.shift,
                alt: self.alt,
                meta: self.meta,
            }
        }
    }

    /// The same keys on `platform` as `other`.
    pub fn same_as(&self, other: &Binding, platform: Platform) -> bool {
        self.key == other.key && self.mods(platform) == other.mods(platform)
    }

    /// For people, as Settings shows it (ui/lib/shortcut.ts): "⌘⇧K" on macOS, "Ctrl+Shift+K"
    /// elsewhere.
    pub fn display(&self, platform: Platform) -> String {
        let mac = platform == Platform::MacOs;
        let mut names: Vec<&str> = Vec::new();
        for (on, mac_name, name) in [
            (self.primary, "⌘", "Ctrl"),
            (self.control, "⌃", "Ctrl"),
            (self.meta, "⌘", "Super"),
            (self.alt, "⌥", "Alt"),
            (self.shift, "⇧", "Shift"),
        ] {
            let name = if mac { mac_name } else { name };
            if on && !names.contains(&name) {
                names.push(name);
            }
        }
        let code = self.key.code().unwrap_or_default();
        let key = code
            .strip_prefix("Key")
            .or_else(|| code.strip_prefix("Digit"))
            .unwrap_or(&code)
            .to_owned();
        if mac {
            format!("{}{key}", names.concat())
        } else {
            names.push(&key);
            names.join("+")
        }
    }
}

/// An action's default bindings on `platform`: the first is the one Settings changes, the rest
/// fixed aliases. Windows leaves Reload, Back and Forward to WebView2's own keys, and Open in
/// browser has no key on Windows and Linux (the toolbar and the tray have it).
pub fn defaults(platform: Platform, action: KeyAction) -> Vec<Binding> {
    let p = |shift: bool, key: Key| Binding::primary(shift, key);
    let alt = |key: Key| Binding {
        primary: false,
        control: false,
        meta: false,
        alt: true,
        shift: false,
        key,
    };
    let plain = |key: Key| Binding {
        primary: false,
        control: false,
        meta: false,
        alt: false,
        shift: false,
        key,
    };
    let mac = platform == Platform::MacOs;
    let linux = platform == Platform::Linux;
    match action {
        KeyAction::Close => vec![p(false, Key::Letter('w'))],
        KeyAction::OpenLocation => vec![p(false, Key::Letter('l'))],
        KeyAction::Translate => vec![p(true, Key::Letter('t'))],
        KeyAction::History if mac => vec![p(false, Key::Letter('y'))],
        KeyAction::History => vec![p(false, Key::Letter('h'))],
        KeyAction::OpenInBrowser if mac => vec![p(true, Key::Letter('o'))],
        KeyAction::Reload if mac => vec![p(false, Key::Letter('r'))],
        KeyAction::Reload if linux => vec![p(false, Key::Letter('r')), plain(Key::F(5))],
        KeyAction::Back if mac => vec![p(false, Key::BracketLeft)],
        KeyAction::Back if linux => vec![alt(Key::Left)],
        KeyAction::Forward if mac => vec![p(false, Key::BracketRight)],
        KeyAction::Forward if linux => vec![alt(Key::Right)],
        KeyAction::OpenInBrowser | KeyAction::Reload | KeyAction::Back | KeyAction::Forward => {
            Vec::new()
        }
    }
}

/// Why a binding or a set of overrides can't be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// No action has this id.
    UnknownAction(String),
    /// Not an accelerator Little Joan can use.
    Unparsable(KeyAction, String),
    /// A key that types (no modifier and not an F-key; or Shift alone with a key that types).
    TypingKey(KeyAction),
    /// macOS: a key needs ⌘ or ⌃.
    NeedsCmdOrCtrl(KeyAction),
    /// Already another action's.
    Duplicate(KeyAction, KeyAction, Binding),
    /// It is the global shortcut that summons Little Joan.
    GlobalShortcut(KeyAction),
}

impl Problem {
    /// What Settings says (save_settings' refusal).
    pub fn message(&self, platform: Platform) -> String {
        let mac = platform == Platform::MacOs;
        match self {
            Problem::UnknownAction(id) => format!("Little Joan has no key called “{id}”."),
            Problem::Unparsable(action, text) => {
                format!(
                    "{}: “{text}” isn't a key Little Joan can use.",
                    plain(*action)
                )
            }
            Problem::TypingKey(action) => format!(
                "{}: plain keys type text — add {}.",
                plain(*action),
                if mac { "⌘ or ⌃" } else { "Ctrl or Alt" }
            ),
            Problem::NeedsCmdOrCtrl(action) => {
                format!("{}: on macOS a key needs ⌘ or ⌃.", plain(*action))
            }
            Problem::Duplicate(action, other, binding) => format!(
                "{}: {} is already {}.",
                plain(*action),
                binding.display(platform),
                plain(*other)
            ),
            Problem::GlobalShortcut(action) => format!(
                "{}: that's the shortcut that summons Little Joan.",
                plain(*action)
            ),
        }
    }
}

/// A title without the menu's "…".
fn plain(action: KeyAction) -> &'static str {
    action.title().trim_end_matches('…')
}

/// Whether `binding` may be one of `action`'s keys at all: never a typing key; on macOS only with
/// ⌘ or ⌃.
pub fn bindable(action: KeyAction, binding: &Binding, platform: Platform) -> Result<(), Problem> {
    let real = binding.primary || binding.control || binding.meta || binding.alt;
    let typing = if real {
        false
    } else if binding.shift {
        binding.key.types_text()
    } else {
        !matches!(binding.key, Key::F(_))
    };
    if typing {
        return Err(Problem::TypingKey(action));
    }
    if platform == Platform::MacOs && !(binding.primary || binding.control || binding.meta) {
        return Err(Problem::NeedsCmdOrCtrl(action));
    }
    Ok(())
}

/// Every action's keys on one OS.
#[derive(Debug, Clone, PartialEq)]
pub struct Keymap {
    platform: Platform,
    bindings: BTreeMap<KeyAction, Vec<Binding>>,
}

impl Keymap {
    /// The defaults with `overrides` (settings' `keys`: action id → accelerator, `""` = no key).
    /// An override replaces the action's first binding and drops its fixed aliases; one for an
    /// unknown action, unparsable, or not bindable is ignored (the default stays); one equal to
    /// the default is the default (its aliases stay).
    pub fn new(platform: Platform, overrides: &BTreeMap<String, String>) -> Keymap {
        let mut bindings = BTreeMap::new();
        for action in KeyAction::ALL {
            let mut list = defaults(platform, action);
            match overrides.get(action.id()).map(|text| text.trim()) {
                Some("") => list.clear(),
                Some(text) => {
                    if let Some(binding) = Binding::parse(text)
                        .filter(|b| bindable(action, b, platform).is_ok())
                        .filter(|b| !list.first().is_some_and(|d| d.same_as(b, platform)))
                    {
                        list = vec![binding];
                    }
                }
                None => {}
            }
            bindings.insert(action, list);
        }
        Keymap { platform, bindings }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// The action for a key press: its key and exactly its modifiers (Ctrl+Shift+L is not Ctrl+L,
    /// and a Ctrl+Alt press — AltGr on many layouts — never matches a Ctrl binding).
    pub fn action_for(&self, key: Key, mods: Mods) -> Option<KeyAction> {
        if key == Key::Other {
            return None;
        }
        KeyAction::ALL.into_iter().find(|action| {
            self.bindings(*action)
                .iter()
                .any(|b| b.key == key && b.mods(self.platform) == mods)
        })
    }

    /// The action's changeable binding (the menu's, the one Settings shows); `None`: no key.
    pub fn binding(&self, action: KeyAction) -> Option<&Binding> {
        self.bindings(action).first()
    }

    /// All of the action's bindings, aliases included.
    pub fn bindings(&self, action: KeyAction) -> &[Binding] {
        self.bindings.get(&action).map_or(&[], Vec::as_slice)
    }
}

/// Checks overrides as Settings saves them, with the global shortcut they must not take: unknown
/// actions, unparsable accelerators, typing keys, macOS keys without ⌘ or ⌃, two actions on one
/// key (aliases included), an action on the global shortcut. `Ok` or every problem found.
pub fn validate(
    platform: Platform,
    overrides: &BTreeMap<String, String>,
    global_shortcut: &str,
) -> Result<(), Vec<Problem>> {
    let mut problems = Vec::new();
    for (id, text) in overrides {
        let Some(action) = KeyAction::from_id(id) else {
            problems.push(Problem::UnknownAction(id.clone()));
            continue;
        };
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        match Binding::parse(text) {
            None => problems.push(Problem::Unparsable(action, text.to_owned())),
            Some(binding) => {
                if let Err(problem) = bindable(action, &binding, platform) {
                    problems.push(problem);
                }
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let keymap = Keymap::new(platform, overrides);
    let summon = Binding::parse(global_shortcut);
    for (i, action) in KeyAction::ALL.into_iter().enumerate() {
        for binding in keymap.bindings(action) {
            if summon.is_some_and(|s| s.same_as(binding, platform)) {
                problems.push(Problem::GlobalShortcut(action));
            }
            for other in KeyAction::ALL.into_iter().skip(i + 1) {
                if keymap
                    .bindings(other)
                    .iter()
                    .any(|b| b.same_as(binding, platform))
                {
                    // Said of the one the user changed (defaults never clash).
                    let (changed, kept) = if overrides.contains_key(other.id()) {
                        (other, action)
                    } else {
                        (action, other)
                    };
                    problems.push(Problem::Duplicate(changed, kept, *binding));
                }
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// One action as `keymap_info` describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionInfo {
    pub id: &'static str,
    pub title: &'static str,
    /// Its changeable binding's accelerator, or `null` for no key.
    pub binding: Option<String>,
    /// Its default bindings' accelerators (the first is what Reset brings back).
    pub defaults: Vec<String>,
}

impl Keymap {
    /// Every action, for Settings (LJ44).
    pub fn info(&self) -> Vec<ActionInfo> {
        KeyAction::ALL
            .into_iter()
            .map(|action| ActionInfo {
                id: action.id(),
                title: action.title(),
                binding: self.binding(action).map(Binding::accelerator),
                defaults: defaults(self.platform, action)
                    .iter()
                    .map(Binding::accelerator)
                    .collect(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
