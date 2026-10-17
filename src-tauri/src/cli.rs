//! Command-line flags. A second launch forwards its arguments to the running instance, so
//! `little-joan --toggle` can be bound to a desktop shortcut where global shortcuts don't work
//! (Linux Wayland).
//!
//! - `--toggle`        same as the global shortcut
//! - `--show [url]`    open (or focus) the window; with an http(s) URL, load it
//! - `--hide`          close the window (hibernate)
//! - `--size WxH`      resize the open window to W×H logical pixels (scripts, tests)
//! - `--start-at-login on|off`  add or remove the OS login entry; never touches the window
//! - `--settings [tab]`  open (or focus) Settings on a tab: general, appearance, extensions,
//!   privacy, about (General without one, or with a name it doesn't know); never touches the
//!   page window
//! - `--quit`          quit Little Joan (the tray's Quit); wins over any window flag
//!
//! The last recognized window flag wins; unknown arguments are ignored (macOS can add `-psn_…`).

use url::Url;

use crate::settings_window::SettingsTab;
use crate::toggle::Command;

/// What a command line asks for. `command` goes through the toggle table (the window);
/// `start_at_login` only changes the OS login entry and is handled before it; `settings` opens
/// the settings window; `quit` comes last and replaces the window command.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Args {
    pub command: Option<Command>,
    pub start_at_login: Option<bool>,
    pub settings: Option<SettingsTab>,
    pub quit: bool,
}

impl Args {
    /// What a second launch does to the window: its window flag, or `--show` when it had no
    /// recognized flag at all. A launch that only changes start at login or opens Settings
    /// leaves the window alone, and one that quits never opens it.
    pub fn forwarded_command(&self) -> Option<Command> {
        if self.quit {
            return None;
        }
        match (&self.command, self.start_at_login, self.settings) {
            (Some(command), _, _) => Some(command.clone()),
            (None, None, None) => Some(Command::Show(None)),
            _ => None,
        }
    }
}

/// Parses `args` (including argv[0]).
pub fn parse_args(args: &[String]) -> Args {
    let mut command = None;
    let mut start_at_login = None;
    let mut settings = None;
    let mut quit = false;
    let mut rest = args.iter().skip(1).peekable();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--toggle" => command = Some(Command::Toggle),
            "--hide" => command = Some(Command::Hide),
            "--quit" => quit = true,
            "--start-at-login" => {
                if let Some(enabled) = rest
                    .next_if(|next| !next.starts_with("--"))
                    .and_then(|value| on_off(value))
                {
                    start_at_login = Some(enabled);
                }
            }
            "--settings" => {
                let tab = rest
                    .next_if(|next| !next.starts_with("--"))
                    .and_then(|value| SettingsTab::from_name(value));
                settings = Some(tab.unwrap_or_default());
            }
            "--size" => {
                if let Some((width, height)) = rest
                    .next_if(|next| !next.starts_with("--"))
                    .and_then(|value| parse_size(value))
                {
                    command = Some(Command::Resize { width, height });
                }
            }
            "--show" => {
                let url = rest
                    .next_if(|next| !next.starts_with("--"))
                    .and_then(|value| web_url(value));
                command = Some(Command::Show(url));
            }
            _ => {}
        }
    }
    Args {
        command,
        start_at_login,
        settings,
        quit,
    }
}

/// `on` / `off` in any case; anything else is not a value.
fn on_off(value: &str) -> Option<bool> {
    if value.eq_ignore_ascii_case("on") {
        Some(true)
    } else if value.eq_ignore_ascii_case("off") {
        Some(false)
    } else {
        None
    }
}

/// `"390x844"` → `(390, 844)`; both sides must be whole numbers from 1 to 10000.
fn parse_size(value: &str) -> Option<(u32, u32)> {
    let (width, height) = value.split_once(['x', 'X'])?;
    let side = |text: &str| {
        text.parse::<u32>()
            .ok()
            .filter(|n| (1..=10_000).contains(n))
    };
    Some((side(width)?, side(height)?))
}

/// Only absolute http(s) URLs are accepted from the command line — never a search, a file, or
/// another scheme.
fn web_url(value: &str) -> Option<Url> {
    Url::parse(value)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
}

#[cfg(test)]
mod tests {
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
}
