//! Command-line flags. A second launch forwards its arguments to the running instance, so
//! `little-joan --toggle` can be bound to a desktop shortcut where global shortcuts don't work
//! (Linux Wayland).
//!
//! - `--toggle`        same as the global shortcut
//! - `--show [url]`    open (or focus) the window; with an http(s) URL, load it
//! - `--hide`          close the window (hibernate)
//! - `--size WxH`      resize the open window to W×H logical pixels (scripts, tests)
//! - `--start-at-login on|off`  add or remove the OS login entry; never touches the window
//! - `--settings [tab]`  open (or focus) Settings on a tab: general, keyboard, appearance,
//!   extensions, privacy, about (General without one, or with a name it doesn't know); never touches the
//!   page window
//! - `--extension <id>`  open an extension's panel (opens the window if needed; an unknown id
//!   does nothing)
//! - `--quit`          quit Little Joan (the tray's Quit); wins over any window flag
//! - `--link <url>`    a link from another app (LJ45: how the OS starts Little Joan as a browser,
//!   per its registration). It takes exactly the next argument as data, whatever it looks like —
//!   `--link --quit` is a link that is refused, never a quit — and a command line with `--link`
//!   does nothing else: every other flag on it is ignored, so a crafted link can't become one.
//!   links.rs decides whether it opens; `--link` with nothing after it is a plain launch.
//!
//! The last recognized window flag wins; unknown arguments are ignored (macOS can add `-psn_…`).

use url::Url;

use crate::settings_window::SettingsTab;
use crate::toggle::Command;

/// The flag the OS's browser registration puts before a link (LJ45).
pub const LINK_FLAG: &str = "--link";

/// What a command line asks for. `command` goes through the toggle table (the window);
/// `start_at_login` only changes the OS login entry and is handled before it; `settings` opens
/// the settings window; `quit` comes last and replaces the window command. With `link`, nothing
/// else is set (`--link` ignores every other flag).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Args {
    pub command: Option<Command>,
    pub start_at_login: Option<bool>,
    pub settings: Option<SettingsTab>,
    pub quit: bool,
    pub link: Option<Link>,
}

/// `--link` and what came after it (LJ45). Untrusted: links.rs checks it before anything opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// `--link` was the last argument (the desktop file's `%u` from the app menu): a plain launch.
    Empty,
    /// The argument after `--link`, as it came — never read as a flag.
    Given(String),
}

impl Args {
    /// What a second launch does to the window: its window flag, or `--show` when it had no
    /// recognized flag at all. A launch that only changes start at login or opens Settings
    /// leaves the window alone, and one that quits never opens it. A link is never a window
    /// command here: links.rs opens it, or not (`--link` alone is a plain launch).
    pub fn forwarded_command(&self) -> Option<Command> {
        if self.quit || matches!(self.link, Some(Link::Given(_))) {
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
    // A link from another app (LJ45): the first `--link` takes the next argument, and nothing else
    // on the command line counts. The registrations put `--link` before the OS's placeholder, so
    // whatever the link holds comes after it — quotes or spaces that split it into more arguments
    // can only add arguments that are ignored here.
    let rest = args.get(1..).unwrap_or_default();
    if let Some(at) = rest.iter().position(|arg| arg == LINK_FLAG) {
        let link = rest
            .get(at + 1)
            .map_or(Link::Empty, |value| Link::Given(value.clone()));
        return Args {
            link: Some(link),
            ..Args::default()
        };
    }
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
            "--extension" => {
                // Checked against the registry when it opens (extensions::session::open).
                if let Some(id) = rest.next_if(|next| !next.starts_with("--")) {
                    command = Some(Command::Extension(id.clone()));
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
        link: None,
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
mod tests;
