//! Every check of `route`: a good request is answered, and a request failing any single check
//! is refused.

use super::*;
use crate::extensions::registry::{self, Bundled};

const HELLO: &[u8] = br#"{ "id": "hello", "name": "Hello", "version": "1", "apiVersion": 1,
    "permissions": ["storage"],
    "contributes": { "commands": [{ "id": "open", "title": "Hello", "panel": "index.html" }] } }"#;
const OTHER: &[u8] = br#"{ "id": "other", "name": "Other", "version": "1", "apiVersion": 1,
    "contributes": { "commands": [{ "id": "open", "title": "Other", "panel": "index.html" }] } }"#;
static FIXTURE: Bundled = &[
    (
        "hello",
        &[
            ("extension.json", HELLO),
            ("index.html", b"<!doctype html>"),
        ],
    ),
    (
        "other",
        &[
            ("extension.json", OTHER),
            ("index.html", b"<!doctype html>"),
        ],
    ),
];

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

/// A good API call from hello's panel on macOS.
fn api(method: &str) -> Incoming {
    Incoming {
        webview: PAGE.to_owned(),
        method: "POST".to_owned(),
        uri: format!("ljext://hello.localhost/__lj/api/v1/{method}"),
        headers: vec![
            ("origin".to_owned(), "ljext://hello.localhost".to_owned()),
            ("x-lj-api".to_owned(), "1".to_owned()),
            ("content-type".to_owned(), "application/json".to_owned()),
        ],
    }
}

/// A good file request from hello's panel.
fn file(path: &str) -> Incoming {
    Incoming {
        webview: PAGE.to_owned(),
        method: "GET".to_owned(),
        uri: format!("ljext://hello.localhost{path}"),
        headers: Vec::new(),
    }
}

fn with_header(mut incoming: Incoming, name: &str, value: &str) -> Incoming {
    incoming.headers.retain(|(n, _)| n != name);
    incoming.headers.push((name.to_owned(), value.to_owned()));
    incoming
}

fn without_header(mut incoming: Incoming, name: &str) -> Incoming {
    incoming.headers.retain(|(n, _)| n != name);
    incoming
}

/// Decides `incoming` on `platform` with the page webview at `page` — the extension the guard
/// would say it shows after admitting that URL (none for anything else) — and `opened` the
/// extension the app opened last.
fn decide_on(
    incoming: &Incoming,
    platform: Platform,
    page: Option<&str>,
    opened: Option<&str>,
) -> Result<Route, Refusal> {
    let registry = registry::build(FIXTURE, None, &[]);
    let showing = page
        .map(url)
        .and_then(|page| crate::extensions::urls::parse(&page, platform))
        .map(|ext| ext.id);
    route(
        incoming,
        &Context {
            platform,
            showing: showing.as_deref(),
            registry: &registry,
            opened,
        },
    )
}

fn decide(incoming: &Incoming) -> Result<Route, Refusal> {
    decide_on(
        incoming,
        Platform::MacOs,
        Some("ljext://hello.localhost/index.html"),
        Some("hello"),
    )
}

fn refusal(result: Result<Route, Refusal>) -> Option<Refusal> {
    result.err()
}

#[test]
fn good_requests_are_answered() {
    assert!(matches!(
        decide(&api("storage.get")),
        Ok(Route::Api(_, Method::StorageGet))
    ));
    assert!(matches!(
        decide(&api("theme")),
        Ok(Route::Api(_, Method::Theme))
    ));
    assert!(matches!(
        decide(&file("/index.html")),
        Ok(Route::File(_, _))
    ));
    assert!(matches!(decide(&file(SDK_PATH)), Ok(Route::Sdk(_))));
    // The engines' own same-origin hints are fine.
    let hinted = with_header(api("storage.keys"), "sec-fetch-site", "same-origin");
    assert!(decide(&hinted).is_ok());
    let document = with_header(file("/index.html"), "sec-fetch-dest", "document");
    assert!(decide(&document).is_ok());
}

#[test]
fn only_the_page_webview_may_ask() {
    for webview in ["joan-bar", "settings", "", "joan-page2", "JOAN-PAGE"] {
        for incoming in [api("storage.get"), file("/index.html"), file(SDK_PATH)] {
            let incoming = Incoming {
                webview: webview.to_owned(),
                ..incoming
            };
            assert_eq!(
                refusal(decide(&incoming)),
                Some(Refusal::Webview),
                "{webview}"
            );
        }
    }
}

#[test]
fn only_known_extensions_are_served() {
    for uri in [
        "ljext://missing.localhost/index.html",
        "ljext://localhost/hello/index.html",
        "ljext://hello/index.html",
        "ljext://evil.com/index.html",
        "http://hello.localhost/index.html",
    ] {
        let incoming = Incoming {
            uri: uri.to_owned(),
            ..file("/index.html")
        };
        assert_eq!(refusal(decide(&incoming)), Some(Refusal::Unknown), "{uri}");
    }
}

#[test]
fn the_page_must_be_that_extension_opened_by_the_app() {
    for (page, opened) in [
        // A web page (or its frames) asking for an extension's files or API.
        (Some("https://example.com/"), Some("hello")),
        (Some("http://127.0.0.1:8123/smoke.html"), Some("hello")),
        // Another extension's panel.
        (Some("ljext://other.localhost/index.html"), Some("other")),
        // The start page, or nothing yet.
        (Some("tauri://localhost/index.html"), Some("hello")),
        (None, Some("hello")),
        // hello's URL, but the app opened another extension (or none).
        (Some("ljext://hello.localhost/index.html"), Some("other")),
        (Some("ljext://hello.localhost/index.html"), None),
        // The Windows form on macOS is not hello's page.
        (
            Some("http://ljext.hello.localhost/index.html"),
            Some("hello"),
        ),
    ] {
        for incoming in [api("storage.get"), file("/index.html"), file(SDK_PATH)] {
            assert_eq!(
                refusal(decide_on(&incoming, Platform::MacOs, page, opened)),
                Some(Refusal::Page),
                "{page:?} {opened:?} {}",
                incoming.uri
            );
        }
    }
}

#[test]
fn files_are_get_only_never_for_frames_and_only_plain_paths_of_known_types() {
    for method in ["POST", "PUT", "DELETE", "OPTIONS", "HEAD"] {
        let incoming = Incoming {
            method: method.to_owned(),
            ..file("/index.html")
        };
        assert_eq!(
            refusal(decide(&incoming)),
            Some(Refusal::HttpMethod),
            "{method}"
        );
    }
    for dest in ["iframe", "frame", "embed", "object", "fencedframe"] {
        let incoming = with_header(file("/index.html"), "sec-fetch-dest", dest);
        assert_eq!(refusal(decide(&incoming)), Some(Refusal::Frame), "{dest}");
    }
    for path in [
        "/",
        "/a/..%2findex.html",
        "/a%2f..%2fb.html",
        "/%5c..%5cindex.html",
        "/a\\..\\index.html",
        "/.hidden",
        "/__lj/sdk/v2.js",
        "/__lj/other",
        "/tool.exe",
        "/README",
    ] {
        assert_eq!(
            refusal(decide(&file(path))),
            Some(Refusal::NotFound),
            "{path}"
        );
    }
}

#[test]
fn dot_segments_resolve_inside_the_extension() {
    // URL parsing (the engine's, then ours) resolves `..` and `.` — also as %2e — before any
    // path is looked at, and never above the extension's root.
    for path in [
        "/../index.html",
        "/%2e%2e/index.html",
        "/a/../index.html",
        "/./index.html",
        "/.%2E/index.html",
    ] {
        match decide(&file(path)) {
            Ok(Route::File(_, rel)) => assert_eq!(rel.as_str(), "index.html", "{path}"),
            other => panic!("{path}: {other:?}"),
        }
    }
}

#[test]
fn the_api_is_post_only() {
    for method in ["GET", "PUT", "OPTIONS", "HEAD"] {
        let incoming = Incoming {
            method: method.to_owned(),
            ..api("storage.get")
        };
        assert_eq!(
            refusal(decide(&incoming)),
            Some(Refusal::HttpMethod),
            "{method}"
        );
    }
}

#[test]
fn the_api_needs_its_header_exactly_once() {
    assert_eq!(
        refusal(decide(&without_header(api("storage.get"), "x-lj-api"))),
        Some(Refusal::Header)
    );
    for value in ["0", "true", "", "1 "] {
        let incoming = with_header(api("storage.get"), "x-lj-api", value);
        assert_eq!(
            refusal(decide(&incoming)),
            Some(Refusal::Header),
            "{value:?}"
        );
    }
    let mut twice = api("storage.get");
    twice.headers.push(("X-LJ-Api".to_owned(), "1".to_owned()));
    assert_eq!(refusal(decide(&twice)), Some(Refusal::Header));
}

#[test]
fn the_origin_must_be_the_extensions_own() {
    for origin in [
        "null",
        "https://example.com",
        "http://127.0.0.1:8123",
        "ljext://other.localhost",
        "ljext://hello.localhost/",
        "ljext://hello.localhost:1",
        "LJEXT://hello.localhost",
        "http://ljext.hello.localhost",
        "tauri://localhost",
        "http://tauri.localhost",
    ] {
        let incoming = with_header(api("storage.get"), "origin", origin);
        assert_eq!(
            refusal(decide(&incoming)),
            Some(Refusal::Origin),
            "{origin}"
        );
    }
    let mut twice = api("storage.get");
    twice
        .headers
        .push(("Origin".to_owned(), "ljext://hello.localhost".to_owned()));
    assert_eq!(refusal(decide(&twice)), Some(Refusal::Origin));
}

#[test]
fn webkit_sends_no_origin_from_the_extension_itself() {
    // What CI saw on macOS and Linux: the panel's own POST carries no Origin and no Referer.
    let bare = without_header(api("storage.get"), "origin");
    for platform in [Platform::MacOs, Platform::Linux] {
        assert!(
            decide_on(
                &bare,
                platform,
                Some("ljext://hello.localhost/index.html"),
                Some("hello")
            )
            .is_ok(),
            "{platform:?}"
        );
        // A Referer, when sent, must be the extension's own.
        let own = with_header(
            bare.clone(),
            "referer",
            "ljext://hello.localhost/index.html",
        );
        assert!(
            decide_on(
                &own,
                platform,
                Some("ljext://hello.localhost/index.html"),
                Some("hello")
            )
            .is_ok()
        );
        for referer in [
            "https://example.com/",
            "ljext://other.localhost/index.html",
            "ljext://hello.localhost.evil.com/",
            "ljext://hello.localhostx/",
            "",
        ] {
            let other = with_header(bare.clone(), "referer", referer);
            assert_eq!(
                refusal(decide_on(
                    &other,
                    platform,
                    Some("ljext://hello.localhost/index.html"),
                    Some("hello")
                )),
                Some(Refusal::Origin),
                "{referer:?}"
            );
        }
        // An empty Origin header is not a missing one.
        let empty = with_header(bare.clone(), "origin", "");
        assert_eq!(
            refusal(decide_on(
                &empty,
                platform,
                Some("ljext://hello.localhost/index.html"),
                Some("hello")
            )),
            Some(Refusal::Origin)
        );
    }
    // WebView2 always sends one: without it, refused.
    assert_eq!(
        refusal(decide_on(
            &bare,
            Platform::Windows,
            Some("http://ljext.hello.localhost/index.html"),
            Some("hello")
        )),
        Some(Refusal::Origin)
    );
    // The other checks still apply to an Origin-less call: the page, the header, the webview.
    assert_eq!(
        refusal(decide_on(
            &bare,
            Platform::MacOs,
            Some("https://example.com/"),
            Some("hello")
        )),
        Some(Refusal::Page)
    );
    assert_eq!(
        refusal(decide(&without_header(bare.clone(), "x-lj-api"))),
        Some(Refusal::Header)
    );
}

#[test]
fn another_site_never_calls_the_api() {
    for site in ["cross-site", "same-site", "none", ""] {
        let incoming = with_header(api("storage.get"), "sec-fetch-site", site);
        assert_eq!(refusal(decide(&incoming)), Some(Refusal::Site), "{site:?}");
    }
}

#[test]
fn only_allowlisted_methods_exist() {
    for name in [
        "start_info",
        "storage",
        "storage.clear",
        "Storage.get",
        "eval",
        "theme/",
        "",
    ] {
        assert_eq!(
            refusal(decide(&api(name))),
            Some(Refusal::NoMethod),
            "{name:?}"
        );
    }
    // `..` resolves out of the API's path first; it is then a file, which is GET only.
    assert_eq!(refusal(decide(&api("../theme"))), Some(Refusal::HttpMethod));
    for method in Method::ALL {
        assert_eq!(Method::from_name(method.name()), Some(method));
    }
}

#[test]
fn methods_need_their_permission() {
    // hello declares storage only.
    for name in ["pageInfo", "clipboard.writeText"] {
        assert_eq!(
            refusal(decide(&api(name))),
            Some(Refusal::Permission),
            "{name}"
        );
    }
    for name in [
        "storage.get",
        "storage.set",
        "storage.remove",
        "storage.keys",
        "theme",
        "openUrl",
        "close",
    ] {
        assert!(decide(&api(name)).is_ok(), "{name}");
    }
    // other declares nothing: no storage.
    let other = Incoming {
        uri: "ljext://other.localhost/__lj/api/v1/storage.get".to_owned(),
        ..with_header(api("storage.get"), "origin", "ljext://other.localhost")
    };
    assert_eq!(
        refusal(decide_on(
            &other,
            Platform::Linux,
            Some("ljext://other.localhost/index.html"),
            Some("other")
        )),
        Some(Refusal::Permission)
    );
}

#[test]
fn windows_uses_its_own_page_url_and_origin() {
    let page = Some("http://ljext.hello.localhost/index.html");
    let windows = with_header(api("storage.get"), "origin", "http://ljext.hello.localhost");
    assert!(decide_on(&windows, Platform::Windows, page, Some("hello")).is_ok());
    assert!(decide_on(&file("/index.html"), Platform::Windows, page, Some("hello")).is_ok());
    // The macOS / Linux origin is not hello's origin on Windows, nor https.
    for origin in ["ljext://hello.localhost", "https://ljext.hello.localhost"] {
        let incoming = with_header(api("storage.get"), "origin", origin);
        assert_eq!(
            refusal(decide_on(&incoming, Platform::Windows, page, Some("hello"))),
            Some(Refusal::Origin),
            "{origin}"
        );
    }
    // A page in the macOS form doesn't count on Windows.
    assert_eq!(
        refusal(decide_on(
            &windows,
            Platform::Windows,
            Some("ljext://hello.localhost/index.html"),
            Some("hello")
        )),
        Some(Refusal::Page)
    );
}

#[test]
fn bodies_are_json_objects_of_limited_size() {
    assert_eq!(params(b"").map(|m| m.len()), Ok(0));
    assert_eq!(params(b" \n").map(|m| m.len()), Ok(0));
    assert_eq!(params(br#"{"key":"a"}"#).map(|m| m.len()), Ok(1));
    for bad in [&b"[]"[..], b"\"x\"", b"{", b"1"] {
        assert_eq!(params(bad), Err((400, "bad-request")));
    }
    let big = vec![b' '; MAX_BODY_BYTES + 1];
    assert_eq!(params(&big), Err((413, "too-large")));
}

#[test]
fn extensions_open_web_addresses_only() {
    for good in ["https://example.com/a?b=1", "http://127.0.0.1:8080/"] {
        assert!(openable(good).is_some(), "{good}");
    }
    for bad in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/",
        "https://ljext.other.localhost/",
        "data:text/html,hi",
        "tauri://localhost/bar.html",
        "example.com",
        "mailto:me@example.com",
    ] {
        assert!(openable(bad).is_none(), "{bad}");
    }
}

#[test]
fn switched_off_extensions_are_unknown() {
    let registry = registry::build(FIXTURE, None, &["hello".to_owned()]);
    for incoming in [api("storage.get"), file("/index.html"), file(SDK_PATH)] {
        let decided = route(
            &incoming,
            &Context {
                platform: Platform::MacOs,
                showing: Some("hello"),
                registry: &registry,
                opened: Some("hello"),
            },
        );
        assert_eq!(refusal(decided), Some(Refusal::Unknown), "{}", incoming.uri);
    }
}

#[test]
fn refusals_have_statuses_and_codes() {
    assert_eq!(Refusal::Webview.status(), 403);
    assert_eq!(Refusal::Unknown.status(), 404);
    assert_eq!(Refusal::HttpMethod.status(), 405);
    assert_eq!(Refusal::Origin.code(), "origin");
}
