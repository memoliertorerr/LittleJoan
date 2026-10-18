use super::*;
use serde_json::json;

fn good() -> Value {
    json!({
        "id": "hello",
        "name": "Hello",
        "version": "1.0.0",
        "description": "Says hello.",
        "author": "Little Joan",
        "icon": "icon.svg",
        "apiVersion": 1,
        "permissions": ["storage", "network:api.example.com", "network:*.example.org"],
        "contributes": {
            "commands": [{ "id": "open", "title": "Say hello", "keywords": ["hi"], "panel": "index.html" }],
            "pageActions": [{ "id": "page", "title": "Hello for this page", "panel": "pages/page.html" }]
        }
    })
}

fn parse_value(value: &Value) -> Result<Manifest, String> {
    parse(value.to_string().as_bytes())
}

fn with(key: &str, value: Value) -> Value {
    let mut manifest = good();
    manifest[key] = value;
    manifest
}

#[test]
fn a_good_manifest_parses() {
    let manifest = parse_value(&good()).expect("valid");
    assert_eq!(manifest.id, "hello");
    assert_eq!(
        manifest.icon.as_ref().map(RelPath::as_str),
        Some("icon.svg")
    );
    assert!(manifest.has(&Permission::Storage));
    assert!(!manifest.has(&Permission::Clipboard));
    assert_eq!(
        manifest.network_hosts(),
        vec!["api.example.com", "*.example.org"]
    );
    assert_eq!(manifest.commands[0].keywords, vec!["hi"]);
    assert_eq!(manifest.page_actions[0].panel.as_str(), "pages/page.html");
    assert_eq!(manifest.named_files().len(), 3);
}

#[test]
fn ids_follow_the_pattern() {
    for id in [
        "hello",
        "2048",
        "clean-link",
        "a1",
        "abcdefghijklmnopqrstuvwxyz012345",
    ] {
        assert!(is_valid_id(id), "{id}");
    }
    for id in [
        "",
        "a",
        "-hello",
        "Hello",
        "he llo",
        "hello_world",
        "hello.world",
        "héllo",
        "abcdefghijklmnopqrstuvwxyz0123456",
    ] {
        assert!(!is_valid_id(id), "{id}");
        assert!(parse_value(&with("id", json!(id))).is_err(), "{id}");
    }
}

#[test]
fn only_api_version_1() {
    for version in [json!(2), json!(0), json!("1"), json!(1.5), Value::Null] {
        assert!(
            parse_value(&with("apiVersion", version.clone())).is_err(),
            "{version}"
        );
    }
    let mut manifest = good();
    manifest
        .as_object_mut()
        .expect("object")
        .remove("apiVersion");
    assert!(parse_value(&manifest).is_err());
}

#[test]
fn permissions_come_from_the_list() {
    for good_list in [
        json!([]),
        json!(["storage", "clipboard", "pageInfo"]),
        json!([
            "network:example.com",
            "network:*.cdn.example.com",
            "network:xn--bcher-kva.de"
        ]),
    ] {
        assert!(
            parse_value(&with("permissions", good_list.clone())).is_ok(),
            "{good_list}"
        );
    }
    for bad in [
        json!(["tabs"]),
        json!(["Storage"]),
        json!(["storage", "storage"]),
        json!(["network:*"]),
        json!(["network:*.com"]),
        json!(["network:com"]),
        json!(["network:localhost"]),
        json!(["network:127.0.0.1"]),
        json!(["network:example.com:8080"]),
        json!(["network:https://example.com"]),
        json!(["network:example.com/path"]),
        json!(["network:Example.com"]),
        json!(["network:-bad.example.com"]),
        json!(["network:*.*.example.com"]),
        json!(["network:"]),
        json!([1]),
        json!("storage"),
    ] {
        assert!(
            parse_value(&with("permissions", bad.clone())).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn panel_and_icon_paths_stay_inside_the_folder() {
    for panel in [
        "../index.html",
        "/etc/passwd.html",
        "a/../../b.html",
        "%2e%2e/x.html",
        "C:\\x.html",
        "__lj/sdk.html",
        ".hidden.html",
        "index.js",
        "",
    ] {
        let manifest = json!({
            "id": "hello", "name": "Hello", "version": "1", "apiVersion": 1,
            "contributes": { "commands": [{ "id": "a", "title": "A", "panel": panel }] }
        });
        assert!(parse_value(&manifest).is_err(), "{panel}");
    }
    for icon in ["../icon.svg", "icon.exe", "/abs/icon.png", "icon%2esvg"] {
        assert!(parse_value(&with("icon", json!(icon))).is_err(), "{icon}");
    }
    assert_eq!(
        parse_value(&with("icon", json!("")))
            .expect("no icon is fine")
            .icon,
        None
    );
}

#[test]
fn unknown_fields_and_bad_shapes_are_refused() {
    assert!(parse_value(&with("permisions", json!([]))).is_err());
    assert!(parse_value(&with("contributes", json!({ "comands": [] }))).is_err());
    assert!(parse_value(&with("contributes", json!([]))).is_err());
    assert!(parse_value(&with("$schema", json!("x"))).is_ok());
    let duplicate = json!({ "commands": [
        { "id": "a", "title": "A", "panel": "a.html" },
        { "id": "a", "title": "B", "panel": "b.html" }
    ]});
    assert!(parse_value(&with("contributes", duplicate)).is_err());
    let extra = json!({ "commands": [{ "id": "a", "title": "A", "panel": "a.html", "run": "x" }]});
    assert!(parse_value(&with("contributes", extra)).is_err());
    assert!(parse(b"not json").is_err());
    assert!(parse(b"[]").is_err());
    assert!(parse(&vec![b' '; MAX_MANIFEST_BYTES + 1]).is_err());
}

#[test]
fn texts_have_limits() {
    assert!(parse_value(&with("name", json!(""))).is_err());
    assert!(parse_value(&with("name", json!("x".repeat(41)))).is_err());
    assert!(parse_value(&with("name", json!("Line\nbreak"))).is_err());
    assert!(parse_value(&with("name", json!(42))).is_err());
    assert!(parse_value(&with("description", json!("d".repeat(201)))).is_err());
    assert!(parse_value(&with("version", json!("1.0 beta"))).is_err());
    assert_eq!(
        parse_value(&with("name", json!("  Hello  ")))
            .expect("trimmed")
            .name,
        "Hello"
    );
    // Markup is just text here; the bar draws it as text (never HTML).
    assert_eq!(
        parse_value(&with("name", json!("<img src=x onerror=alert(1)>")))
            .expect("text")
            .name,
        "<img src=x onerror=alert(1)>"
    );
    let many = json!({ "commands": [{ "id": "a", "title": "A", "panel": "a.html",
        "keywords": ["1", "2", "3", "4", "5", "6", "7", "8", "9"] }]});
    assert!(parse_value(&with("contributes", many)).is_err());
}

#[test]
fn ids_are_never_windows_device_names() {
    for id in ["con", "nul", "aux", "prn", "com1", "com9", "lpt1", "lpt9"] {
        assert!(!is_valid_id(id), "{id}");
        assert!(parse_value(&with("id", json!(id))).is_err(), "{id}");
    }
    for id in ["console", "com10", "lpt", "nulls", "aux-tools"] {
        assert!(is_valid_id(id), "{id}");
    }
}

#[test]
fn texts_refuse_invisible_and_bidi_characters() {
    for hidden in [
        "Hel\u{202E}lo",
        "Hel\u{2066}lo",
        "Hel\u{200F}lo",
        "Hel\u{200E}lo",
        "Hel\u{200B}lo",
        "Hel\u{FEFF}lo",
        "Hel\u{00AD}lo",
        "Hel\u{E0041}lo",
        "Hel\u{0007}lo",
    ] {
        assert!(
            parse_value(&with("name", json!(hidden))).is_err(),
            "{hidden:?}"
        );
        assert!(
            parse_value(&with("description", json!(hidden))).is_err(),
            "{hidden:?}"
        );
        assert!(
            parse_value(&with("author", json!(hidden))).is_err(),
            "{hidden:?}"
        );
        let titled = json!({ "commands": [{ "id": "a", "title": hidden, "panel": "a.html" }] });
        assert!(
            parse_value(&with("contributes", titled)).is_err(),
            "{hidden:?}"
        );
        let keyword = json!({ "commands": [{ "id": "a", "title": "A", "keywords": [hidden], "panel": "a.html" }] });
        assert!(
            parse_value(&with("contributes", keyword)).is_err(),
            "{hidden:?}"
        );
    }
    // Ordinary letters of any script are fine.
    assert!(parse_value(&with("name", json!("Héllo · 你好 · مرحبا"))).is_ok());
}

#[test]
fn names_never_read_as_web_addresses() {
    for address in [
        "paypal.com",
        "PayPal.com",
        "accounts.google.com",
        "www.example",
        "https://example.com",
        "ljext://hello.localhost",
        "127.0.0.1",
        "::1",
        "bank.co.uk.",
        "bücher.de",
    ] {
        assert!(looks_like_address(address), "{address}");
        assert!(
            parse_value(&with("name", json!(address))).is_err(),
            "{address}"
        );
    }
    for name in [
        "Hello",
        "QR code",
        "2048",
        "Clean link",
        "Notes 2.0",
        "v1.2",
        "Hello (unpacked)",
    ] {
        assert!(!looks_like_address(name), "{name}");
    }
}

#[test]
fn open_browser_is_a_permission_of_its_own() {
    let manifest = parse_value(&with("permissions", json!(["openBrowser"]))).expect("valid");
    assert!(manifest.has(&Permission::OpenBrowser));
    assert!(parse_value(&with("permissions", json!(["openbrowser"]))).is_err());
}
