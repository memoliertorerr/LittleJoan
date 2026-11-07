//! macOS: a sign-in popup's `window.close()` (LJ47). WebKit tells the web view's UI delegate
//! (`-webViewDidClose:`), which wry's delegate doesn't implement, so the call did nothing. The
//! method is added to wry's delegate class once, before the first popup is built — WebKit reads
//! which methods a delegate has when it is set on a web view — and it closes the popup whose web
//! view it names; for any other web view (the page calling `window.close()`) it does nothing, as
//! before.

use std::cell::RefCell;
use std::sync::{Once, OnceLock};

use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
use objc2::{msg_send, sel};
use tauri::webview::NewWindowFeatures;
use tauri::{AppHandle, Runtime, WebviewWindow};

/// `-webViewDidClose:`'s signature: (self, _cmd, the WKWebView).
type DidClose = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject);

static ADDED: Once = Once::new();
/// What a popup's `window.close()` does: closes the popup whose WKWebView it is.
static ON_CLOSE: OnceLock<Box<dyn Fn(usize) + Send + Sync>> = OnceLock::new();

thread_local! {
    /// Each popup's WKWebView and label (main thread only: the engine's callbacks run there).
    static VIEWS: RefCell<Vec<(usize, String)>> = const { RefCell::new(Vec::new()) };
}

/// The added `-webViewDidClose:`.
unsafe extern "C-unwind" fn did_close(_this: *mut AnyObject, _cmd: Sel, webview: *mut AnyObject) {
    if let Some(on_close) = ON_CLOSE.get() {
        on_close(webview as usize);
    }
}

/// Before a popup is built: `-webViewDidClose:` on wry's UI delegate class (the opener's web view
/// has one), once.
pub fn prepare<R: Runtime>(app: &AppHandle<R>, features: &NewWindowFeatures) {
    let app = app.clone();
    let _ = ON_CLOSE.set(Box::new(move |view| {
        let label = VIEWS.with(|views| {
            views
                .borrow()
                .iter()
                .find(|(known, _)| *known == view)
                .map(|(_, label)| label.clone())
        });
        if let Some(label) = label {
            crate::bar::popup::closed_by_page(&app, &label);
        }
    }));
    ADDED.call_once(|| {
        // The web view that asked for the popup (tauri's new-window opener, not the opener
        // plugin: browsers/tests.rs keeps that one to browsers.rs).
        let opener = &NewWindowFeatures::opener(features).webview;
        // SAFETY: the opener's live WKWebView; -UIDelegate returns its delegate (wry's) or nil.
        let delegate: *mut AnyObject = unsafe { msg_send![&**opener, UIDelegate] };
        let class: Option<&AnyClass> = if delegate.is_null() {
            AnyClass::get(c"WryWebViewUIDelegate")
        } else {
            // SAFETY: a live Objective-C object (the delegate the web view holds).
            Some(unsafe { (*delegate).class() })
        };
        let Some(class) = class else {
            eprintln!("little-joan: no web view delegate; a popup's window.close() does nothing");
            return;
        };
        if class.instance_method(sel!(webViewDidClose:)).is_some() {
            return;
        }
        // SAFETY: `did_close` has the method's signature (an object argument, no result) and its
        // type encoding is "v@:@"; adding a method a class lacks is what class_addMethod is for.
        let added = unsafe {
            objc2::ffi::class_addMethod(
                std::ptr::from_ref(class).cast_mut(),
                sel!(webViewDidClose:),
                std::mem::transmute::<DidClose, Imp>(did_close),
                c"v@:@".as_ptr(),
            )
        };
        if !added.as_bool() {
            eprintln!("little-joan: could not follow a popup's window.close()");
        }
    });
}

/// A popup was built: its WKWebView is remembered for `-webViewDidClose:`.
pub fn install<R: Runtime>(popup: &WebviewWindow<R>) {
    let label = popup.label().to_owned();
    let result = popup.with_webview(move |webview| {
        let view = webview.inner() as usize;
        VIEWS.with(|views| {
            let mut views = views.borrow_mut();
            // A new web view may reuse a gone one's address.
            views.retain(|(known, _)| *known != view);
            views.push((view, label));
        });
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the popup: {err}");
    }
}

/// A popup is gone: its WKWebView is forgotten.
pub fn forget(label: &str) {
    VIEWS.with(|views| views.borrow_mut().retain(|(_, known)| known != label));
}
