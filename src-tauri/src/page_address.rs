//! The page's address, kept from the page webview's own load events (pure, tested).
//!
//! Reading the engine's address (`Webview::url`) is risky on macOS: wry 0.57 unwraps WKWebView's
//! `URL`, which is unset until a first navigation commits (and again after a first navigation
//! that never did), and that panic ends the app. So the address comes from the load events
//! (`Started` and `Finished` carry it) and from the navigations the app starts itself. The
//! engine is asked only for in-page changes that raise no event (history.pushState, #anchors),
//! and only while the last event was `Finished` — so never before the first page loaded, and
//! never in the middle of a navigation. A navigation of the app's that turns into a download
//! (LJ15) leaves the page where it was, so its address goes back to what it was before (WKWebView
//! raises no event for the page that stays).

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
    /// What the address was before the app's last navigation (`navigating`), until an event.
    before: Option<(Option<Url>, Load)>,
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
            before: None,
        }
    }

    fn event(&mut self, url: Url, load: Load) {
        self.url = Some(url);
        self.load = load;
        self.seq = self.seq.wrapping_add(1);
        self.before = None;
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
        let before = (self.url.clone(), self.load);
        self.event(url, Load::Loading);
        self.before = Some(before);
    }

    /// `url` turned out to be a download (LJ15): if it is the address the app just navigated to,
    /// the page stayed where it was, and so does the address. Returns whether it changed.
    pub fn download(&mut self, url: &Url) -> bool {
        if self.load != Load::Loading || self.url.as_ref() != Some(url) {
            return false;
        }
        let Some((before, load)) = self.before.take() else {
            return false;
        };
        self.url = before;
        self.load = load;
        self.seq = self.seq.wrapping_add(1);
        true
    }

    /// The app started a navigation whose URL isn't the address until the page's own events say
    /// so (an extension's panel, a page an extension opens): the address stays, and the engine
    /// isn't asked until a load finished.
    pub fn pending(&mut self) {
        self.load = Load::Loading;
        self.seq = self.seq.wrapping_add(1);
        self.before = None;
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

    #[test]
    fn a_pending_navigation_keeps_the_address_until_its_events_arrive() {
        let mut page = PageAddress::new();
        page.finished(url("https://a.test/"));
        page.pending();
        assert_eq!(page.current(), Some(&url("https://a.test/")));
        assert_eq!(
            page.read_token(),
            None,
            "the engine isn't asked mid-navigation"
        );
        page.started(url("ljext://hello.localhost/index.html?lj-open=1"));
        assert_eq!(
            page.current().map(Url::as_str),
            Some("ljext://hello.localhost/index.html?lj-open=1")
        );
    }

    #[test]
    fn a_navigation_that_became_a_download_leaves_the_address_where_it_was() {
        let mut page = PageAddress::new();
        page.finished(url("https://a.test/page"));
        page.navigating(url("https://files.test/report.pdf"));
        assert_eq!(page.current(), Some(&url("https://files.test/report.pdf")));
        assert!(!page.download(&url("https://files.test/other.pdf")));
        assert!(page.download(&url("https://files.test/report.pdf")));
        assert_eq!(page.current(), Some(&url("https://a.test/page")));
        assert!(page.read_token().is_some(), "the page is finished again");
        // Once only, and never after the page's own events moved on.
        assert!(!page.download(&url("https://files.test/report.pdf")));
        page.navigating(url("https://files.test/b.zip"));
        page.started(url("https://files.test/b.zip"));
        assert!(!page.download(&url("https://files.test/b.zip")));
        // A download from the start page goes back to no address.
        let mut fresh = PageAddress::new();
        fresh.navigating(url("https://files.test/c.zip"));
        assert!(fresh.download(&url("https://files.test/c.zip")));
        assert_eq!(fresh.current(), None);
        assert!(!fresh.started_any());
    }
}
