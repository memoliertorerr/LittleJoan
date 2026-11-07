//! Saved logins per engine (LJ51; logins.rs decides when): Windows' WebView2 switches its password
//! saving and form autofill per profile and removes what it saved (win/logins.rs). WKWebView and
//! WebKitGTK offer an app nothing to switch (macOS: Apple's browser entitlement; Linux: no password
//! manager), so there is nothing to do there and removing says it isn't available.

use tauri::{Runtime, Webview};

use crate::logins::Place;

/// Sets the engine's password saving (`save`) and form autofill (`autofill`) on `webview` and logs
/// what it reads back (`logins …`). Queued on the main thread; never waits.
#[cfg(windows)]
pub fn apply_logins<R: Runtime>(webview: &Webview<R>, save: bool, autofill: bool, place: Place) {
    super::win::logins::apply(webview, save, autofill, place);
}

#[cfg(not(windows))]
pub fn apply_logins<R: Runtime>(
    _webview: &Webview<R>,
    _save: bool,
    _autofill: bool,
    _place: Place,
) {
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
