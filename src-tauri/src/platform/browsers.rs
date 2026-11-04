//! The OS's browsers for web links (LJ46), read-only, and starting one without a shell: the
//! readers are mac/browsers.rs (NSWorkspace), win/browsers.rs (the registry's StartMenuInternet
//! and UserChoice), linux/browsers.rs (the XDG desktop files, `xdg-mime`); their pure parts are in
//! crate::browsers::parse. Nothing here writes anything.

use std::io;
use std::process::{Command, Stdio};

use url::Url;

use crate::browsers::{Browser, Browsers};

#[cfg(target_os = "linux")]
use super::linux::browsers as os;
#[cfg(target_os = "macos")]
use super::mac::browsers as os;
#[cfg(windows)]
use super::win::browsers as os;

/// Every browser the OS lists for https links, its default, and which are Little Joan.
pub fn read() -> Browsers {
    os::read()
}

/// Opens `url` with `browser` (one argument, no shell).
pub fn open_with(browser: &Browser, url: &Url) -> io::Result<()> {
    os::open_with(browser, url)
}

/// Opens the OS's default-apps settings — a settings URI through `open_uri` (macOS, Windows), or
/// GNOME's / KDE's settings program — and says what it opened; `None` when this desktop has none.
pub fn open_default_apps(
    open_uri: impl FnOnce(&'static str) -> io::Result<()>,
) -> io::Result<Option<&'static str>> {
    os::open_default_apps(open_uri)
}

/// Whether `open_default_apps` has something to open here.
pub fn can_open_default_apps() -> bool {
    os::can_open_default_apps()
}

/// Starts `program` with `args` and leaves it running (a thread reaps it when it ends).
pub fn spawn_detached<S: AsRef<std::ffi::OsStr>>(program: S, args: &[S]) -> io::Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    std::thread::Builder::new()
        .name("little-joan-browser".into())
        .spawn(move || {
            let _ = child.wait();
        })
        .map(drop)
}
