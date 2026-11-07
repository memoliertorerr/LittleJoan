//! WebView2's find (LJ49): `ICoreWebView2_28::Find` → `ICoreWebView2Find` — `Start` with options
//! from `ICoreWebView2Environment15::CreateFindOptions` (the query, case-insensitive, every match
//! highlighted, WebView2's own find dialog suppressed), `FindNext`, `FindPrevious`, `Stop`. The
//! match count and the active match (1-based) come from its `MatchCountChanged` and
//! `ActiveMatchIndexChanged` events, connected once per page (`install`), and are read again when a
//! `Start` completes. A runtime without `ICoreWebView2_28` has no find: the strip says so. Ctrl+F
//! never reaches WebView2's own find bar: the keymap catches it in `AcceleratorKeyPressed`.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, Runtime, Webview};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2, ICoreWebView2_2, ICoreWebView2_28, ICoreWebView2Controller,
    ICoreWebView2Environment15, ICoreWebView2Find,
};
use webview2_com::{
    FindActiveMatchIndexChangedEventHandler, FindMatchCountChangedEventHandler,
    FindStartCompletedHandler,
};
use windows::core::{HSTRING, Interface};

use crate::bar;
use crate::find::{self, EngineOp, Report};
use crate::smoke;

/// The engine's line went to the smoke log (once per process).
static LOGGED: AtomicBool = AtomicBool::new(false);

/// The page's find, where its runtime has one.
///
/// SAFETY: COM calls on a live core webview, on the UI thread.
unsafe fn find_of(core: &ICoreWebView2) -> Option<ICoreWebView2Find> {
    // SAFETY: as above.
    unsafe { core.cast::<ICoreWebView2_28>().ok()?.Find().ok() }
}

/// Smoke log, once: `find-engine webview2 <runtime version> find=<bool>` (CI records the runtime).
///
/// SAFETY: as `find_of`.
unsafe fn log_engine(core: &ICoreWebView2, available: bool) {
    if !smoke::enabled() || LOGGED.swap(true, Ordering::SeqCst) {
        return;
    }
    // SAFETY: as above.
    let version = unsafe { super::engine_version(core) }.unwrap_or_else(|| "unknown".to_owned());
    smoke::log(
        "find-engine",
        &format!("webview2 {version} find={available}"),
    );
}

/// Reports what the engine says now: the match count, and the active match while a find session
/// exists (`MatchCount` is −1 without one; `ActiveMatchIndex` −1 with no active match).
fn report_now<R: Runtime>(app: &AppHandle<R>, generation: Option<u64>, finder: &ICoreWebView2Find) {
    let (mut matches, mut active) = (-1i32, -1i32);
    // SAFETY: plain COM getters on a live find object, on the UI thread (its events and the
    // completion handler run there).
    let read = unsafe {
        finder
            .MatchCount(&mut matches)
            .and_then(|()| finder.ActiveMatchIndex(&mut active))
    };
    if read.is_err() {
        return;
    }
    let Ok(matches) = u32::try_from(matches) else {
        return;
    };
    bar::find::report(app, generation, Report::Count(matches));
    bar::find::report(
        app,
        generation,
        Report::Active(u32::try_from(active).unwrap_or(0)),
    );
}

/// The page's find events (`install_page`): its match count and active match, reported for the
/// current find.
pub(super) fn install<R: Runtime>(controller: &ICoreWebView2Controller, app: AppHandle<R>) {
    // SAFETY: COM calls on the live controller of this webview, on the UI thread (with_webview).
    let Ok(core) = (unsafe { controller.CoreWebView2() }) else {
        return;
    };
    // SAFETY: as above.
    let Some(finder) = (unsafe { find_of(&core) }) else {
        // SAFETY: as above.
        unsafe { log_engine(&core, false) };
        return;
    };
    // SAFETY: as above.
    unsafe { log_engine(&core, true) };
    let counted = app.clone();
    let on_count = FindMatchCountChangedEventHandler::create(Box::new(move |sender, _| {
        if let Some(finder) = sender {
            report_now(&counted, None, &finder);
        }
        Ok(())
    }));
    let on_active = FindActiveMatchIndexChangedEventHandler::create(Box::new(move |sender, _| {
        if let Some(finder) = sender {
            report_now(&app, None, &finder);
        }
        Ok(())
    }));
    let mut token = 0i64;
    // SAFETY: registers handlers on the live find object of this webview (UI thread).
    unsafe {
        if let Err(err) = finder.add_MatchCountChanged(&on_count, &mut token) {
            eprintln!("little-joan: could not follow find's match count: {err}");
        }
        if let Err(err) = finder.add_ActiveMatchIndexChanged(&on_active, &mut token) {
            eprintln!("little-joan: could not follow find's active match: {err}");
        }
    }
}

/// Runs `op` on the page's find, on the UI thread.
pub fn run<R: Runtime>(page: &Webview<R>, generation: u64, op: EngineOp) {
    let app = page.app_handle().clone();
    let result = page.with_webview(move |webview| {
        let controller = webview.controller();
        // SAFETY: with_webview runs on the UI thread while the webview (and its controller) lives.
        if let Err(err) = unsafe { start(&controller, &app, generation, op) } {
            eprintln!("little-joan: find failed: {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the page for find: {err}");
    }
}

/// `op` on the live controller's find; no find in this runtime: the strip is told.
///
/// SAFETY: `controller` is the live controller of a webview, and this runs on the UI thread.
unsafe fn start<R: Runtime>(
    controller: &ICoreWebView2Controller,
    app: &AppHandle<R>,
    generation: u64,
    op: EngineOp,
) -> windows::core::Result<()> {
    // SAFETY: as above: COM calls on the live controller, its core webview, its environment and
    // its find object.
    unsafe {
        let core = controller.CoreWebView2()?;
        let finder = find_of(&core);
        log_engine(&core, finder.is_some());
        let options = match (&finder, &op) {
            (Some(_), EngineOp::Start(_)) => core
                .cast::<ICoreWebView2_2>()
                .and_then(|core| core.Environment())
                .and_then(|environment| environment.cast::<ICoreWebView2Environment15>())
                .and_then(|environment| environment.CreateFindOptions())
                .ok(),
            _ => None,
        };
        let Some(finder) = finder else {
            if op != EngineOp::Stop {
                bar::find::report(app, Some(generation), Report::Unavailable);
            }
            return Ok(());
        };
        match op {
            EngineOp::Start(query) => {
                let Some(options) = options else {
                    bar::find::report(app, Some(generation), Report::Unavailable);
                    return Ok(());
                };
                options.SetFindTerm(&HSTRING::from(query.as_str()))?;
                options.SetIsCaseSensitive(find::CASE_SENSITIVE)?;
                options.SetShouldMatchWord(false)?;
                options.SetShouldHighlightAllMatches(true)?;
                options.SetSuppressDefaultFindDialog(true)?;
                let app = app.clone();
                let started = finder.clone();
                let handler = FindStartCompletedHandler::create(Box::new(move |result| {
                    if result.is_ok() {
                        report_now(&app, Some(generation), &started);
                    }
                    Ok(())
                }));
                finder.Start(&options, &handler)
            }
            EngineOp::Next(_) => finder.FindNext(),
            EngineOp::Previous(_) => finder.FindPrevious(),
            EngineOp::Stop => finder.Stop(),
        }
    }
}
