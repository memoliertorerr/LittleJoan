//! The `siteModes` field of settings.json: per site, whether the page gets the phone or the
//! desktop user agent, or follows the window's width (phone mode, phone.rs). Shape:
//! `{ "<host>": "auto" | "phone" | "desktop" }`. Keys are lower-case host names without `www.` and
//! without a port — a mode is per host, not per port —, IP addresses as they are (`127.0.0.1`,
//! `[::1]`). Parsing is tolerant like the rest of the settings: a bad entry is dropped alone, and
//! at most `MAX_SITE_MODES` are kept. A lookup walks a name up to its last two labels
//! (`en.m.wikipedia.org` → `m.wikipedia.org` → `wikipedia.org`), so an entry covers its
//! subdomains; an IP address is never walked.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::{Host, Url};

/// The most entries kept.
pub const MAX_SITE_MODES: usize = 500;

/// What a site gets: the width decides (auto), or always the phone / the desktop user agent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SiteMode {
    #[default]
    Auto,
    Phone,
    Desktop,
}

impl SiteMode {
    /// The name settings.json uses.
    pub fn name(self) -> &'static str {
        match self {
            SiteMode::Auto => "auto",
            SiteMode::Phone => "phone",
            SiteMode::Desktop => "desktop",
        }
    }

    fn from_name(name: &str) -> Option<SiteMode> {
        match name {
            "auto" => Some(SiteMode::Auto),
            "phone" => Some(SiteMode::Phone),
            "desktop" => Some(SiteMode::Desktop),
            _ => None,
        }
    }
}

/// `siteModes` from settings.json: valid entries only (keys through `site_key`, values one of the
/// three names), the first of two keys that name the same site, at most `MAX_SITE_MODES`. Anything
/// but an object is an empty table.
pub fn parse(value: Option<&Value>) -> BTreeMap<String, SiteMode> {
    let mut kept = BTreeMap::new();
    for (raw, mode) in value.and_then(Value::as_object).into_iter().flatten() {
        if kept.len() >= MAX_SITE_MODES {
            break;
        }
        let (Some(key), Some(mode)) = (site_key(raw), mode.as_str().and_then(SiteMode::from_name))
        else {
            continue;
        };
        kept.entry(key).or_insert(mode);
    }
    kept
}

/// A settings key as the table keeps it: a host name in lower case without `www.` (IDNs in
/// their ASCII form) or an IP address; `None` for anything with a scheme, a port, a path, a user,
/// or no host at all.
pub fn site_key(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 253 || raw.contains(['/', '\\', '?', '#', '@', ' ']) {
        return None;
    }
    let url = Url::parse(&format!("http://{raw}/")).ok()?;
    // "host:80" parses with no port (the default one); any other port is refused.
    if url.port().is_some() || raw.contains(':') && !raw.starts_with('[') {
        return None;
    }
    match url.host()? {
        Host::Domain(name) => {
            let name = name.trim_end_matches('.');
            let name = name.strip_prefix("www.").unwrap_or(name);
            let labels_ok = !name.is_empty()
                && name.split('.').all(|label| {
                    !label.is_empty()
                        && label
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                });
            labels_ok.then(|| name.to_owned())
        }
        Host::Ipv4(ip) => Some(ip.to_string()),
        Host::Ipv6(ip) => Some(format!("[{ip}]")),
    }
}

/// The names a page's host is looked up under, most specific first: the host, then each parent
/// down to its last two labels. An IP address (or a single label) is only itself.
pub fn candidates(host: &str) -> Vec<&str> {
    let host = host.trim_end_matches('.');
    if host.starts_with('[') || host.parse::<Ipv4Addr>().is_ok() {
        return vec![host];
    }
    let mut names = vec![host];
    let mut rest = host;
    while let Some((_, parent)) = rest.split_once('.') {
        if !parent.contains('.') {
            break;
        }
        names.push(parent);
        rest = parent;
    }
    names
}

/// The mode `table` gives `host` (a page URL's host, already lower case): the entry for the host
/// itself, else for its nearest parent (`candidates`).
pub fn lookup(table: &BTreeMap<String, SiteMode>, host: &str) -> Option<SiteMode> {
    candidates(host)
        .into_iter()
        .find_map(|name| table.get(name).copied())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keys_are_hosts_without_www_or_ports() {
        assert_eq!(site_key("YouTube.com").as_deref(), Some("youtube.com"));
        assert_eq!(
            site_key(" www.Example.org ").as_deref(),
            Some("example.org")
        );
        assert_eq!(site_key("m.youtube.com").as_deref(), Some("m.youtube.com"));
        assert_eq!(site_key("localhost").as_deref(), Some("localhost"));
        assert_eq!(site_key("bücher.de").as_deref(), Some("xn--bcher-kva.de"));
        assert_eq!(site_key("127.0.0.1").as_deref(), Some("127.0.0.1"));
        assert_eq!(site_key("[::1]").as_deref(), Some("[::1]"));
        for bad in [
            "",
            "https://youtube.com",
            "youtube.com/watch",
            "youtube.com:8080",
            "127.0.0.1:3000",
            "user@youtube.com",
            "you tube.com",
            "::1",
            "a..b",
        ] {
            assert_eq!(site_key(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn bad_entries_are_dropped_one_by_one() {
        let table = parse(Some(&json!({
            "YouTube.com": "desktop",
            "en.m.wikipedia.org": "phone",
            "127.0.0.1": "auto",
            "example.com:8080": "phone",
            "example.org": "tablet",
            "example.net": 3,
        })));
        assert_eq!(
            table.into_iter().collect::<Vec<_>>(),
            vec![
                ("127.0.0.1".to_owned(), SiteMode::Auto),
                ("en.m.wikipedia.org".to_owned(), SiteMode::Phone),
                ("youtube.com".to_owned(), SiteMode::Desktop),
            ]
        );
        assert!(parse(Some(&json!(["youtube.com"]))).is_empty());
        assert!(parse(Some(&json!("desktop"))).is_empty());
        assert!(parse(None).is_empty());
    }

    #[test]
    fn two_spellings_of_one_site_keep_one_entry() {
        let table = parse(Some(
            &json!({ "example.com": "phone", "www.example.com": "desktop" }),
        ));
        assert_eq!(table.len(), 1);
        assert_eq!(table.get("example.com"), Some(&SiteMode::Phone));
    }

    #[test]
    fn at_most_500_entries() {
        let many: serde_json::Map<String, Value> = (0..600)
            .map(|i| (format!("site{i:03}.example"), json!("desktop")))
            .collect();
        assert_eq!(parse(Some(&Value::Object(many))).len(), MAX_SITE_MODES);
    }

    #[test]
    fn names_walk_up_to_their_last_two_labels() {
        assert_eq!(
            candidates("a.b.youtube.com"),
            vec!["a.b.youtube.com", "b.youtube.com", "youtube.com"]
        );
        assert_eq!(candidates("youtube.com"), vec!["youtube.com"]);
        assert_eq!(candidates("youtube.com."), vec!["youtube.com"]);
        assert_eq!(candidates("localhost"), vec!["localhost"]);
    }

    #[test]
    fn ip_addresses_are_never_walked() {
        assert_eq!(candidates("127.0.0.1"), vec!["127.0.0.1"]);
        assert_eq!(candidates("10.1.2.3"), vec!["10.1.2.3"]);
        assert_eq!(candidates("[::1]"), vec!["[::1]"]);
        // "0.1" is an IPv4 address too (0.0.0.1), and never a parent of 10.0.0.1.
        let table = parse(Some(&json!({ "0.1": "desktop" })));
        assert_eq!(table.keys().collect::<Vec<_>>(), ["0.0.0.1"]);
        assert_eq!(lookup(&table, "10.0.0.1"), None);
    }

    #[test]
    fn the_nearest_entry_wins() {
        let table = parse(Some(&json!({
            "youtube.com": "phone",
            "music.youtube.com": "desktop",
        })));
        assert_eq!(lookup(&table, "music.youtube.com"), Some(SiteMode::Desktop));
        assert_eq!(
            lookup(&table, "eu.music.youtube.com"),
            Some(SiteMode::Desktop)
        );
        assert_eq!(lookup(&table, "m.youtube.com"), Some(SiteMode::Phone));
        assert_eq!(lookup(&table, "www.youtube.com"), Some(SiteMode::Phone));
        assert_eq!(lookup(&table, "youtube.de"), None);
        // A parent never reaches past two labels: "com" isn't a site.
        let com = parse(Some(&json!({ "com": "desktop" })));
        assert_eq!(lookup(&com, "youtube.com"), None);
    }
}
