//! Windows: the browsers registered for Default apps — `Software\Clients\StartMenuInternet\*`
//! under HKCU, then HKLM — and the user's choice for https (`UserChoiceLatest`'s ProgId, else
//! `UserChoice`'s), read-only. A browser starts through the ProgId its Capabilities give https to
//! (that ProgId's `shell\open\command`, `%1` = the URL as one argument), else its own command with
//! the URL appended; never through a shell, never an interpreter or a non-`.exe` (parse.rs).

use std::io;

use url::Url;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

use crate::browsers::parse::{
    CLIENT_ID, PROGID, default_from_progid, is_own_client, launchable_program, pick_progid,
    split_command, start_menu_entry, with_url,
};
use crate::browsers::{Browser, Browsers, Launch};
use crate::platform::browsers::spawn_detached;

const CLIENTS: &str = r"Software\Clients\StartMenuInternet";
const USER_CHOICE: &str =
    r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice";
/// Newer Windows 11 keeps the effective choice here (with `ProgId` as a value or a subkey's).
const USER_CHOICE_LATEST: &str =
    r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoiceLatest";

/// A ProgId's `shell\open\command`, HKCU's before HKLM's (the merged `Classes` view).
fn progid_command(progid: &str) -> Option<String> {
    [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
        .into_iter()
        .find_map(|root| {
            let key = RegKey::predef(root)
                .open_subkey_with_flags(format!(r"Software\Classes\{progid}"), KEY_READ)
                .ok()?;
            value(&key, r"shell\open\command", "")
        })
}

/// The ProgId the user chose for https: `UserChoiceLatest` first, then `UserChoice`.
fn chosen_progid() -> Option<String> {
    let user = RegKey::predef(HKEY_CURRENT_USER);
    let latest = user
        .open_subkey_with_flags(USER_CHOICE_LATEST, KEY_READ)
        .ok()
        .and_then(|key| value(&key, "", "ProgId").or_else(|| value(&key, "ProgId", "ProgId")));
    let legacy = user
        .open_subkey_with_flags(USER_CHOICE, KEY_READ)
        .ok()
        .and_then(|key| value(&key, "", "ProgId"));
    pick_progid(latest, legacy)
}

fn value(key: &RegKey, path: &str, name: &str) -> Option<String> {
    if path.is_empty() {
        key.get_value(name).ok()
    } else {
        key.open_subkey_with_flags(path, KEY_READ)
            .ok()?
            .get_value(name)
            .ok()
    }
}

pub fn read() -> Browsers {
    let own = std::env::current_exe().ok();
    let mut browsers = Browsers {
        self_ids: vec![CLIENT_ID.to_owned(), PROGID.to_owned()],
        ..Browsers::default()
    };
    let mut associations: Vec<(String, String)> = Vec::new();
    for root in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
        let Ok(clients) = RegKey::predef(root).open_subkey_with_flags(CLIENTS, KEY_READ) else {
            continue;
        };
        for id in clients.enum_keys().filter_map(Result::ok) {
            if browsers
                .installed
                .iter()
                .any(|b| b.id.eq_ignore_ascii_case(&id))
            {
                continue;
            }
            let Ok(key) = clients.open_subkey_with_flags(&id, KEY_READ) else {
                continue;
            };
            let command = value(&key, r"shell\open\command", "");
            let https = value(&key, r"Capabilities\URLAssociations", "https");
            let launch = https.as_deref().and_then(progid_command);
            if let Some(https) = https {
                associations.push((id.clone(), https));
            }
            let entry = start_menu_entry(
                &id,
                value(&key, "", "").as_deref(),
                value(&key, "Capabilities", "ApplicationName").as_deref(),
                command.as_deref(),
                launch.as_deref(),
            );
            if let Some(browser) = entry {
                if is_own_client(&id, command.as_deref().unwrap_or_default(), own.as_deref()) {
                    browsers.self_ids.push(id.clone());
                }
                browsers.installed.push(browser);
            }
        }
    }
    browsers.os_default = chosen_progid().map(|progid| default_from_progid(&progid, &associations));
    browsers
}

pub fn open_with(browser: &Browser, url: &Url) -> io::Result<()> {
    let Launch::Command(line) = &browser.launch else {
        return Err(io::Error::other("not a registered browser"));
    };
    let (program, args) =
        split_command(line).ok_or_else(|| io::Error::other("no command for the browser"))?;
    if !launchable_program(&program) {
        return Err(io::Error::other("not a browser program"));
    }
    spawn_detached(program, &with_url(&args, url.as_str()))
}

/// Settings → Default apps, on Little Joan's own page where Windows 11 has one.
pub fn open_default_apps(
    open_uri: impl FnOnce(&'static str) -> io::Result<()>,
) -> io::Result<Option<&'static str>> {
    let uri = "ms-settings:defaultapps?registeredAppUser=Little%20Joan";
    open_uri(uri).map(|()| Some(uri))
}

pub fn can_open_default_apps() -> bool {
    true
}
