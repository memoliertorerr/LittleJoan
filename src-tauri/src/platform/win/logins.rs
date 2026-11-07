//! Windows: WebView2's own password saving and form autofill (LJ51; Ground Rule 26).
//! `ICoreWebView2Settings4` switches `IsPasswordAutosaveEnabled` and `IsGeneralAutofillEnabled` on a
//! webview — WebView2 shares both across the webviews of a profile — and reads them back for the
//! smoke log; `ICoreWebView2Profile2::ClearBrowsingData(PASSWORD_AUTOSAVE | GENERAL_AUTOFILL)` asks
//! the engine to remove what it saved, through the settings window's webview (the page's profile:
//! `ICoreWebView2_13::Profile`'s `ProfilePath`, which CI compares). Nothing here reads, exports or
//! logs a saved password or form entry: only the two switches and the engine's answer.

use std::sync::mpsc;
use std::time::Duration;

use tauri::{Runtime, Webview};
use webview2_com::ClearBrowsingDataCompletedHandler;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_BROWSING_DATA_KINDS_GENERAL_AUTOFILL,
    COREWEBVIEW2_BROWSING_DATA_KINDS_PASSWORD_AUTOSAVE, ICoreWebView2_13, ICoreWebView2Controller,
    ICoreWebView2Profile, ICoreWebView2Profile2, ICoreWebView2Settings4,
};
use windows::core::{BOOL, Interface, PWSTR};

use crate::logins::{Place, line};
use crate::smoke;

/// How long removing waits for the engine's answer.
const CLEAR_WAIT: Duration = Duration::from_secs(15);

/// The webview's settings, where its runtime has the two switches.
///
/// SAFETY: COM calls on a live controller, on the UI thread.
unsafe fn settings_of(
    controller: &ICoreWebView2Controller,
) -> windows::core::Result<ICoreWebView2Settings4> {
    // SAFETY: as above.
    unsafe { controller.CoreWebView2()?.Settings()?.cast() }
}

/// The two switches as the engine says they are: (password autosave, general autofill).
///
/// SAFETY: COM getters on a live webview's settings, on the UI thread.
unsafe fn read(settings: &ICoreWebView2Settings4) -> windows::core::Result<(bool, bool)> {
    let (mut save, mut autofill) = (BOOL::default(), BOOL::default());
    // SAFETY: as above.
    unsafe {
        settings.IsPasswordAutosaveEnabled(&mut save)?;
        settings.IsGeneralAutofillEnabled(&mut autofill)?;
    }
    Ok((save.as_bool(), autofill.as_bool()))
}

/// Sets both switches, then reads them back.
///
/// SAFETY: COM calls on a live controller, on the UI thread.
unsafe fn set_and_read(
    controller: &ICoreWebView2Controller,
    save: bool,
    autofill: bool,
) -> windows::core::Result<(bool, bool)> {
    // SAFETY: as above.
    unsafe {
        let settings = settings_of(controller)?;
        settings.SetIsPasswordAutosaveEnabled(save)?;
        settings.SetIsGeneralAutofillEnabled(autofill)?;
        read(&settings)
    }
}

/// The webview's profile.
///
/// SAFETY: COM calls on a live controller, on the UI thread.
unsafe fn profile_of(
    controller: &ICoreWebView2Controller,
) -> windows::core::Result<ICoreWebView2Profile> {
    // SAFETY: as above.
    unsafe {
        controller
            .CoreWebView2()?
            .cast::<ICoreWebView2_13>()?
            .Profile()
    }
}

/// An engine error on one line, for the smoke log.
fn one_line(err: &windows::core::Error) -> String {
    err.message()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Sets the switches on `webview` (queued on the main thread, never waited for): `logins
/// password-autosave=<bool> autofill=<bool> webview=<place>` as read back, or `logins failed …`.
pub fn apply<R: Runtime>(webview: &Webview<R>, save: bool, autofill: bool, place: Place) {
    let reached = webview.with_webview(move |platform| {
        // SAFETY: with_webview runs on the UI thread while the webview (and its controller) lives.
        match unsafe { set_and_read(&platform.controller(), save, autofill) } {
            Ok((save, autofill)) => smoke::log("logins", &line(save, autofill, place)),
            Err(err) => {
                eprintln!("little-joan: could not switch the engine's password saving: {err}");
                smoke::log(
                    "logins",
                    &format!("failed webview={} {}", place.name(), one_line(&err)),
                );
            }
        }
    });
    if let Err(err) = reached {
        eprintln!("little-joan: could not reach the webview: {err}");
    }
}

/// Smoke mode only: `webview-profile <place>=<the profile's folder>` and `logins-seen …` (the
/// switches as they are, nothing set).
pub fn report<R: Runtime>(webview: &Webview<R>, place: Place) {
    if !smoke::enabled() {
        return;
    }
    let _ = webview.with_webview(move |platform| {
        let controller = platform.controller();
        let mut raw = PWSTR::null();
        // SAFETY: COM calls on the live controller (UI thread); ProfilePath hands back a string
        // WebView2 allocated, which take_pwstr frees.
        let path = unsafe { profile_of(&controller).and_then(|p| p.ProfilePath(&mut raw)) }
            .map(|()| webview2_com::take_pwstr(raw));
        let detail = match path {
            Ok(path) => format!("{}={path}", place.name()),
            Err(err) => format!("{}=unknown {}", place.name(), one_line(&err)),
        };
        smoke::log("webview-profile", &detail);
        // SAFETY: as above.
        if let Ok((save, autofill)) = unsafe { settings_of(&controller).and_then(|s| read(&s)) } {
            smoke::log("logins-seen", &line(save, autofill, place));
        }
    });
}

/// Starts removing on the UI thread; the engine answers on `reply`.
///
/// SAFETY: COM calls on a live controller, on the UI thread.
unsafe fn start_clear(
    controller: &ICoreWebView2Controller,
    reply: mpsc::Sender<Result<(), String>>,
) -> windows::core::Result<()> {
    // SAFETY: as above.
    unsafe {
        let profile: ICoreWebView2Profile2 = profile_of(controller)?.cast()?;
        let handler = ClearBrowsingDataCompletedHandler::create(Box::new(move |result| {
            let _ = reply.send(result.map_err(|err| one_line(&err)));
            Ok(())
        }));
        profile.ClearBrowsingData(
            COREWEBVIEW2_BROWSING_DATA_KINDS_PASSWORD_AUTOSAVE
                | COREWEBVIEW2_BROWSING_DATA_KINDS_GENERAL_AUTOFILL,
            &handler,
        )
    }
}

/// Removes the passwords and form data the engine saved in `webview`'s profile — nothing else of
/// it (cookies and site data stay) — and waits for the engine's answer. Never on the main thread.
pub fn clear<R: Runtime>(webview: &Webview<R>) -> Result<(), String> {
    if super::on_main_thread() {
        return Err("on the main thread".to_owned());
    }
    let (sent, answer) = mpsc::channel::<Result<(), String>>();
    let reached = webview.with_webview(move |platform| {
        let failed = sent.clone();
        // SAFETY: with_webview runs on the UI thread while the webview (and its controller) lives.
        if let Err(err) = unsafe { start_clear(&platform.controller(), sent) } {
            let _ = failed.send(Err(one_line(&err)));
        }
    });
    reached.map_err(|err| err.to_string())?;
    answer
        .recv_timeout(CLEAR_WAIT)
        .map_err(|_| "Windows' web engine didn't answer.".to_owned())?
}
