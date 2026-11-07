//! The `pinnedSites` field (LJ50): the sites that lead the command bar's empty state, in the order
//! they were pinned — at most 8 `{ url, title }`, absolute http(s) addresses only, one per URL.
//! Tolerant: a bad entry is dropped on its own, as are repeats and anything past the eighth.
//! Pinned from the command bar's quick commands ("Pin this site", the open web page) and unpinned
//! there or with Shift+Delete on a pinned row (commands/quick.rs); never from a page.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

/// The most pinned sites.
pub const MAX_PINNED: usize = 8;
/// The longest title kept (characters).
pub const MAX_TITLE_CHARS: usize = 300;
/// The longest address kept (bytes), as history keeps them.
pub const MAX_URL_BYTES: usize = 4_096;

/// One pinned site: what it opens, and what its row says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedSite {
    pub url: String,
    pub title: String,
}

/// `text` as a pinned address: an absolute http(s) URL with a host and no user name or password,
/// its fragment dropped, at most 4,096 bytes; anything else is none.
pub fn clean_url(text: &str) -> Option<String> {
    let mut url = Url::parse(text.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none_or(str::is_empty)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    url.set_fragment(None);
    let url = String::from(url);
    (url.len() <= MAX_URL_BYTES).then_some(url)
}

/// A title as a row shows it: on one line (control characters become spaces), without the
/// direction overrides that could make it read as something else, trimmed, at most 300 characters.
pub fn clean_title(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !matches!(c, '\u{200e}' | '\u{200f}' | '\u{061c}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .chars()
        .take(MAX_TITLE_CHARS)
        .collect()
}

/// A site to pin, or none when the address can't be one.
pub fn site(url: &str, title: &str) -> Option<PinnedSite> {
    Some(PinnedSite {
        url: clean_url(url)?,
        title: clean_title(title),
    })
}

/// settings.json's list: each entry an object with a `url` (and a `title`, else none) — bad ones,
/// repeats and those past the eighth dropped; a wrong type is an empty list.
pub fn parse(value: Option<&Value>) -> Vec<PinnedSite> {
    let mut kept: Vec<PinnedSite> = Vec::new();
    for item in value.and_then(Value::as_array).into_iter().flatten() {
        if kept.len() == MAX_PINNED {
            break;
        }
        let Some(url) = item.get("url").and_then(Value::as_str) else {
            continue;
        };
        let title = item.get("title").and_then(Value::as_str).unwrap_or("");
        if let Some(site) = site(url, title)
            && !kept.iter().any(|k| k.url == site.url)
        {
            kept.push(site);
        }
    }
    kept
}

/// Whether `url` is pinned.
pub fn is_pinned(list: &[PinnedSite], url: &str) -> bool {
    clean_url(url).is_some_and(|url| list.iter().any(|site| site.url == url))
}

/// The list with `site` pinned last — a URL pinned already keeps its place and gets the new title.
/// Refused with eight pinned.
pub fn pin(list: &[PinnedSite], site: PinnedSite) -> Result<Vec<PinnedSite>, &'static str> {
    let mut next = list.to_vec();
    if let Some(known) = next.iter_mut().find(|known| known.url == site.url) {
        known.title = site.title;
        return Ok(next);
    }
    if next.len() >= MAX_PINNED {
        return Err("Eight sites are pinned already: unpin one first (Shift+Delete on its row).");
    }
    next.push(site);
    Ok(next)
}

/// The list without `url` (nothing changes when it isn't pinned).
pub fn unpin(list: &[PinnedSite], url: &str) -> Vec<PinnedSite> {
    let url = clean_url(url);
    list.iter()
        .filter(|site| Some(&site.url) != url.as_ref())
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sites(urls: &[&str]) -> Vec<PinnedSite> {
        urls.iter()
            .map(|url| site(url, "T").expect("a site"))
            .collect()
    }

    #[test]
    fn only_web_addresses_are_pinned() {
        assert_eq!(
            clean_url(" https://example.com/a?b=1#top "),
            Some("https://example.com/a?b=1".into())
        );
        assert_eq!(
            clean_url("http://127.0.0.1:5000/smoke.html"),
            Some("http://127.0.0.1:5000/smoke.html".into())
        );
        for bad in [
            "",
            "example.com",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,hi",
            "ljext://hello.localhost/index.html",
            "tauri://localhost/index.html",
            "https://user:secret@example.com/",
            "https://user@example.com/",
            "about:blank",
        ] {
            assert_eq!(clean_url(bad), None, "{bad}");
        }
        let long = format!("https://example.com/{}", "a".repeat(MAX_URL_BYTES));
        assert_eq!(clean_url(&long), None);
    }

    #[test]
    fn titles_are_text_on_one_line() {
        assert_eq!(clean_title("  Example\nDomain\u{202e} "), "Example Domain");
        assert_eq!(
            clean_title(&"é".repeat(400)).chars().count(),
            MAX_TITLE_CHARS
        );
    }

    #[test]
    fn the_list_is_parsed_tolerantly() {
        let value = json!([
            { "url": "https://a.example/", "title": "A" },
            { "url": "https://a.example/#again", "title": "A again" },
            { "url": "ftp://b.example/", "title": "B" },
            { "title": "no url" },
            "https://c.example/",
            { "url": "https://d.example/" },
            { "url": 3, "title": "D" },
        ]);
        let got = parse(Some(&value));
        assert_eq!(
            got,
            vec![
                PinnedSite {
                    url: "https://a.example/".into(),
                    title: "A".into()
                },
                PinnedSite {
                    url: "https://d.example/".into(),
                    title: String::new()
                },
            ]
        );
        assert!(parse(Some(&json!({ "url": "https://a.example/" }))).is_empty());
        assert!(parse(None).is_empty());
    }

    #[test]
    fn at_most_eight_are_kept() {
        let value: Vec<Value> = (0..12)
            .map(|i| json!({ "url": format!("https://site{i}.example/"), "title": i.to_string() }))
            .collect();
        let got = parse(Some(&Value::Array(value)));
        assert_eq!(got.len(), MAX_PINNED);
        assert_eq!(got[0].url, "https://site0.example/");
        assert_eq!(got[7].url, "https://site7.example/");
    }

    #[test]
    fn pinning_adds_last_once_and_refuses_a_ninth() {
        let list = sites(&["https://a.example/", "https://b.example/"]);
        let more = pin(&list, site("https://c.example/", "C").expect("a site")).expect("room");
        assert_eq!(
            more.last().map(|s| s.url.as_str()),
            Some("https://c.example/")
        );
        let again = pin(
            &more,
            site("https://a.example/#x", "A, renamed").expect("a site"),
        )
        .expect("pinned already");
        assert_eq!(again.len(), 3, "one per URL");
        assert_eq!(again[0].title, "A, renamed");
        let full = sites(&[
            "https://1.example/",
            "https://2.example/",
            "https://3.example/",
            "https://4.example/",
            "https://5.example/",
            "https://6.example/",
            "https://7.example/",
            "https://8.example/",
        ]);
        assert!(pin(&full, site("https://9.example/", "9").expect("a site")).is_err());
        assert!(pin(&full, site("https://8.example/", "8").expect("a site")).is_ok());
    }

    #[test]
    fn unpinning_removes_that_url_only() {
        let list = sites(&["https://a.example/", "https://b.example/"]);
        assert_eq!(
            unpin(&list, "https://a.example/#frag"),
            sites(&["https://b.example/"])
        );
        assert_eq!(unpin(&list, "https://c.example/"), list);
        assert_eq!(unpin(&list, "not a url"), list);
        assert!(is_pinned(&list, "https://b.example/"));
        assert!(!is_pinned(&list, "https://b.example/other"));
    }

    #[test]
    fn a_change_pins_and_unpins_and_nothing_else_touches_the_pins() {
        use crate::settings::{Change, Pin, Settings, apply_change};
        let current = Settings::default();
        let pinned = apply_change(
            &current,
            &Change {
                pin: Some(Pin::Add(site("https://a.example/", "T").expect("a site"))),
                ..Change::default()
            },
        )
        .expect("room");
        assert_eq!(pinned.pinned_sites, sites(&["https://a.example/"]));
        assert_eq!(
            Settings {
                pinned_sites: Vec::new(),
                ..pinned.clone()
            },
            current,
            "nothing else moved"
        );
        // Settings' Save sends no pins: they stay.
        let saved = apply_change(
            &pinned,
            &Change {
                mobile_sites: Some(false),
                ..Change::default()
            },
        )
        .expect("valid");
        assert_eq!(saved.pinned_sites, pinned.pinned_sites);
        let unpinned = apply_change(
            &saved,
            &Change {
                pin: Some(Pin::Remove("https://a.example/".into())),
                ..Change::default()
            },
        )
        .expect("valid");
        assert!(unpinned.pinned_sites.is_empty());
        // A ninth is refused, and then nothing changes.
        let full = Settings {
            pinned_sites: (1..=8)
                .map(|i| site(&format!("https://{i}.example/"), "").expect("a site"))
                .collect(),
            ..Settings::default()
        };
        assert!(
            apply_change(
                &full,
                &Change {
                    pin: Some(Pin::Add(site("https://9.example/", "").expect("a site"))),
                    ..Change::default()
                },
            )
            .is_err()
        );
    }

    #[test]
    fn it_round_trips_as_settings_json_writes_it() {
        let list = vec![PinnedSite {
            url: "https://a.example/".into(),
            title: "A".into(),
        }];
        let json = serde_json::to_value(&list).expect("json");
        assert_eq!(json, json!([{ "url": "https://a.example/", "title": "A" }]));
        assert_eq!(parse(Some(&json)), list);
    }
}
