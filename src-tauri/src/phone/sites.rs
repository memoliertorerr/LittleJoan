//! Phone mode per site (pure; LJ36, LJ40): which agent a site gets (`siteModes`, then a short
//! built-in list on WebKit), which key a request for "this site" is kept under, what changing a
//! site's mode does to the table, what the toolbar and the command bar are told about it, and
//! what the page must do when its agent changed.
//!
//! - **The site of a phone address is its desktop one:** `m.youtube.com` is kept as `youtube.com`,
//!   `en.m.wikipedia.org` as `en.wikipedia.org`, and any `m.` / `mobile.` in front of a name as the
//!   name without it (`m.facebook.com` → `facebook.com`). Otherwise "Request desktop site" on a
//!   phone host would be kept for that host alone, and a site that sends a desktop agent from
//!   `m.` to `www.` and a phone agent back again would bounce between the two forever. A lookup
//!   asks the page's own host first, then that site.

use std::collections::BTreeMap;

use serde::Serialize;
use url::Url;

use crate::keys::Platform;
use crate::settings::SiteMode;
use crate::settings::site_modes::{self, MAX_SITE_MODES};

/// Sites that get the desktop agent on the WebKit engines (macOS, Linux) unless `siteModes` says
/// otherwise; their subdomains too.
pub const WEBKIT_DESKTOP_SITES: &[&str] = &["youtube.com"];

/// The built-in mode for `host` on `platform`: the desktop agent for `WEBKIT_DESKTOP_SITES` (and
/// their subdomains) on macOS and Linux, nothing elsewhere.
pub fn built_in(host: &str, platform: Platform) -> Option<SiteMode> {
    let webkit = matches!(platform, Platform::MacOs | Platform::Linux);
    let listed = site_modes::candidates(host)
        .iter()
        .any(|name| WEBKIT_DESKTOP_SITES.contains(name));
    (webkit && listed).then_some(SiteMode::Desktop)
}

/// The user's entry for `host`: for the host or its nearest parent, else for its site
/// (`mode_site`: a phone address's desktop one) or that site's nearest parent.
fn user_entry(overrides: &BTreeMap<String, SiteMode>, host: &str) -> Option<SiteMode> {
    site_modes::lookup(overrides, host).or_else(|| {
        let site = mode_site(host)?;
        site_modes::lookup(overrides, &site)
    })
}

/// The mode for a page on `host` (a URL's host, lower case): the user's entry (`"auto"` included,
/// so a user can undo a built-in), else the built-in list on WebKit, else auto.
pub fn site_mode(
    host: &str,
    platform: Platform,
    overrides: &BTreeMap<String, SiteMode>,
) -> SiteMode {
    user_entry(overrides, host)
        .or_else(|| built_in(host, platform))
        .unwrap_or_default()
}

/// The desktop host of a known phone host: `m.youtube.com` → `www.youtube.com`,
/// `<lang>.m.wikipedia.org` → `<lang>.wikipedia.org`.
fn known_desktop_host(host: &str) -> Option<String> {
    if host == "m.youtube.com" {
        return Some("www.youtube.com".to_owned());
    }
    let lang = host
        .strip_suffix(".m.wikipedia.org")
        .filter(|lang| !lang.is_empty() && !lang.contains('.'))?;
    Some(format!("{lang}.wikipedia.org"))
}

/// The desktop address of a page on a known phone host — `m.youtube.com` → `www.youtube.com`,
/// `<lang>.m.wikipedia.org` → `<lang>.wikipedia.org` — with everything else kept; `None` for
/// every other URL.
pub fn desktop_url(url: &Url) -> Option<Url> {
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let desktop = known_desktop_host(url.host_str()?)?;
    let mut rewritten = url.clone();
    rewritten.set_host(Some(&desktop)).ok()?;
    Some(rewritten)
}

/// The key a mode for `host`'s site is kept under (settings' `siteModes`): `www.` and the port
/// dropped (site_modes::site_key), a known phone host's desktop one, and an `m.` or `mobile.`
/// label in front of a name of two labels or more dropped. `None` for what can't be a key.
pub fn mode_site(host: &str) -> Option<String> {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    let desktop = known_desktop_host(&host).unwrap_or_else(|| {
        ["m.", "mobile."]
            .iter()
            .find_map(|prefix| host.strip_prefix(prefix))
            .filter(|rest| rest.contains('.'))
            .map_or_else(|| host.clone(), str::to_owned)
    });
    site_modes::site_key(&desktop)
}

/// Why a mode couldn't be kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page isn't a web page (the start page, an extension's panel), or has no host.
    NoSite,
    /// The table holds `MAX_SITE_MODES` sites already.
    Full,
}

impl Refusal {
    pub fn message(self) -> &'static str {
        match self {
            Refusal::NoSite => "Phone or desktop works on a web page.",
            Refusal::Full => "Little Joan keeps at most 500 sites; remove some in Settings first.",
        }
    }
}

/// The table after asking for `mode` on the site of a page on `host`: `phone` and `desktop` are
/// kept for the site (`mode_site`); `auto` removes the site's entry — and keeps `"auto"` for it
/// when something else would still decide (a built-in, or an entry for a parent domain), so the
/// width really decides afterwards.
pub fn with_site_mode(
    table: &BTreeMap<String, SiteMode>,
    host: &str,
    mode: SiteMode,
    platform: Platform,
) -> Result<BTreeMap<String, SiteMode>, Refusal> {
    let key = mode_site(host).ok_or(Refusal::NoSite)?;
    let mut next = table.clone();
    match mode {
        SiteMode::Auto => {
            next.remove(&key);
            if site_mode(host, platform, &next) != SiteMode::Auto {
                next.insert(key.clone(), SiteMode::Auto);
            }
        }
        SiteMode::Phone | SiteMode::Desktop => {
            next.insert(key.clone(), mode);
        }
    }
    if next.len() > MAX_SITE_MODES && !table.contains_key(&key) {
        return Err(Refusal::Full);
    }
    Ok(next)
}

/// What the toolbar's pill and the command bar's rows are told (`site_mode_info`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteInfo {
    /// The site (`mode_site`): what a request is kept under.
    pub host: String,
    /// The site's mode now (its entry, a built-in, or auto).
    pub mode: SiteMode,
    /// The mode comes from the built-in list (no entry of the user's applies).
    pub built_in: bool,
    /// The agent the page has: "phone" or "desktop".
    pub agent: &'static str,
}

/// The info for a page on `host` whose agent is the phone one (`mobile`) or not.
pub fn site_info(
    host: &str,
    platform: Platform,
    overrides: &BTreeMap<String, SiteMode>,
    mobile: bool,
) -> Option<SiteInfo> {
    let site = mode_site(host)?;
    let entry = user_entry(overrides, host);
    let built_in = entry.is_none() && built_in(host, platform).is_some();
    Some(SiteInfo {
        host: site,
        mode: site_mode(host, platform, overrides),
        built_in,
        agent: if mobile { "phone" } else { "desktop" },
    })
}

/// What the open page must do after its agent changed (`switched`: the new agent, `Some(true)` =
/// phone; `None` = it didn't change).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reaction {
    /// Nothing: the agent is the one it had.
    Stay,
    /// Load the page again, so the site answers for the new agent.
    Reload,
    /// Load this address instead: the desktop one of a known phone address, going to desktop.
    Load(Url),
}

/// The page at `url` after `switched` (see `Reaction`).
pub fn reaction(switched: Option<bool>, url: &Url) -> Reaction {
    match switched {
        None => Reaction::Stay,
        Some(true) => Reaction::Reload,
        Some(false) => desktop_url(url).map_or(Reaction::Reload, Reaction::Load),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUTO: SiteMode = SiteMode::Auto;
    const PLATFORMS: [Platform; 3] = [Platform::MacOs, Platform::Linux, Platform::Windows];

    fn table(entries: &[(&str, SiteMode)]) -> BTreeMap<String, SiteMode> {
        entries
            .iter()
            .map(|(host, mode)| ((*host).to_owned(), *mode))
            .collect()
    }

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn youtube_gets_the_desktop_agent_on_webkit_only() {
        let none = BTreeMap::new();
        for host in [
            "youtube.com",
            "www.youtube.com",
            "m.youtube.com",
            "music.youtube.com",
        ] {
            assert_eq!(
                site_mode(host, Platform::MacOs, &none),
                SiteMode::Desktop,
                "{host}"
            );
            assert_eq!(
                site_mode(host, Platform::Linux, &none),
                SiteMode::Desktop,
                "{host}"
            );
            assert_eq!(site_mode(host, Platform::Windows, &none), AUTO, "{host}");
        }
        for host in [
            "notyoutube.com",
            "youtube.com.example",
            "example.com",
            "127.0.0.1",
        ] {
            assert_eq!(site_mode(host, Platform::MacOs, &none), AUTO, "{host}");
        }
    }

    #[test]
    fn a_user_entry_wins_for_the_host_or_a_parent() {
        let phone = table(&[("youtube.com", SiteMode::Phone)]);
        assert_eq!(
            site_mode("m.youtube.com", Platform::MacOs, &phone),
            SiteMode::Phone
        );
        assert_eq!(
            site_mode("youtube.com", Platform::Linux, &phone),
            SiteMode::Phone
        );
        let desktop = table(&[("wikipedia.org", SiteMode::Desktop)]);
        assert_eq!(
            site_mode("en.m.wikipedia.org", Platform::Windows, &desktop),
            SiteMode::Desktop
        );
        let ip = table(&[("127.0.0.1", SiteMode::Desktop)]);
        assert_eq!(
            site_mode("127.0.0.1", Platform::Windows, &ip),
            SiteMode::Desktop
        );
    }

    #[test]
    fn auto_undoes_a_built_in() {
        let auto = table(&[("youtube.com", AUTO)]);
        assert_eq!(site_mode("www.youtube.com", Platform::MacOs, &auto), AUTO);
        let one = table(&[("m.youtube.com", AUTO)]);
        assert_eq!(site_mode("m.youtube.com", Platform::MacOs, &one), AUTO);
        assert_eq!(
            site_mode("www.youtube.com", Platform::MacOs, &one),
            SiteMode::Desktop,
            "only that host"
        );
    }

    #[test]
    fn a_phone_address_follows_its_sites_entry() {
        // The entries are kept for the desktop site; its phone address finds them too.
        let wiki = table(&[("en.wikipedia.org", SiteMode::Phone)]);
        assert_eq!(
            site_mode("en.m.wikipedia.org", Platform::Windows, &wiki),
            SiteMode::Phone
        );
        let facebook = table(&[("facebook.com", SiteMode::Desktop)]);
        for host in ["m.facebook.com", "www.facebook.com", "mobile.facebook.com"] {
            assert_eq!(
                site_mode(host, Platform::Windows, &facebook),
                SiteMode::Desktop,
                "{host}"
            );
        }
    }

    #[test]
    fn known_phone_addresses_go_to_their_desktop_ones() {
        assert_eq!(
            desktop_url(&url("https://m.youtube.com/watch?v=x")).map(String::from),
            Some("https://www.youtube.com/watch?v=x".to_owned())
        );
        assert_eq!(
            desktop_url(&url("https://en.m.wikipedia.org/wiki/X#History")).map(String::from),
            Some("https://en.wikipedia.org/wiki/X#History".to_owned())
        );
        for other in [
            "https://www.youtube.com/watch?v=x",
            "https://m.example.com/",
            "https://mobile.twitter.com/",
            "https://m.wikipedia.org/",
            "https://a.b.m.wikipedia.org/",
            "https://en.wikipedia.org/wiki/X",
            "ljext://m.youtube.com/",
        ] {
            assert_eq!(desktop_url(&url(other)), None, "{other}");
        }
    }

    #[test]
    fn a_sites_key_is_its_desktop_name() {
        for (host, key) in [
            ("www.youtube.com", "youtube.com"),
            ("m.youtube.com", "youtube.com"),
            ("en.m.wikipedia.org", "en.wikipedia.org"),
            ("m.facebook.com", "facebook.com"),
            ("mobile.twitter.com", "twitter.com"),
            ("WWW.Example.COM", "example.com"),
            ("127.0.0.1", "127.0.0.1"),
            ("[::1]", "[::1]"),
            // A name that is only "m." something keeps it: there is nothing left to be the site.
            ("m.com", "m.com"),
            ("news.m.example.org", "news.m.example.org"),
        ] {
            assert_eq!(mode_site(host).as_deref(), Some(key), "{host}");
        }
        assert_eq!(mode_site(""), None);
        assert_eq!(mode_site("a b"), None);
    }

    #[test]
    fn asking_for_phone_or_desktop_keeps_it_for_the_site() {
        for platform in PLATFORMS {
            let got = with_site_mode(&BTreeMap::new(), "127.0.0.1", SiteMode::Desktop, platform);
            assert_eq!(got, Ok(table(&[("127.0.0.1", SiteMode::Desktop)])));
            let got = with_site_mode(
                &table(&[("127.0.0.1", SiteMode::Desktop)]),
                "127.0.0.1",
                SiteMode::Phone,
                platform,
            );
            assert_eq!(got, Ok(table(&[("127.0.0.1", SiteMode::Phone)])));
            let got = with_site_mode(&BTreeMap::new(), "m.youtube.com", SiteMode::Phone, platform);
            assert_eq!(got, Ok(table(&[("youtube.com", SiteMode::Phone)])));
        }
    }

    #[test]
    fn automatic_removes_the_entry_or_keeps_auto_over_a_built_in() {
        let mine = table(&[
            ("127.0.0.1", SiteMode::Desktop),
            ("example.com", SiteMode::Phone),
        ]);
        let got = with_site_mode(&mine, "127.0.0.1", AUTO, Platform::Windows);
        assert_eq!(got, Ok(table(&[("example.com", SiteMode::Phone)])));
        // YouTube on WebKit: without an entry the built-in would decide, so "auto" is kept.
        for platform in [Platform::MacOs, Platform::Linux] {
            let got = with_site_mode(&BTreeMap::new(), "www.youtube.com", AUTO, platform);
            assert_eq!(got, Ok(table(&[("youtube.com", AUTO)])));
            assert_eq!(
                site_mode("www.youtube.com", platform, &got.unwrap_or_default()),
                AUTO
            );
        }
        // On Windows nothing is built in: the entry just goes.
        let got = with_site_mode(
            &table(&[("youtube.com", SiteMode::Phone)]),
            "m.youtube.com",
            AUTO,
            Platform::Windows,
        );
        assert_eq!(got, Ok(BTreeMap::new()));
        // A parent's entry would still decide: "auto" for this site.
        let parent = table(&[("wikipedia.org", SiteMode::Desktop)]);
        let got = with_site_mode(&parent, "en.wikipedia.org", AUTO, Platform::Windows);
        assert_eq!(
            got,
            Ok(table(&[
                ("wikipedia.org", SiteMode::Desktop),
                ("en.wikipedia.org", AUTO)
            ]))
        );
    }

    #[test]
    fn a_page_without_a_site_or_a_full_table_is_refused() {
        assert_eq!(
            with_site_mode(&BTreeMap::new(), "", SiteMode::Phone, Platform::MacOs),
            Err(Refusal::NoSite)
        );
        let full: BTreeMap<String, SiteMode> = (0..MAX_SITE_MODES)
            .map(|i| (format!("site{i}.example"), SiteMode::Phone))
            .collect();
        assert_eq!(
            with_site_mode(&full, "new.example", SiteMode::Phone, Platform::MacOs),
            Err(Refusal::Full)
        );
        // An existing site can still change, and removing always works.
        assert!(with_site_mode(&full, "site7.example", SiteMode::Desktop, Platform::MacOs).is_ok());
        assert!(with_site_mode(&full, "new.example", AUTO, Platform::MacOs).is_ok());
    }

    #[test]
    fn the_info_says_the_site_its_mode_where_it_comes_from_and_the_agent() {
        let none = BTreeMap::new();
        assert_eq!(
            site_info("m.youtube.com", Platform::MacOs, &none, false),
            Some(SiteInfo {
                host: "youtube.com".to_owned(),
                mode: SiteMode::Desktop,
                built_in: true,
                agent: "desktop",
            })
        );
        let mine = table(&[("youtube.com", SiteMode::Phone)]);
        let info = site_info("www.youtube.com", Platform::MacOs, &mine, true).expect("info");
        assert_eq!(
            (info.mode, info.built_in, info.agent),
            (SiteMode::Phone, false, "phone")
        );
        let info = site_info("127.0.0.1", Platform::Windows, &none, true).expect("info");
        assert_eq!(
            (info.host.as_str(), info.mode, info.built_in),
            ("127.0.0.1", AUTO, false)
        );
        let json = serde_json::to_value(&info).expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "host": "127.0.0.1", "mode": "auto", "builtIn": false, "agent": "phone" })
        );
    }

    #[test]
    fn a_changed_agent_reloads_once_or_goes_to_the_desktop_address() {
        let page = url("https://example.com/a");
        assert_eq!(reaction(None, &page), Reaction::Stay);
        assert_eq!(reaction(Some(true), &page), Reaction::Reload);
        assert_eq!(reaction(Some(false), &page), Reaction::Reload);
        let phone = url("https://m.youtube.com/watch?v=x");
        assert_eq!(
            reaction(Some(false), &phone),
            Reaction::Load(url("https://www.youtube.com/watch?v=x"))
        );
        assert_eq!(
            reaction(Some(true), &phone),
            Reaction::Reload,
            "to phone: as it is"
        );
        assert_eq!(reaction(None, &phone), Reaction::Stay);
    }
}
