use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use url::Url;

use super::parse::*;
use super::target::*;

fn browser(id: &str) -> Browser {
    Browser {
        id: id.to_owned(),
        name: format!("{id} browser"),
        launch: Launch::Bundle,
    }
}

fn os(installed: &[&str], os_default: Option<&str>) -> Browsers {
    Browsers {
        installed: installed.iter().map(|id| browser(id)).collect(),
        os_default: os_default.map(str::to_owned),
        self_ids: vec![BUNDLE_ID.to_owned()],
    }
}

fn target_id<'a>(decision: &Decision<'a>) -> Option<&'a str> {
    match decision.target {
        Target::Browser(b) => Some(b.id.as_str()),
        _ => None,
    }
}

#[test]
fn an_installed_main_browser_wins() {
    let browsers = os(
        &["com.apple.Safari", "org.mozilla.firefox", BUNDLE_ID],
        Some(BUNDLE_ID),
    );
    let decision = handoff_target(Some("org.mozilla.firefox"), &browsers);
    assert_eq!(target_id(&decision), Some("org.mozilla.firefox"));
    assert!(!decision.refused_self);
    // Ids compare without case (Launch Services may lower-case them).
    let lower = handoff_target(Some("ORG.MOZILLA.FIREFOX"), &browsers);
    assert_eq!(target_id(&lower), Some("org.mozilla.firefox"));
}

#[test]
fn else_the_os_default_unless_it_is_little_joan() {
    let browsers = os(&["com.apple.Safari", BUNDLE_ID], Some("com.apple.Safari"));
    assert_eq!(handoff_target(None, &browsers).target, Target::OsDefault);
    // A main browser that isn't installed any more: the default.
    assert_eq!(
        handoff_target(Some("gone.browser"), &browsers).target,
        Target::OsDefault
    );
    // A default that can't be read: the opener, as before (links.rs's guard catches a bounce).
    assert_eq!(
        handoff_target(None, &os(&["com.apple.Safari"], None)).target,
        Target::OsDefault
    );
}

#[test]
fn with_little_joan_the_default_the_first_other_browser_in_the_os_order() {
    let browsers = os(
        &[BUNDLE_ID, "com.apple.Safari", "org.mozilla.firefox"],
        Some(BUNDLE_ID),
    );
    let decision = handoff_target(None, &browsers);
    assert_eq!(target_id(&decision), Some("com.apple.Safari"));
    assert!(decision.refused_self, "the default pointed at Little Joan");
}

#[test]
fn nowhere_when_little_joan_is_the_only_browser() {
    let only_us = os(&[BUNDLE_ID], Some(BUNDLE_ID));
    let decision = handoff_target(None, &only_us);
    assert_eq!(decision.target, Target::Nowhere);
    assert!(decision.refused_self);
    assert_eq!(
        handoff_target(None, &os(&[], Some(BUNDLE_ID))).target,
        Target::Nowhere
    );
}

#[test]
fn little_joan_itself_is_always_refused() {
    let browsers = Browsers {
        self_ids: vec![CLIENT_ID.to_owned(), PROGID.to_owned()],
        ..os(&["LittleJoan", "Google Chrome"], Some("LittleJoan"))
    };
    // As the main browser (settings.json could say so), as the default, as "the first other".
    let decision = handoff_target(Some("littlejoan"), &browsers);
    assert_eq!(target_id(&decision), Some("Google Chrome"));
    assert!(decision.refused_self);
    let only_us = Browsers {
        installed: vec![browser("LittleJoan")],
        ..browsers.clone()
    };
    assert_eq!(
        handoff_target(Some("LittleJoan"), &only_us).target,
        Target::Nowhere
    );
    for main in [None, Some("LittleJoan"), Some("Google Chrome"), Some("x")] {
        for default in [
            None,
            Some("LittleJoan"),
            Some("LittleJoanURL"),
            Some("MSEdgeHTM"),
        ] {
            let browsers = Browsers {
                os_default: default.map(str::to_owned),
                ..browsers.clone()
            };
            if let Target::Browser(b) = handoff_target(main, &browsers).target {
                assert!(!browsers.is_self(&b.id), "{main:?} {default:?}");
            }
        }
    }
    assert_eq!(browsers.others().count(), 1);
}

#[test]
fn the_default_is_named_for_settings_and_the_log() {
    let browsers = os(&["com.apple.Safari", BUNDLE_ID], Some("com.apple.safari"));
    assert_eq!(browsers.default_kind(), "other");
    assert_eq!(browsers.default_name(), Some("com.apple.Safari browser"));
    assert_eq!(os(&[], Some(BUNDLE_ID)).default_kind(), "self");
    assert_eq!(os(&[], None).default_kind(), "none");
}

#[test]
fn a_page_handed_on_bounces_back_once_within_the_window() {
    let url = Url::parse("https://example.com/a").expect("valid");
    let other = Url::parse("https://example.com/b").expect("valid");
    let start = Instant::now();
    let mut recent = Recent::new();
    recent.handed_off(&url, start);
    assert!(!recent.bounced(&other, start));
    assert!(recent.bounced(&url, start + Duration::from_secs(2)));
    assert!(
        !recent.bounced(&url, start + Duration::from_secs(3)),
        "once"
    );
    recent.handed_off(&url, start);
    assert!(!recent.bounced(&url, start + BOUNCE_WINDOW), "too late");
    for n in 0..40 {
        let page = Url::parse(&format!("https://example.com/{n}")).expect("valid");
        recent.handed_off(&page, start);
    }
    assert!(recent.bounced(&Url::parse("https://example.com/39").expect("valid"), start));
    assert!(!recent.bounced(&Url::parse("https://example.com/0").expect("valid"), start));
}

#[test]
fn windows_start_menu_entries() {
    let chrome_exe = r#""C:\Program Files\Google\Chrome\Application\chrome.exe""#;
    let chrome_html =
        r#""C:\Program Files\Google\Chrome\Application\chrome.exe" --single-argument %1"#;
    let chrome = start_menu_entry(
        "Google Chrome",
        Some("Google Chrome"),
        Some("Google Chrome"),
        Some(chrome_exe),
        Some(chrome_html),
    )
    .expect("a browser");
    assert_eq!(chrome.name, "Google Chrome");
    // Through the ProgId its Capabilities give https to (phase 21's security review).
    assert_eq!(chrome.launch, Launch::Command(chrome_html.into()));
    // Without one: its own command, the URL appended.
    let own = start_menu_entry("Google Chrome", None, None, Some(chrome_exe), None);
    assert_eq!(
        own.expect("a browser").launch,
        Launch::Command(chrome_exe.into())
    );
    let edge = start_menu_entry(
        "Microsoft Edge",
        None,
        Some("@C:\\edge.dll,-1"),
        Some("edge.exe"),
        None,
    );
    assert_eq!(edge.expect("a browser").name, "Microsoft Edge");
    let named = start_menu_entry("X", Some(" "), Some("X Browser"), Some("x.exe"), None);
    assert_eq!(named.expect("a browser").name, "X Browser");
    assert_eq!(start_menu_entry("Y", Some("Y"), None, None, None), None);
    assert_eq!(
        start_menu_entry("Y", Some("Y"), None, Some("  "), None),
        None
    );
}

#[test]
fn windows_browsers_never_start_an_interpreter() {
    for program in [
        "C:\\Program Files\\Mozilla Firefox\\firefox.exe",
        "msedge.exe",
        "C:/Tools/BROWSER.EXE",
        "C:\\hostedtoolcache\\node\\24\\node.exe",
    ] {
        assert!(launchable_program(program), "{program}");
    }
    for program in [
        "cmd.exe",
        "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
        "PWSH.EXE",
        "wscript.exe",
        "cscript.exe",
        "mshta.exe",
        "C:\\Windows\\System32\\rundll32.exe",
        "regsvr32.exe",
        "C:\\x\\browser.bat",
        "C:\\x\\browser.cmd",
        "C:\\x\\browser.ps1",
        "C:\\x\\browser",
        ".exe",
    ] {
        assert!(!launchable_program(program), "{program}");
    }
    // An entry whose only command runs an interpreter isn't offered at all.
    assert_eq!(
        start_menu_entry(
            "Evil",
            Some("Evil"),
            None,
            Some(r#"cmd.exe /c start %1"#),
            Some(r#"powershell.exe -c "%1""#)
        ),
        None
    );
    // A ProgId that runs an interpreter falls back to the entry's own launchable command.
    let fallback = start_menu_entry(
        "B",
        None,
        None,
        Some("b.exe"),
        Some(r#"cmd.exe /c b.exe %1"#),
    );
    assert_eq!(
        fallback.expect("a browser").launch,
        Launch::Command("b.exe".into())
    );
}

#[test]
fn windows_user_choice_latest_first() {
    assert_eq!(
        pick_progid(Some("ChromeHTML".into()), Some("MSEdgeHTM".into())),
        Some("ChromeHTML".into())
    );
    assert_eq!(
        pick_progid(None, Some("MSEdgeHTM".into())),
        Some("MSEdgeHTM".into())
    );
    assert_eq!(
        pick_progid(Some(" ".into()), Some("MSEdgeHTM".into())),
        Some("MSEdgeHTM".into())
    );
    assert_eq!(pick_progid(None, None), None);
}

#[test]
fn windows_user_choice() {
    let associations = vec![
        ("Microsoft Edge".to_owned(), "MSEdgeHTM".to_owned()),
        ("Google Chrome".to_owned(), "ChromeHTML".to_owned()),
    ];
    assert_eq!(
        default_from_progid("ChromeHTML", &associations),
        "Google Chrome"
    );
    assert_eq!(
        default_from_progid("msedgehtm", &associations),
        "Microsoft Edge"
    );
    assert_eq!(
        default_from_progid("LittleJoanURL", &associations),
        CLIENT_ID
    );
    assert_eq!(default_from_progid("OtherHTML", &associations), "OtherHTML");
    let own = Path::new(r"C:\Users\me\AppData\Local\Little Joan\little-joan.exe");
    assert!(is_own_client("LittleJoan", "x.exe", None));
    assert!(is_own_client(
        "Renamed",
        r#""C:\Users\me\AppData\Local\Little Joan\little-joan.exe""#,
        Some(own)
    ));
    assert!(!is_own_client(
        "Google Chrome",
        r#""C:\chrome.exe""#,
        Some(own)
    ));
}

#[test]
fn windows_command_lines_without_a_shell() {
    let line = r#""C:\Program Files\B\b.exe" --single-argument "%1" -x"#;
    let (program, args) = split_command(line).expect("a command");
    assert_eq!(program, r"C:\Program Files\B\b.exe");
    assert_eq!(args, ["--single-argument", "%1", "-x"]);
    let url = "https://example.com/?q=a&b=c";
    assert_eq!(with_url(&args, url), ["--single-argument", url, "-x"]);
    let (program, args) = split_command(r"C:\b.exe").expect("a command");
    assert_eq!((program.as_str(), args.len()), (r"C:\b.exe", 0));
    assert_eq!(with_url(&args, url), [url]);
    // MSVC's rules: \" is a quote, backslashes before a quote halve.
    let (_, args) = split_command(r#"b.exe "a b" c\"d e\\"f g""#).expect("a command");
    assert_eq!(args, ["a b", "c\"d", "e\\f g"]);
    assert_eq!(split_command("  "), None);
    assert_eq!(split_command(r#""unclosed"#), None);
}

#[test]
fn linux_desktop_files() {
    let firefox = "[Desktop Entry]\nType=Application\nName=Firefox\nExec=firefox %u\nMimeType=text/html;x-scheme-handler/http;x-scheme-handler/https;\n[Desktop Action new]\nName=New\n";
    let path = Path::new("/usr/share/applications/firefox.desktop");
    match desktop_browser("firefox.desktop", path, firefox) {
        DesktopFile::Browser(b) => {
            assert_eq!(b.name, "Firefox");
            assert_eq!(
                b.launch,
                Launch::Desktop {
                    path: path.to_owned(),
                    exec: "firefox %u".into()
                }
            );
        }
        other => panic!("{other:?}"),
    }
    for text in [
        firefox.replace("Type=Application", "Type=Link"),
        firefox.replace("x-scheme-handler/https;", ""),
        firefox.replace("Exec=firefox %u\n", ""),
        firefox.replace("Name=Firefox\n", "Name=Firefox\nNoDisplay=true\n"),
        firefox.replace("Name=Firefox\n", "Name=Firefox\nHidden=true\n"),
    ] {
        assert_eq!(
            desktop_browser("x.desktop", path, &text),
            DesktopFile::Other,
            "{text}"
        );
    }
}

#[test]
fn linux_exec_lines() {
    assert_eq!(
        exec_args("firefox %u", "https://x.test/a b"),
        Some(vec!["firefox".into(), "https://x.test/a b".into()])
    );
    assert_eq!(
        exec_args(r#""/opt/my app/run" --new %U %i --pct=%%"#, "u"),
        Some(vec![
            "/opt/my app/run".into(),
            "--new".into(),
            "u".into(),
            "--pct=%".into()
        ])
    );
    assert_eq!(exec_args(r#"app "%u""#, "u"), None);
    assert_eq!(exec_args(r#"app "open"#, "u"), None);
    assert_eq!(exec_args("", "u"), None);
    // The URL is one argument, whatever it holds.
    let tricky = "https://x.test/?a=1;b=$(id)`x`\"";
    assert_eq!(
        exec_args("b %u", tricky),
        Some(vec!["b".into(), tricky.into()])
    );
}

#[test]
fn linux_little_joan_itself() {
    let own = [PathBuf::from("/tmp/LittleJoan-0.3.0-linux-x86_64.AppImage")];
    assert!(is_own_desktop("Little Joan.desktop", "x", &own));
    assert!(is_own_desktop(
        "io.github.memoliertorerr.littlejoan.desktop",
        "x",
        &own
    ));
    assert!(is_own_desktop(
        "renamed.desktop",
        "little-joan --link %u",
        &own
    ));
    assert!(is_own_desktop(
        "renamed.desktop",
        "/usr/bin/little-joan --link %u",
        &own
    ));
    assert!(is_own_desktop(
        "appimagekit_1-Little_Joan.desktop",
        "/tmp/LittleJoan-0.3.0-linux-x86_64.AppImage --link %u",
        &own
    ));
    assert!(!is_own_desktop("firefox.desktop", "firefox %u", &own));
}

#[test]
fn linux_xdg_mime_answers() {
    assert_eq!(
        xdg_answer("firefox.desktop\n"),
        Some("firefox.desktop".into())
    );
    assert_eq!(
        xdg_answer("\nLittle Joan.desktop\n"),
        Some("Little Joan.desktop".into())
    );
    assert_eq!(xdg_answer(""), None);
    assert_eq!(xdg_answer("xdg-mime: no method available\n"), None);
    assert_eq!(xdg_answer("/usr/share/applications/x.desktop"), None);
}

#[test]
fn linux_mimeapps_without_xdg_mime() {
    let user = "[Added Associations]\nx-scheme-handler/https=chrome.desktop;\n[Default Applications]\ntext/html=x.desktop\nx-scheme-handler/https = Little Joan.desktop;firefox.desktop\n";
    let system = "[Default Applications]\nx-scheme-handler/https=firefox.desktop\n";
    assert_eq!(
        mimeapps_default([user, system]),
        Some("Little Joan.desktop".into())
    );
    // A file that sets no https default doesn't count; the next one does.
    assert_eq!(
        mimeapps_default(["[Default Applications]\ntext/html=x.desktop\n", system]),
        Some("firefox.desktop".into())
    );
    assert_eq!(
        mimeapps_default(["[Added Associations]\nx-scheme-handler/https=chrome.desktop\n"]),
        None
    );
    assert_eq!(
        mimeapps_default(["[Default Applications]\nx-scheme-handler/https=/x/y.desktop\n"]),
        None
    );
    assert_eq!(
        mimeapps_default(["[Default Applications]\nx-scheme-handler/http=a.desktop\n"]),
        None
    );
    assert_eq!(mimeapps_default(Vec::<&str>::new()), None);
}

/// Every Rust file under `dir`.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).expect("read src") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    found
}

#[test]
fn every_handoff_goes_through_browsers_rs() {
    // LJ46 and its security review: the opener (which hands a URL to the OS default — maybe Little
    // Joan itself) is used only by browsers.rs's glue; elsewhere only the two "show in folder"
    // calls may name the opener plugin at all.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let glue = src.join("browsers.rs");
    let reveals = [
        src.join("commands.rs"),
        src.join("commands").join("downloads.rs"),
    ];
    let mut calls_in_glue = 0;
    for path in rust_files(&src) {
        if path.ends_with("browsers/tests.rs") {
            continue;
        }
        let text = fs::read_to_string(&path).expect("read a source file");
        let uses = text.matches("tauri_plugin_opener::").count();
        let other_ways = text.matches("OpenerExt").count() + text.matches(".opener()").count();
        assert_eq!(other_ways, 0, "{}: no opener extension", path.display());
        if path == glue {
            calls_in_glue = text.matches("tauri_plugin_opener::open_url(").count();
            assert_eq!(uses, calls_in_glue, "browsers.rs: open_url only");
        } else if reveals.contains(&path) {
            assert_eq!(
                uses,
                text.matches("tauri_plugin_opener::reveal_item_in_dir(")
                    .count(),
                "{}: reveal_item_in_dir only",
                path.display()
            );
            assert_eq!(uses, 1, "{}", path.display());
        } else {
            assert_eq!(
                uses,
                0,
                "{}: hand pages on through browsers.rs",
                path.display()
            );
        }
    }
    assert_eq!(
        calls_in_glue, 2,
        "browsers.rs: the OS default and the default-apps settings"
    );
}
