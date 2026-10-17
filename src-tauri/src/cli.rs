//! Command-line flags. A second launch forwards its arguments to the running instance, so
//! `little-joan --toggle` can be bound to a desktop shortcut where global shortcuts don't work
//! (Linux Wayland).
//!
//! - `--toggle`        same as the global shortcut
//! - `--show [url]`    open (or focus) the window; with an http(s) URL, load it
//! - `--hide`          close the window (hibernate)
//!
//! The last recognized flag wins; unknown arguments are ignored (macOS can add `-psn_…`).

use url::Url;

use crate::toggle::Command;

/// Parses `args` (including argv[0]). Returns `None` when no flag is recognized.
pub fn parse_args(args: &[String]) -> Option<Command> {
    let mut command = None;
    let mut rest = args.iter().skip(1).peekable();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--toggle" => command = Some(Command::Toggle),
            "--hide" => command = Some(Command::Hide),
            "--show" => {
                let url = rest
                    .next_if(|next| !next.starts_with("--"))
                    .and_then(|value| web_url(value));
                command = Some(Command::Show(url));
            }
            _ => {}
        }
    }
    command
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

    fn parse(args: &[&str]) -> Option<Command> {
        let args: Vec<String> = std::iter::once("little-joan")
            .chain(args.iter().copied())
            .map(String::from)
            .collect();
        parse_args(&args)
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
}
