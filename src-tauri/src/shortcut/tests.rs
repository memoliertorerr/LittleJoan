//! Tests for shortcut.rs: the swap with rollback, the launch's fallback, parsing.

use super::*;
use std::cell::RefCell;
use std::collections::BTreeSet;

/// A fake registrar: `taken` shortcuts belong to "another app".
#[derive(Default)]
struct Fake {
    registered: RefCell<BTreeSet<String>>,
    taken: BTreeSet<String>,
}

impl Registrar for Fake {
    fn register(&self, accelerator: &str) -> Result<(), String> {
        if self.taken.contains(accelerator) {
            return Err("taken".into());
        }
        self.registered.borrow_mut().insert(accelerator.to_owned());
        Ok(())
    }

    fn unregister(&self, accelerator: &str) -> Result<(), String> {
        self.registered.borrow_mut().remove(accelerator);
        Ok(())
    }
}

fn registered(fake: &Fake) -> Vec<String> {
    fake.registered.borrow().iter().cloned().collect()
}

#[test]
fn swap_replaces_the_old_shortcut() {
    let fake = Fake::default();
    fake.register("Alt+KeyJ").expect("free");
    assert_eq!(swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"), Ok(()));
    assert_eq!(registered(&fake), ["Alt+KeyK"]);
}

#[test]
fn swap_restores_the_old_shortcut_when_the_new_one_is_taken() {
    let fake = Fake {
        taken: BTreeSet::from(["Alt+KeyK".to_owned()]),
        ..Fake::default()
    };
    fake.register("Alt+KeyJ").expect("free");
    assert_eq!(
        swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"),
        Err(SwapError {
            message: TAKEN.to_owned(),
            old_kept: true
        })
    );
    assert_eq!(registered(&fake), ["Alt+KeyJ"]);
}

#[test]
fn swap_says_when_the_old_shortcut_couldnt_come_back() {
    // Another app took the old one in the moment it was free.
    let fake = Fake {
        taken: BTreeSet::from(["Alt+KeyK".to_owned(), "Alt+KeyJ".to_owned()]),
        ..Fake::default()
    };
    fake.registered.borrow_mut().insert("Alt+KeyJ".to_owned());
    assert_eq!(
        swap(&fake, Some("Alt+KeyJ"), "Alt+KeyK"),
        Err(SwapError {
            message: TAKEN.to_owned(),
            old_kept: false
        })
    );
    assert!(registered(&fake).is_empty());
    let none = Fake {
        taken: BTreeSet::from(["Alt+KeyK".to_owned()]),
        ..Fake::default()
    };
    assert_eq!(
        swap(&none, None, "Alt+KeyK").map_err(|err| err.old_kept),
        Err(false)
    );
}

#[test]
fn swap_rejects_unparseable_shortcuts_without_touching_anything() {
    let fake = Fake::default();
    fake.register("Alt+KeyJ").expect("free");
    assert_eq!(
        swap(&fake, Some("Alt+KeyJ"), "Ctrl+Nope"),
        Err(SwapError {
            message: INVALID.to_owned(),
            old_kept: true
        })
    );
    assert_eq!(registered(&fake), ["Alt+KeyJ"]);
}

#[test]
fn swap_to_the_same_shortcut_is_a_no_op() {
    let fake = Fake::default();
    fake.register("Alt+KeyJ").expect("free");
    assert_eq!(swap(&fake, Some("Alt+KeyJ"), "Alt+KeyJ"), Ok(()));
    assert_eq!(registered(&fake), ["Alt+KeyJ"]);
}

#[test]
fn swap_works_when_no_shortcut_was_active() {
    let fake = Fake::default();
    assert_eq!(swap(&fake, None, "Alt+KeyJ"), Ok(()));
    assert_eq!(registered(&fake), ["Alt+KeyJ"]);
}

#[test]
fn a_shortcut_that_fails_at_launch_falls_back_and_says_why() {
    let free = Fake::default();
    assert_eq!(
        settle(&free, "Alt+KeyJ"),
        (Some("Alt+KeyJ".to_owned()), None)
    );
    let taken = Fake {
        taken: BTreeSet::from(["Alt+KeyJ".to_owned()]),
        ..Fake::default()
    };
    assert_eq!(
        settle(&taken, "Alt+KeyJ"),
        (
            Some(DEFAULT_SHORTCUT.to_owned()),
            Some(Problem {
                wanted: "Alt+KeyJ".into(),
                active: Some(DEFAULT_SHORTCUT.into()),
                reason: Failure::Taken
            })
        )
    );
    assert_eq!(registered(&taken), [DEFAULT_SHORTCUT]);
    let invalid = settle(&Fake::default(), "Ctrl+Nope");
    assert_eq!(invalid.0.as_deref(), Some(DEFAULT_SHORTCUT));
    assert_eq!(invalid.1.map(|p| p.reason), Some(Failure::Invalid));
    let both = Fake {
        taken: BTreeSet::from(["Alt+KeyJ".to_owned(), DEFAULT_SHORTCUT.to_owned()]),
        ..Fake::default()
    };
    let (active, problem) = settle(&both, "Alt+KeyJ");
    assert_eq!(active, None, "no shortcut at all");
    assert_eq!(problem.and_then(|p| p.active), None);
    let json = serde_json::to_value(Problem {
        wanted: "Alt+KeyJ".into(),
        active: None,
        reason: Failure::Taken,
    })
    .expect("json");
    assert_eq!(
        json,
        serde_json::json!({ "wanted": "Alt+KeyJ", "active": null, "reason": "taken" })
    );
    assert_eq!(
        TAKEN,
        "Another app or the system already uses that shortcut. Try another one."
    );
}

#[test]
fn the_default_and_common_shortcuts_parse() {
    for accelerator in [
        DEFAULT_SHORTCUT,
        "Alt+Shift+KeyJ",
        "Control+Alt+Space",
        "Super+Digit1",
        "CommandOrControl+F12",
    ] {
        assert!(parse(accelerator).is_ok(), "{accelerator}");
    }
}

#[test]
fn malformed_shortcuts_are_rejected() {
    for accelerator in ["", "Shift+", "Ctrl+Shift+Nope", "Ctrl+A+B"] {
        assert!(parse(accelerator).is_err(), "{accelerator}");
    }
}
