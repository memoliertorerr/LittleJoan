//! The extension open in the page window, and the navigation guard that keeps `ljext:` pages to
//! the ones the app opens.
//!
//! Opening an extension (the command line's `--extension`, the command bar) closes the command
//! bar and navigates the page webview to the panel's URL — after telling the guard to expect
//! exactly that URL (`NavGate`). That URL carries a fresh single-use nonce (`?lj-open=<128 random
//! bits>`, nonce.rs) that only the app knows, so no document can claim the expectation first: the
//! guard admits an extension URL only when it is the expected one, nonce included, once, within a
//! few seconds. A web page (or a panel) therefore can't navigate the page webview — or, on macOS
//! and Linux where the guard also sees frames, a frame — to an extension URL, in any form
//! (`urls::is_ext_like`). While a panel is shown
//! it can't turn into a `blob:` or `data:` document either (one would keep its origin outside
//! the files the policy allows). The bar's toolbar shows the extension's name and icon instead of
//! a domain. All of it is per window: a new window starts with nothing opened.
//!
//! **Which extension the page shows** has two answers. For serving files and the API (api.rs)
//! it is what the guard admitted (`NavGate::showing`): an admitted extension URL makes it that
//! extension, any other navigation the guard lets through makes it none — every navigation passes
//! the guard first and in order, while load events can arrive late (on Windows CI saw the start
//! page's `Finished` land after the app had started opening a panel). For what the user sees —
//! the toolbar's extension name and icon, and Reload — it is what a load event of the page then
//! **committed** (`NavGate::committed`): the admitted extension's URL arriving in the page's own
//! `Started` / `Finished`. A navigation that was refused or cancelled never commits, so the toolbar
//! keeps showing the real page.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use super::nonce;
use super::registry;
use super::urls;
use crate::bar;
use crate::keys::{self, Platform};
use crate::navigation;
use crate::smoke;

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

    /// A load event (`Started` / `Finished`) of the page's main frame at `url`: the page shows the
    /// admitted extension once its URL arrives that way; any other URL means no extension.
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

/// The page a page action was opened for (`pageInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageInfo {
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone)]
struct Session {
    id: String,
    /// What was opened, so Reload can open it again (with a new nonce).
    target: Target,
    page: Option<PageInfo>,
    /// The web page that was open when the panel opened: `close` goes back to it.
    before: Option<Url>,
}

static GATE: Mutex<NavGate> = Mutex::new(NavGate::new());
static SESSION: Mutex<Option<Session>> = Mutex::new(None);
/// What the toolbar was last told (`Some(None)` = no extension); reset per window.
static LABEL_SHOWN: Mutex<Option<Option<String>>> = Mutex::new(None);

fn session() -> std::sync::MutexGuard<'static, Option<Session>> {
    SESSION.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A new window, or a closed one: nothing opened, nothing expected.
pub fn reset() {
    *session() = None;
    *GATE.lock().unwrap_or_else(PoisonError::into_inner) = NavGate::new();
    *LABEL_SHOWN.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

/// The page webview's navigation guard (bar.rs). Refusals go to the smoke log.
pub fn allow_page_navigation(url: &Url) -> bool {
    let allowed = page_may_navigate(
        url,
        &mut GATE.lock().unwrap_or_else(PoisonError::into_inner),
        Instant::now(),
        Platform::current(),
    );
    if !allowed {
        smoke::log("navigation-refused", url.as_str());
    }
    allowed
}

/// The extension whose panel the page shows, as the guard saw it (`NavGate::showing`).
pub fn showing() -> Option<String> {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .showing()
        .map(str::to_owned)
}

/// The extension whose panel a load event committed (`NavGate::committed`): what the toolbar
/// shows.
pub fn committed() -> Option<String> {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .committed()
        .map(str::to_owned)
}

/// A load event of the page webview (bar.rs: `Started` and `Finished` of the main frame).
pub fn page_event(url: &Url) {
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .commit(url, Platform::current());
}

/// The extension the app opened last in this window.
pub fn opened() -> Option<String> {
    session().as_ref().map(|s| s.id.clone())
}

/// The page the open panel was opened for, if any.
pub fn page_info() -> Option<PageInfo> {
    session().as_ref().and_then(|s| s.page.clone())
}

/// What to open of an extension.
#[derive(Debug, Clone)]
pub enum Target {
    /// Its first command (`--extension <id>`).
    First,
    /// One of its commands (the command bar).
    Command(String),
    /// One of its page actions, for the web page that is open (the command bar).
    PageAction(String, PageInfo),
}

/// Opens `target` of the extension `id` in the page (switched-off extensions can't be). A page
/// action's panel gets the page (`pageInfo`) only when its manifest asks for it. The window must
/// exist.
pub fn open<R: Runtime>(app: &AppHandle<R>, id: &str, target: Target) -> Result<(), String> {
    let registry = registry::current(app);
    let extension = registry
        .get(id)
        .ok_or_else(|| format!("there is no extension `{id}` (or it is switched off)"))?;
    let manifest = &extension.manifest;
    let (contribution, page) = match &target {
        Target::First => (manifest.commands.first(), None),
        Target::Command(command) => (manifest.commands.iter().find(|c| &c.id == command), None),
        Target::PageAction(action, page) => (
            manifest.page_actions.iter().find(|c| &c.id == action),
            manifest
                .has(&super::manifest::Permission::PageInfo)
                .then(|| page.clone()),
        ),
    };
    let contribution =
        contribution.ok_or_else(|| format!("the extension `{id}` has no such command"))?;
    let url = panel_url(id, contribution.panel.as_str(), &nonce::random_hex())
        .ok_or_else(|| "no URL for the panel".to_owned())?;
    let before = bar::page_url(app).filter(|current| {
        navigation::is_external_openable(current) && !keys::is_app_url(&keys::app_origin(), current)
    });
    *session() = Some(Session {
        id: id.to_owned(),
        target,
        page,
        before,
    });
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .expect(&url, Instant::now());
    smoke::log("extension-opened", id);
    // The page's address and the toolbar change when the panel's own load events arrive.
    bar::navigate_pending(app, url).map_err(|err| err.to_string())
}

/// The URL the app opens a panel at: the panel file in this platform's form, with the
/// single-use nonce (`?lj-open=<hex>`) the guard will expect.
pub fn panel_url(id: &str, panel: &str, nonce: &str) -> Option<Url> {
    let mut url = urls::file_url(id, panel, Platform::current())?;
    url.set_query(Some(&format!("lj-open={nonce}")));
    Some(url)
}

/// The Reload page action: on a panel that a load event committed, the app opens the same thing
/// again — a new nonce, never the old address. Returns whether it did (otherwise the page reloads
/// as a web page would; a panel that never committed isn't reloaded at all, the guard refuses it).
pub fn reopen<R: Runtime>(app: &AppHandle<R>) -> bool {
    let Some(committed) = committed() else {
        return false;
    };
    let Some(open_again) = session().as_ref().filter(|s| s.id == committed).cloned() else {
        return false;
    };
    if let Err(err) = open(app, &open_again.id, open_again.target) {
        eprintln!("little-joan: could not open the extension again: {err}");
    }
    true
}

/// littleJoan.close(): back to the web page the panel was opened from, or to the command bar.
pub fn close<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let before = session().take().and_then(|s| s.before);
    match before {
        Some(url) => bar::navigate_pending(app, url).map_err(|err| err.to_string()),
        None => {
            let start = keys::app_origin()
                .join("index.html")
                .map_err(|err| err.to_string())?;
            bar::navigate_pending(app, start).map_err(|err| err.to_string())?;
            bar::focus_bar(app);
            Ok(())
        }
    }
}

/// What the toolbar shows for an extension's panel.
#[derive(Debug, Clone, Serialize)]
struct Label {
    name: String,
    icon: Option<String>,
}

/// Tells the bar which extension the page shows — only once a load event committed it
/// (`committed`; `None`: a web page or nothing) — when that changed, or always with `force` (the
/// bar's script just started).
pub fn publish_label<R: Runtime>(app: &AppHandle<R>, force: bool) {
    let id = committed();
    {
        let mut shown = LABEL_SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
        if !force && shown.as_ref() == Some(&id) {
            return;
        }
        *shown = Some(id.clone());
    }
    smoke::log("toolbar-label", id.as_deref().unwrap_or("none"));
    let label = id
        .and_then(|id| registry::current(app).get(&id))
        .map(|extension| Label {
            name: extension.manifest.name.clone(),
            icon: extension.icon_data_uri(),
        });
    let Some(bar) = app.get_webview(bar::BAR) else {
        return;
    };
    // The name is checked text and the icon a base64 data: URI; JSON makes a safe script
    // literal of both, and the bar draws the name as text.
    let json = serde_json::to_string(&label).unwrap_or_else(|_| "null".to_owned());
    let _ = bar.eval(format!(
        "window.littleJoan && window.littleJoan.setExtension && window.littleJoan.setExtension({json})"
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn a_racing_document_cant_claim_the_expected_panel() {
        // The reviewer's case: while the app opens hello, a web page loops iframes (or a panel
        // navigates and stops) at hello's panel URL. Without the nonce, none is admitted, the
        // expectation survives, and the app's own navigation still is.
        let now = Instant::now();
        let mut gate = NavGate::new();
        let mine =
            url("ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdef");
        gate.expect(&mine, now);
        for racer in [
            "ljext://hello.localhost/index.html",
            "ljext://hello.localhost/index.html?lj-open=",
            "ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdee",
            "ljext://hello.localhost/index.html?lj-open=0123456789abcdef0123456789abcdef&x=1",
            "ljext://hello.localhost/index.html#lj-open=0123456789abcdef0123456789abcdef",
        ] {
            assert!(
                !page_may_navigate(&url(racer), &mut gate, now, Platform::MacOs),
                "{racer}"
            );
            assert_eq!(gate.showing(), None, "{racer}");
        }
        assert!(page_may_navigate(&mine, &mut gate, now, Platform::MacOs));
        assert_eq!(gate.showing(), Some("hello"));
        // Used up: the same URL again (a reload, a back) isn't admitted.
        assert!(!page_may_navigate(&mine, &mut gate, now, Platform::MacOs));
    }

    #[test]
    fn the_toolbar_follows_what_a_load_event_committed() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        let panel = url("ljext://hello.localhost/index.html?lj-open=00");
        // Load events of a page that is not the admitted panel commit nothing.
        assert!(!gate.commit(&url("https://example.com/"), Platform::MacOs));
        assert!(
            !gate.commit(&panel, Platform::MacOs),
            "not admitted: no extension"
        );
        assert_eq!(gate.committed(), None);
        gate.expect(&panel, now);
        assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
        assert_eq!(gate.committed(), None, "admitted is not committed");
        // A late event of the page before (the race CI saw) doesn't commit the panel either.
        assert!(!gate.commit(&url("tauri://localhost/index.html"), Platform::MacOs));
        assert!(gate.commit(&panel, Platform::MacOs));
        assert_eq!(gate.committed(), Some("hello"));
        // Leaving: the guard clears what it admitted, and the next load event clears the toolbar.
        assert!(page_may_navigate(
            &url("https://example.com/"),
            &mut gate,
            now,
            Platform::MacOs
        ));
        assert!(gate.commit(&url("https://example.com/"), Platform::MacOs));
        assert_eq!(gate.committed(), None);
        // Another extension's URL never commits as the admitted one.
        gate.expect(&panel, now);
        assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
        assert!(!gate.commit(&url("ljext://other.localhost/index.html"), Platform::MacOs));
        assert_eq!(gate.committed(), None);
    }

    #[test]
    fn panel_urls_carry_the_nonce() {
        let panel = panel_url("hello", "index.html", "abc").expect("a URL");
        assert!(panel.as_str().ends_with("/index.html?lj-open=abc"));
        assert_eq!(
            urls::parse(&panel, Platform::current())
                .map(|e| e.path)
                .as_deref(),
            Some("/index.html")
        );
    }

    #[test]
    fn only_the_expected_extension_url_is_admitted_once() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        let panel = url("ljext://hello.localhost/index.html");
        assert!(!gate.admit(&panel, now), "nothing expected yet");
        gate.expect(&panel, now);
        assert!(!gate.admit(&url("ljext://other.localhost/index.html"), now));
        assert!(!gate.admit(&url("ljext://hello.localhost/other.html"), now));
        assert!(gate.admit(&panel, now), "the expected one, still expected");
        assert!(!gate.admit(&panel, now), "only once");
    }

    #[test]
    fn an_expected_url_goes_stale() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        let panel = url("http://ljext.hello.localhost/index.html");
        gate.expect(&panel, now);
        assert!(!gate.admit(&panel, now + EXPECT_FOR + Duration::from_millis(1)));
        gate.expect(&panel, now);
        assert!(gate.admit(&panel, now + EXPECT_FOR));
    }

    #[test]
    fn web_pages_can_never_open_extension_urls() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        for s in [
            "ljext://hello.localhost/index.html",
            "http://ljext.hello.localhost/index.html",
            "https://ljext.hello.localhost/index.html",
            "http://ljext.localhost/hello/index.html",
            "http://ljext.evil.example.com/",
        ] {
            assert!(
                !page_may_navigate(&url(s), &mut gate, now, Platform::Windows),
                "{s}"
            );
        }
        // The expected URL in one form doesn't admit another form.
        gate.expect(&url("http://ljext.hello.localhost/index.html"), now);
        assert!(!page_may_navigate(
            &url("https://ljext.hello.localhost/index.html"),
            &mut gate,
            now,
            Platform::Windows
        ));
        assert!(page_may_navigate(
            &url("http://ljext.hello.localhost/index.html"),
            &mut gate,
            now,
            Platform::Windows
        ));
    }

    #[test]
    fn the_guard_decides_which_extension_the_page_shows() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        let panel = url("ljext://hello.localhost/index.html");
        gate.expect(&panel, now);
        assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
        assert_eq!(gate.showing(), Some("hello"));
        // Refused navigations change nothing.
        for refused in [
            "file:///etc/passwd",
            "ljext://other.localhost/index.html",
            "blob:ljext://hello.localhost/1",
            "data:text/html,hi",
        ] {
            assert!(
                !page_may_navigate(&url(refused), &mut gate, now, Platform::MacOs),
                "{refused}"
            );
        }
        assert_eq!(gate.showing(), Some("hello"));
        // Leaving for anything else the guard allows: no extension any more.
        assert!(page_may_navigate(
            &url("https://example.com/"),
            &mut gate,
            now,
            Platform::MacOs
        ));
        assert_eq!(gate.showing(), None);
        // The start page (the command bar, `close`) is not a panel either.
        gate.expect(&panel, now);
        assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
        assert!(page_may_navigate(
            &url("tauri://localhost/index.html"),
            &mut gate,
            now,
            Platform::MacOs
        ));
        assert_eq!(gate.showing(), None);
        // Another platform's form, even if expected, shows no extension on this one.
        let windows_form = url("http://ljext.hello.localhost/index.html");
        gate.expect(&windows_form, now);
        assert!(page_may_navigate(
            &windows_form,
            &mut gate,
            now,
            Platform::MacOs
        ));
        assert_eq!(gate.showing(), None);
    }

    #[test]
    fn web_rules_still_apply_and_panels_get_no_blob_or_data_documents() {
        let now = Instant::now();
        for (s, on_web, on_panel) in [
            ("https://example.com/", true, true),
            ("about:blank", true, true),
            ("blob:https://example.com/0f8f", true, false),
            ("data:text/html,hi", true, false),
            ("file:///etc/passwd", false, false),
            ("javascript:alert(1)", false, false),
            ("tauri://localhost/index.html", true, true),
        ] {
            let mut web = NavGate::new();
            assert_eq!(
                page_may_navigate(&url(s), &mut web, now, Platform::Linux),
                on_web,
                "{s} on the web"
            );
            let mut shown = NavGate::new();
            let opened = url("ljext://hello.localhost/index.html");
            shown.expect(&opened, now);
            assert!(page_may_navigate(&opened, &mut shown, now, Platform::Linux));
            assert_eq!(
                page_may_navigate(&url(s), &mut shown, now, Platform::Linux),
                on_panel,
                "{s} on a panel"
            );
        }
    }
}
