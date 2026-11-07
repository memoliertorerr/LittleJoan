use super::*;

fn url(s: &str) -> Url {
    Url::parse(s).expect("test URL parses")
}

fn web(s: &str) -> Opener {
    Opener::of(Some(&url(s)))
}

const SIGNIN: &str = "http://127.0.0.1:8124/signin.html";
const SIZED: Size = Size::Given(480.0, 600.0);

fn open(open: usize) -> Counts {
    Counts { open, opened: 0 }
}

fn decided(opener: &Opener, target: &str, size: Size, now: usize) -> Decision {
    decide(opener, &url(target), size, open(now), || true).decision
}

#[test]
fn labels_are_numbered_and_recognised() {
    assert_eq!(label(1), "joan-popup-1");
    assert_eq!(label(42), "joan-popup-42");
    assert!(is_popup_label(&label(7)));
    for other in [
        "joan",
        "joan-page",
        "joan-bar",
        "settings",
        "joan-popup-",
        "joan-popup-x",
    ] {
        assert!(!is_popup_label(other), "{other}");
    }
}

#[test]
fn only_a_web_page_in_the_page_webview_is_an_opener() {
    assert_eq!(
        web("https://example.com/a?b"),
        Opener::WebPage(url("https://example.com/").origin())
    );
    assert!(matches!(
        web("http://127.0.0.1:8123/smoke.html"),
        Opener::WebPage(_)
    ));
    for other in [
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/index.html",
        "tauri://localhost/index.html",
        "http://tauri.localhost/index.html",
        "about:blank",
        "data:text/html,hi",
    ] {
        assert_eq!(web(other), Opener::Other, "{other}");
    }
    let start = keys::app_origin().join("index.html").expect("joins");
    assert_eq!(Opener::of(Some(&start)), Opener::Other);
    assert_eq!(Opener::of(None), Opener::Other);
}

#[test]
fn a_sized_request_from_a_web_page_opens_inside() {
    let page = web("https://shop.example/");
    assert_eq!(decided(&page, SIGNIN, SIZED, 0), Decision::InApp);
    assert_eq!(
        decided(&page, "https://auth.example/authorize?x=1", SIZED, 1),
        Decision::InApp
    );
}

#[test]
fn an_unsized_request_goes_to_the_browser_as_before() {
    let page = web("https://shop.example/");
    assert_eq!(decided(&page, SIGNIN, Size::None, 0), Decision::Browser);
    assert_eq!(
        decided(&page, "https://example.com/next", Size::None, 0),
        Decision::Browser
    );
}

#[test]
fn known_sign_in_flows_open_inside_without_a_size() {
    let page = web("https://shop.example/");
    for target in [
        "https://accounts.google.com/o/oauth2/v2/auth?client_id=x",
        "https://appleid.apple.com/auth/authorize?client_id=x",
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
        "https://login.live.com/oauth20_authorize.srf",
        "https://github.com/login/oauth/authorize?client_id=x",
        "https://www.facebook.com/v19.0/dialog/oauth?client_id=x",
        "https://www.facebook.com/dialog/oauth?client_id=x",
        "https://discord.com/oauth2/authorize?client_id=x",
        "https://x.com/i/oauth2/authorize?client_id=x",
        "https://twitter.com/i/oauth2/authorize?client_id=x",
        "https://slack.com/oauth/v2/authorize?client_id=x",
    ] {
        assert!(known_sign_in(&url(target)), "{target}");
        assert_eq!(
            decided(&page, target, Size::None, 0),
            Decision::InApp,
            "{target}"
        );
        // WebKitGTK can't say the size: a known flow needs none.
        assert_eq!(
            decided(&page, target, Size::Unknown, 0),
            Decision::InApp,
            "{target}"
        );
    }
}

#[test]
fn look_alikes_are_not_sign_in_flows() {
    for target in [
        "http://accounts.google.com/",
        "https://accounts.google.com:8443/",
        "https://accounts.google.com.evil.example/",
        "https://evil.example/accounts.google.com",
        "https://github.com/settings",
        "https://github.com/login",
        "https://www.facebook.com/dialogs/oauth",
        "https://www.facebook.com/vx/dialog/oauth",
        "https://www.facebook.com/profile/dialog/oauth",
        "https://discord.com/channels/oauth2",
        "https://x.com/oauth2",
        "https://slack.com/apps/oauth",
        "https://example.com/login/oauth/authorize",
    ] {
        assert!(!known_sign_in(&url(target)), "{target}");
    }
}

#[test]
fn a_blank_popup_needs_a_size() {
    let page = web("https://shop.example/");
    assert_eq!(decided(&page, "about:blank", SIZED, 0), Decision::InApp);
    assert_eq!(
        decided(&page, "about:blank", Size::None, 0),
        Decision::Refuse
    );
    assert_eq!(
        decided(&page, "about:blank", Size::Unknown, 0),
        Decision::Pending
    );
}

#[test]
fn webkitgtk_requests_wait_for_the_size() {
    let page = web("https://shop.example/");
    assert_eq!(decided(&page, SIGNIN, Size::Unknown, 0), Decision::Pending);
    assert_eq!(settle(&url(SIGNIN), true), Decision::InApp);
    assert_eq!(settle(&url(SIGNIN), false), Decision::Browser);
    assert_eq!(settle(&url("about:blank"), true), Decision::InApp);
    assert_eq!(settle(&url("about:blank"), false), Decision::Refuse);
    assert_eq!(
        settle(&url("https://accounts.google.com/o/oauth2/auth"), false),
        Decision::InApp
    );
}

#[test]
fn webkitgtk_asked_for_a_size_only_when_its_features_gave_one() {
    // What WebKitGTK reported in CI run 42398526340 (`popup-gtk` in the smoke log) for
    // `window.open(url, name, "width=480,height=600")` and for a target=_blank link (or a bare
    // window.open(url)): (features right after `create`, [toolbar, location, menubar] and the
    // geometry at ready-to-show), on 2.50.4 (Ubuntu 22.04) and 2.52.6 (24.04, the AppImage).
    struct Seen {
        features: (i32, i32),
        bars: [bool; 3],
        geometry: (i32, i32),
    }
    let sized = |toolbar| Seen {
        features: (480, 600),
        bars: [toolbar, true, true],
        geometry: (480, 600),
    };
    // A link's geometry is the popup window's own default size (480 × 640 for a pending one).
    let link = |toolbar| Seen {
        features: (0, 0),
        bars: [toolbar, true, true],
        geometry: (480, 640),
    };
    for toolbar in [false, true] {
        let (sized, link) = (sized(toolbar), link(toolbar));
        // At ready-to-show both have a geometry and the same bars: neither can say (CI runs
        // 42390685720 and 42393350263).
        assert_eq!(sized.bars, link.bars);
        assert!(link.geometry.0 > 0 && link.geometry.1 > 0);
        assert!(sized.geometry.0 > 0 && sized.geometry.1 > 0);
        // The features' geometry does.
        assert_eq!(gtk_asked(Some(sized.features)), Some((480.0, 600.0)));
        assert_eq!(gtk_asked(Some(link.features)), None);
        assert_eq!(
            settle(&url(SIGNIN), gtk_asked(Some(sized.features)).is_some()),
            Decision::InApp
        );
        assert_eq!(
            settle(&url(SIGNIN), gtk_asked(Some(link.features)).is_some()),
            Decision::Browser
        );
        assert_eq!(
            settle(
                &url("about:blank"),
                gtk_asked(Some(sized.features)).is_some()
            ),
            Decision::InApp
        );
    }
    // Only one of the two (or none read at all): no size asked.
    assert_eq!(gtk_asked(Some((480, 0))), None);
    assert_eq!(gtk_asked(Some((0, 600))), None);
    assert_eq!(gtk_asked(None), None);
    // The wait before an undecided popup settles as unsized is far longer than ready-to-show takes.
    assert!(GTK_SETTLE_WAIT >= Duration::from_secs(5));
}

#[test]
fn panels_and_app_pages_keep_todays_behaviour() {
    let other = Opener::Other;
    assert_eq!(decided(&other, SIGNIN, SIZED, 0), Decision::Browser);
    assert_eq!(
        decided(
            &other,
            "https://accounts.google.com/o/oauth2/auth",
            Size::None,
            0
        ),
        Decision::Browser
    );
    assert_eq!(decided(&other, "about:blank", SIZED, 0), Decision::Refuse);
    assert_eq!(
        decided(&other, "ljext://hello.localhost/index.html", SIZED, 0),
        Decision::Refuse
    );
}

#[test]
fn a_popup_never_opens_on_what_it_may_not_load() {
    let page = web("https://shop.example/");
    // Not web pages: nowhere, as before.
    for target in [
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/index.html",
        "tauri://localhost/index.html",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,hi",
    ] {
        assert_eq!(
            decided(&page, target, SIZED, 0),
            Decision::Refuse,
            "{target}"
        );
    }
    // Addresses Windows hands to the app's protocols: never a popup (the browser, as before).
    for target in ["http://tauri.evil.example/", "http://ipc.localhost/x"] {
        assert_eq!(
            decided(&page, target, SIZED, 0),
            Decision::Browser,
            "{target}"
        );
    }
    let app = keys::app_origin().join("index.html").expect("joins");
    assert_ne!(
        decide(&page, &app, SIZED, open(0), || true).decision,
        Decision::InApp
    );
}

#[test]
fn at_most_two_popups_at_once() {
    let page = web("https://shop.example/");
    assert_eq!(decided(&page, SIGNIN, SIZED, MAX_OPEN - 1), Decision::InApp);
    let full = decide(&page, &url(SIGNIN), SIZED, open(MAX_OPEN), || true);
    assert_eq!(full.decision, Decision::Browser);
    assert_eq!(full.limit, Some(Limit::Cap));
    let blank = decide(&page, &url("about:blank"), SIZED, open(MAX_OPEN), || true);
    assert_eq!(blank.decision, Decision::Refuse);
    assert_eq!(blank.limit, Some(Limit::Cap));
}

#[test]
fn at_most_thirty_popups_in_a_pages_life() {
    let page = web("https://shop.example/");
    let used = |opened| Counts { open: 0, opened };
    assert_eq!(
        decide(&page, &url(SIGNIN), SIZED, used(LIFETIME_MAX - 1), || true).decision,
        Decision::InApp
    );
    let spent = decide(&page, &url(SIGNIN), SIZED, used(LIFETIME_MAX), || true);
    assert_eq!(spent.decision, Decision::Browser);
    assert_eq!(spent.limit, Some(Limit::Lifetime));
    assert_eq!(spent.limit.map(Limit::name), Some("lifetime"));
    // A known sign-in flow goes to the browser too; the rate isn't counted.
    let mut asked = false;
    let google = decide(
        &page,
        &url("https://accounts.google.com/o/oauth2/auth"),
        Size::None,
        used(LIFETIME_MAX),
        || {
            asked = true;
            true
        },
    );
    assert_eq!(google.decision, Decision::Browser);
    assert!(!asked);
}

#[test]
fn openings_are_rate_limited_and_counted_only_for_a_popup() {
    let page = web("https://shop.example/");
    let limited = decide(&page, &url(SIGNIN), SIZED, open(0), || false);
    assert_eq!(limited.decision, Decision::Browser);
    assert_eq!(limited.limit, Some(Limit::Rate));
    let mut asked = 0;
    for (opener, target, size, now) in [
        (web("https://shop.example/"), SIGNIN, Size::None, 0),
        (Opener::Other, SIGNIN, SIZED, 0),
        (web("https://shop.example/"), "about:blank", Size::None, 0),
        (web("https://shop.example/"), SIGNIN, SIZED, MAX_OPEN),
        (
            web("https://shop.example/"),
            "ljext://hello.localhost/",
            SIZED,
            0,
        ),
    ] {
        let _ = decide(&opener, &url(target), size, open(now), || {
            asked += 1;
            true
        });
    }
    assert_eq!(asked, 0, "only a request that would get a popup counts");
    let _ = decide(&page, &url(SIGNIN), SIZED, open(0), || {
        asked += 1;
        true
    });
    assert_eq!(asked, 1);
}

#[test]
fn a_popup_loads_web_pages_and_blank_only() {
    for allowed in [
        "https://accounts.google.com/signin",
        "http://127.0.0.1:8124/signin.html",
        "about:blank",
        "about:srcdoc",
    ] {
        assert!(may_navigate(&url(allowed)), "{allowed}");
    }
    for refused in [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,hi",
        "blob:https://example.com/0f8f6e1c",
        "about:config",
        "tauri://localhost/index.html",
        "http://tauri.localhost/index.html",
        "http://tauri.evil.example/",
        "http://ipc.localhost/open_input",
        "http://tauri.@evil.example/",
        "ljext://hello.localhost/index.html",
        "http://ljext.hello.localhost/index.html",
        "https://ljext.hello.localhost/index.html",
        "mailto:me@example.com",
        "ftp://example.com/",
        "zoommtg://zoom.us/join",
    ] {
        assert!(!may_navigate(&url(refused)), "{refused}");
    }
    let app = keys::app_origin().join("index.html").expect("joins");
    assert!(!may_navigate(&app));
    let bar = keys::app_origin().join("bar.html").expect("joins");
    assert!(!may_navigate(&bar));
}

#[test]
fn the_title_is_where_the_popup_is() {
    assert_eq!(
        title_for(&url("https://accounts.google.com/o/oauth2/auth?x=1")),
        "accounts.google.com"
    );
    assert_eq!(
        title_for(&url("https://user:pw@login.example:8443/a")),
        "login.example:8443"
    );
    assert_eq!(
        title_for(&url("http://127.0.0.1:8124/signin.html")),
        "http://127.0.0.1:8124"
    );
    assert_eq!(title_for(&url("about:blank")), "about:blank");
    // International names show as the engine resolves them (punycode), never the page's title.
    assert_eq!(
        title_for(&url("https://xn--80ak6aa92e.com/")),
        "xn--80ak6aa92e.com"
    );
    assert_eq!(title_for(&url("https://[::1]:9000/")), "[::1]:9000");
}

#[test]
fn a_popup_is_built_titled_by_its_opener_never_by_its_target() {
    // The review's spoof: window.open(target), stop(), document.write(a fake form) — nothing of
    // the target ever commits, so its host must never be the title.
    let opener = url("https://evil.example/start").origin();
    for target in [
        "https://accounts.google.com/",
        "https://appleid.apple.com/auth/authorize",
        "http://127.0.0.1:8124/signin.html",
    ] {
        let host = url(target).host_str().expect("a host").to_owned();
        let built = opener_title(&opener);
        assert_eq!(built, "evil.example");
        assert!(!built.contains(&host), "{target}");
    }
    assert_eq!(
        opener_title(&url("http://127.0.0.1:8123/smoke.html").origin()),
        "http://127.0.0.1:8123"
    );
    // A blank or srcdoc document is the opener's (it inherits its origin); a committed page is
    // its own.
    assert_eq!(title_at(&url("about:blank"), &opener), "evil.example");
    assert_eq!(title_at(&url("about:srcdoc"), &opener), "evil.example");
    assert_eq!(
        title_at(&url("https://accounts.google.com/signin"), &opener),
        "accounts.google.com"
    );
}

#[test]
fn a_long_host_keeps_its_end() {
    let long = "accounts.google.com.verify-session-0123.evil.example";
    let title = title_for(&url(&format!("https://{long}/login")));
    assert_eq!(title, "…evil.example");
    assert!(title.ends_with("evil.example"));
    assert!(!title.contains("google"));
    assert!(title.chars().count() <= HOST_MAX);
    // As many whole labels as fit, the registrable domain last.
    assert_eq!(
        short_host("accounts.google.com.verify.evil.example"),
        "…com.verify.evil.example"
    );
    // Short hosts stay whole (the known providers among them).
    for host in [
        "accounts.google.com",
        "login.microsoftonline.com",
        "appleid.apple.com",
        "a-thirty-character-host.example",
    ] {
        if host.chars().count() <= HOST_MAX {
            assert_eq!(short_host(host), host);
        }
    }
    // The last two labels stay even when they are long; one label longer than the room keeps its
    // own end.
    let two = format!("www.{}.example", "a".repeat(40));
    let shown = short_host(&two);
    assert!(
        shown.starts_with('…') && shown.ends_with("aaaa.example"),
        "{shown}"
    );
    assert_eq!(shown.chars().count(), HOST_MAX);
    // With a port, and over http: the scheme and the port stay.
    assert_eq!(
        title_for(&url(&format!("http://{long}:8080/"))),
        "http://…evil.example:8080"
    );
    // IP addresses are never cut.
    assert_eq!(
        title_for(&url("http://127.0.0.1:8124/")),
        "http://127.0.0.1:8124"
    );
}

#[test]
fn sizes_are_clamped() {
    assert_eq!(clamp_size(Some((480.0, 600.0))), (480.0, 600.0));
    assert_eq!(clamp_size(Some((10.0, 10.0))), MIN_SIZE);
    assert_eq!(clamp_size(Some((5000.0, 5000.0))), MAX_SIZE);
    assert_eq!(clamp_size(Some((f64::NAN, f64::INFINITY))), MIN_SIZE);
    assert_eq!(clamp_size(None), DEFAULT_SIZE);
}

#[test]
fn a_popup_is_centred_over_the_window_and_kept_on_its_screen() {
    let window = Rect {
        x: 100.0,
        y: 100.0,
        width: 480.0,
        height: 640.0,
    };
    let screen = Rect {
        x: 0.0,
        y: 25.0,
        width: 1440.0,
        height: 875.0,
    };
    assert_eq!(place(window, (400.0, 500.0), Some(screen)), (140.0, 170.0));
    // Wider than the window: still centred over it, but on the screen.
    assert_eq!(place(window, (900.0, 600.0), Some(screen)), (0.0, 120.0));
    // At the screen's far edge.
    let right = Rect {
        x: 1300.0,
        ..window
    };
    assert_eq!(place(right, (480.0, 600.0), Some(screen)), (960.0, 120.0));
    // Bigger than the screen: its top-left corner.
    assert_eq!(place(window, (2000.0, 1000.0), Some(screen)), (0.0, 25.0));
    // No screen known: centred over the window only.
    assert_eq!(place(window, (400.0, 500.0), None), (140.0, 170.0));
    // A second monitor on the left (negative coordinates).
    let left = Rect {
        x: -1920.0,
        y: 0.0,
        width: 1920.0,
        height: 1080.0,
    };
    let there = Rect {
        x: -1000.0,
        y: 200.0,
        width: 480.0,
        height: 640.0,
    };
    assert_eq!(place(there, (480.0, 600.0), Some(left)), (-1000.0, 220.0));
}

#[test]
fn a_popup_closes_when_its_opener_leaves_for_another_origin() {
    let opener = url("https://shop.example/cart").origin();
    // Reloads and same-origin navigations keep it (sign-in flows reload the opener).
    for same in [
        "https://shop.example/cart",
        "https://shop.example/account?signed-in=1",
        "https://shop.example:443/#done",
    ] {
        assert!(!opener_left(&opener, &url(same)), "{same}");
    }
    for other in [
        "https://other.example/",
        "http://shop.example/cart",
        "https://shop.example:8443/",
        "https://www.shop.example/",
        "about:blank",
        "tauri://localhost/index.html",
        "ljext://hello.localhost/index.html",
    ] {
        assert!(opener_left(&opener, &url(other)), "{other}");
    }
}

#[test]
fn in_a_popup_only_close_acts_and_it_closes_the_popup() {
    for action in KeyAction::ALL {
        let in_popup = route_key(true, action);
        if action == KeyAction::Close {
            assert_eq!(in_popup, KeyRoute::ClosePopup);
        } else {
            assert_eq!(in_popup, KeyRoute::Nothing, "{action:?}");
        }
        // In the page window every key does what it always did.
        assert_eq!(
            route_key(false, action),
            KeyRoute::Page(action.page_action())
        );
    }
}
