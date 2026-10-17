//! Session cookies kept in memory while Little Joan hibernates (pure, tested).
//!
//! A session cookie has no expiry: a site's "until you close the browser" login. For Little Joan
//! the browser is the app run, not the window — but WebView2 gives every window its own browser
//! environment, which ends when the window closes, taking the session cookies with it. So
//! `joan::destroy` reads the page webview's cookies and keeps the session ones here (in
//! `AppState`), and `joan::create` sets them again before the new page loads.
//!
//! They live only in memory: this module has no file or serialization code (a test reads its
//! source to keep it that way), and `SessionCookies` can't be printed with its values. Quitting
//! forgets them, as quitting a browser does. Uses the `cookie` crate's types through Tauri's
//! re-export (`tauri::webview::cookie`), so no runtime types and no new dependency.

use std::collections::HashSet;
use std::fmt;

use tauri::webview::cookie::{Cookie, SameSite};

/// The most session cookies kept across one close → summon.
pub const MAX_KEPT: usize = 500;

/// Whether the engine itself keeps session cookies from one window to the next within an app
/// run, so Little Joan doesn't have to: WebKit on macOS (the default data store lives as long
/// as the app) and WebKitGTK (Tauri keeps the web context for the app's lifetime) do; WebView2's
/// per-window environment doesn't. Measured by the CI smoke test (LJ20).
pub const ENGINE_KEEPS_SESSION_COOKIES: bool = !cfg!(windows);

/// Session cookies ready to be set again, in memory only.
#[derive(Default, Clone, PartialEq)]
pub struct SessionCookies(Vec<Cookie<'static>>);

/// Never prints names or values (they are logins): only how many there are.
impl fmt::Debug for SessionCookies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SessionCookies({} kept)", self.0.len())
    }
}

impl SessionCookies {
    /// The session cookies among `all` (every cookie the webview has): one per name + domain +
    /// path — the last one read wins — at most `MAX_KEPT`, each made settable again. Persistent
    /// cookies are left out: the engine keeps those on disk itself.
    pub fn from_all(all: impl IntoIterator<Item = Cookie<'static>>) -> Self {
        let sessions: Vec<Cookie<'static>> = all
            .into_iter()
            .filter(is_session)
            .filter_map(|cookie| restorable(&cookie))
            .collect();
        // Walk from the end so the last of each name + domain + path is the one kept.
        let mut seen = HashSet::new();
        let mut kept: Vec<Cookie<'static>> = sessions
            .into_iter()
            .rev()
            .filter(|cookie| seen.insert(key(cookie)))
            .collect();
        kept.reverse();
        kept.truncate(MAX_KEPT);
        Self(kept)
    }

    /// The cookies to set again, in the order they were read.
    pub fn cookies(&self) -> &[Cookie<'static>] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A session cookie: neither an expiry date nor a max-age.
pub fn is_session(cookie: &Cookie<'_>) -> bool {
    cookie.max_age().is_none() && cookie.expires_datetime().is_none()
}

/// What makes two cookies the same cookie: name, domain (case-insensitive) and path.
fn key(cookie: &Cookie<'_>) -> (String, String, String) {
    (
        cookie.name().to_owned(),
        cookie.domain().unwrap_or_default().to_ascii_lowercase(),
        cookie.path().unwrap_or("/").to_owned(),
    )
}

/// A fresh session cookie with `cookie`'s name, value, domain, path and flags, or `None` when it
/// can't be set again (no name or no domain; WKWebView refuses a cookie without a domain).
/// Chromium (so WebView2) refuses `SameSite=None` without `Secure`, so a non-secure one is set
/// without SameSite (the engine's Lax default, as for a cookie that never named one).
fn restorable(cookie: &Cookie<'_>) -> Option<Cookie<'static>> {
    let name = cookie.name().trim();
    let domain = cookie.domain().map(str::trim).unwrap_or_default();
    if name.is_empty() || domain.is_empty() {
        return None;
    }
    let path = cookie.path().filter(|p| p.starts_with('/')).unwrap_or("/");
    let secure = cookie.secure().unwrap_or(false);
    let mut builder = Cookie::build((name.to_owned(), cookie.value().to_owned()))
        .domain(domain.to_owned())
        .path(path.to_owned())
        .secure(secure)
        .http_only(cookie.http_only().unwrap_or(false));
    match cookie.same_site() {
        Some(SameSite::None) if !secure => {}
        Some(same_site) => builder = builder.same_site(same_site),
        None => {}
    }
    Some(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::webview::cookie::Expiration;
    use tauri::webview::cookie::time::{Duration, OffsetDateTime};

    fn session(name: &str, value: &str, domain: &str, path: &str) -> Cookie<'static> {
        Cookie::build((name.to_owned(), value.to_owned()))
            .domain(domain.to_owned())
            .path(path.to_owned())
            .expires(Expiration::Session)
            .build()
    }

    #[test]
    fn a_cookie_without_an_expiry_is_a_session_cookie() {
        assert!(is_session(&Cookie::new("a", "1")));
        // Engines report session cookies with an explicit "session" expiration (wry).
        assert!(is_session(&session("a", "1", "x.test", "/")));
        let dated = Cookie::build(("a", "1"))
            .expires(OffsetDateTime::from_unix_timestamp(2_000_000_000).expect("time"))
            .build();
        assert!(!is_session(&dated));
        let aged = Cookie::build(("a", "1"))
            .max_age(Duration::days(365))
            .build();
        assert!(!is_session(&aged));
    }

    #[test]
    fn only_session_cookies_are_kept() {
        let persistent = Cookie::build(("lj_p", "t"))
            .domain("127.0.0.1")
            .path("/")
            .max_age(Duration::days(365))
            .build();
        let kept = SessionCookies::from_all([persistent, session("lj_s", "t", "127.0.0.1", "/")]);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept.cookies()[0].name(), "lj_s");
        assert_eq!(kept.cookies()[0].value(), "t");
        assert_eq!(kept.cookies()[0].domain(), Some("127.0.0.1"));
        assert_eq!(kept.cookies()[0].path(), Some("/"));
        assert!(is_session(&kept.cookies()[0]));
    }

    #[test]
    fn entries_are_unique_by_name_domain_and_path_and_the_last_one_wins() {
        let kept = SessionCookies::from_all([
            session("sid", "old", "example.com", "/"),
            session("sid", "other-path", "example.com", "/app"),
            session("sid", "other-host", "www.example.com", "/"),
            session("other", "x", "example.com", "/"),
            session("sid", "new", "EXAMPLE.com", "/"),
        ]);
        let values: Vec<_> = kept.cookies().iter().map(Cookie::value).collect();
        assert_eq!(values, ["other-path", "other-host", "x", "new"]);
    }

    #[test]
    fn at_most_500_are_kept() {
        let many = (0..MAX_KEPT + 20).map(|i| session(&format!("c{i}"), "v", "x.test", "/"));
        let kept = SessionCookies::from_all(many);
        assert_eq!(kept.len(), MAX_KEPT);
        assert_eq!(kept.cookies()[0].name(), "c0");
        assert_eq!(
            kept.cookies()[MAX_KEPT - 1].name(),
            format!("c{}", MAX_KEPT - 1)
        );
    }

    #[test]
    fn cookies_that_cant_be_set_again_are_dropped() {
        let no_domain = Cookie::new("a", "1");
        let empty_name = session("", "1", "x.test", "/");
        let blank_domain = session("a", "1", "  ", "/");
        assert!(SessionCookies::from_all([no_domain, empty_name, blank_domain]).is_empty());
    }

    #[test]
    fn flags_are_kept_and_a_missing_path_becomes_the_root() {
        let secure = Cookie::build(("s", "1"))
            .domain("x.test")
            .secure(true)
            .http_only(true)
            .same_site(SameSite::None)
            .build();
        let lax = Cookie::build(("l", "1"))
            .domain("x.test")
            .path("relative")
            .same_site(SameSite::Strict)
            .build();
        let kept = SessionCookies::from_all([secure, lax]);
        let [secure, lax] = kept.cookies() else {
            panic!("two cookies, got {kept:?}");
        };
        assert_eq!(secure.secure(), Some(true));
        assert_eq!(secure.http_only(), Some(true));
        assert_eq!(secure.same_site(), Some(SameSite::None));
        assert_eq!(secure.path(), Some("/"));
        assert_eq!(lax.same_site(), Some(SameSite::Strict));
        assert_eq!(lax.secure(), Some(false));
        assert_eq!(lax.path(), Some("/"));
    }

    #[test]
    fn same_site_none_without_secure_is_set_with_the_engine_default() {
        let cookie = Cookie::build(("a", "1"))
            .domain("x.test")
            .same_site(SameSite::None)
            .build();
        let kept = SessionCookies::from_all([cookie]);
        assert_eq!(kept.cookies()[0].same_site(), None);
    }

    #[test]
    fn debug_output_never_shows_values() {
        let kept = SessionCookies::from_all([session("sid", "secret-token", "x.test", "/")]);
        let printed = format!("{kept:?}");
        assert_eq!(printed, "SessionCookies(1 kept)");
        assert!(!printed.contains("secret-token"));
    }

    #[test]
    fn nothing_is_ever_written_to_disk() {
        // The module's own code (above the tests) has no file, process or serialization calls.
        let source = include_str!("cookies.rs");
        let code = source.split("#[cfg(test)]").next().unwrap_or(source);
        for banned in [
            "std::fs",
            "File",
            "write(",
            "serde",
            "Serialize",
            "OpenOptions",
            "Command",
        ] {
            assert!(!code.contains(banned), "cookies.rs mentions {banned}");
        }
    }
}
