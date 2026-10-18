//! An extension's `extension.json`: its types, parsing and validation (pure, tested).
//!
//! Strict on purpose — the API is experimental, so a typo should fail loudly rather than be
//! ignored: unknown fields are refused, every text has a length limit and no control characters,
//! permissions come from a short list, and every path must be a plain path inside the folder
//! (paths.rs). A manifest that fails is skipped with the reason (registry.rs), never a crash.

use std::collections::HashSet;

use serde_json::{Map, Value};

use super::paths::{self, RelPath};

/// The only API version there is.
pub const API_VERSION: u64 = 1;
/// The largest manifest read, in bytes.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
/// At most this many permissions, commands and page actions each.
pub const MAX_ITEMS: usize = 16;
/// At most this many keywords per command.
pub const MAX_KEYWORDS: usize = 8;

/// What an extension may use beyond its own panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    /// `storage`: littleJoan.storage (1 MB).
    Storage,
    /// `clipboard`: littleJoan.clipboard.writeText.
    Clipboard,
    /// `pageInfo`: the URL and title of the page a page action was opened for.
    PageInfo,
    /// `openBrowser`: littleJoan.openUrl(url, { where: "browser" }) — hand a web address to the
    /// default browser (at most one every 2 s).
    OpenBrowser,
    /// `network:<host>`: fetch from `https://<host>` (`*.` allowed in front).
    Network(String),
}

impl Permission {
    fn parse(text: &str) -> Option<Permission> {
        match text {
            "storage" => Some(Permission::Storage),
            "clipboard" => Some(Permission::Clipboard),
            "pageInfo" => Some(Permission::PageInfo),
            "openBrowser" => Some(Permission::OpenBrowser),
            _ => {
                let host = text.strip_prefix("network:")?;
                is_valid_network_host(host).then(|| Permission::Network(host.to_owned()))
            }
        }
    }
}

/// A command (opens a panel) or a page action (opens a panel for the open web page).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contribution {
    pub id: String,
    pub title: String,
    pub keywords: Vec<String>,
    pub panel: RelPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: Option<RelPath>,
    pub permissions: Vec<Permission>,
    pub commands: Vec<Contribution>,
    pub page_actions: Vec<Contribution>,
}

impl Manifest {
    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    /// The hosts it may fetch from (`network:` permissions), in order.
    pub fn network_hosts(&self) -> Vec<&str> {
        self.permissions
            .iter()
            .filter_map(|p| match p {
                Permission::Network(host) => Some(host.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Every file the manifest names (the icon and the panels), for the registry to check.
    pub fn named_files(&self) -> Vec<&RelPath> {
        self.icon
            .iter()
            .chain(self.commands.iter().map(|c| &c.panel))
            .chain(self.page_actions.iter().map(|c| &c.panel))
            .collect()
    }
}

/// An extension id: `^[a-z0-9][a-z0-9-]{1,31}$` (2 to 32 characters), and never a Windows device
/// name (`con`, `nul`, `com1` …), which would name a device as a file (its storage file).
pub fn is_valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (2..=32).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
        && !paths::DEVICES.contains(&id.to_ascii_lowercase().as_str())
}

/// Unicode's format characters (general category Cf, Unicode 15): invisible, and some — the bidi
/// controls U+202A–U+202E and U+2066–U+2069, U+200E/F — reorder the text around them.
const FORMAT_CHARS: [(u32, u32); 21] = [
    (0x00AD, 0x00AD),
    (0x0600, 0x0605),
    (0x061C, 0x061C),
    (0x06DD, 0x06DD),
    (0x070F, 0x070F),
    (0x0890, 0x0891),
    (0x08E2, 0x08E2),
    (0x180E, 0x180E),
    (0x200B, 0x200F),
    (0x202A, 0x202E),
    (0x2060, 0x2064),
    (0x2066, 0x206F),
    (0xFEFF, 0xFEFF),
    (0xFFF9, 0xFFFB),
    (0x110BD, 0x110BD),
    (0x110CD, 0x110CD),
    (0x13430, 0x1343F),
    (0x1BCA0, 0x1BCA3),
    (0x1D173, 0x1D17A),
    (0xE0001, 0xE0001),
    (0xE0020, 0xE007F),
];

/// Whether `c` is invisible or reorders text: a control (Cc) or a format character (Cf).
pub fn is_hidden_char(c: char) -> bool {
    let code = u32::from(c);
    c.is_control()
        || FORMAT_CHARS
            .iter()
            .any(|(low, high)| (*low..=*high).contains(&code))
}

/// Whether a name reads as a web address — a URL, a host name like `paypal.com`, an IP address —
/// which the toolbar could otherwise pass off as the page's domain.
pub fn looks_like_address(name: &str) -> bool {
    let name = name.trim().trim_end_matches('.').to_ascii_lowercase();
    if name.contains("://") || name.starts_with("www.") || name.parse::<std::net::IpAddr>().is_ok()
    {
        return true;
    }
    let labels: Vec<&str> = name.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty() && label.chars().all(|c| c.is_alphanumeric() || c == '-')
        })
        && labels
            .last()
            .is_some_and(|tld| tld.chars().count() >= 2 && tld.chars().all(char::is_alphabetic))
}

/// A contribution id: `^[a-z0-9][a-z0-9-]{0,31}$`.
fn is_valid_item_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (1..=32).contains(&bytes.len())
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// A host for `network:` — lower-case DNS names only, with at least two labels after an optional
/// `*.` (so `*.com` is refused); no IP addresses, ports, paths or schemes. Fetches go to
/// `https://<host>` only.
pub fn is_valid_network_host(host: &str) -> bool {
    let name = host.strip_prefix("*.").unwrap_or(host);
    if name.len() > 253 {
        return false;
    }
    let labels: Vec<&str> = name.split('.').collect();
    let good_label = |label: &&str| {
        let bytes = label.as_bytes();
        (1..=63).contains(&bytes.len())
            && bytes
                .iter()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    };
    labels.len() >= 2
        && labels.iter().all(good_label)
        // The last label is a name, never a number (no IPv4 addresses).
        && labels
            .last()
            .is_some_and(|tld| !tld.bytes().all(|b| b.is_ascii_digit()))
}

/// Text from the manifest: trimmed, 1..=`max` characters (or 0 when `optional`), no control
/// characters.
fn text(map: &Map<String, Value>, key: &str, max: usize, optional: bool) -> Result<String, String> {
    let value = match map.get(key) {
        None if optional => return Ok(String::new()),
        None => return Err(format!("`{key}` is missing")),
        Some(value) => value,
    };
    let Some(text) = value.as_str() else {
        return Err(format!("`{key}` must be text"));
    };
    let text = text.trim();
    let chars = text.chars().count();
    if text.chars().any(is_hidden_char) {
        return Err(format!(
            "`{key}` has control or invisible formatting characters"
        ));
    }
    if chars > max || (chars == 0 && !optional) {
        return Err(format!("`{key}` must be 1 to {max} characters"));
    }
    Ok(text.to_owned())
}

fn only_keys(map: &Map<String, Value>, allowed: &[&str], what: &str) -> Result<(), String> {
    match map.keys().find(|key| !allowed.contains(&key.as_str())) {
        Some(key) => Err(format!("{what} has an unknown field `{key}`")),
        None => Ok(()),
    }
}

/// A path in the manifest: relative to the extension's folder (no leading `/`), plain names.
fn path(value: &str, what: &str) -> Result<RelPath, String> {
    if value.starts_with('/') {
        return Err(format!(
            "{what} `{value}`: paths are relative to the extension's folder"
        ));
    }
    paths::clean(value).map_err(|err| format!("{what} `{value}`: {err}"))
}

fn contributions(value: Option<&Value>, what: &str) -> Result<Vec<Contribution>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let Some(list) = value.as_array() else {
        return Err(format!("`contributes.{what}` must be a list"));
    };
    if list.len() > MAX_ITEMS {
        return Err(format!("at most {MAX_ITEMS} {what}"));
    }
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for item in list {
        let Some(map) = item.as_object() else {
            return Err(format!("each of `contributes.{what}` must be an object"));
        };
        only_keys(map, &["id", "title", "keywords", "panel"], what)?;
        let id = text(map, "id", 32, false)?;
        if !is_valid_item_id(&id) || !seen.insert(id.clone()) {
            return Err(format!("{what}: `{id}` is not a unique id (a-z, 0-9, -)"));
        }
        let title = text(map, "title", 60, false)?;
        let keywords = match map.get("keywords") {
            None => Vec::new(),
            Some(Value::Array(words)) if words.len() <= MAX_KEYWORDS => words
                .iter()
                .map(|word| match word.as_str().map(str::trim) {
                    Some(w)
                        if (1..=32).contains(&w.chars().count())
                            && !w.chars().any(is_hidden_char) =>
                    {
                        Ok(w.to_owned())
                    }
                    _ => Err(format!("{what} `{id}`: keywords are 1 to 32 characters")),
                })
                .collect::<Result<_, _>>()?,
            Some(_) => return Err(format!("{what} `{id}`: at most {MAX_KEYWORDS} keywords")),
        };
        let panel = path(&text(map, "panel", 200, false)?, "panel")?;
        if panel.extension().as_deref() != Some("html") {
            return Err(format!("{what} `{id}`: the panel must be an .html file"));
        }
        items.push(Contribution {
            id,
            title,
            keywords,
            panel,
        });
    }
    Ok(items)
}

/// Parses and checks `extension.json`.
pub fn parse(bytes: &[u8]) -> Result<Manifest, String> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err("extension.json is larger than 64 KB".to_owned());
    }
    let value: Value =
        serde_json::from_slice(bytes).map_err(|err| format!("extension.json: {err}"))?;
    let Some(map) = value.as_object() else {
        return Err("extension.json must be an object".to_owned());
    };
    only_keys(
        map,
        &[
            "$schema",
            "id",
            "name",
            "version",
            "description",
            "author",
            "icon",
            "apiVersion",
            "permissions",
            "contributes",
        ],
        "extension.json",
    )?;
    if map.get("apiVersion").and_then(Value::as_u64) != Some(API_VERSION) {
        return Err(format!("`apiVersion` must be {API_VERSION}"));
    }
    let id = text(map, "id", 32, false)?;
    if !is_valid_id(&id) {
        return Err(format!(
            "`id` `{id}` must be 2 to 32 characters: a-z, 0-9 and -, not starting with -"
        ));
    }
    let version = text(map, "version", 32, false)?;
    if !version
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+'))
    {
        return Err("`version` may use letters, digits, . - +".to_owned());
    }
    let icon = match text(map, "icon", 200, true)? {
        icon if icon.is_empty() => None,
        icon => {
            let icon = path(&icon, "icon")?;
            if !matches!(icon.extension().as_deref(), Some("svg" | "png")) {
                return Err("`icon` must be an .svg or .png file".to_owned());
            }
            Some(icon)
        }
    };
    let permissions = match map.get("permissions") {
        None => Vec::new(),
        Some(Value::Array(list)) if list.len() <= MAX_ITEMS => {
            let mut permissions: Vec<Permission> = Vec::new();
            for item in list {
                let name = item.as_str().unwrap_or_default();
                let permission = Permission::parse(name)
                    .ok_or_else(|| format!("unknown permission `{name}`"))?;
                if permissions.contains(&permission) {
                    return Err(format!("permission `{name}` is listed twice"));
                }
                permissions.push(permission);
            }
            permissions
        }
        Some(_) => {
            return Err(format!(
                "`permissions` must be a list of at most {MAX_ITEMS}"
            ));
        }
    };
    let (commands, page_actions) = match map.get("contributes") {
        None => (Vec::new(), Vec::new()),
        Some(Value::Object(contributes)) => {
            only_keys(contributes, &["commands", "pageActions"], "`contributes`")?;
            (
                contributions(contributes.get("commands"), "commands")?,
                contributions(contributes.get("pageActions"), "pageActions")?,
            )
        }
        Some(_) => return Err("`contributes` must be an object".to_owned()),
    };
    let name = text(map, "name", 40, false)?;
    if looks_like_address(&name) {
        return Err(format!(
            "`name` `{name}` reads like a web address; give the extension a name"
        ));
    }
    Ok(Manifest {
        name,
        description: text(map, "description", 200, true)?,
        author: text(map, "author", 80, true)?,
        id,
        version,
        icon,
        permissions,
        commands,
        page_actions,
    })
}

#[cfg(test)]
mod tests;
