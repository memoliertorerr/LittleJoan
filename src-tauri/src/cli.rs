//! Command-line flags. A second launch forwards its arguments to the running instance, so
//! `little-joan --toggle` can be bound to a desktop shortcut where global shortcuts don't work
//! (Linux Wayland).
//!
//! - `--toggle`        same as the global shortcut
//! - `--show [url]`    open (or focus) the window; with an http(s) URL, load it
//! - `--hide`          close the window (hibernate)
//! - `--size WxH`      resize the open window to W×H logical pixels (scripts, tests)
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
    command
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
}
