//! The extension open in the page window, and the navigation guard that keeps `ljext:` pages to
//! the ones the app opens.
//!
//! Opening an extension (the command line's `--extension`, from LJ30 the command bar) closes the
//! command bar and navigates the page webview to the panel's URL — after telling the guard to
//! expect exactly that URL (`NavGate`). The guard admits an extension URL only when it is the
//! expected one, once, within a few seconds; a web page (or a panel) can never navigate the page
//! webview to an extension URL itself, in any form (`urls::is_ext_like`). While a panel is shown
//! it can't turn into a `blob:` or `data:` document either (one would keep its origin outside
//! the files the policy allows). The bar's toolbar shows the extension's name and icon instead of
//! a domain. All of it is per window: a new window starts with nothing opened.
//!
//! **Which extension the page shows** is decided here too, by the guard (`NavGate::showing`): an
//! admitted extension URL makes it that extension, any other navigation the guard lets through
//! makes it none. Every navigation of the page webview passes the guard first, in order — the
//! app's own, links, reloads, back / forward — while load events can arrive late: on Windows CI
//! saw the start page's `Finished` land after the app had started opening a panel, and a page
//! check based on the page's address refused the panel's own document. api.rs checks this.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use super::registry;
use super::urls;
use crate::bar;
use crate::keys::{self, Platform};
use crate::navigation;
use crate::smoke;

/// How long an expected extension URL stays expected.
pub const EXPECT_FOR: Duration = Duration::from_secs(5);

/// The app's "this extension URL is mine" flag, and which extension the page shows (pure,
/// tested).
#[derive(Debug, Default)]
pub struct NavGate {
    expected: Option<(String, Instant)>,
    showing: Option<String>,
}

impl NavGate {
    pub const fn new() -> Self {
        NavGate {
            expected: None,
            showing: None,
        }
    }

    /// The extension whose panel the guard last let the page navigate to, until it let the page
    /// go anywhere else.
    pub fn showing(&self) -> Option<&str> {
        self.showing.as_deref()
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
    let url = urls::file_url(id, contribution.panel.as_str(), Platform::current())
        .ok_or_else(|| "no URL for the panel".to_owned())?;
    let before = bar::page_url(app).filter(|current| {
        navigation::is_external_openable(current) && !keys::is_app_url(&keys::app_origin(), current)
    });
    *session() = Some(Session {
        id: id.to_owned(),
        page,
        before,
    });
    GATE.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .expect(&url, Instant::now());
    smoke::log("extension-opened", id);
    bar::navigate(app, url).map_err(|err| err.to_string())
}

/// The Reload page action on a panel: the reload is the app's own navigation.
pub fn expect_reload(current: &Url) {
    if urls::parse(current, Platform::current()).is_some() {
        GATE.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .expect(current, Instant::now());
    }
}

/// littleJoan.close(): back to the web page the panel was opened from, or to the command bar.
pub fn close<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let before = session().take().and_then(|s| s.before);
    match before {
        Some(url) => bar::navigate(app, url).map_err(|err| err.to_string()),
        None => {
            let start = keys::app_origin()
                .join("index.html")
                .map_err(|err| err.to_string())?;
            bar::navigate(app, start).map_err(|err| err.to_string())?;
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

/// Tells the bar which extension the page shows (`None`: a web page or nothing), when that
/// changed — or always, with `force` (the bar's script just started).
pub fn publish_label<R: Runtime>(app: &AppHandle<R>, page: Option<&Url>, force: bool) {
    let id = page
        .and_then(|page| urls::parse(page, Platform::current()))
        .map(|ext| ext.id);
    {
        let mut shown = LABEL_SHOWN.lock().unwrap_or_else(PoisonError::into_inner);
        if !force && shown.as_ref() == Some(&id) {
            return;
        }
        *shown = Some(id.clone());
    }
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
