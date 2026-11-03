//! Linux: the desktop files in the XDG data directories whose MimeType has x-scheme-handler/https
//! (shown ones; the first file of an id masks the later ones), `xdg-mime query default` for the
//! default, read-only; a browser starts with `gio launch <file> <url>` (GLib ≥ 2.68), or its own
//! `Exec` with the URL when gio can't (no shell either way).

use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use url::Url;

use crate::browsers::parse::{
    DESKTOP_IDS, DesktopFile, desktop_browser, exec_args, is_own_desktop, xdg_answer,
};
use crate::browsers::{Browser, Browsers, Launch};
use crate::platform::browsers::spawn_detached;

/// `$XDG_DATA_HOME` (or ~/.local/share), then `$XDG_DATA_DIRS` (or /usr/local/share:/usr/share).
fn data_dirs() -> Vec<PathBuf> {
    let set = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let home = set("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| set("HOME").map(|home| PathBuf::from(home).join(".local").join("share")));
    let system = set("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".to_owned());
    home.into_iter()
        .chain(
            system
                .split(':')
                .filter(|d| !d.is_empty())
                .map(PathBuf::from),
        )
        .collect()
}

pub fn read() -> Browsers {
    let own: Vec<PathBuf> = std::env::current_exe()
        .ok()
        .into_iter()
        .chain(std::env::var_os("APPIMAGE").map(PathBuf::from))
        .collect();
    let mut browsers = Browsers {
        self_ids: DESKTOP_IDS.iter().map(|id| (*id).to_owned()).collect(),
        ..Browsers::default()
    };
    let mut seen: Vec<String> = Vec::new();
    for dir in data_dirs() {
        let Ok(entries) = std::fs::read_dir(dir.join("applications")) else {
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "desktop") && p.is_file())
            .collect();
        files.sort();
        for path in files {
            let Some(id) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
                continue;
            };
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let DesktopFile::Browser(browser) = desktop_browser(&id, &path, &text) {
                if let Launch::Desktop { exec, .. } = &browser.launch
                    && is_own_desktop(&id, exec, &own)
                {
                    browsers.self_ids.push(id.clone());
                }
                browsers.installed.push(browser);
            }
        }
    }
    browsers.os_default = Command::new("xdg-mime")
        .args(["query", "default", "x-scheme-handler/https"])
        .output()
        .ok()
        .and_then(|out| xdg_answer(&String::from_utf8_lossy(&out.stdout)));
    browsers
}

/// How long gio may take to say it launched the browser before it is left to it.
const GIO_WAIT: Duration = Duration::from_secs(5);

pub fn open_with(browser: &Browser, url: &Url) -> io::Result<()> {
    let Launch::Desktop { path, exec } = &browser.launch else {
        return Err(io::Error::other("not a desktop file"));
    };
    if let Ok(mut gio) = Command::new("gio")
        .arg("launch")
        .arg(path)
        .arg(url.as_str())
        .spawn()
    {
        let started = Instant::now();
        loop {
            match gio.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(_)) | Err(_) => break,
                Ok(None) if started.elapsed() > GIO_WAIT => {
                    let _ = std::thread::spawn(move || gio.wait());
                    return Ok(());
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            }
        }
    }
    // No gio (or one without `launch`): the file's own Exec, the URL as one argument.
    let args = exec_args(exec, url.as_str()).ok_or_else(|| io::Error::other("bad Exec"))?;
    let (program, rest) = args
        .split_first()
        .ok_or_else(|| io::Error::other("empty Exec"))?;
    spawn_detached(
        program.as_str(),
        &rest.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|path| std::env::split_paths(&path).any(|dir| dir.join(program).is_file()))
}

/// GNOME's Default Apps, or KDE's default applications; nothing on other desktops.
fn settings_program() -> Option<(&'static str, &'static str)> {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let kde = (desktop.to_ascii_uppercase().contains("KDE") || !on_path("gnome-control-center"))
        && on_path("systemsettings");
    if kde {
        Some(("systemsettings", "kcm_componentchooser"))
    } else if on_path("gnome-control-center") {
        Some(("gnome-control-center", "default-apps"))
    } else {
        None
    }
}

pub fn open_default_apps(
    _open_uri: impl FnOnce(&'static str) -> io::Result<()>,
) -> io::Result<Option<&'static str>> {
    match settings_program() {
        Some((program, panel)) => spawn_detached(program, &[panel]).map(|()| Some(program)),
        None => Ok(None),
    }
}

pub fn can_open_default_apps() -> bool {
    settings_program().is_some()
}
