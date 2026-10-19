//! The zoom keys (LJ16): keymap actions like every other page key.

use super::*;

const WIN: Platform = Platform::Windows;
const LINUX: Platform = Platform::Linux;
const MAC: Platform = Platform::MacOs;

const fn with(ctrl: bool, shift: bool, alt: bool, meta: bool) -> Mods {
    Mods {
        ctrl,
        shift,
        alt,
        meta,
    }
}

fn accelerators(platform: Platform, action: KeyAction) -> Vec<String> {
    defaults(platform, action)
        .iter()
        .map(Binding::accelerator)
        .collect()
}

fn overrides(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn zoom_keys_default_to_ctrl_plus_minus_and_zero_with_aliases() {
    for platform in [WIN, LINUX] {
        assert_eq!(
            accelerators(platform, KeyAction::ZoomIn),
            [
                "CommandOrControl+Equal",
                "CommandOrControl+Shift+Equal",
                "CommandOrControl+NumpadAdd"
            ]
        );
        assert_eq!(
            accelerators(platform, KeyAction::ZoomOut),
            ["CommandOrControl+Minus", "CommandOrControl+NumpadSubtract"]
        );
        assert_eq!(
            accelerators(platform, KeyAction::ZoomReset),
            ["CommandOrControl+Digit0", "CommandOrControl+Numpad0"]
        );
    }
    // macOS: one key each, the View menu's Zoom In, Zoom Out, Actual Size.
    assert_eq!(
        accelerators(MAC, KeyAction::ZoomIn),
        ["CommandOrControl+Equal"]
    );
    assert_eq!(
        accelerators(MAC, KeyAction::ZoomOut),
        ["CommandOrControl+Minus"]
    );
    assert_eq!(
        accelerators(MAC, KeyAction::ZoomReset),
        ["CommandOrControl+Digit0"]
    );
    assert_eq!(KeyAction::ZoomIn.title(), "Zoom In");
    assert_eq!(KeyAction::ZoomOut.title(), "Zoom Out");
    assert_eq!(KeyAction::ZoomReset.title(), "Actual Size");
    assert_eq!(KeyAction::ZoomIn.page_action(), PageAction::ZoomIn);
    assert_eq!(KeyAction::ZoomOut.page_action(), PageAction::ZoomOut);
    assert_eq!(KeyAction::ZoomReset.page_action(), PageAction::ZoomReset);
    for (action, id) in [
        (KeyAction::ZoomIn, "zoom-in"),
        (KeyAction::ZoomOut, "zoom-out"),
        (KeyAction::ZoomReset, "zoom-reset"),
    ] {
        assert_eq!(action.id(), id);
        assert_eq!(KeyAction::from_id(id), Some(action));
    }
}

#[test]
fn zoom_presses_match_exactly() {
    let ctrl = with(true, false, false, false);
    let ctrl_shift = with(true, true, false, false);
    let ctrl_alt = with(true, false, true, false);
    for platform in [WIN, LINUX] {
        let keymap = Keymap::new(platform, &BTreeMap::new());
        let at = |key, m| keymap.action_for(key, m);
        assert_eq!(at(Key::Equal, ctrl), Some(KeyAction::ZoomIn));
        assert_eq!(at(Key::Equal, ctrl_shift), Some(KeyAction::ZoomIn));
        assert_eq!(at(Key::NumpadAdd, ctrl), Some(KeyAction::ZoomIn));
        assert_eq!(at(Key::Minus, ctrl), Some(KeyAction::ZoomOut));
        assert_eq!(at(Key::NumpadSubtract, ctrl), Some(KeyAction::ZoomOut));
        assert_eq!(at(Key::Digit(0), ctrl), Some(KeyAction::ZoomReset));
        assert_eq!(at(Key::Numpad(0), ctrl), Some(KeyAction::ZoomReset));
        // Exact: Ctrl+Alt (AltGr) never matches, a plain key is typing, Ctrl+Shift+- isn't zoom.
        assert_eq!(at(Key::Equal, ctrl_alt), None);
        assert_eq!(at(Key::Digit(0), ctrl_alt), None);
        assert_eq!(at(Key::Equal, Mods::default()), None);
        assert_eq!(at(Key::Minus, ctrl_shift), None);
        assert_eq!(at(Key::Digit(0), ctrl_shift), None);
    }
    // GDK names Ctrl+Shift+= as "plus": the same key on a US layout.
    assert_eq!(Key::from_gdk_name("plus"), Key::Equal);
    assert_eq!(Key::from_gdk_name("KP_Add"), Key::NumpadAdd);
    assert_eq!(Key::from_gdk_name("KP_0"), Key::Numpad(0));
    // Windows: VK_OEM_PLUS (the = key), VK_OEM_MINUS, VK_ADD, VK_SUBTRACT, VK_NUMPAD0.
    assert_eq!(Key::from_windows_vk(0xBB), Key::Equal);
    assert_eq!(Key::from_windows_vk(0xBD), Key::Minus);
    assert_eq!(Key::from_windows_vk(0x6B), Key::NumpadAdd);
    assert_eq!(Key::from_windows_vk(0x6D), Key::NumpadSubtract);
    assert_eq!(Key::from_windows_vk(0x60), Key::Numpad(0));
    let mac = Keymap::new(MAC, &BTreeMap::new());
    let cmd = with(false, false, false, true);
    assert_eq!(mac.action_for(Key::Equal, cmd), Some(KeyAction::ZoomIn));
    assert_eq!(mac.action_for(Key::Minus, cmd), Some(KeyAction::ZoomOut));
    assert_eq!(
        mac.action_for(Key::Digit(0), cmd),
        Some(KeyAction::ZoomReset)
    );
    assert_eq!(mac.action_for(Key::Equal, ctrl), None);
}

#[test]
fn a_zoom_override_replaces_the_first_binding_and_drops_the_aliases() {
    let ctrl = with(true, false, false, false);
    let keymap = Keymap::new(
        WIN,
        &overrides(&[
            ("zoom-in", "CommandOrControl+Shift+KeyI"),
            ("zoom-reset", ""),
        ]),
    );
    assert_eq!(
        keymap.action_for(Key::Letter('i'), with(true, true, false, false)),
        Some(KeyAction::ZoomIn)
    );
    for old in [Key::Equal, Key::NumpadAdd] {
        assert_eq!(keymap.action_for(old, ctrl), None, "{old:?}");
    }
    assert_eq!(
        keymap.action_for(Key::Equal, with(true, true, false, false)),
        None
    );
    assert_eq!(keymap.bindings(KeyAction::ZoomIn).len(), 1);
    // "" is no key: Ctrl+0 and the keypad's 0 do nothing now; zoom-out keeps both of its.
    assert_eq!(keymap.binding(KeyAction::ZoomReset), None);
    assert_eq!(keymap.action_for(Key::Digit(0), ctrl), None);
    assert_eq!(keymap.action_for(Key::Numpad(0), ctrl), None);
    assert_eq!(keymap.bindings(KeyAction::ZoomOut).len(), 2);
    // The default written out is the default (its aliases stay).
    let same = Keymap::new(LINUX, &overrides(&[("zoom-in", "CommandOrControl+Equal")]));
    assert_eq!(same.bindings(KeyAction::ZoomIn).len(), 3);
}

#[test]
fn zoom_keys_follow_the_keymaps_rules() {
    // Two actions on one key: refused; so is the summon shortcut; and a plain = types text.
    let clash = validate(
        LINUX,
        &overrides(&[("history", "CommandOrControl+NumpadAdd")]),
        "CommandOrControl+Shift+Space",
    )
    .expect_err("Ctrl+keypad + is zoom in's");
    assert!(clash.iter().any(|p| matches!(
        p,
        Problem::Duplicate(KeyAction::History, KeyAction::ZoomIn, _)
    )));
    let summon = validate(
        WIN,
        &overrides(&[("zoom-out", "CommandOrControl+Alt+KeyZ")]),
        "CommandOrControl+Alt+KeyZ",
    )
    .expect_err("the summon shortcut");
    assert!(summon.contains(&Problem::GlobalShortcut(KeyAction::ZoomOut)));
    let typing = validate(MAC, &overrides(&[("zoom-in", "Equal")]), "").expect_err("a typing key");
    assert!(typing.contains(&Problem::TypingKey(KeyAction::ZoomIn)));
    assert!(validate(WIN, &BTreeMap::new(), "CommandOrControl+Shift+Space").is_ok());
    // keymap_info lists them, so Settings → Keyboard shows them.
    let info = Keymap::new(MAC, &BTreeMap::new()).info();
    let ids: Vec<&str> = info.iter().map(|a| a.id).collect();
    assert_eq!(
        ids,
        [
            "open-location",
            "history",
            "translate",
            "reload",
            "back",
            "forward",
            "zoom-in",
            "zoom-out",
            "zoom-reset",
            "open-in-browser",
            "close"
        ]
    );
    let reset = info
        .iter()
        .find(|a| a.id == "zoom-reset")
        .expect("zoom-reset");
    assert_eq!(reset.title, "Actual Size");
    assert_eq!(reset.binding.as_deref(), Some("CommandOrControl+Digit0"));
    // As Settings and the macOS menu log show them.
    let mac_in = Keymap::new(MAC, &BTreeMap::new());
    let shown = mac_in
        .binding(KeyAction::ZoomIn)
        .map(|b| b.display(MAC))
        .expect("a key");
    assert_eq!(shown, "⌘=");
    let win_out = Keymap::new(WIN, &BTreeMap::new());
    let shown = win_out
        .binding(KeyAction::ZoomOut)
        .map(|b| b.display(WIN))
        .expect("a key");
    assert_eq!(shown, "Ctrl+-");
}
