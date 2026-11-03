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

pub mod parse;
pub mod target;

pub use target::{Browser, Browsers, Decision, Launch, Recent, Target, handoff_target};

/// How a handoff went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Opened,
    /// No browser but Little Joan: nothing opened.
    Nowhere,
    Failed(String),
}

/// What the OS says now, logged for the smoke test.
pub fn read() -> Browsers {
    let browsers = platform::browsers::read();
    smoke::log(
        "browsers-listed",
        &format!(
            "{} default={}",
            browsers.others().count(),
            browsers.default_kind()
        ),
    );
    browsers
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
    let opened = match decision.target {
        Target::Nowhere => {
            smoke::log("handoff-none", "");
            bar::tell_handoff_nowhere(app);
            return Outcome::Nowhere;
        }
        Target::OsDefault => {
            smoke::log("handoff", "os-default");
            tauri_plugin_opener::open_url(url.as_str(), None::<&str>).map_err(|e| e.to_string())
        }
        Target::Browser(browser) => {
            smoke::log("handoff", &format!("browser={}", browser.id));
            platform::browsers::open_with(browser, url).map_err(|e| e.to_string())
        }
    };
    match opened {
        Ok(()) => {
            recent().handed_off(url, Instant::now());
            Outcome::Opened
        }
        Err(err) => {
            eprintln!("little-joan: could not hand the page to a browser: {err}");
            Outcome::Failed(err)
        }
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
