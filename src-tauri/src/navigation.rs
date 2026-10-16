//! Which URLs the page window may navigate to, and which may be handed to the OS browser.

use url::Url;

/// Whether the webview may navigate (top level or frame) to `url`.
///
/// Web schemes and the engine's own internal schemes (`about:blank`, `about:srcdoc`, `blob:`,
/// `data:` for frames) are allowed, as is the app's own origin (`tauri://localhost` on macOS and
/// Linux; on Windows it is `http://tauri.localhost`, already covered by `http`). Everything else —
/// `file:`, `javascript:`, and custom schemes that would launch other apps — is refused.
pub fn is_allowed_navigation(url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" | "about" | "blob" | "data" => true,
        "tauri" => url.host_str() == Some("localhost"),
        _ => false,
    }
}

/// Whether `url` may be opened in the system's default browser: plain web URLs only.
pub fn is_external_openable(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https") && url.has_host()
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
            "http://tauri.localhost/index.html",
            "tauri://localhost/index.html?u=x",
            "about:blank",
            "about:srcdoc",
            "blob:https://example.com/0f8f6e1c",
            "data:text/html,hi",
        ] {
            assert!(is_allowed_navigation(&url(s)), "{s} should be allowed");
        }
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
    fn only_web_urls_go_to_the_default_browser() {
        assert!(is_external_openable(&url("https://example.com/a")));
        assert!(is_external_openable(&url("http://example.com/")));
        assert!(!is_external_openable(&url("file:///tmp/x")));
        assert!(!is_external_openable(&url("mailto:me@example.com")));
        assert!(!is_external_openable(&url("tauri://localhost/")));
        assert!(!is_external_openable(&url("javascript:alert(1)")));
    }
}
