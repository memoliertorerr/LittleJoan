//! Windows: the browsers registered for Default apps — `Software\Clients\StartMenuInternet\*`
//! under HKCU, then HKLM — and the user's choice for https (`UserChoice`'s ProgId), read-only;
//! a browser starts as its registered command with the URL as one argument (no shell).

use std::io;

use url::Url;
use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

use crate::browsers::parse::{
    CLIENT_ID, PROGID, default_from_progid, is_own_client, split_command, start_menu_entry,
    with_url,
};
use crate::browsers::{Browser, Browsers, Launch};
use crate::platform::browsers::spawn_detached;

const CLIENTS: &str = r"Software\Clients\StartMenuInternet";
const USER_CHOICE: &str =
    r"Software\Microsoft\Windows\Shell\Associations\UrlAssociations\https\UserChoice";

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
            if let Some(https) = value(&key, r"Capabilities\URLAssociations", "https") {
                associations.push((id.clone(), https));
            }
            let entry = start_menu_entry(
                &id,
                value(&key, "", "").as_deref(),
                value(&key, "Capabilities", "ApplicationName").as_deref(),
                command.as_deref(),
            );
            if let Some(browser) = entry {
                if is_own_client(&id, command.as_deref().unwrap_or_default(), own.as_deref()) {
                    browsers.self_ids.push(id.clone());
                }
                browsers.installed.push(browser);
            }
        }
    }
    browsers.os_default = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(USER_CHOICE, KEY_READ)
        .ok()
        .and_then(|key| key.get_value::<String, _>("ProgId").ok())
        .map(|progid| default_from_progid(&progid, &associations));
    browsers
}

pub fn open_with(browser: &Browser, url: &Url) -> io::Result<()> {
    let Launch::Command(line) = &browser.launch else {
        return Err(io::Error::other("not a registered browser"));
    };
    let (program, args) =
        split_command(line).ok_or_else(|| io::Error::other("no command for the browser"))?;
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
