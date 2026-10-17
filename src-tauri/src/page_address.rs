//! The page's address, kept from the page webview's own load events (pure, tested).
//!
//! Reading the engine's address (`Webview::url`) is risky on macOS: wry 0.57 unwraps WKWebView's
//! `URL`, which is unset until a first navigation commits (and again after a first navigation
//! that never did), and that panic ends the app. So the address comes from the load events
//! (`Started` and `Finished` carry it) and from the navigations the app starts itself. The
//! engine is asked only for in-page changes that raise no event (history.pushState, #anchors),
//! and only while the last event was `Finished` — so never before the first page loaded, and
//! never in the middle of a navigation.

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Load {
    /// No load event yet in this window.
    None,
    /// A navigation is under way (it started, or the app asked for it).
    Loading,
    /// The last event was `Finished`: the engine's address is set and stable.
    Finished,
}

/// What the page webview is showing, as far as its events tell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageAddress {
    url: Option<Url>,
    load: Load,
    /// Bumped by every event, so an engine read that raced with a new navigation is dropped.
    seq: u64,
}

/// Permission to ask the engine for the address once (see [`PageAddress::read_token`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadToken(u64);

impl Default for PageAddress {
    fn default() -> Self {
        Self::new()
    }
}

impl PageAddress {
    /// A new window: nothing loaded yet.
    pub const fn new() -> Self {
        PageAddress {
            url: None,
            load: Load::None,
            seq: 0,
        }
    }

    fn event(&mut self, url: Url, load: Load) {
        self.url = Some(url);
        self.load = load;
        self.seq = self.seq.wrapping_add(1);
    }

    /// The page started loading `url` (the `Started` event).
    pub fn started(&mut self, url: Url) {
        self.event(url, Load::Loading);
    }

    /// The page finished loading `url` (the `Finished` event; after redirects, the final one).
    pub fn finished(&mut self, url: Url) {
        self.event(url, Load::Finished);
    }

    /// The app is about to load `url` in the page: it is the address from now on, and the
    /// engine isn't asked until that load finished.
    pub fn navigating(&mut self, url: Url) {
        self.event(url, Load::Loading);
    }

    /// The page's address, or `None` before anything loaded.
    pub fn current(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    /// Whether the page has started loading anything in this window.
    pub fn started_any(&self) -> bool {
        self.load != Load::None
    }

    /// A token to ask the engine for the address, only while the last event was `Finished`.
    pub fn read_token(&self) -> Option<ReadToken> {
        (self.load == Load::Finished).then_some(ReadToken(self.seq))
    }

    /// What the engine answered for `token`. It counts only if no event came in between (a
    /// newer navigation wins). Returns whether the address changed.
    pub fn observed(&mut self, token: ReadToken, url: Url) -> bool {
        if token.0 != self.seq || self.load != Load::Finished || self.url.as_ref() == Some(&url) {
            return false;
        }
        self.url = Some(url);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn nothing_is_known_or_read_before_the_first_load_event() {
        let address = PageAddress::new();
        assert_eq!(address.current(), None);
        assert!(!address.started_any());
        assert_eq!(address.read_token(), None);
    }

    #[test]
    fn load_events_carry_the_address() {
        let mut address = PageAddress::new();
        address.started(url("tauri://localhost/index.html"));
        assert_eq!(
            address.current(),
            Some(&url("tauri://localhost/index.html"))
        );
        assert!(address.started_any());
        assert_eq!(address.read_token(), None, "not while it loads");
        address.finished(url("https://example.com/after-redirect"));
        assert_eq!(
            address.current(),
            Some(&url("https://example.com/after-redirect"))
        );
        assert!(address.read_token().is_some());
    }

    #[test]
    fn the_apps_own_navigation_shows_at_once_and_stops_engine_reads() {
        let mut address = PageAddress::new();
        address.started(url("tauri://localhost/index.html"));
        // Create(url): the app navigates while the start page is still loading.
        address.navigating(url("http://127.0.0.1:5000/smoke.html"));
        assert_eq!(
            address.current(),
            Some(&url("http://127.0.0.1:5000/smoke.html"))
        );
        assert_eq!(address.read_token(), None);
        address.finished(url("http://127.0.0.1:5000/smoke.html"));
        assert!(address.read_token().is_some());
        address.navigating(url("https://example.org/"));
        assert_eq!(address.read_token(), None);
    }

    #[test]
    fn in_page_changes_are_read_from_the_engine_after_a_finished_load() {
        let mut address = PageAddress::new();
        address.finished(url("https://example.com/a"));
        let token = address.read_token().expect("finished");
        assert!(address.observed(token, url("https://example.com/a#top")));
        assert_eq!(address.current(), Some(&url("https://example.com/a#top")));
        // The same answer again changes nothing; reads go on while nothing else happens.
        let token = address.read_token().expect("still finished");
        assert!(!address.observed(token, url("https://example.com/a#top")));
        assert!(address.observed(token, url("https://example.com/b")));
    }

    #[test]
    fn an_engine_read_that_raced_with_a_new_navigation_is_dropped() {
        let mut address = PageAddress::new();
        address.finished(url("https://example.com/a"));
        let token = address.read_token().expect("finished");
        address.started(url("https://example.com/next"));
        assert!(!address.observed(token, url("https://example.com/a")));
        assert_eq!(address.current(), Some(&url("https://example.com/next")));
        // Even once that load finished, the old token stays stale.
        address.finished(url("https://example.com/next"));
        assert!(!address.observed(token, url("https://example.com/a")));
        assert_eq!(address.current(), Some(&url("https://example.com/next")));
    }
}
