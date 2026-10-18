use super::*;
use crate::extensions::manifest::Permission;

const HELLO: &[u8] = br#"{ "id": "hello", "name": "Hello", "version": "1", "apiVersion": 1,
    "icon": "icon.svg",
    "contributes": { "commands": [{ "id": "open", "title": "Say hello", "panel": "index.html" }] } }"#;
static GOOD: Bundled = &[(
    "hello",
    &[
        ("extension.json", HELLO),
        ("index.html", b"<!doctype html>"),
        ("icon.svg", b"<svg/>"),
    ],
)];

#[test]
fn the_shipped_extensions_are_all_valid() {
    let registry = build(embedded::BUNDLED, None, &[]);
    assert_eq!(
        registry.skipped,
        Vec::new(),
        "every bundled extension must load"
    );
    let hello = registry.get("hello").expect("hello is bundled");
    assert_eq!(hello.kind, Kind::Bundled);
    assert!(
        hello
            .read(&paths::clean("index.html").expect("clean"))
            .is_some()
    );
    assert!(
        hello
            .icon_data_uri()
            .is_some_and(|uri| uri.starts_with("data:image/svg+xml;base64,"))
    );
    // Every bundled extension: its icon and every panel it names are served, and it declares no
    // network host (bundled extensions are local tools).
    for extension in registry.all() {
        let id = extension.id();
        assert!(extension.icon_data_uri().is_some(), "{id} has an icon");
        let manifest = &extension.manifest;
        for item in manifest.commands.iter().chain(&manifest.page_actions) {
            assert!(
                extension.read(&item.panel).is_some(),
                "{id}: {}",
                item.panel.as_str()
            );
        }
        assert!(manifest.network_hosts().is_empty(), "{id} fetches nothing");
    }
    for id in [
        "snake",
        "2048",
        "notes",
        "focus",
        "clock",
        "qr",
        "clean-link",
        "wayback",
        "markdown-link",
        "json",
        "text-tools",
        "color",
    ] {
        assert!(registry.get(id).is_some(), "{id} is bundled");
    }
    // The page actions get the page; nothing bundled may hand an address to the default browser.
    for id in ["qr", "clean-link", "wayback", "markdown-link"] {
        let extension = registry.get(id).expect("bundled");
        assert!(
            !extension.manifest.page_actions.is_empty(),
            "{id} is a page action"
        );
        assert!(
            extension.manifest.has(&Permission::PageInfo),
            "{id} reads the page"
        );
    }
    assert!(
        registry
            .all()
            .iter()
            .all(|e| !e.manifest.has(&Permission::OpenBrowser)),
        "no bundled extension opens the default browser"
    );
}

#[test]
fn bundled_files_are_served_by_exact_path() {
    let registry = build(GOOD, None, &[]);
    let hello = registry.get("hello").expect("loaded");
    let read = |p: &str| hello.read(&paths::clean(p).expect("clean"));
    assert_eq!(read("index.html").as_deref(), Some(&b"<!doctype html>"[..]));
    assert!(read("Index.html").is_none());
    assert!(read("missing.js").is_none());
    assert_eq!(
        hello.icon_data_uri().as_deref(),
        Some("data:image/svg+xml;base64,PHN2Zy8+")
    );
}

#[test]
fn bad_bundled_extensions_are_skipped_with_a_reason() {
    static BAD: Bundled = &[
        ("nomanifest", &[("index.html", b"x")]),
        (
            "wrong-folder",
            &[
                ("extension.json", HELLO),
                ("index.html", b"x"),
                ("icon.svg", b"x"),
            ],
        ),
        ("hello", &[("extension.json", HELLO), ("icon.svg", b"x")]),
        ("broken", &[("extension.json", b"{")]),
    ];
    let registry = build(BAD, None, &[]);
    assert!(registry.all().is_empty());
    let names: Vec<&str> = registry.skipped.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["nomanifest", "wrong-folder", "hello", "broken"]);
    assert!(registry.skipped[2].reason.contains("index.html"));
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "little-joan-test-registry-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create");
    dir
}

fn unpacked(dir: &Path, folder: &str, manifest: &str, files: &[&str]) {
    let root = dir.join(folder);
    fs::create_dir_all(&root).expect("create");
    fs::write(root.join("extension.json"), manifest).expect("write");
    for file in files {
        fs::write(root.join(file), "x").expect("write");
    }
}

#[test]
fn developer_extensions_load_from_their_folders() {
    let dir = temp_dir("unpacked");
    let notes = r#"{ "id": "my-notes", "name": "Notes", "version": "0.1", "apiVersion": 1,
        "contributes": { "commands": [{ "id": "open", "title": "Notes", "panel": "index.html" }] } }"#;
    unpacked(&dir, "my-notes", notes, &["index.html"]);
    // Takes a bundled extension's id: skipped.
    unpacked(
        &dir,
        "hello",
        &String::from_utf8_lossy(HELLO),
        &["index.html", "icon.svg"],
    );
    // A panel that isn't there: skipped.
    unpacked(
        &dir,
        "my-notes-2",
        &notes.replace("my-notes", "my-notes-2"),
        &[],
    );
    fs::create_dir_all(dir.join(".git")).expect("hidden folders are ignored");
    let registry = build(GOOD, Some(&dir), &[]);
    let ids: Vec<&str> = registry.all().iter().map(|e| e.id()).collect();
    assert_eq!(ids, vec!["hello", "my-notes"]);
    assert_eq!(
        registry.get("my-notes").map(|e| e.kind),
        Some(Kind::Developer)
    );
    assert_eq!(registry.get("hello").map(|e| e.kind), Some(Kind::Bundled));
    let names: Vec<&str> = registry.skipped.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["hello", "my-notes-2"]);
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn developer_files_never_come_from_outside_their_folder() {
    use std::os::unix::fs::symlink;
    let dir = temp_dir("escape");
    let manifest = r#"{ "id": "leaky", "name": "Leaky", "version": "1", "apiVersion": 1,
        "contributes": { "commands": [{ "id": "open", "title": "Leaky", "panel": "index.html" }] } }"#;
    unpacked(&dir, "leaky", manifest, &["index.html"]);
    fs::write(dir.join("secret.txt"), "secret").expect("write");
    symlink(dir.join("secret.txt"), dir.join("leaky").join("secret.txt")).expect("symlink");
    let registry = build(GOOD, Some(&dir), &[]);
    let leaky = registry.get("leaky").expect("loaded");
    assert!(
        leaky
            .read(&paths::clean("index.html").expect("clean"))
            .is_some()
    );
    assert!(
        leaky
            .read(&paths::clean("secret.txt").expect("clean"))
            .is_none()
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn base64_matches_rfc_4648() {
    for (input, want) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ] {
        assert_eq!(base64(input.as_bytes()), want, "{input}");
    }
    assert_eq!(base64(&[0xff, 0xfe, 0xfd]), "//79");
}

#[test]
fn switched_off_extensions_are_listed_but_never_found() {
    let registry = build(GOOD, None, &["hello".to_owned(), "builtin:calc".to_owned()]);
    assert!(
        registry.get("hello").is_none(),
        "opening, serving and the API use get"
    );
    let hello = registry.find("hello").expect("still listed");
    assert!(!hello.enabled);
    assert_eq!(registry.all().len(), 1);
    assert!(
        build(GOOD, None, &[])
            .get("hello")
            .is_some_and(|e| e.enabled)
    );
}
