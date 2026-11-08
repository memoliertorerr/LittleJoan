//! Saved logins per engine (LJ51; logins.rs decides when): Windows' WebView2 switches its password
//! saving and form autofill per profile and removes what it saved (win/logins.rs). WKWebView and
//! WebKitGTK offer an app nothing to switch (macOS: Apple's browser entitlement; Linux: no password
//! manager), so there is nothing to do there and removing says it isn't available.

use tauri::{Runtime, Webview};

use crate::logins::Place;

/// Sets the engine's password saving and form autofill on `webview` to what `switches` says
/// (`(save, autofill)`, asked on the main thread when the engine is reached) and logs what it reads
/// back (`logins …`). Queued on the main thread; never waits.
#[cfg(windows)]
pub fn apply_logins<R, F>(webview: &Webview<R>, place: Place, switches: F)
where
    R: Runtime,
    F: FnOnce() -> (bool, bool) + Send + 'static,
{
    super::win::logins::apply(webview, place, switches);
}

#[cfg(not(windows))]
pub fn apply_logins<R, F>(_webview: &Webview<R>, _place: Place, _switches: F)
where
    R: Runtime,
    F: FnOnce() -> (bool, bool) + Send + 'static,
{
}

/// Smoke log only: the webview's profile folder (`webview-profile <place>=<path>`) and its switches
/// as they are (`logins-seen …`).
#[cfg(windows)]
pub fn report_logins<R: Runtime>(webview: &Webview<R>, place: Place) {
    super::win::logins::report(webview, place);
}

#[cfg(not(windows))]
pub fn report_logins<R: Runtime>(_webview: &Webview<R>, _place: Place) {}

/// Asks the engine to remove the passwords and form data it saved in `webview`'s profile, and
/// waits for its answer (never on the main thread).
#[cfg(windows)]
pub fn clear_saved_logins<R: Runtime>(webview: &Webview<R>) -> Result<(), String> {
    super::win::logins::clear(webview)
}

#[cfg(not(windows))]
pub fn clear_saved_logins<R: Runtime>(_webview: &Webview<R>) -> Result<(), String> {
    Err("Saving passwords isn't available in Little Joan on this system.".to_owned())
}
