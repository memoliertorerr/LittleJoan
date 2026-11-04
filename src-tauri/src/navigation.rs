//! Which URLs the page window may navigate to, and which may be handed to the OS browser.
//!
//! Extension URLs — `ljext:` and WebView2's `http(s)://ljext.*` form — are never web pages
//! here: the page webview reaches them only when the app opened them (extensions/session.rs),
//! and the bar never does. Nor is any `http://` address WebView2 would hand to one of the app's own
//! protocols: wry intercepts `http://<scheme>.*` **by prefix** — `tauri.`, `ipc.`, `ljext.`, also
//! as user info (`http://tauri.@host/`) — so only the app's exact origin passes (phase 21's
//! security review).

use url::Url;

use crate::extensions::urls::is_ext_like;
use crate::keys;

/// The prefixes WebView2 routes to the app's own protocols when they follow `http://` (Tauri
/// serves custom schemes as `http://<scheme>.localhost` on Windows; wry's filter is
/// `http://<scheme>.*`). Https isn't used for them (`useHttpsScheme` is off).
const INTERCEPTED_PREFIXES: [&str; 3] = ["tauri.", "ipc.", "ljext."];

/// Whether `url`, as text, would reach one of the app's own protocols on Windows — the host's
/// first label, or user info written where a host is expected. Refused everywhere, so links and
/// typed addresses behave alike on every OS.
pub fn intercepted_by_app(url: &Url) -> bool {
    url.scheme() == "http"
        && url.as_str().get("http://".len()..).is_some_and(|rest| {
            let rest = rest.to_ascii_lowercase();
            INTERCEPTED_PREFIXES
                .iter()
                .any(|prefix| rest.starts_with(prefix))
        })
}

/// Whether the webview may navigate (top level or frame) to `url`.
///
/// Web schemes and the engine's own internal schemes (`about:blank`, `about:srcdoc`, `blob:`,
/// `data:` for frames) are allowed, as is the app's own origin (`tauri://localhost` on macOS and
/// Linux; on Windows it is `http://tauri.localhost`, already covered by `http`). Everything else —
/// `file:`, `javascript:`, and custom schemes that would launch other apps — is refused, and so
/// is every extension URL (only extensions/session.rs's gate admits one).
pub fn is_allowed_navigation(url: &Url) -> bool {
    if is_ext_like(url) || (intercepted_by_app(url) && !keys::is_app_url(&keys::app_origin(), url))
    {
        return false;
    }
    match url.scheme() {
        "http" | "https" | "about" | "blob" | "data" => true,
        "tauri" => url.host_str() == Some("localhost"),
        _ => false,
    }
}

/// Whether `url` may be opened in the system's default browser: plain web URLs only (never an
/// extension URL in WebView2's http form).
pub fn is_external_openable(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https") && url.has_host() && !is_ext_like(url)
}

/// Whether the bar (`joan-bar`, trusted) may navigate to `url`: the app's own pages, and
/// `about:blank` (to unload it when the window closes on macOS). Never an extension URL.
pub fn is_allowed_bar_navigation(url: &Url, app_origin: &Url) -> bool {
    !is_ext_like(url) && (keys::is_app_url(app_origin, url) || url.as_str() == "about:blank")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("test URL parses")
    }

    #[test]
    fn web_and_engine_internal_urls_are_allowed() {
        for s in [
            "https://example.com/",
            "http://localhost:3000/",
            "tauri://localhost/index.html?u=x",
            "about:blank",
            "about:srcdoc",
            "blob:https://example.com/0f8f6e1c",
            "data:text/html,hi",
        ] {
            assert!(is_allowed_navigation(&url(s)), "{s} should be allowed");
        }
        // The app's own origin (`http://tauri.localhost` on Windows): only there is it the app's.
        let start = keys::app_origin().join("index.html").expect("joins");
        assert!(is_allowed_navigation(&start));
        assert_eq!(
            is_allowed_navigation(&url("http://tauri.localhost/index.html")),
            cfg!(windows)
        );
    }

    #[test]
    fn local_files_scripts_and_app_launchers_are_refused() {
        for s in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "ms-settings:privacy",
            "mailto:me@example.com",
            "zoommtg://zoom.us/join",
            "tauri://evil.example/",
            "ftp://example.com/",
        ] {
            assert!(!is_allowed_navigation(&url(s)), "{s} should be refused");
        }
    }

    #[test]
    fn extension_urls_are_never_web_pages() {
        for s in [
            "ljext://hello.localhost/index.html",
            "http://ljext.hello.localhost/index.html",
            "https://ljext.hello.localhost/index.html",
            "http://ljext.localhost/",
            "http://ljext.evil.example.com/",
        ] {
            assert!(!is_allowed_navigation(&url(s)), "{s}");
            assert!(!is_external_openable(&url(s)), "{s}");
        }
    }

    #[test]
    fn the_bar_stays_on_the_app_and_never_opens_an_extension() {
        for origin in ["tauri://localhost/", "http://tauri.localhost/"] {
            let origin = url(origin);
            let own = origin.join("bar.html").expect("join");
            assert!(is_allowed_bar_navigation(&own, &origin));
            assert!(is_allowed_bar_navigation(&url("about:blank"), &origin));
            for s in [
                "ljext://hello.localhost/index.html",
                "http://ljext.hello.localhost/index.html",
                "https://example.com/",
                "about:srcdoc",
                "data:text/html,hi",
                "tauri://elsewhere/",
            ] {
                assert!(!is_allowed_bar_navigation(&url(s), &origin), "{s}");
            }
        }
    }

    #[test]
    fn only_web_urls_go_to_the_default_browser() {
        assert!(is_external_openable(&url("https://example.com/a")));
        assert!(is_external_openable(&url("http://example.com/")));
        assert!(!is_external_openable(&url("file:///tmp/x")));
        assert!(!is_external_openable(&url("mailto:me@example.com")));
        assert!(!is_external_openable(&url("tauri://localhost/")));
        assert!(!is_external_openable(&url("javascript:alert(1)")));
    }

    #[test]
    fn addresses_windows_would_hand_to_the_app_are_refused() {
        for s in [
            "http://tauri.evil.example/",
            "http://TAURI.localhost.evil/",
            "http://ipc.localhost/open_input",
            "http://ipc.example.com/",
            "http://tauri.@evil.example/",
            "http://ipc.:x@evil.example/",
            "http://ljext.@evil.example/",
        ] {
            let parsed = url(s);
            assert!(intercepted_by_app(&parsed), "{s}");
            assert!(!is_allowed_navigation(&parsed), "{s}");
        }
        // Not intercepted: https, other first labels, the app's own origin on Windows.
        for s in [
            "https://tauri.app/",
            "https://ipc.example.com/",
            "http://example.com/tauri.",
            "http://mytauri.example/",
            "http://user@tauri-apps.example/",
        ] {
            assert!(!intercepted_by_app(&url(s)), "{s}");
            assert!(is_allowed_navigation(&url(s)), "{s}");
        }
        let origin = keys::app_origin();
        assert!(is_allowed_navigation(
            &origin.join("index.html").expect("joins")
        ));
    }
}
