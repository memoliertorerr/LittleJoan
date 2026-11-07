//! The find key (LJ49): a keymap action like every other page key.

use super::*;

const WIN: Platform = Platform::Windows;
const LINUX: Platform = Platform::Linux;
const MAC: Platform = Platform::MacOs;

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

fn overrides(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

#[test]
fn find_is_ctrl_f_and_cmd_f() {
    for platform in [WIN, LINUX, MAC] {
        let accelerators: Vec<String> = defaults(platform, KeyAction::Find)
            .iter()
            .map(Binding::accelerator)
            .collect();
        assert_eq!(accelerators, ["CommandOrControl+KeyF"], "{platform:?}");
    }
    // Windows catches it in AcceleratorKeyPressed (so WebView2's own find bar never opens), Linux
    // in the window's key hook, macOS through the View menu's Find….
    for (platform, mods) in [(WIN, CTRL), (LINUX, CTRL), (MAC, CMD)] {
        let keymap = Keymap::new(platform, &BTreeMap::new());
        assert_eq!(
            keymap.action_for(Key::Letter('f'), mods),
            Some(KeyAction::Find),
            "{platform:?}"
        );
    }
    let shift = Mods {
        shift: true,
        ..CTRL
    };
    assert_eq!(
        Keymap::new(WIN, &BTreeMap::new()).action_for(Key::Letter('f'), shift),
        None,
        "Ctrl+Shift+F is not Ctrl+F"
    );
    assert_eq!(KeyAction::Find.page_action(), PageAction::Find);
    assert_eq!(KeyAction::Find.id(), "find");
    assert_eq!(KeyAction::Find.title(), "Find…");
    assert_eq!(KeyAction::from_id("find"), Some(KeyAction::Find));
}

#[test]
fn find_can_be_changed_and_clashes_are_refused() {
    let keymap = Keymap::new(
        LINUX,
        &overrides(&[("find", "CommandOrControl+Shift+KeyF")]),
    );
    assert_eq!(keymap.action_for(Key::Letter('f'), CTRL), None);
    assert_eq!(
        keymap.action_for(
            Key::Letter('f'),
            Mods {
                shift: true,
                ..CTRL
            }
        ),
        Some(KeyAction::Find)
    );
    let none = Keymap::new(MAC, &overrides(&[("find", "")]));
    assert_eq!(none.binding(KeyAction::Find), None);
    // Another action on Ctrl+F is a clash Settings refuses.
    let clash = validate(WIN, &overrides(&[("history", "CommandOrControl+KeyF")]), "")
        .expect_err("a clash");
    assert!(clash.contains(&Problem::Duplicate(
        KeyAction::History,
        KeyAction::Find,
        defaults(WIN, KeyAction::Find)[0]
    )));
    // A key someone gave an older action before Find existed still does that action.
    let older = Keymap::new(WIN, &overrides(&[("translate", "CommandOrControl+KeyF")]));
    assert_eq!(
        older.action_for(Key::Letter('f'), CTRL),
        Some(KeyAction::Translate)
    );
    // Settings lists it, last.
    let info = Keymap::new(MAC, &BTreeMap::new()).info();
    let last = info.last().expect("actions");
    assert_eq!(last.id, "find");
    assert_eq!(last.binding.as_deref(), Some("CommandOrControl+KeyF"));
    let shown = Keymap::new(MAC, &BTreeMap::new())
        .binding(KeyAction::Find)
        .map(|b| b.display(MAC));
    assert_eq!(shown.as_deref(), Some("⌘F"));
}
