//! The navigation gate, the nonce in panel URLs, the commit rule and the wait to leave a web
//! page (session.rs).

use std::time::Duration;

use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

#[test]
fn a_racing_document_cant_claim_the_expected_panel() {
    // The reviewer's case: while the app opens hello, a web page loops iframes (or a panel
    // navigates and stops) at hello's panel URL. Without the nonce, none is admitted, the
    // expectation survives, and the app's own navigation still is.
    let now = Instant::now();
    let mut gate = NavGate::new();
    let mine = url("ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdef");
    gate.expect(&mine, now);
    for racer in [
        "ljext://hello.localhost/index.html",
        "ljext://hello.localhost/index.html?lj-open=",
        "ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdee",
        "ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdef&x=1",
        "ljext://hello.localhost/index.html#lj-open=0123456789abcdef0123456789abcdef",
    ] {
        assert!(
            !page_may_navigate(&url(racer), &mut gate, now, Platform::MacOs),
            "{racer}"
        );
        assert_eq!(gate.showing(), None, "{racer}");
    }
    assert!(page_may_navigate(&mine, &mut gate, now, Platform::MacOs));
    assert_eq!(gate.showing(), Some("hello"));
    // Used up: the same URL again (a reload, a back) isn't admitted.
    assert!(!page_may_navigate(&mine, &mut gate, now, Platform::MacOs));
}

#[test]
fn the_toolbar_follows_what_a_load_event_committed() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    let panel = url("ljext://hello.localhost/index.html?lj-open=00");
    // Load events of a page that is not the admitted panel commit nothing.
    assert!(!gate.commit(&url("https://example.com/"), Platform::MacOs));
    assert!(
        !gate.commit(&panel, Platform::MacOs),
        "not admitted: no extension"
    );
    assert_eq!(gate.committed(), None);
    gate.expect(&panel, now);
    assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
    assert_eq!(gate.committed(), None, "admitted is not committed");
    // A late event of the page before (the race CI saw) doesn't commit the panel either.
    assert!(!gate.commit(&url("tauri://localhost/index.html"), Platform::MacOs));
    assert!(gate.commit(&panel, Platform::MacOs));
    assert_eq!(gate.committed(), Some("hello"));
    // Leaving: the guard clears what it admitted, and the next load event clears the toolbar.
    assert!(page_may_navigate(
        &url("https://example.com/"),
        &mut gate,
        now,
        Platform::MacOs
    ));
    assert!(gate.commit(&url("https://example.com/"), Platform::MacOs));
    assert_eq!(gate.committed(), None);
    // Another extension's URL never commits as the admitted one.
    gate.expect(&panel, now);
    assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
    assert!(!gate.commit(&url("ljext://other.localhost/index.html"), Platform::MacOs));
    assert_eq!(gate.committed(), None);
}

#[test]
fn panel_urls_carry_the_nonce() {
    let panel = panel_url("hello", "index.html", "abc").expect("a URL");
    assert!(panel.as_str().ends_with("/index.html?lj-open=abc"));
    assert_eq!(
        urls::parse(&panel, Platform::current())
            .map(|e| e.path)
            .as_deref(),
        Some("/index.html")
    );
}

#[test]
fn only_the_expected_extension_url_is_admitted_once() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    let panel = url("ljext://hello.localhost/index.html");
    assert!(!gate.admit(&panel, now), "nothing expected yet");
    gate.expect(&panel, now);
    assert!(!gate.admit(&url("ljext://other.localhost/index.html"), now));
    assert!(!gate.admit(&url("ljext://hello.localhost/other.html"), now));
    assert!(gate.admit(&panel, now), "the expected one, still expected");
    assert!(!gate.admit(&panel, now), "only once");
}

#[test]
fn an_expected_url_goes_stale() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    let panel = url("http://ljext.hello.localhost/index.html");
    gate.expect(&panel, now);
    assert!(!gate.admit(&panel, now + EXPECT_FOR + Duration::from_millis(1)));
    gate.expect(&panel, now);
    assert!(gate.admit(&panel, now + EXPECT_FOR));
}

#[test]
fn web_pages_can_never_open_extension_urls() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    for s in [
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/index.html",
        "https://ljext.hello.localhost/index.html",
        "http://ljext.localhost/hello/index.html",
        "http://ljext.evil.example.com/",
    ] {
        assert!(
            !page_may_navigate(&url(s), &mut gate, now, Platform::Windows),
            "{s}"
        );
    }
    // The expected URL in one form doesn't admit another form.
    gate.expect(&url("http://ljext.hello.localhost/index.html"), now);
    assert!(!page_may_navigate(
        &url("https://ljext.hello.localhost/index.html"),
        &mut gate,
        now,
        Platform::Windows
    ));
    assert!(page_may_navigate(
        &url("http://ljext.hello.localhost/index.html"),
        &mut gate,
        now,
        Platform::Windows
    ));
}

#[test]
fn the_guard_decides_which_extension_the_page_shows() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    let panel = url("ljext://hello.localhost/index.html");
    gate.expect(&panel, now);
    assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
    assert_eq!(gate.showing(), Some("hello"));
    // Refused navigations change nothing.
    for refused in [
        "file:///etc/passwd",
        "ljext://other.localhost/index.html",
        "blob:ljext://hello.localhost/1",
        "data:text/html,hi",
    ] {
        assert!(
            !page_may_navigate(&url(refused), &mut gate, now, Platform::MacOs),
            "{refused}"
        );
    }
    assert_eq!(gate.showing(), Some("hello"));
    // Leaving for anything else the guard allows: no extension any more.
    assert!(page_may_navigate(
        &url("https://example.com/"),
        &mut gate,
        now,
        Platform::MacOs
    ));
    assert_eq!(gate.showing(), None);
    // The start page (the command bar, `close`) is not a panel either.
    gate.expect(&panel, now);
    assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
    assert!(page_may_navigate(
        &url("tauri://localhost/index.html"),
        &mut gate,
        now,
        Platform::MacOs
    ));
    assert_eq!(gate.showing(), None);
    // Another platform's form, even if expected, shows no extension on this one.
    let windows_form = url("http://ljext.hello.localhost/index.html");
    gate.expect(&windows_form, now);
    assert!(page_may_navigate(
        &windows_form,
        &mut gate,
        now,
        Platform::MacOs
    ));
    assert_eq!(gate.showing(), None);
}

#[test]
fn web_rules_still_apply_and_panels_get_no_blob_or_data_documents() {
    let now = Instant::now();
    for (s, on_web, on_panel) in [
        ("https://example.com/", true, true),
        ("about:blank", true, true),
        ("blob:https://example.com/0f8f", true, false),
        ("data:text/html,hi", true, false),
        ("file:///etc/passwd", false, false),
        ("javascript:alert(1)", false, false),
        ("tauri://localhost/index.html", true, true),
    ] {
        let mut web = NavGate::new();
        assert_eq!(
            page_may_navigate(&url(s), &mut web, now, Platform::Linux),
            on_web,
            "{s} on the web"
        );
        let mut shown = NavGate::new();
        let opened = url("ljext://hello.localhost/index.html");
        shown.expect(&opened, now);
        assert!(page_may_navigate(&opened, &mut shown, now, Platform::Linux));
        assert_eq!(
            page_may_navigate(&url(s), &mut shown, now, Platform::Linux),
            on_panel,
            "{s} on a panel"
        );
    }
}

#[test]
fn an_opening_waits_for_the_page_to_leave_its_web_page() {
    let mut leaving = Leaving::new();
    assert_eq!(leaving.left(), None, "nothing waits");
    let first = leaving.wait(url("ljext://hello.localhost/index.html?lj-open=1"));
    assert_eq!(
        leaving.left(),
        Some(url("ljext://hello.localhost/index.html?lj-open=1")),
        "the blank page committed: open it"
    );
    assert_eq!(
        leaving.timed_out(first),
        None,
        "already opened: the timer does nothing"
    );
    // A second opening replaces the first; the first one's timer can't open it early.
    let second = leaving.wait(url("ljext://hello.localhost/index.html?lj-open=2"));
    let third = leaving.wait(url("ljext://other.localhost/index.html?lj-open=3"));
    assert_eq!(leaving.timed_out(second), None);
    assert_eq!(
        leaving.timed_out(third),
        Some(url("ljext://other.localhost/index.html?lj-open=3"))
    );
    assert_eq!(leaving.left(), None);
}

#[test]
fn a_new_opening_is_uncommitted_until_its_own_commit() {
    let now = Instant::now();
    let mut gate = NavGate::new();
    let first = url("ljext://hello.localhost/index.html?lj-open=11111111111111111111111111111111");
    gate.expect(&first, now);
    assert!(page_may_navigate(&first, &mut gate, now, Platform::MacOs));
    gate.commit(&first, Platform::MacOs);
    assert_eq!(gate.committed(), Some("hello"));
    // Reload opens the same extension again: admitted, but not committed until its commit event —
    // the old panel's (or a late) commit for the same extension no longer counts for it.
    let again = url("ljext://hello.localhost/index.html?lj-open=22222222222222222222222222222222");
    gate.expect(&again, now);
    assert!(page_may_navigate(&again, &mut gate, now, Platform::MacOs));
    assert_eq!(gate.showing(), Some("hello"));
    assert_eq!(gate.committed(), None);
    // A late commit event of the first opening (its nonce) doesn't commit the second.
    gate.commit(&first, Platform::MacOs);
    assert_eq!(gate.committed(), None);
    gate.commit(&url("ljext://hello.localhost/index.html"), Platform::MacOs);
    assert_eq!(gate.committed(), None, "no nonce: not the admitted URL");
    gate.commit(&again, Platform::MacOs);
    assert_eq!(gate.committed(), Some("hello"));
    // A refused extension URL changes nothing.
    assert!(!page_may_navigate(&first, &mut gate, now, Platform::MacOs));
    assert_eq!(gate.committed(), Some("hello"));
}

#[test]
fn an_opening_waits_for_the_bar_then_opens() {
    // LJ38: a panel opened with a new window waits until the bar's script runs (start_info).
    let ready = std::sync::Arc::new(crate::bar::Ready::new());
    let marker = std::sync::Arc::clone(&ready);
    let started = Instant::now();
    let setter = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(60));
        marker.set();
    });
    let mut opened = None;
    after_bar(&ready, BAR_WAIT, |was_ready| opened = Some(was_ready));
    setter.join().expect("the setter thread");
    assert_eq!(opened, Some(true));
    let waited = started.elapsed();
    assert!(waited >= Duration::from_millis(50), "{waited:?}");
    assert!(waited < BAR_WAIT, "{waited:?}");
}

#[test]
fn an_opening_goes_on_without_the_bar_after_the_limit() {
    let ready = crate::bar::Ready::new();
    let started = Instant::now();
    let mut opened = None;
    after_bar(&ready, Duration::from_millis(80), |was_ready| {
        opened = Some(was_ready);
    });
    // It opens anyway, saying the bar wasn't ready (the glue logs `bar-ready timeout`).
    assert_eq!(opened, Some(false));
    assert!(started.elapsed() >= Duration::from_millis(80));
    assert_eq!(BAR_WAIT, Duration::from_secs(2));
}

#[test]
fn with_the_bar_ready_an_opening_goes_at_once() {
    let ready = crate::bar::Ready::new();
    ready.set();
    let started = Instant::now();
    let mut opened = None;
    after_bar(&ready, BAR_WAIT, |was_ready| opened = Some(was_ready));
    assert_eq!(opened, Some(true));
    assert!(started.elapsed() < Duration::from_millis(50));
}
