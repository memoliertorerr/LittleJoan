//! The `keys` field of settings.json (LJ43): the user's own keys for Little Joan's actions,
//! `{ "<action-id>": "<accelerator>" | "" }` — `""` means no key (keymap.rs has the actions, the
//! defaults and what an override does). Parsing is tolerant like the rest of the settings: an entry
//! for an unknown action, or whose accelerator doesn't parse, is dropped alone; the accelerators
//! kept are written the keymap's way (`CommandOrControl`, never the resolved modifier).

use std::collections::BTreeMap;

use serde_json::Value;

use crate::keymap::{Binding, KeyAction};

/// The entries of `value` that name an action and a key (or `""`), as the keymap writes them.
pub fn parse(value: Option<&Value>) -> BTreeMap<String, String> {
    let mut kept = BTreeMap::new();
    for (id, text) in value.and_then(Value::as_object).into_iter().flatten() {
        let (Some(action), Some(text)) = (KeyAction::from_id(id), text.as_str()) else {
            continue;
        };
        let text = text.trim();
        let accelerator = if text.is_empty() {
            Some(String::new())
        } else {
            Binding::parse(text).map(|binding| binding.accelerator())
        };
        if let Some(accelerator) = accelerator {
            kept.insert(action.id().to_owned(), accelerator);
        }
    }
    kept
}

/// What Settings sends, refused whole when it isn't an object of strings (each entry is then
/// checked by keymap::validate).
pub fn from_input(value: &Value) -> Option<BTreeMap<String, String>> {
    value
        .as_object()?
        .iter()
        .map(|(id, text)| Some((id.clone(), text.as_str()?.trim().to_owned())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entries_are_kept_one_by_one_in_the_keymaps_spelling() {
        let keys = parse(Some(&json!({
            "history": "cmdorctrl+shift+KeyK",
            "close": "",
            "reload": "Ctrl+Nope",
            "zoom": "Ctrl+KeyZ",
            "back": 3,
        })));
        assert_eq!(
            keys,
            BTreeMap::from([
                ("close".to_owned(), String::new()),
                (
                    "history".to_owned(),
                    "CommandOrControl+Shift+KeyK".to_owned()
                ),
            ])
        );
        assert!(parse(Some(&json!("history"))).is_empty());
        assert!(parse(None).is_empty());
    }

    #[test]
    fn input_must_be_an_object_of_strings() {
        assert_eq!(
            from_input(&json!({ "history": " Alt+KeyK " })),
            Some(BTreeMap::from([(
                "history".to_owned(),
                "Alt+KeyK".to_owned()
            )]))
        );
        assert_eq!(from_input(&json!({ "history": 1 })), None);
        assert_eq!(from_input(&json!(["history"])), None);
    }
}
