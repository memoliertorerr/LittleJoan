//! Ground Rule 18: the page webview (`joan-page`) has no IPC at all. Tauri treats custom-scheme
//! pages as local, so extensions (which will run there as `ljext:` pages) would get every local
//! capability that reaches the page webview — by its label, or through a window-level scope that
//! covers the page window (it applies to every webview in it). These tests read every capability
//! the build can see: the files under `capabilities/` (tauri-build takes `capabilities/**/*`)
//! and inline ones in tauri.conf.json.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{bar, joan, settings_window};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every file under `dir`, recursively.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).expect("read the capabilities folder") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            found.extend(files(&path));
        } else {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// The capabilities in one file: a single capability, a list, or `{ "capabilities": [...] }`
/// (the formats tauri-utils accepts).
fn capabilities_in(value: Value) -> Vec<Value> {
    match value {
        Value::Array(list) => list,
        Value::Object(mut map) if map.contains_key("capabilities") => {
            match map.remove("capabilities") {
                Some(Value::Array(list)) => list,
                other => panic!("`capabilities` must be a list, got {other:?}"),
            }
        }
        single => vec![single],
    }
}

/// `(where, capability)` for every capability file and every inline capability in the config.
fn all_capabilities() -> Vec<(String, Value)> {
    let mut all = Vec::new();
    for path in files(&manifest_dir().join("capabilities")) {
        let name = path.display().to_string();
        // tauri-build also reads TOML (and JSON5); only JSON is checked here, so only JSON may
        // be there.
        assert_eq!(
            path.extension().and_then(|e| e.to_str()),
            Some("json"),
            "{name}: capability files must be JSON so this test can check them"
        );
        let text = fs::read_to_string(&path).expect("read a capability file");
        let value: Value = serde_json::from_str(&text).expect("a capability file is JSON");
        for capability in capabilities_in(value) {
            all.push((name.clone(), capability));
        }
    }
    let config: Value = serde_json::from_str(
        &fs::read_to_string(manifest_dir().join("tauri.conf.json")).expect("read tauri.conf.json"),
    )
    .expect("tauri.conf.json is JSON");
    if let Some(Value::Array(entries)) = config.pointer("/app/security/capabilities") {
        for entry in entries {
            // A string names a file's capability (checked above); an object is inline.
            if entry.is_object() {
                all.push(("tauri.conf.json".to_owned(), entry.clone()));
            }
        }
    }
    all
}

/// Whether a capability's window or webview pattern matches `label`. Tauri matches them as glob
/// patterns (the `glob` crate): `*` and `?` are handled here; character classes are refused, so
/// no pattern can slip past this check.
fn pattern_matches(pattern: &str, label: &str) -> bool {
    assert!(
        !pattern.contains('['),
        "{pattern:?}: character classes in capability scopes aren't allowed (this test can't check them)"
    );
    fn go(p: &[char], l: &[char]) -> bool {
        match p.split_first() {
            None => l.is_empty(),
            Some(('*', rest)) => (0..=l.len()).any(|skip| go(rest, &l[skip..])),
            Some(('?', rest)) => !l.is_empty() && go(rest, &l[1..]),
            Some((c, rest)) => l.first() == Some(c) && go(rest, &l[1..]),
        }
    }
    let p: Vec<char> = pattern.chars().collect();
    let l: Vec<char> = label.chars().collect();
    go(&p, &l)
}

fn scopes<'a>(capability: &'a Value, key: &str) -> Vec<&'a str> {
    match capability.get(key) {
        None => Vec::new(),
        Some(Value::Array(list)) => list
            .iter()
            .map(|v| v.as_str().expect("scope entries are strings"))
            .collect(),
        Some(other) => panic!("`{key}` must be a list, got {other}"),
    }
}

#[test]
fn the_page_webview_has_no_capability() {
    let all = all_capabilities();
    assert!(!all.is_empty(), "no capabilities found at all");
    for (place, capability) in &all {
        for pattern in scopes(capability, "webviews") {
            assert!(
                !pattern_matches(pattern, bar::PAGE),
                "{place}: {pattern:?} grants the page webview ({}) IPC",
                bar::PAGE
            );
        }
        // A window-level scope covers every webview in that window, the page included.
        for pattern in scopes(capability, "windows") {
            assert!(
                !pattern_matches(pattern, joan::LABEL),
                "{place}: {pattern:?} is a window scope over the page window ({}), which holds the page webview",
                joan::LABEL
            );
        }
    }
}

#[test]
fn no_capability_reaches_remote_urls() {
    for (place, capability) in all_capabilities() {
        assert!(
            capability.get("remote").is_none(),
            "{place}: remote pages never get IPC (Ground Rule 10)"
        );
    }
}

#[test]
fn the_bar_and_settings_keep_their_own_capabilities() {
    let all = all_capabilities();
    let reaches = |label: &str, key: &str| {
        all.iter()
            .any(|(_, c)| scopes(c, key).iter().any(|p| pattern_matches(p, label)))
    };
    assert!(reaches(bar::BAR, "webviews"), "the bar lost its capability");
    assert!(
        reaches(settings_window::LABEL, "windows"),
        "the settings window lost its capability"
    );
}

/// The permission identifiers a capability lists (strings, or objects with an `identifier`).
fn permissions(capability: &Value) -> Vec<&str> {
    match capability.get("permissions") {
        None => Vec::new(),
        Some(Value::Array(list)) => list
            .iter()
            .map(|p| {
                p.as_str()
                    .or_else(|| p.get("identifier").and_then(Value::as_str))
                    .expect("a permission is a string or has an identifier")
            })
            .collect(),
        Some(other) => panic!("`permissions` must be a list, got {other}"),
    }
}

#[test]
fn only_the_bar_may_toggle_maximize() {
    // A double click on the toolbar's drag region zooms the window (Tauri's drag script calls
    // internal_toggle_maximize, LJ37). Only the bar gets it — it, or a default set that holds it.
    let grants = |p: &str| {
        matches!(
            p,
            "core:window:allow-internal-toggle-maximize" | "core:window:default" | "core:default"
        )
    };
    let mut bar_has_it = false;
    for (place, capability) in all_capabilities() {
        if !permissions(&capability).into_iter().any(grants) {
            continue;
        }
        let webviews = scopes(&capability, "webviews");
        let only_the_bar = capability.get("identifier").and_then(Value::as_str) == Some("bar")
            && webviews == [bar::BAR]
            && scopes(&capability, "windows").is_empty();
        assert!(
            only_the_bar,
            "{place}: only the bar's capability may toggle maximize"
        );
        bar_has_it = true;
    }
    assert!(
        bar_has_it,
        "the bar lost core:window:allow-internal-toggle-maximize"
    );
}

#[test]
fn no_capability_is_added_at_runtime() {
    // A capability added in code (Manager::add_capability) would escape the checks above.
    for path in files(&manifest_dir().join("src")) {
        if path.file_name().and_then(|n| n.to_str()) == Some("capability_tests.rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_default();
        assert!(
            !text.contains("add_capability"),
            "{}: capabilities are only declared in files",
            path.display()
        );
    }
}

#[test]
fn the_pattern_matcher_matches_like_a_glob() {
    assert!(pattern_matches("joan-page", "joan-page"));
    assert!(pattern_matches("*", "joan-page"));
    assert!(pattern_matches("joan-*", "joan-page"));
    assert!(pattern_matches("joan?page", "joan-page"));
    assert!(pattern_matches("*page", "joan-page"));
    assert!(!pattern_matches("joan-bar", "joan-page"));
    assert!(!pattern_matches("joan", "joan-page"));
    assert!(!pattern_matches("settings", "joan"));
    assert!(pattern_matches("jo*", "joan"));
}
