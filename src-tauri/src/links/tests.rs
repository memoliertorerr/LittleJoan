use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

#[test]
fn web_links_open() {
    for s in [
        "https://example.com/",
        "http://example.com/a/b?c=d#e",
        "https://user@example.com:8443/x",
        "http://127.0.0.1:8123/smoke.html?link=os",
        "http://localhost:3000/",
        "https://xn--bcher-kva.example/",
    ] {
        assert_eq!(accept(&url(s)), Some(url(s)), "{s}");
        assert_eq!(parse(s), Ok(url(s)), "{s}");
    }
}

#[test]
fn only_http_and_https_open() {
    for (s, scheme) in [
        ("file:///etc/passwd", "file"),
        ("FILE:///C:/Windows/win.ini", "file"),
        ("javascript:alert(1)", "javascript"),
        ("data:text/html,<script>alert(1)</script>", "data"),
        ("about:blank", "about"),
        ("blob:https://example.com/0f8f", "blob"),
        ("ljext://hello/index.html", "ljext"),
        ("tauri://localhost/index.html", "tauri"),
        ("ftp://example.com/", "ftp"),
        ("mailto:a@example.com", "mailto"),
        ("vbscript:msgbox", "vbscript"),
        ("ms-settings:defaultapps", "other"),
        (
            "x-apple.systempreferences:com.apple.Desktop-Settings.extension",
            "other",
        ),
        ("littlejoanurl:https://example.com/", "other"),
    ] {
        assert_eq!(accept(&url(s)), None, "{s}");
        assert_eq!(refusal(&url(s)), Some(Refusal::Scheme(scheme)), "{s}");
        assert_eq!(parse(s), Err(Refusal::Scheme(scheme)), "{s}");
    }
}

#[test]
fn the_app_and_extensions_never_open_from_a_link() {
    for (s, why) in [
        (
            "http://ljext.hello.localhost/index.html",
            Refusal::Extension,
        ),
        ("https://LJEXT.hello.localhost/", Refusal::Extension),
        ("http://tauri.localhost/index.html", Refusal::App),
        ("http://ipc.localhost/open_input", Refusal::App),
        ("http://IPC.localhost/", Refusal::App),
    ] {
        assert_eq!(accept(&url(s)), None, "{s}");
        assert_eq!(refusal(&url(s)), Some(why), "{s}");
    }
    // The app's own origin on this platform (tauri://localhost elsewhere: a scheme refusal).
    let origin = keys::app_origin();
    assert_eq!(accept(&origin.join("index.html").expect("joins")), None);
}

#[test]
fn a_link_is_never_a_search_or_a_flag() {
    for s in [
        "",
        " ",
        "--quit",
        "--toggle",
        "example.com",
        "hello world",
        "//example.com/",
        "/etc/passwd",
    ] {
        assert_eq!(parse(s), Err(Refusal::Invalid), "{s:?}");
    }
    // A Windows path parses with the drive letter as its scheme: refused as a scheme.
    assert_eq!(parse("C:\\Windows\\win.ini"), Err(Refusal::Scheme("other")));
}

#[test]
fn long_links_are_refused() {
    let base = "https://example.com/?q=";
    let fits = format!("{base}{}", "a".repeat(MAX_LINK_BYTES - base.len()));
    assert_eq!(fits.len(), MAX_LINK_BYTES);
    assert!(parse(&fits).is_ok());
    let long = format!("{fits}a");
    assert_eq!(parse(&long), Err(Refusal::TooLong));
    assert_eq!(refusal(&url(&long)), Some(Refusal::TooLong));
    // Percent-encoding counts: the URL as it would load.
    let spaced = format!("{base}{}x", " ".repeat(MAX_LINK_BYTES / 2));
    assert_eq!(refusal(&url(&spaced)), Some(Refusal::TooLong));
}

#[test]
fn the_log_never_holds_the_link() {
    let secret = "https://example.com/?token=SECRET";
    for why in [
        Refusal::Invalid,
        Refusal::TooLong,
        Refusal::Scheme(scheme_name("secretscheme")),
        Refusal::Scheme(scheme_name("file")),
        Refusal::NoHost,
        Refusal::App,
        Refusal::Extension,
        Refusal::NotWeb,
    ] {
        let detail = why.detail();
        assert!(!detail.contains("SECRET") && !detail.contains("secretscheme"));
        assert!(
            detail
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == ' ' || c == '-'),
            "{detail}"
        );
    }
    assert_eq!(parse(secret), Ok(url(secret)));
    assert_eq!(scheme_name("secretscheme"), "other");
}

#[test]
fn from_a_batch_only_the_first_acceptable_link_opens() {
    let batch = pick(&[
        url("file:///etc/passwd"),
        url("https://a.example/"),
        url("javascript:alert(1)"),
        url("https://b.example/"),
        url("https://c.example/"),
    ]);
    assert_eq!(batch.open, Some(url("https://a.example/")));
    assert_eq!(
        batch.refused,
        [Refusal::Scheme("file"), Refusal::Scheme("javascript")]
    );
    assert_eq!(batch.ignored, 2);
    let none = pick(&[url("about:blank")]);
    assert_eq!(none.open, None);
    assert_eq!(none.ignored, 0);
    assert_eq!(
        pick(&[]),
        Batch {
            open: None,
            refused: Vec::new(),
            ignored: 0
        }
    );
}
