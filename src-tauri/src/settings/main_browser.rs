//! The `mainBrowser` field (LJ46): where Little Joan hands pages on — a browser's id as the OS
//! lists it (a macOS bundle id, a Windows StartMenuInternet key, a Linux desktop file id), or
//! `null` for the system default. Tolerant: anything that can't be an id is the system default.
//! It only chooses from the OS's own list (browsers.rs): an id is never run.

use serde_json::Value;

/// The longest id kept (characters).
pub const MAX_ID_CHARS: usize = 255;

/// Text that may name a browser: not empty, at most 255 characters, no control characters or
/// path separators, not starting with `-`.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id == id.trim()
        && id.chars().count() <= MAX_ID_CHARS
        && !id.starts_with('-')
        && !id.chars().any(|c| c.is_control() || c == '/' || c == '\\')
}

/// settings.json's value: a valid id, else `None` (the system default).
pub fn parse(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|id| is_valid_id(id))
        .map(str::to_owned)
}

/// Settings' choice: `""` = the system default, else a valid id; anything else is refused.
pub fn from_input(text: &str) -> Result<Option<String>, &'static str> {
    match text {
        "" => Ok(None),
        id if is_valid_id(id) => Ok(Some(id.to_owned())),
        _ => Err("Pick a browser from the list."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_as_each_os_lists_them_are_kept() {
        for id in [
            "com.apple.Safari",
            "Google Chrome",
            "Firefox-308046B0AF4A39CB",
            "firefox_firefox.desktop",
            "lj-fake-browser.desktop",
        ] {
            assert_eq!(parse(Some(&json!(id))), Some(id.to_owned()), "{id}");
            assert_eq!(from_input(id), Ok(Some(id.to_owned())), "{id}");
        }
    }

    #[test]
    fn anything_else_is_the_system_default() {
        for value in [
            json!(null),
            json!(""),
            json!(" padded "),
            json!(42),
            json!(["x"]),
            json!("--quit"),
            json!("../../bin/sh"),
            json!("C:\\x.exe"),
            json!("a\nb"),
            json!("x".repeat(MAX_ID_CHARS + 1)),
        ] {
            assert_eq!(parse(Some(&value)), None, "{value}");
        }
        assert_eq!(parse(None), None);
        assert_eq!(from_input(""), Ok(None));
        assert!(from_input("--quit").is_err());
        assert!(from_input("a/b").is_err());
    }
}
