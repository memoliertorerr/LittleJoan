use super::*;

fn parse_all(args: &[&str]) -> Args {
    let args: Vec<String> = std::iter::once("little-joan")
        .chain(args.iter().copied())
        .map(String::from)
        .collect();
    parse_args(&args)
}

fn parse(args: &[&str]) -> Option<Command> {
    parse_all(args).command
}

fn url(s: &str) -> Url {
    Url::parse(s).expect("valid")
}

#[test]
fn no_flags() {
    assert_eq!(parse(&[]), None);
    assert_eq!(parse(&["-psn_0_12345", "whatever"]), None);
}

#[test]
fn single_flags() {
    assert_eq!(parse(&["--toggle"]), Some(Command::Toggle));
    assert_eq!(parse(&["--hide"]), Some(Command::Hide));
    assert_eq!(parse(&["--show"]), Some(Command::Show(None)));
    assert_eq!(
        parse(&["--show", "https://x.test/a"]),
        Some(Command::Show(Some(url("https://x.test/a"))))
    );
}

#[test]
fn last_flag_wins() {
    assert_eq!(
        parse(&["--show", "https://x.test", "--toggle"]),
        Some(Command::Toggle)
    );
    assert_eq!(parse(&["--toggle", "--show"]), Some(Command::Show(None)));
    assert_eq!(parse(&["--show", "--hide"]), Some(Command::Hide));
}

#[test]
fn size_takes_width_x_height() {
    assert_eq!(
        parse(&["--size", "390x600"]),
        Some(Command::Resize {
            width: 390,
            height: 600
        })
    );
    assert_eq!(
        parse(&["--size", "900X600"]),
        Some(Command::Resize {
            width: 900,
            height: 600
        })
    );
    for bad in ["390", "x600", "0x600", "390x-1", "390x600x2", "wide"] {
        assert_eq!(parse(&["--size", bad]), None, "{bad}");
    }
    assert_eq!(parse(&["--size"]), None);
}

#[test]
fn show_accepts_only_web_urls() {
    assert_eq!(
        parse(&["--show", "file:///etc/passwd"]),
        Some(Command::Show(None))
    );
    assert_eq!(
        parse(&["--show", "javascript:alert(1)"]),
        Some(Command::Show(None))
    );
    assert_eq!(parse(&["--show", "example.com"]), Some(Command::Show(None)));
    assert_eq!(
        parse(&["--show", "http://127.0.0.1:8123/smoke.html"]),
        Some(Command::Show(Some(url("http://127.0.0.1:8123/smoke.html"))))
    );
}

#[test]
fn start_at_login_takes_on_or_off_in_any_case() {
    for (value, want) in [
        ("on", true),
        ("ON", true),
        ("On", true),
        ("off", false),
        ("OFF", false),
    ] {
        assert_eq!(
            parse_all(&["--start-at-login", value]).start_at_login,
            Some(want),
            "{value}"
        );
    }
    for bad in ["yes", "true", "1", "", "onn"] {
        assert_eq!(
            parse_all(&["--start-at-login", bad]).start_at_login,
            None,
            "{bad}"
        );
    }
    assert_eq!(parse_all(&["--start-at-login"]).start_at_login, None);
    assert_eq!(
        parse_all(&["--start-at-login", "on", "--start-at-login", "off"]).start_at_login,
        Some(false)
    );
}

#[test]
fn start_at_login_never_touches_the_window() {
    for args in [
        &["--start-at-login", "on"][..],
        &["--start-at-login", "off"],
        &["--start-at-login", "maybe"],
    ] {
        let parsed = parse_all(args);
        assert_eq!(parsed.command, None, "{args:?}");
    }
    assert_eq!(
        parse_all(&["--start-at-login", "on"]).forwarded_command(),
        None
    );
    assert_eq!(
        parse_all(&["--start-at-login", "off"]).forwarded_command(),
        None
    );
    // A flag without a value is no flag: a plain second launch still shows the window.
    assert_eq!(
        parse_all(&["--start-at-login", "maybe"]).forwarded_command(),
        Some(Command::Show(None))
    );
    // A missing value doesn't swallow the next flag.
    assert_eq!(
        parse_all(&["--start-at-login", "--toggle"]),
        Args {
            command: Some(Command::Toggle),
            start_at_login: None,
            settings: None,
            quit: false,
            link: None,
        }
    );
}

#[test]
fn window_flags_still_apply_next_to_start_at_login() {
    assert_eq!(
        parse_all(&["--start-at-login", "on", "--hide"]),
        Args {
            command: Some(Command::Hide),
            start_at_login: Some(true),
            settings: None,
            quit: false,
            link: None,
        }
    );
    assert_eq!(
        parse_all(&["--hide", "--start-at-login", "off"]).forwarded_command(),
        Some(Command::Hide)
    );
}

#[test]
fn a_plain_second_launch_shows_the_window() {
    assert_eq!(
        parse_all(&[]).forwarded_command(),
        Some(Command::Show(None))
    );
    assert_eq!(
        parse_all(&["-psn_0_12345"]).forwarded_command(),
        Some(Command::Show(None))
    );
    assert_eq!(
        parse_all(&["--toggle"]).forwarded_command(),
        Some(Command::Toggle)
    );
}

#[test]
fn settings_opens_on_a_tab_and_leaves_the_window_alone() {
    assert_eq!(
        parse_all(&["--settings"]).settings,
        Some(SettingsTab::General)
    );
    for (value, tab) in [
        ("general", SettingsTab::General),
        ("keyboard", SettingsTab::Keyboard),
        ("KEYBOARD", SettingsTab::Keyboard),
        ("appearance", SettingsTab::Appearance),
        ("Extensions", SettingsTab::Extensions),
        ("PRIVACY", SettingsTab::Privacy),
        ("about", SettingsTab::About),
        // A name it doesn't know still opens Settings, on General.
        ("advanced", SettingsTab::General),
    ] {
        let parsed = parse_all(&["--settings", value]);
        assert_eq!(parsed.settings, Some(tab), "{value}");
        assert_eq!(parsed.command, None, "{value}");
        assert_eq!(parsed.forwarded_command(), None, "{value}");
    }
    // A missing tab doesn't swallow the next flag; window flags still apply next to it.
    let both = parse_all(&["--settings", "--toggle"]);
    assert_eq!(both.settings, Some(SettingsTab::General));
    assert_eq!(both.forwarded_command(), Some(Command::Toggle));
    assert_eq!(parse_all(&["--toggle"]).settings, None);
    assert_eq!(
        parse_all(&["--settings", "about", "--quit"]).forwarded_command(),
        None
    );
}

#[test]
fn extension_takes_an_id() {
    assert_eq!(
        parse(&["--extension", "hello"]),
        Some(Command::Extension("hello".into()))
    );
    // No id: no flag (a plain second launch shows the window).
    assert_eq!(parse(&["--extension"]), None);
    assert_eq!(parse(&["--extension", "--toggle"]), Some(Command::Toggle));
    assert_eq!(
        parse_all(&["--extension", "hello"]).forwarded_command(),
        Some(Command::Extension("hello".into()))
    );
    assert_eq!(
        parse_all(&["--extension", "hello", "--quit"]).forwarded_command(),
        None
    );
}

#[test]
fn quit_never_opens_the_window() {
    let quit = parse_all(&["--quit"]);
    assert!(quit.quit);
    assert_eq!(quit.command, None);
    assert_eq!(quit.forwarded_command(), None);
    // It wins over window flags in any order; start at login still applies first.
    for args in [
        &["--show", "https://x.test", "--quit"][..],
        &["--quit", "--toggle"],
        &["--quit", "--start-at-login", "on"],
    ] {
        assert_eq!(parse_all(args).forwarded_command(), None, "{args:?}");
    }
    assert_eq!(
        parse_all(&["--quit", "--start-at-login", "off"]).start_at_login,
        Some(false)
    );
    assert!(!parse_all(&["--toggle"]).quit);
    assert!(!parse_all(&["--quitting"]).quit);
}

fn given(text: &str) -> Option<Link> {
    Some(Link::Given(text.to_owned()))
}

#[test]
fn link_takes_exactly_the_next_argument_as_data() {
    assert_eq!(
        parse_all(&["--link", "https://x.test/a?b=c"]).link,
        given("https://x.test/a?b=c")
    );
    // Even when it looks like a flag: `--link --quit` is a link (refused later), not a quit.
    for flag in [
        "--quit",
        "--toggle",
        "--settings",
        "--extension",
        "--link",
        "-psn_0_1",
    ] {
        let parsed = parse_all(&["--link", flag]);
        assert_eq!(parsed.link, given(flag), "{flag}");
        assert!(!parsed.quit, "{flag}");
        assert_eq!(parsed.command, None, "{flag}");
        assert_eq!(parsed.settings, None, "{flag}");
        assert_eq!(parsed.forwarded_command(), None, "{flag}");
    }
    // Anything is data, even what isn't a URL: links.rs refuses it.
    for text in [
        "",
        " ",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "x\" --quit",
    ] {
        assert_eq!(parse_all(&["--link", text]).link, given(text), "{text:?}");
    }
}

#[test]
fn a_command_line_with_a_link_does_nothing_else() {
    // Whatever else is on it, before or after: a link from another app never toggles, hides,
    // quits, opens Settings or an extension, or changes start at login.
    for args in [
        &["--link", "https://x.test/", "--quit"][..],
        &["--quit", "--link", "https://x.test/"],
        &["--link", "https://x.test/", "--toggle", "--hide"],
        &["--settings", "privacy", "--link", "https://x.test/"],
        &["--link", "https://x.test/", "--extension", "hello"],
        &["--start-at-login", "on", "--link", "https://x.test/"],
        &["--link", "https://x.test/\"", "--quit", "\""],
        &[
            "--show",
            "https://y.test/",
            "--link",
            "https://x.test/",
            "--size",
            "390x600",
        ],
    ] {
        let parsed = parse_all(args);
        assert_eq!(
            parsed,
            Args {
                link: parsed.link.clone(),
                ..Args::default()
            },
            "{args:?}: nothing but the link"
        );
        assert!(
            matches!(&parsed.link, Some(Link::Given(text)) if text.starts_with("https://x.test/")),
            "{args:?}"
        );
        assert_eq!(parsed.forwarded_command(), None, "{args:?}");
    }
    // The first `--link` is the one: a second is data, or ignored.
    assert_eq!(
        parse_all(&["--link", "https://x.test/", "--link", "https://y.test/"]).link,
        given("https://x.test/")
    );
}

#[test]
fn link_with_nothing_after_it_is_a_plain_launch() {
    // The desktop file's `Exec=little-joan --link %u` from the app menu: `%u` is nothing.
    let parsed = parse_all(&["--link"]);
    assert_eq!(parsed.link, Some(Link::Empty));
    assert_eq!(parsed.command, None);
    assert!(!parsed.quit);
    // A second launch like that shows the window, as one without flags.
    assert_eq!(parsed.forwarded_command(), Some(Command::Show(None)));
    // Still nothing else counts.
    let parsed = parse_all(&["--quit", "--link"]);
    assert!(!parsed.quit);
    assert_eq!(parsed.forwarded_command(), Some(Command::Show(None)));
    assert_eq!(parse_all(&["--toggle"]).link, None);
    assert_eq!(parse_all(&["--linked", "https://x.test/"]).link, None);
}
