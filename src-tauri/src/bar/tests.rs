use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

#[test]
fn the_start_page_shows_an_empty_bar() {
    for (origin, start) in [
        ("tauri://localhost/", "tauri://localhost/index.html"),
        (
            "http://tauri.localhost/",
            "http://tauri.localhost/index.html",
        ),
    ] {
        assert_eq!(bar_text(&url(start), &url(origin)), None);
    }
    assert_eq!(
        bar_text(&url("about:blank"), &url("tauri://localhost/")),
        None
    );
}

#[test]
fn web_pages_show_their_full_address() {
    let origin = url("http://tauri.localhost/");
    assert_eq!(
        bar_text(&url("https://example.com/a?b=1#c"), &origin).as_deref(),
        Some("https://example.com/a?b=1#c")
    );
    assert_eq!(
        bar_text(&url("http://127.0.0.1:8123/smoke.html"), &origin).as_deref(),
        Some("http://127.0.0.1:8123/smoke.html")
    );
}
