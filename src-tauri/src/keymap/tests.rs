use super::*;

const WIN: Platform = Platform::Windows;
const LINUX: Platform = Platform::Linux;
const MAC: Platform = Platform::MacOs;

fn mods(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Mods {
    Mods {
        ctrl,
        shift,
        alt,
        meta,
    }
}

const CTRL: Mods = Mods {
    ctrl: true,
    shift: false,
    alt: false,
    meta: false,
};
const CMD: Mods = Mods {
    ctrl: false,
    shift: false,
    alt: false,
    meta: true,
};
const NONE: Mods = Mods {
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
};

fn overrides(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

fn accelerators(platform: Platform, action: KeyAction) -> Vec<String> {
    defaults(platform, action)
        .iter()
        .map(Binding::accelerator)
        .collect()
}

#[test]
fn the_defaults_are_todays_keys() {
    // Windows and Linux: Ctrl+W, Ctrl+L, Ctrl+Shift+T, Ctrl+H; Linux also Ctrl+R, F5, Alt+← / →.
    for platform in [WIN, LINUX] {
        assert_eq!(
            accelerators(platform, KeyAction::Close),
            ["CommandOrControl+KeyW"]
        );
        assert_eq!(
            accelerators(platform, KeyAction::OpenLocation),
            ["CommandOrControl+KeyL"]
        );
        assert_eq!(
            accelerators(platform, KeyAction::Translate),
            ["CommandOrControl+Shift+KeyT"]
        );
        assert_eq!(
            accelerators(platform, KeyAction::History),
            ["CommandOrControl+KeyH"]
        );
        assert!(accelerators(platform, KeyAction::OpenInBrowser).is_empty());
    }
    assert_eq!(
        accelerators(LINUX, KeyAction::Reload),
        ["CommandOrControl+KeyR", "F5"]
    );
    assert_eq!(accelerators(LINUX, KeyAction::Back), ["Alt+ArrowLeft"]);
    assert_eq!(accelerators(LINUX, KeyAction::Forward), ["Alt+ArrowRight"]);
    for action in [KeyAction::Reload, KeyAction::Back, KeyAction::Forward] {
        assert!(
            accelerators(WIN, action).is_empty(),
            "WebView2 handles {action:?} itself"
        );
    }
    // macOS: Cmd+W, Cmd+L, Cmd+Shift+T, Cmd+Shift+O, Cmd+R, Cmd+[, Cmd+], Cmd+Y.
    let mac: Vec<(KeyAction, Vec<String>)> = KeyAction::ALL
        .into_iter()
        .map(|a| (a, accelerators(MAC, a)))
        .collect();
    let want = [
        (KeyAction::OpenLocation, "CommandOrControl+KeyL"),
        (KeyAction::History, "CommandOrControl+KeyY"),
        (KeyAction::Translate, "CommandOrControl+Shift+KeyT"),
        (KeyAction::Reload, "CommandOrControl+KeyR"),
        (KeyAction::Back, "CommandOrControl+BracketLeft"),
        (KeyAction::Forward, "CommandOrControl+BracketRight"),
        (KeyAction::OpenInBrowser, "CommandOrControl+Shift+KeyO"),
        (KeyAction::Close, "CommandOrControl+KeyW"),
    ];
    for ((action, got), (want_action, want_key)) in mac.iter().zip(want) {
        assert_eq!(*action, want_action);
        assert_eq!(got, &[want_key.to_owned()], "{action:?}");
    }
}

#[test]
fn the_defaults_match_presses_as_before() {
    let ctrl_shift = mods(true, true, false, false);
    for platform in [WIN, LINUX] {
        let keymap = Keymap::new(platform, &BTreeMap::new());
        let at = |key, m| keymap.action_for(key, m);
        assert_eq!(at(Key::Letter('w'), CTRL), Some(KeyAction::Close));
        assert_eq!(at(Key::Letter('l'), CTRL), Some(KeyAction::OpenLocation));
        assert_eq!(at(Key::Letter('t'), ctrl_shift), Some(KeyAction::Translate));
        assert_eq!(at(Key::Letter('h'), CTRL), Some(KeyAction::History));
        assert_eq!(
            at(Key::Letter('y'), CTRL),
            None,
            "Ctrl+Y is the page's redo"
        );
        assert_eq!(at(Key::Letter('t'), CTRL), None);
    }
    let linux = Keymap::new(LINUX, &BTreeMap::new());
    assert_eq!(
        linux.action_for(Key::Letter('r'), CTRL),
        Some(KeyAction::Reload)
    );
    assert_eq!(linux.action_for(Key::F(5), NONE), Some(KeyAction::Reload));
    let alt = mods(false, false, true, false);
    assert_eq!(linux.action_for(Key::Left, alt), Some(KeyAction::Back));
    assert_eq!(linux.action_for(Key::Right, alt), Some(KeyAction::Forward));
    let win = Keymap::new(WIN, &BTreeMap::new());
    assert_eq!(win.action_for(Key::Letter('r'), CTRL), None);
    assert_eq!(win.action_for(Key::F(5), NONE), None);
    assert_eq!(win.action_for(Key::Left, alt), None);
    let mac = Keymap::new(MAC, &BTreeMap::new());
    assert_eq!(
        mac.action_for(Key::Letter('y'), CMD),
        Some(KeyAction::History)
    );
    assert_eq!(
        mac.action_for(Key::Letter('y'), CTRL),
        None,
        "macOS's primary is Cmd"
    );
}

#[test]
fn matching_is_exact() {
    for platform in [WIN, LINUX] {
        let keymap = Keymap::new(platform, &BTreeMap::new());
        let w = Key::Letter('w');
        assert_eq!(keymap.action_for(w, NONE), None, "plain W types a letter");
        assert_eq!(keymap.action_for(w, mods(true, true, false, false)), None);
        assert_eq!(
            keymap.action_for(w, mods(true, false, true, false)),
            None,
            "Ctrl+Alt is AltGr"
        );
        assert_eq!(keymap.action_for(w, mods(true, false, false, true)), None);
        assert_eq!(
            keymap.action_for(Key::Letter('l'), mods(true, true, false, false)),
            None,
            "Ctrl+Shift+L is not Ctrl+L"
        );
        assert_eq!(keymap.action_for(Key::Other, CTRL), None);
    }
}

#[test]
fn bindings_are_accelerators_with_codes() {
    let b = Binding::parse("CommandOrControl+KeyL").expect("parses");
    assert_eq!(b.key, Key::Letter('l'));
    assert!(b.primary && !b.shift);
    assert_eq!(b.mods(WIN), CTRL);
    assert_eq!(b.mods(LINUX), CTRL);
    assert_eq!(b.mods(MAC), CMD, "CommandOrControl is Cmd on macOS");
    let alt_left = Binding::parse("Alt+ArrowLeft").expect("parses");
    assert_eq!(alt_left.key, Key::Left);
    assert_eq!(alt_left.mods(LINUX), mods(false, false, true, false));
    assert_eq!(Binding::parse("F5").map(|b| b.key), Some(Key::F(5)));
    assert_eq!(
        Binding::parse(" cmdorctrl + shift + KeyK ").map(|b| b.accelerator()),
        Some("CommandOrControl+Shift+KeyK".to_owned()),
        "any case, spaces around; stored as CommandOrControl"
    );
    assert_eq!(
        Binding::parse("Ctrl+Alt+Shift+Super+Digit1").map(|b| b.accelerator()),
        Some("Control+Super+Alt+Shift+Digit1".to_owned())
    );
    for bad in [
        "",
        "Ctrl+",
        "Ctrl+Nope",
        "Ctrl+KeyA+KeyB",
        "Hyper+KeyA",
        "Ctrl+Ctrl+KeyA",
        "Ctrl+Space",
        "Ctrl+l",
        "+KeyA",
    ] {
        assert_eq!(Binding::parse(bad), None, "{bad:?}");
    }
    let mac_ctrl = Binding::parse("Control+KeyL").expect("parses");
    assert!(!mac_ctrl.same_as(&b, MAC), "⌃L is not ⌘L on macOS");
    assert!(mac_ctrl.same_as(&b, WIN), "but Ctrl+L elsewhere");
}

#[test]
fn bindings_display_like_settings() {
    let k = Binding::parse("CommandOrControl+Shift+KeyK").expect("parses");
    assert_eq!(k.display(MAC), "⌘⇧K");
    assert_eq!(k.display(WIN), "Ctrl+Shift+K");
    assert_eq!(
        Binding::parse("Alt+ArrowLeft")
            .expect("parses")
            .display(LINUX),
        "Alt+←"
    );
    assert_eq!(
        Binding::parse("CommandOrControl+BracketLeft")
            .expect("parses")
            .display(MAC),
        "⌘[",
        "as ui/lib/shortcut.ts shows it"
    );
    assert_eq!(
        Binding::parse("CommandOrControl+Digit1")
            .expect("parses")
            .display(MAC),
        "⌘1"
    );
}

#[test]
fn overrides_replace_the_first_binding_and_drop_aliases() {
    let keymap = Keymap::new(
        LINUX,
        &overrides(&[
            ("history", "CommandOrControl+Shift+KeyK"),
            ("reload", "CommandOrControl+Shift+KeyR"),
            ("close", ""),
            ("bogus", "CommandOrControl+KeyB"),
            ("translate", "Ctrl+Nope"),
        ]),
    );
    let ctrl_shift = mods(true, true, false, false);
    assert_eq!(
        keymap.action_for(Key::Letter('k'), ctrl_shift),
        Some(KeyAction::History)
    );
    assert_eq!(
        keymap.action_for(Key::Letter('h'), CTRL),
        None,
        "the old key is gone"
    );
    assert_eq!(
        keymap.action_for(Key::Letter('r'), ctrl_shift),
        Some(KeyAction::Reload)
    );
    assert_eq!(
        keymap.action_for(Key::F(5), NONE),
        None,
        "F5 stays only while reload has its default"
    );
    assert_eq!(keymap.binding(KeyAction::Close), None, "\"\" means no key");
    assert_eq!(keymap.action_for(Key::Letter('w'), CTRL), None);
    assert_eq!(
        keymap
            .binding(KeyAction::Translate)
            .map(Binding::accelerator),
        Some("CommandOrControl+Shift+KeyT".to_owned()),
        "an unparsable override is dropped"
    );
    // An override equal to the default keeps the aliases.
    let same = Keymap::new(LINUX, &overrides(&[("reload", "Ctrl+KeyR")]));
    assert_eq!(same.action_for(Key::F(5), NONE), Some(KeyAction::Reload));
    // A typing key from a hand-edited file is never bound.
    let typing = Keymap::new(WIN, &overrides(&[("history", "KeyK")]));
    assert_eq!(typing.action_for(Key::Letter('k'), NONE), None);
    assert_eq!(
        typing.action_for(Key::Letter('h'), CTRL),
        Some(KeyAction::History)
    );
}

#[test]
fn validation_refuses_clashes_and_typing_keys() {
    let ok = validate(
        WIN,
        &overrides(&[("history", "CommandOrControl+Shift+KeyK"), ("close", "")]),
        "CommandOrControl+Shift+Space",
    );
    assert_eq!(ok, Ok(()));
    // Two actions on one key.
    let dup = validate(
        WIN,
        &overrides(&[("history", "CommandOrControl+KeyL")]),
        "CommandOrControl+Shift+Space",
    );
    assert_eq!(
        dup,
        Err(vec![Problem::Duplicate(
            KeyAction::History,
            KeyAction::OpenLocation,
            Binding::parse("CommandOrControl+KeyL").expect("parses")
        )])
    );
    assert_eq!(
        dup.unwrap_err()[0].message(MAC),
        "History: ⌘L is already Open Location."
    );
    // An alias counts: F5 is Linux's reload.
    assert!(validate(LINUX, &overrides(&[("history", "F5")]), "Alt+KeyJ").is_err());
    assert!(validate(WIN, &overrides(&[("history", "F5")]), "Alt+KeyJ").is_ok());
    // The global shortcut.
    assert_eq!(
        validate(LINUX, &overrides(&[("history", "Alt+KeyJ")]), "Alt+KeyJ"),
        Err(vec![Problem::GlobalShortcut(KeyAction::History)])
    );
    assert_eq!(
        validate(LINUX, &BTreeMap::new(), "CommandOrControl+KeyL"),
        Err(vec![Problem::GlobalShortcut(KeyAction::OpenLocation)]),
        "a default key taken as the global shortcut"
    );
    // Typing keys: no modifier and not an F-key, or Shift alone with a key that types.
    for typing in [
        "KeyK",
        "Digit1",
        "Shift+KeyK",
        "Shift+Digit1",
        "Shift+Equal",
        "ArrowLeft",
    ] {
        assert_eq!(
            validate(WIN, &overrides(&[("history", typing)]), "Alt+KeyJ"),
            Err(vec![Problem::TypingKey(KeyAction::History)]),
            "{typing}"
        );
    }
    for fine in ["F6", "Shift+F6", "Alt+KeyK", "Shift+ArrowLeft"] {
        assert!(
            validate(WIN, &overrides(&[("history", fine)]), "Alt+KeyJ").is_ok(),
            "{fine}"
        );
    }
    // macOS: ⌘ or ⌃ needed.
    assert_eq!(
        validate(
            MAC,
            &overrides(&[("history", "Alt+Shift+KeyK")]),
            "Alt+KeyJ"
        ),
        Err(vec![Problem::NeedsCmdOrCtrl(KeyAction::History)])
    );
    assert!(validate(MAC, &overrides(&[("history", "Control+KeyK")]), "Alt+KeyJ").is_ok());
    assert!(validate(MAC, &overrides(&[("history", "F6")]), "Alt+KeyJ").is_err());
    // Unknown actions and unparsable accelerators.
    assert_eq!(
        validate(WIN, &overrides(&[("zoom", "Ctrl+KeyZ")]), "Alt+KeyJ"),
        Err(vec![Problem::UnknownAction("zoom".into())])
    );
    assert_eq!(
        validate(WIN, &overrides(&[("history", "Ctrl+Nope")]), "Alt+KeyJ"),
        Err(vec![Problem::Unparsable(
            KeyAction::History,
            "Ctrl+Nope".into()
        )])
    );
}

#[test]
fn defaults_never_clash() {
    for platform in [WIN, LINUX, MAC] {
        assert_eq!(
            validate(platform, &BTreeMap::new(), "CommandOrControl+Shift+Space"),
            Ok(()),
            "{platform:?}"
        );
        for action in KeyAction::ALL {
            for binding in defaults(platform, action) {
                assert_eq!(
                    bindable(action, &binding, platform),
                    Ok(()),
                    "{action:?} {binding:?}"
                );
            }
        }
    }
}

/// Every key the Settings recorder can produce (ui/lib/shortcut.ts' codes).
fn recordable_keys() -> Vec<Key> {
    let mut keys: Vec<Key> = ('a'..='z').map(Key::Letter).collect();
    keys.extend((0..=9).map(Key::Digit));
    keys.extend((1..=12).map(Key::F));
    keys.extend((0..=9).map(Key::Numpad));
    keys.extend([
        Key::Equal,
        Key::Minus,
        Key::BracketLeft,
        Key::BracketRight,
        Key::Comma,
        Key::Period,
        Key::Slash,
        Key::Semicolon,
        Key::Quote,
        Key::Backquote,
        Key::Backslash,
        Key::Left,
        Key::Right,
        Key::Up,
        Key::Down,
        Key::Home,
        Key::End,
        Key::PageUp,
        Key::PageDown,
        Key::NumpadAdd,
        Key::NumpadSubtract,
        Key::NumpadMultiply,
        Key::NumpadDivide,
        Key::NumpadDecimal,
    ]);
    keys
}

#[test]
fn every_default_and_recordable_binding_parses_as_a_menu_accelerator() {
    // Tauri's menu items parse accelerators with muda (and drop one that fails, silently); the
    // global shortcut's parser (global-hotkey) has the same modifier and key names for everything
    // `accelerator()` writes — checked in both crates' sources (muda 0.20.0 accelerator/mod.rs,
    // global-hotkey 0.8.0 hotkey.rs) —, and muda isn't a direct dependency, so this parses with
    // the one the app has.
    let mut all: Vec<Binding> = [WIN, LINUX, MAC]
        .into_iter()
        .flat_map(|p| KeyAction::ALL.into_iter().flat_map(move |a| defaults(p, a)))
        .collect();
    for key in recordable_keys() {
        for (primary, control, meta, alt, shift) in [
            (true, false, false, false, false),
            (true, false, false, false, true),
            (false, true, false, true, false),
            (false, false, true, false, true),
            (false, false, false, true, false),
        ] {
            all.push(Binding {
                primary,
                control,
                meta,
                alt,
                shift,
                key,
            });
        }
    }
    for binding in all {
        let text = binding.accelerator();
        assert!(
            crate::shortcut::parse(&text).is_ok(),
            "{text} doesn't parse as a menu accelerator"
        );
        assert_eq!(Binding::parse(&text), Some(binding), "{text} round-trips");
    }
}

#[test]
fn every_action_maps_to_its_page_action_and_back_by_id() {
    for action in KeyAction::ALL {
        assert_eq!(KeyAction::from_id(action.id()), Some(action));
        assert!(!action.title().is_empty());
    }
    assert_eq!(KeyAction::History.page_action(), PageAction::History);
    assert_eq!(KeyAction::Close.page_action(), PageAction::Close);
    assert_eq!(KeyAction::from_id("History"), None);
}

#[test]
fn info_lists_every_action_for_settings() {
    let keymap = Keymap::new(
        LINUX,
        &overrides(&[("history", "CommandOrControl+Shift+KeyK"), ("close", "")]),
    );
    let info = keymap.info();
    assert_eq!(info.len(), KeyAction::ALL.len());
    let history = info.iter().find(|a| a.id == "history").expect("history");
    assert_eq!(
        history.binding.as_deref(),
        Some("CommandOrControl+Shift+KeyK")
    );
    assert_eq!(history.defaults, ["CommandOrControl+KeyH"]);
    let close = info.iter().find(|a| a.id == "close").expect("close");
    assert_eq!(close.binding, None);
    assert_eq!(close.title, "Close Window");
    let reload = info.iter().find(|a| a.id == "reload").expect("reload");
    assert_eq!(reload.defaults, ["CommandOrControl+KeyR", "F5"]);
}
