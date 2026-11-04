//! The main browser (LJ46): every page Little Joan hands on goes through `hand_off` — "Open in
//! browser" (page_actions.rs), the page's `target=_blank` and `window.open` (bar/page.rs),
//! Settings' links (settings_window.rs), extensions' `openUrl(where: "browser")`
//! (extensions/protocol.rs). Since Little Joan can be the system's browser itself (LJ45), the OS
//! default may be Little Joan, which would bring the page straight back: `handoff_target`
//! (browsers/target.rs, pure) never picks Little Joan — the user's main browser (settings'
//! `mainBrowser`) if installed, else the OS default unless it is Little Joan, else the first other
//! browser the OS lists, else nowhere (the toolbar says so; nothing opens in Little Joan). This file
//! is the one place that calls the opener (a source test checks it), and the default-apps button's
//! glue: opening the OS's own settings is all it does — Little Joan never writes a default.
//!
//! The browser list, the default and a launch come from platform/browsers.rs (read-only:
//! NSWorkspace, the registry's StartMenuInternet and UserChoice, the XDG desktop files and
//! `xdg-mime`), never from a page. Browsers start without a shell: `open -b`, the registered
//! program with the URL as one argument, `gio launch`. Smoke log: `browsers-listed <n>
//! default=<self|other|none>`, `handoff os-default | browser=<id>`, `handoff-none`,
//! `handoff-self-refused` — never the URL.

use std::sync::{Mutex, PoisonError};
use std::time::Instant;

use tauri::{AppHandle, Runtime};
use url::Url;

use crate::bar;
use crate::navigation;
use crate::platform;
use crate::smoke;
use crate::state;

pub mod limits;
pub mod parse;
pub mod target;

pub use limits::HandoffGate;
pub use target::{Browser, Browsers, Decision, Launch, Recent, Target, handoff_target};

/// How a handoff went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Opened,
    /// No browser but Little Joan: nothing opened.
    Nowhere,
    Failed(String),
}

/// The last read of the OS's list, reused for `limits::LIST_TTL`.
static LISTED: Mutex<Option<(Instant, Browsers)>> = Mutex::new(None);

/// What the OS says (read again after `limits::LIST_TTL`), logged for the smoke test when read.
pub fn read() -> Browsers {
    let now = Instant::now();
    if let Some((at, browsers)) = LISTED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        && limits::fresh(*at, now)
    {
        return browsers.clone();
    }
    let browsers = platform::browsers::read();
    smoke::log(
        "browsers-listed",
        &format!(
            "{} default={}",
            browsers.others().count(),
            browsers.default_kind()
        ),
    );
    *LISTED.lock().unwrap_or_else(PoisonError::into_inner) = Some((now, browsers.clone()));
    browsers
}

/// Who asked for a new window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWindowFrom {
    /// The page webview: a web page — or an extension's panel, which gets `openUrl`'s checks.
    Page,
    /// The settings window's own links.
    Settings,
}

/// The new-window gate (limits.rs): one under way, one a second.
static GATE: Mutex<Option<HandoffGate>> = Mutex::new(None);

/// Ends the gate's handoff however the task ends.
struct GateTurn;

impl Drop for GateTurn {
    fn drop(&mut self) {
        if let Some(gate) = GATE.lock().unwrap_or_else(PoisonError::into_inner).as_mut() {
            gate.end();
        }
    }
}

/// A new-window request (`target=_blank`, `window.open`, a Settings link): through the gate — the
/// rest dropped, `handoff-throttled` —; from an extension's panel only with `openUrl`'s checks
/// (extensions/popups.rs); then `hand_off_now`.
pub fn hand_off_new_window<R: Runtime>(app: &AppHandle<R>, url: Url, from: NewWindowFrom) {
    let started = GATE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert_with(HandoffGate::default)
        .try_start(Instant::now());
    if !started {
        smoke::log("handoff-throttled", "");
        return;
    }
    let turn = GateTurn;
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _turn = turn;
        let panel = (from == NewWindowFrom::Page)
            .then(|| {
                crate::extensions::session::showing().or_else(crate::extensions::session::committed)
            })
            .flatten();
        let url = match panel {
            Some(id) => match crate::extensions::popups::check(&app, &id, &url) {
                Ok(url) => url,
                Err(why) => {
                    smoke::log("handoff-refused", why.name());
                    return;
                }
            },
            None => url,
        };
        hand_off_now(&app, &url);
    });
}

/// Hands `url` on (see the module docs), then `then` with how it went — off the calling thread:
/// reading the OS's list may take a moment, and callers run on the main thread.
pub fn hand_off<R: Runtime>(
    app: &AppHandle<R>,
    url: Url,
    then: impl FnOnce(&AppHandle<R>, Outcome) + Send + 'static,
) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let outcome = hand_off_now(&app, &url);
        then(&app, outcome);
    });
}

/// `hand_off` on this thread (extensions' requests already run on a blocking one). Web URLs only.
pub fn hand_off_now<R: Runtime>(app: &AppHandle<R>, url: &Url) -> Outcome {
    if !navigation::is_external_openable(url) {
        return Outcome::Failed("only web addresses are handed on".to_owned());
    }
    let main = state::with(app, |s| s.settings.main_browser.clone());
    let browsers = read();
    let decision = handoff_target(main.as_deref(), &browsers);
    if decision.refused_self {
        smoke::log("handoff-self-refused", "");
    }
    if decision.target == Target::Nowhere {
        smoke::log("handoff-none", "");
        bar::tell_handoff_nowhere(app);
        return Outcome::Nowhere;
    }
    // Remembered before the launch: macOS's `open` can deliver the link back before it returns.
    recent().handed_off(url, Instant::now());
    let opened = match decision.target {
        Target::Browser(browser) => {
            smoke::log("handoff", &format!("browser={}", browser.id));
            platform::browsers::open_with(browser, url).is_ok()
        }
        _ => {
            smoke::log("handoff", "os-default");
            tauri_plugin_opener::open_url(url.as_str(), None::<&str>).is_ok()
        }
    };
    if opened {
        Outcome::Opened
    } else {
        recent().forget(url);
        // A fixed message: the launcher's own error text can hold the command line, and so the URL.
        eprintln!("little-joan: could not hand the page to a browser");
        Outcome::Failed("The browser couldn't be started.".to_owned())
    }
}

/// The pages just handed on (the loop guard: links.rs refuses one that comes straight back).
static RECENT: Mutex<Recent> = Mutex::new(Recent::new());

fn recent() -> std::sync::MutexGuard<'static, Recent> {
    RECENT.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Whether a link from another app is a page Little Joan just handed on, coming back (links.rs).
pub fn bounced(url: &Url) -> bool {
    recent().bounced(url, Instant::now())
}

/// Opens the OS's own default-apps settings — the only thing the button does (Ground Rule 21):
/// what it opened, or `None` when this desktop has nothing to open (Settings says where to choose).
pub fn open_default_apps_settings() -> Result<Option<&'static str>, String> {
    platform::browsers::open_default_apps(|uri| {
        tauri_plugin_opener::open_url(uri, None::<&str>).map_err(std::io::Error::other)
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
