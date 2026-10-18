//! The pure core of the extension session (session.rs has the glue): `NavGate`, the page
//! webview's "this extension URL is mine" flag and which extension the page shows (admitted by
//! the guard, committed by the page's commit event); `page_may_navigate`, the guard's decision;
//! and `Leaving`, an opening that waits for the page to leave a web page. All tested in
//! session/tests.rs.

use std::time::{Duration, Instant};

use url::Url;

use super::urls;
use crate::keys::Platform;
use crate::navigation;

/// How long an expected extension URL stays expected.
pub const EXPECT_FOR: Duration = Duration::from_secs(5);

/// The app's "this extension URL is mine" flag, and which extension the page shows: as admitted
/// by the guard, and as committed by a load event (pure, tested).
#[derive(Debug, Default)]
pub struct NavGate {
    expected: Option<(String, Instant)>,
    showing: Option<String>,
    committed: Option<String>,
}

impl NavGate {
    pub const fn new() -> Self {
        NavGate {
            expected: None,
            showing: None,
            committed: None,
        }
    }

    /// The extension whose panel the guard last let the page navigate to, until it let the page
    /// go anywhere else.
    pub fn showing(&self) -> Option<&str> {
        self.showing.as_deref()
    }

    /// The extension whose panel a load event of the page committed (see `commit`).
    pub fn committed(&self) -> Option<&str> {
        self.committed.as_deref()
    }

    /// The page's main frame committed a document at `url` (its `Started` event): the page shows
    /// the admitted extension once its URL arrives that way; any other URL means no extension.
    /// Returns whether that changed.
    pub fn commit(&mut self, url: &Url, platform: Platform) -> bool {
        let id = urls::parse(url, platform)
            .map(|ext| ext.id)
            .filter(|id| self.showing.as_deref() == Some(id.as_str()));
        let changed = self.committed != id;
        self.committed = id;
        changed
    }

    /// The app is about to navigate the page to `url`.
    pub fn expect(&mut self, url: &Url, now: Instant) {
        self.expected = Some((url.as_str().to_owned(), now));
    }

    /// Whether the extension URL `url` is the one the app is opening (and still fresh). Admitting
    /// it uses the flag up; another URL leaves it (unless it went stale).
    pub fn admit(&mut self, url: &Url, now: Instant) -> bool {
        match self.expected.take() {
            Some((want, at)) if now.saturating_duration_since(at) <= EXPECT_FOR => {
                if want == url.as_str() {
                    true
                } else {
                    self.expected = Some((want, at));
                    false
                }
            }
            _ => false,
        }
    }
}

/// Whether the page webview may navigate (top level or frame) to `url`: an extension URL only
/// when the gate admits it (the page then shows that extension); while a panel is shown, never
/// `blob:` or `data:`; everything else as for web pages (navigation.rs), and the page then shows
/// no extension.
pub fn page_may_navigate(url: &Url, gate: &mut NavGate, now: Instant, platform: Platform) -> bool {
    if urls::is_ext_like(url) {
        let admitted = gate.admit(url, now);
        if admitted {
            gate.showing = urls::parse(url, platform).map(|ext| ext.id);
        }
        return admitted;
    }
    if gate.showing.is_some() && matches!(url.scheme(), "blob" | "data") {
        return false;
    }
    let allowed = navigation::is_allowed_navigation(url);
    if allowed {
        gate.showing = None;
    }
    allowed
}

/// An opening waiting for the page to leave a web page (pure, tested): the panel's URL, numbered
/// so a stale timer can't open a later one early.
#[derive(Debug, Default)]
pub struct Leaving {
    seq: u64,
    url: Option<Url>,
}

impl Leaving {
    pub const fn new() -> Self {
        Leaving { seq: 0, url: None }
    }

    /// Waits to open `url`; returns its number.
    pub fn wait(&mut self, url: Url) -> u64 {
        self.seq += 1;
        self.url = Some(url);
        self.seq
    }

    /// The page left the web page: the panel to open now, if one waits.
    pub fn left(&mut self) -> Option<Url> {
        self.url.take()
    }

    /// The wait for `seq` ran out: its panel, if it still waits (not a later one).
    pub fn timed_out(&mut self, seq: u64) -> Option<Url> {
        if self.seq == seq {
            self.url.take()
        } else {
            None
        }
    }
}

/// How long an opening waits for the page to leave a web page before it opens the panel anyway.
pub const LEAVE_FOR: Duration = Duration::from_secs(1);
