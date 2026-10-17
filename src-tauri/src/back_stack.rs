//! Whether the toolbar's back button has somewhere to go (pure, tested).
//!
//! Counted natively from this window's own page loads, since the page webview has no IPC and
//! Tauri 2.12 has no "can go back" query: a trail of the web pages that finished loading. A load
//! of the page just before the last one is a step back (the trail shrinks), a load of the last
//! one again is a reload, anything else is a new page. The app's own pages and `about:` pages
//! don't count, so going back to the start page never looks like somewhere to go.

use url::Url;

use crate::keys;

/// The longest trail kept (the oldest pages are forgotten first).
pub const MAX_PAGES: usize = 100;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackStack {
    pages: Vec<String>,
}

impl BackStack {
    /// A new window: nowhere to go back to.
    pub const fn new() -> Self {
        BackStack { pages: Vec::new() }
    }

    /// The page finished loading `url`. Returns whether `can_go_back` changed.
    pub fn loaded(&mut self, url: &Url, app_origin: &Url) -> bool {
        if !matches!(url.scheme(), "http" | "https") || keys::is_app_url(app_origin, url) {
            return false;
        }
        let mut page = url.clone();
        page.set_fragment(None);
        let page = String::from(page);
        let before = self.can_go_back();
        let count = self.pages.len();
        if self.pages.last() == Some(&page) {
            // A reload, or the same page again.
        } else if count >= 2 && self.pages.get(count - 2) == Some(&page) {
            self.pages.pop();
        } else {
            self.pages.push(page);
            if self.pages.len() > MAX_PAGES {
                self.pages.remove(0);
            }
        }
        before != self.can_go_back()
    }

    /// Whether there is an earlier web page in this window to go back to.
    pub fn can_go_back(&self) -> bool {
        self.pages.len() >= 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    fn origin() -> Url {
        url("tauri://localhost/")
    }

    #[test]
    fn a_new_window_has_nowhere_to_go_back_to() {
        let mut stack = BackStack::new();
        assert!(!stack.can_go_back());
        // The start page and the first web page: still nothing before it but the start page.
        assert!(!stack.loaded(&url("tauri://localhost/index.html"), &origin()));
        assert!(!stack.loaded(&url("https://a.test/"), &origin()));
        assert!(!stack.can_go_back());
    }

    #[test]
    fn a_second_page_enables_back_and_going_back_disables_it() {
        let mut stack = BackStack::new();
        stack.loaded(&url("https://a.test/"), &origin());
        assert!(
            stack.loaded(&url("https://a.test/next"), &origin()),
            "changed"
        );
        assert!(stack.can_go_back());
        // Back: the page before the last one loads again.
        assert!(stack.loaded(&url("https://a.test/"), &origin()), "changed");
        assert!(!stack.can_go_back());
    }

    #[test]
    fn reloads_and_fragments_change_nothing() {
        let mut stack = BackStack::new();
        stack.loaded(&url("https://a.test/x"), &origin());
        stack.loaded(&url("https://a.test/y"), &origin());
        assert!(!stack.loaded(&url("https://a.test/y"), &origin()));
        assert!(!stack.loaded(&url("https://a.test/y#part"), &origin()));
        assert!(stack.can_go_back());
        assert_eq!(stack.pages.len(), 2);
    }

    #[test]
    fn several_pages_go_back_one_at_a_time() {
        let mut stack = BackStack::new();
        for page in ["https://a.test/1", "https://a.test/2", "https://a.test/3"] {
            stack.loaded(&url(page), &origin());
        }
        assert!(
            !stack.loaded(&url("https://a.test/2"), &origin()),
            "still one before"
        );
        assert!(stack.can_go_back());
        assert!(stack.loaded(&url("https://a.test/1"), &origin()));
        assert!(!stack.can_go_back());
        // A new page after going back starts a new trail from there.
        assert!(stack.loaded(&url("https://b.test/"), &origin()));
        assert!(stack.can_go_back());
    }

    #[test]
    fn app_pages_and_blank_pages_never_count() {
        let mut stack = BackStack::new();
        stack.loaded(&url("https://a.test/"), &origin());
        for other in [
            "tauri://localhost/index.html",
            "about:blank",
            "data:text/plain,x",
        ] {
            assert!(!stack.loaded(&url(other), &origin()), "{other}");
        }
        // Windows' app origin is plain http.
        let windows = url("http://tauri.localhost/");
        assert!(!stack.loaded(&url("http://tauri.localhost/index.html"), &windows));
        assert!(!stack.can_go_back());
    }

    #[test]
    fn the_trail_is_capped() {
        let mut stack = BackStack::new();
        for i in 0..(MAX_PAGES + 20) {
            stack.loaded(&url(&format!("https://a.test/{i}")), &origin());
        }
        assert_eq!(stack.pages.len(), MAX_PAGES);
        assert!(stack.can_go_back());
    }
}
