//! Extension URLs and origins (pure, tested).
//!
//! Every extension has an origin of its own, so one extension's localStorage, cookies and
//! fetches never reach another's:
//!
//! | | URL | Origin |
//! |---|---|---|
//! | macOS, Linux | `ljext://<id>.localhost/<path>` | `ljext://<id>.localhost` |
//! | Windows | `http://ljext.<id>.localhost/<path>` | `http://ljext.<id>.localhost` |
//!
//! WebView2 can't load custom schemes, so wry serves every `http://ljext.*` request from the
//! `ljext` handler (custom_protocol_workaround.rs: it turns `http://ljext.` back into `ljext://`
//! before the handler sees it). The `.localhost` suffix keeps the Windows form on localhost: a
//! host like `ljext.app` could be upgraded to https by Chromium (`.app` is HSTS-preloaded), miss
//! the handler and reach the internet; `*.localhost` is never upgraded, never leaves the machine
//! and is a secure context. Every `http(s)://ljext.*` URL therefore counts as an extension URL
//! (never plain web), on every platform (`is_ext_like`).

use url::Url;

use super::manifest::is_valid_id;
use crate::keys::Platform;

/// The custom scheme every platform registers (WebView2 reaches it through `http://ljext.*`).
pub const SCHEME: &str = "ljext";
/// The host prefix of WebView2's form.
const WINDOWS_PREFIX: &str = "ljext.";
const LOCALHOST: &str = ".localhost";

/// An extension URL: whose it is, and its path as it appears in the URL (percent-encoded,
/// starting with `/`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtUrl {
    pub id: String,
    pub path: String,
}

/// Whether `url` is an extension URL in any form: `ljext:`, or `http(s)` with a host starting
/// `ljext.` (WebView2's form). Such URLs are never treated as web pages: the navigation guard
/// refuses them unless the app opened that very URL, and they are never recorded in history,
/// counted by the back button or handed to the default browser.
pub fn is_ext_like(url: &Url) -> bool {
    match url.scheme() {
        SCHEME => true,
        "http" | "https" => url
            .host_str()
            .is_some_and(|host| host.to_ascii_lowercase().starts_with(WINDOWS_PREFIX)),
        _ => false,
    }
}

/// The extension `url` belongs to, in this platform's form only (see the module docs): no port,
/// no user name or password, and a valid extension id. Anything else is `None`.
pub fn parse(url: &Url, platform: Platform) -> Option<ExtUrl> {
    if url.port().is_some() || !url.username().is_empty() || url.password().is_some() {
        return None;
    }
    let host = url.host_str()?;
    let id = match platform {
        Platform::Windows if url.scheme() == "http" => {
            host.strip_prefix(WINDOWS_PREFIX)?.strip_suffix(LOCALHOST)?
        }
        Platform::MacOs | Platform::Linux if url.scheme() == SCHEME => {
            host.strip_suffix(LOCALHOST)?
        }
        _ => return None,
    };
    is_valid_id(id).then(|| ExtUrl {
        id: id.to_owned(),
        path: url.path().to_owned(),
    })
}

/// A request's URI as the `ljext` handler receives it: `ljext://<id>.localhost/<path>` on every
/// platform (wry turns WebView2's `http://ljext.` form back before the handler runs).
pub fn parse_request(uri: &str) -> Option<ExtUrl> {
    parse(&Url::parse(uri).ok()?, Platform::MacOs)
}

/// The extension's origin as the engine writes it in the `Origin` header.
pub fn origin(id: &str, platform: Platform) -> String {
    match platform {
        Platform::Windows => format!("http://{WINDOWS_PREFIX}{id}{LOCALHOST}"),
        Platform::MacOs | Platform::Linux => format!("{SCHEME}://{id}{LOCALHOST}"),
    }
}

/// The URL of the extension's file `path` (a checked relative path, paths.rs) in this platform's
/// form.
pub fn file_url(id: &str, path: &str, platform: Platform) -> Option<Url> {
    Url::parse(&format!("{}/{path}", origin(id, platform))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn each_platform_has_its_own_form() {
        assert_eq!(
            parse(&url("ljext://hello.localhost/index.html"), Platform::MacOs),
            Some(ExtUrl {
                id: "hello".into(),
                path: "/index.html".into()
            })
        );
        assert_eq!(
            parse(&url("ljext://hello.localhost/a/b.js"), Platform::Linux)
                .map(|e| e.path)
                .as_deref(),
            Some("/a/b.js")
        );
        assert_eq!(
            parse(
                &url("http://ljext.hello.localhost/index.html"),
                Platform::Windows
            )
            .map(|e| e.id)
            .as_deref(),
            Some("hello")
        );
        // The other platform's form is not this platform's extension URL.
        assert_eq!(
            parse(&url("http://ljext.hello.localhost/"), Platform::MacOs),
            None
        );
        assert_eq!(
            parse(&url("ljext://hello.localhost/"), Platform::Windows),
            None
        );
        // https is never the Windows form (wry intercepts http only).
        assert_eq!(
            parse(&url("https://ljext.hello.localhost/"), Platform::Windows),
            None
        );
    }

    #[test]
    fn hosts_ports_and_ids_are_strict() {
        for (s, platform) in [
            ("ljext://hello/index.html", Platform::MacOs),
            ("ljext://hello.localhost:8080/", Platform::MacOs),
            ("ljext://user@hello.localhost/", Platform::MacOs),
            ("ljext://HELLO.localhost/", Platform::Linux),
            ("ljext://he%6Clo.localhost/", Platform::Linux),
            ("ljext://a.b.localhost/", Platform::Linux),
            ("ljext://-x.localhost/", Platform::Linux),
            ("ljext://localhost/hello/", Platform::MacOs),
            ("http://ljext.localhost/hello/", Platform::Windows),
            ("http://ljext.hello.localhost:81/", Platform::Windows),
            ("http://ljext.hello.example.com/", Platform::Windows),
            ("http://ljext.evil.com/", Platform::Windows),
        ] {
            assert_eq!(parse(&url(s), platform), None, "{s}");
        }
    }

    #[test]
    fn requests_come_in_the_ljext_form_everywhere() {
        assert_eq!(
            parse_request("ljext://hello.localhost/__lj/api/v1/theme").map(|e| (e.id, e.path)),
            Some(("hello".into(), "/__lj/api/v1/theme".into()))
        );
        // What wry makes of a web page's request to http://ljext.evil.com/ on Windows.
        assert_eq!(parse_request("ljext://evil.com/x"), None);
        assert_eq!(parse_request("not a url"), None);
    }

    #[test]
    fn the_windows_form_is_never_plain_web() {
        for s in [
            "ljext://hello.localhost/",
            "http://ljext.hello.localhost/index.html",
            "https://ljext.hello.localhost/",
            "http://ljext.localhost/",
            "HTTP://LJEXT.Evil.COM/",
            "https://ljext.example.com/page",
        ] {
            assert!(is_ext_like(&url(s)), "{s}");
        }
        for s in [
            "https://example.com/",
            "http://tauri.localhost/index.html",
            "https://notljext.example.com/",
            "https://example.com/ljext.html",
            "about:blank",
        ] {
            assert!(!is_ext_like(&url(s)), "{s}");
        }
    }

    #[test]
    fn origins_and_file_urls_round_trip() {
        for platform in [Platform::MacOs, Platform::Linux, Platform::Windows] {
            let file = file_url("hello", "index.html", platform).expect("a URL");
            assert_eq!(
                parse(&file, platform),
                Some(ExtUrl {
                    id: "hello".into(),
                    path: "/index.html".into()
                })
            );
            assert!(file.as_str().starts_with(&origin("hello", platform)));
            assert!(is_ext_like(&file));
        }
        assert_eq!(origin("hello", Platform::MacOs), "ljext://hello.localhost");
        assert_eq!(
            origin("hello", Platform::Windows),
            "http://ljext.hello.localhost"
        );
    }
}
