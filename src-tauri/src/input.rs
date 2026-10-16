//! Turns what the user typed into the start field into a URL: a web address, or a search.
//!
//! Only `http` and `https` URLs ever come out of here as addresses. Anything else — including
//! `javascript:`, `file:`, `data:`, or `about:` — is treated as text to search for, so the start
//! field can never be used to open a non-web URL.

use serde::{Deserialize, Serialize};
use url::Url;

/// The search engines the start field can use. Serialized as lowercase names in settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    #[default]
    Google,
    DuckDuckGo,
    Bing,
    Brave,
}

impl SearchEngine {
    pub const ALL: [SearchEngine; 4] = [
        SearchEngine::Google,
        SearchEngine::DuckDuckGo,
        SearchEngine::Bing,
        SearchEngine::Brave,
    ];

    /// The display name shown in settings.
    pub fn name(self) -> &'static str {
        match self {
            SearchEngine::Google => "Google",
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Bing => "Bing",
            SearchEngine::Brave => "Brave",
        }
    }

    fn endpoint(self) -> &'static str {
        match self {
            SearchEngine::Google => "https://www.google.com/search",
            SearchEngine::DuckDuckGo => "https://duckduckgo.com/",
            SearchEngine::Bing => "https://www.bing.com/search",
            SearchEngine::Brave => "https://search.brave.com/search",
        }
    }

    /// The results page for `query` (form-encoded into `q`).
    pub fn search_url(self, query: &str) -> Url {
        // The endpoints are constant, valid URLs (covered by tests), so parsing cannot fail.
        Url::parse_with_params(self.endpoint(), &[("q", query)])
            .expect("search endpoints are valid URLs")
    }
}

/// Normalizes start-field input. Returns `None` for empty input.
///
/// Rules, in order: an absolute `http`/`https` URL is used as is; something that looks like a
/// host (with optional port and path) gets `https://` — or `http://` for `localhost`,
/// `*.localhost`, and IP literals; everything else becomes a search with `engine`.
pub fn normalize(input: &str, engine: SearchEngine) -> Option<Url> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(url) = Url::parse(text)
        && matches!(url.scheme(), "http" | "https")
        && url.has_host()
    {
        return Some(url);
    }
    if let Some(host) = host_of(text) {
        let scheme = if is_local_host(host) { "http" } else { "https" };
        if let Ok(url) = Url::parse(&format!("{scheme}://{text}"))
            && url.has_host()
        {
            return Some(url);
        }
    }
    Some(engine.search_url(text))
}

/// Returns the host part if `text` looks like `host[:port][/path…]`, else `None`.
fn host_of(text: &str) -> Option<&str> {
    if text.chars().any(char::is_whitespace) || text.contains('@') {
        return None;
    }
    let end = text.find(['/', '?', '#']).unwrap_or(text.len());
    let host_port = &text[..end];
    let host = if host_port.starts_with('[') {
        // IPv6 literal: [::1] or [::1]:8080
        let close = host_port.find(']')?;
        let rest = &host_port[close + 1..];
        if !rest.is_empty() && !is_port(rest.strip_prefix(':')?) {
            return None;
        }
        &host_port[..=close]
    } else {
        match host_port.rsplit_once(':') {
            Some((host, port)) if is_port(port) => host,
            Some(_) => return None, // "javascript:…", "mailto:…", "file:" — a scheme, not a port
            None => host_port,
        }
    };
    let looks_like_host = host == "localhost"
        || host.parse::<std::net::Ipv4Addr>().is_ok()
        || (host.starts_with('[') && host.ends_with(']'))
        || is_domain(host);
    looks_like_host.then_some(host)
}

fn is_port(text: &str) -> bool {
    !text.is_empty() && text.len() <= 5 && text.chars().all(|c| c.is_ascii_digit())
}

/// A dotted name whose labels are letters/digits/hyphens and whose last label (the TLD) has at
/// least two characters and is not all digits ("v1.2" is a version, not a host).
fn is_domain(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() < 2 {
        return false;
    }
    let label_ok = |label: &&str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_alphanumeric() || c == '-')
    };
    let tld = labels.last().copied().unwrap_or_default();
    labels.iter().all(label_ok)
        && tld.chars().count() >= 2
        && !tld.chars().all(|c| c.is_ascii_digit())
}

fn is_local_host(host: &str) -> bool {
    host == "localhost"
        || host.ends_with(".localhost")
        || host.parse::<std::net::Ipv4Addr>().is_ok()
        || host.starts_with('[')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn go(input: &str) -> String {
        normalize(input, SearchEngine::Google)
            .map(|u| u.to_string())
            .unwrap_or_default()
    }

    #[test]
    fn empty_input_is_none() {
        assert_eq!(normalize("   ", SearchEngine::Google), None);
        assert_eq!(normalize("", SearchEngine::Bing), None);
    }

    #[test]
    fn bare_domains_get_https() {
        assert_eq!(go("example.com"), "https://example.com/");
        assert_eq!(
            go("  news.ycombinator.com/item?id=1  "),
            "https://news.ycombinator.com/item?id=1"
        );
        assert_eq!(go("example.com:8443/a#b"), "https://example.com:8443/a#b");
        assert_eq!(go("münchen.de"), "https://xn--mnchen-3ya.de/");
    }

    #[test]
    fn local_hosts_get_http() {
        assert_eq!(go("localhost:3000"), "http://localhost:3000/");
        assert_eq!(go("localhost"), "http://localhost/");
        assert_eq!(go("app.localhost/x"), "http://app.localhost/x");
        assert_eq!(go("192.168.1.1"), "http://192.168.1.1/");
        assert_eq!(go("[::1]:8080/"), "http://[::1]:8080/");
    }

    #[test]
    fn full_web_urls_are_kept() {
        assert_eq!(go("https://example.com/a?b=1"), "https://example.com/a?b=1");
        assert_eq!(go("HTTP://Example.COM"), "http://example.com/");
    }

    #[test]
    fn words_become_searches() {
        assert_eq!(
            go("what is rust"),
            "https://www.google.com/search?q=what+is+rust"
        );
        assert_eq!(go("v1.2"), "https://www.google.com/search?q=v1.2");
        assert_eq!(go("rust"), "https://www.google.com/search?q=rust");
        assert_eq!(
            go("me@example.com"),
            "https://www.google.com/search?q=me%40example.com"
        );
    }

    #[test]
    fn non_web_schemes_are_never_navigable() {
        for input in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "file:///C:/Windows/win.ini",
            "data:text/html,<b>x</b>",
            "about:blank",
            "ftp://example.com/x.txt",
            "mailto:me@example.com",
            "ms-settings:privacy",
        ] {
            let url = normalize(input, SearchEngine::Google).expect("non-empty input");
            assert_eq!(
                url.host_str(),
                Some("www.google.com"),
                "{input} must become a search"
            );
            assert_eq!(url.scheme(), "https");
        }
    }

    #[test]
    fn every_engine_builds_a_search_url() {
        let expected = [
            "https://www.google.com/search?q=a+b",
            "https://duckduckgo.com/?q=a+b",
            "https://www.bing.com/search?q=a+b",
            "https://search.brave.com/search?q=a+b",
        ];
        for (engine, want) in SearchEngine::ALL.iter().zip(expected) {
            assert_eq!(engine.search_url("a b").as_str(), want);
        }
    }

    #[test]
    fn engines_serialize_as_lowercase_names() {
        assert_eq!(
            serde_json::to_string(&SearchEngine::DuckDuckGo).unwrap(),
            "\"duckduckgo\""
        );
        assert_eq!(
            serde_json::from_str::<SearchEngine>("\"brave\"").unwrap(),
            SearchEngine::Brave
        );
    }
}
