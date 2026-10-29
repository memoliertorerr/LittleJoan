//! Windows, LJ19.4: the bar's `ProcessFailed` (a ready window whose bar is gone is built anew at
//! the next summon), the bar's visibility and memory target while the window rests, and the page's
//! downloads noted (`DownloadStarting`) so closing the page cancels them — the window's WebView2
//! environment, which used to take them along, now outlives the page.

use std::cell::RefCell;

use tauri::{AppHandle, Runtime, Webview};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
    COREWEBVIEW2_PROCESS_FAILED_KIND, COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED,
    COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED,
    COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE, ICoreWebView2_4,
    ICoreWebView2_19, ICoreWebView2Controller, ICoreWebView2DownloadOperation,
};
use webview2_com::{DownloadStartingEventHandler, ProcessFailedEventHandler};
use windows::core::Interface;

use crate::smoke;

/// The bar's processes failing (LJ19.4): its renderer exited or hangs, or the browser process
/// exited. A ready window's bar is then built anew at the next summon (joan.rs); a GPU or utility
/// process WebView2 restarts by itself doesn't count.
pub(super) fn install_process_failed<R: Runtime>(
    controller: &ICoreWebView2Controller,
    app: AppHandle<R>,
) {
    // SAFETY: reads the core webview of a live controller.
    let core = match unsafe { controller.CoreWebView2() } {
        Ok(core) => core,
        Err(err) => {
            eprintln!("little-joan: no core webview for the bar's process watch: {err}");
            return;
        }
    };
    let handler = ProcessFailedEventHandler::create(Box::new(move |_, args| {
        let mut kind = COREWEBVIEW2_PROCESS_FAILED_KIND::default();
        if let Some(args) = args {
            // SAFETY: a plain COM getter on the event args WebView2 passed to this handler.
            unsafe { args.ProcessFailedKind(&mut kind)? };
        }
        let gone = [
            COREWEBVIEW2_PROCESS_FAILED_KIND_BROWSER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_EXITED,
            COREWEBVIEW2_PROCESS_FAILED_KIND_RENDER_PROCESS_UNRESPONSIVE,
        ];
        if gone.contains(&kind) {
            crate::joan::bar_process_ended(&app, &format!("kind={}", kind.0));
        } else {
            smoke::log("bar-process-failed", &format!("kind={} (restarts)", kind.0));
        }
        Ok(())
    }));
    let mut token = 0i64;
    // SAFETY: registers a handler on the live core webview.
    if let Err(err) = unsafe { core.add_ProcessFailed(&handler, &mut token) } {
        eprintln!("little-joan: could not watch the bar's processes: {err}");
    }
}

thread_local! {
    /// The page's downloads since it was built (the UI thread only): cancelled when it closes.
    static DOWNLOADS: RefCell<Vec<ICoreWebView2DownloadOperation>> = const { RefCell::new(Vec::new()) };
}

/// Notes every download the page starts (wry's own `DownloadStarting` handler decides where it
/// goes), so closing the page can cancel the ones under way (`cancel_downloads`).
pub(super) fn note_downloads(controller: &ICoreWebView2Controller) {
    // SAFETY: reads the core webview of a live controller; DownloadStarting is ICoreWebView2_4's.
    let core = match unsafe { controller.CoreWebView2() }
        .and_then(|core| core.cast::<ICoreWebView2_4>())
    {
        Ok(core) => core,
        Err(err) => {
            eprintln!("little-joan: downloads can't be followed: {err}");
            return;
        }
    };
    let handler = DownloadStartingEventHandler::create(Box::new(|_, args| {
        if let Some(args) = args {
            // SAFETY: a plain COM getter on the event args WebView2 passed to this handler.
            let operation = unsafe { args.DownloadOperation()? };
            DOWNLOADS.with(|noted| noted.borrow_mut().push(operation));
            smoke::log("download-noted", "");
        }
        Ok(())
    }));
    let mut token = 0i64;
    // SAFETY: registers a handler on the live core webview.
    if let Err(err) = unsafe { core.add_DownloadStarting(&handler, &mut token) } {
        eprintln!("little-joan: downloads can't be followed: {err}");
    }
}

/// The page closes: every noted download is cancelled on the UI thread (a finished one refuses,
/// which changes nothing), and WebView2 removes a cancelled one's partial file.
pub fn cancel_downloads<R: Runtime>(app: &AppHandle<R>) {
    let result = app.run_on_main_thread(|| {
        let noted = DOWNLOADS.with(|noted| std::mem::take(&mut *noted.borrow_mut()));
        for operation in noted {
            // SAFETY: a download operation of this process's WebView2, on the UI thread.
            let _ = unsafe { operation.Cancel() };
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not cancel the page's downloads: {err}");
    }
}

/// Smoke tests only: `LITTLEJOAN_SMOKE_WIN_REST=plain` leaves the bar as it is at rest, for CI to
/// compare memory and summon times with and without `bar_at_rest` (LJ19.4's design 9).
fn plain_rest() -> bool {
    smoke::enabled() && std::env::var("LITTLEJOAN_SMOKE_WIN_REST").is_ok_and(|v| v == "plain")
}

/// The bar while the ready window rests (LJ19.4): hidden — its controller's `IsVisible` off, so
/// WebView2 stops drawing it — and its memory target low (`ICoreWebView2_19`), so WebView2 may
/// trim what the bar's renderer holds; back to normal and shown before the window shows again.
/// Smoke log: `bar-rest <low|normal>`.
pub fn bar_at_rest<R: Runtime>(bar: &Webview<R>, rest: bool) {
    if plain_rest() {
        smoke::log("bar-rest", "plain");
        return;
    }
    if rest {
        let _ = bar.hide();
        set_memory_target(bar, true);
    } else {
        set_memory_target(bar, false);
        let _ = bar.show();
    }
    smoke::log("bar-rest", if rest { "low" } else { "normal" });
}

fn set_memory_target<R: Runtime>(bar: &Webview<R>, low: bool) {
    let level = if low {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
    } else {
        COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
    };
    let result = bar.with_webview(move |webview| {
        // SAFETY: COM calls on the live controller of this webview, on the UI thread;
        // MemoryUsageTargetLevel is ICoreWebView2_19's (WebView2 Runtime 114+).
        let set = unsafe {
            webview
                .controller()
                .CoreWebView2()
                .and_then(|core| core.cast::<ICoreWebView2_19>())
                .and_then(|core| core.SetMemoryUsageTargetLevel(level))
        };
        if let Err(err) = set {
            eprintln!("little-joan: could not set the bar's memory target: {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the bar: {err}");
    }
}
