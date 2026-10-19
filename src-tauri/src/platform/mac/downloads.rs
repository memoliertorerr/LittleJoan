//! macOS: the page's downloads end with the window (LJ15). A `WKDownload` lives in WebKit's
//! network process, not in the web view: CI saw a download go on after the window — and its
//! WKWebView — had gone, still writing the file. wry keeps the WKDownloads to itself (its download
//! delegate gets them; Tauri's `on_download` gets only the URL), so Little Joan follows them where
//! every one passes: wry gives each its delegate with `-[WKDownload setDelegate:]`, whose
//! implementation is wrapped once (`follow`) to note the download, then set the delegate as
//! before. When the window closes (`cancel`), each noted download is cancelled (on a finished
//! one that does nothing) and the partial files of those that were under way are removed.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Once, OnceLock};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
use objc2::{msg_send, sel};
use tauri::{AppHandle, Runtime};

use crate::smoke;

/// `-[WKDownload setDelegate:]`'s signature: (self, _cmd, the delegate).
type SetDelegate = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject);

static FOLLOW: Once = Once::new();
/// The setter's own implementation, called after the download is noted.
static ORIGINAL: OnceLock<Imp> = OnceLock::new();

thread_local! {
    /// The WKDownloads given a delegate since the window opened (main thread only).
    static NOTED: RefCell<Vec<Retained<AnyObject>>> = const { RefCell::new(Vec::new()) };
}

/// The wrapped `-[WKDownload setDelegate:]`: notes the download, then sets its delegate as WebKit
/// would.
unsafe extern "C-unwind" fn set_delegate(this: *mut AnyObject, cmd: Sel, delegate: *mut AnyObject) {
    // SAFETY: `this` is the WKDownload the setter was sent to (WebKit and wry call it on the main
    // thread); retaining it keeps it for `cancel`.
    if let Some(download) = unsafe { Retained::retain(this) } {
        NOTED.with(|noted| noted.borrow_mut().push(download));
        smoke::log("download-noted", "");
    }
    if let Some(original) = ORIGINAL.get() {
        // SAFETY: the setter's original implementation, which has exactly this signature.
        unsafe {
            let original = std::mem::transmute::<Imp, SetDelegate>(*original);
            original(this, cmd, delegate);
        }
    }
}

/// Wraps `-[WKDownload setDelegate:]` once (the Objective-C runtime is thread-safe for this).
pub fn follow() {
    FOLLOW.call_once(|| {
        let Some(class) = AnyClass::get(c"WKDownload") else {
            eprintln!("little-joan: no WKDownload class; downloads go on after the window closes");
            return;
        };
        let Some(method) = class.instance_method(sel!(setDelegate:)) else {
            eprintln!("little-joan: no -[WKDownload setDelegate:]; downloads go on after closing");
            return;
        };
        if ORIGINAL.set(method.implementation()).is_err() {
            return;
        }
        // SAFETY: `set_delegate` has the setter's signature (an object argument, no result), and
        // after noting the download it calls the original implementation with the same arguments.
        unsafe {
            method.set_implementation(std::mem::transmute::<SetDelegate, Imp>(set_delegate));
        }
    });
}

/// The window is closing: cancels the noted downloads (main thread) and removes the partial files
/// of those that were under way (`partial`), once WebKit has had a moment to stop writing.
pub fn cancel<R: Runtime>(app: &AppHandle<R>, partial: Vec<PathBuf>) {
    let result = app.run_on_main_thread(move || {
        let noted = NOTED.with(|noted| std::mem::take(&mut *noted.borrow_mut()));
        if partial.is_empty() {
            return;
        }
        for download in &noted {
            // SAFETY: a WKDownload; -cancel: takes a block that may be nil (no resume data
            // wanted), sent through -performSelector:withObject: with nil.
            let _: *mut AnyObject = unsafe {
                msg_send![&**download, performSelector: sel!(cancel:), withObject: std::ptr::null::<AnyObject>()]
            };
        }
        smoke::log("download-cancelled", &noted.len().to_string());
        let removing = std::thread::Builder::new()
            .name("little-joan-download-partial".into())
            .spawn(move || {
                std::thread::sleep(Duration::from_millis(500));
                for path in partial {
                    let _ = std::fs::remove_file(&path);
                }
            });
        if let Err(err) = removing {
            eprintln!("little-joan: could not remove partial downloads: {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not cancel the downloads: {err}");
    }
}
