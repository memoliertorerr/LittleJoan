//! macOS: WKWebView's `customUserAgent` (nil = the engine's own user agent), WebKit's element
//! fullscreen (`fullscreenState`), and the general pasteboard (extensions' clipboard.writeText).

use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, sel};
use objc2_foundation::NSString;
use tauri::{AppHandle, Runtime, Webview};

use crate::bar;
use crate::fullscreen::WebKitFullscreen;
use crate::keys::Platform;
use crate::phone;
use crate::smoke;

pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    let result = page.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        if view.is_null() {
            return;
        }
        let user_agent =
            mobile.then(|| NSString::from_str(&phone::mobile_user_agent(Platform::MacOs, None)));
        // SAFETY: `view` is the live WKWebView of this webview (with_webview runs on the main
        // thread while it exists); setCustomUserAgent: takes an NSString or nil.
        unsafe {
            let _: () = msg_send![&*view, setCustomUserAgent: user_agent.as_deref()];
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not change the user agent: {err}");
    }
}

/// `[view fullscreenState]` where the WKWebView has it (macOS 13+).
fn fullscreen_state(view: *mut AnyObject) -> Option<WebKitFullscreen> {
    // SAFETY: `view` is the live WKWebView handed to a with_webview closure (main thread);
    // respondsToSelector: takes a selector, and fullscreenState (asked only when it exists)
    // returns a WKFullscreenState, an NSInteger.
    unsafe {
        let view = &*view;
        let known: bool = msg_send![view, respondsToSelector: sel!(fullscreenState)];
        if !known {
            return None;
        }
        let raw: isize = msg_send![view, fullscreenState];
        Some(WebKitFullscreen::from_raw(raw))
    }
}

/// The last `wk-fullscreen` state the smoke log got (-1: none yet, else `WebKitFullscreen` as
/// 0–3).
static LOGGED: AtomicI8 = AtomicI8::new(-1);
static WATCHING: AtomicBool = AtomicBool::new(false);

/// Smoke mode only (CI): logs `wk-fullscreen <not|entering|in|exiting>` whenever WebKit's
/// fullscreen state changes, polled every 100 ms while a page window exists. Nothing runs
/// without the smoke log, nor at rest.
pub fn watch_webkit_fullscreen<R: Runtime>(app: &AppHandle<R>) {
    if !smoke::enabled() || WATCHING.swap(true, Ordering::SeqCst) {
        return;
    }
    LOGGED.store(-1, Ordering::SeqCst);
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-wk-fullscreen".into())
        .spawn(move || {
            while let Some(page) = bar::page(&app) {
                let _ = page.with_webview(|webview| {
                    let view = webview.inner().cast::<AnyObject>();
                    if view.is_null() {
                        return;
                    }
                    let state = fullscreen_state(view).unwrap_or(WebKitFullscreen::Not);
                    let code = state as i8;
                    if LOGGED.swap(code, Ordering::SeqCst) != code {
                        smoke::log("wk-fullscreen", state.name());
                    }
                });
                std::thread::sleep(Duration::from_millis(100));
            }
            WATCHING.store(false, Ordering::SeqCst);
        });
    if spawned.is_err() {
        WATCHING.store(false, Ordering::SeqCst);
    }
}

/// Whether the smoke log's last `wk-fullscreen` line was anything but `not` (smoke mode only).
pub fn webkit_fullscreen_logged() -> bool {
    LOGGED.load(Ordering::SeqCst) > 0
}

/// Puts `text` on the general pasteboard as plain text. Main thread only (AppKit).
pub fn write_clipboard(text: &str) -> Result<(), String> {
    let class = AnyClass::get(c"NSPasteboard").ok_or_else(|| "no NSPasteboard".to_owned())?;
    let text = NSString::from_str(text);
    // NSPasteboardTypeString's value.
    let kind = NSString::from_str("public.utf8-plain-text");
    // SAFETY: +generalPasteboard returns the shared pasteboard; -clearContents takes nothing and
    // -setString:forType: two NSStrings; AppKit is linked (tao) and this runs on the main thread.
    unsafe {
        let board: Option<Retained<AnyObject>> = msg_send![class, generalPasteboard];
        let board = board.ok_or_else(|| "no pasteboard".to_owned())?;
        let _: isize = msg_send![&*board, clearContents];
        let written: bool = msg_send![&*board, setString: &*text, forType: &*kind];
        if written {
            Ok(())
        } else {
            Err("the pasteboard refused the text".to_owned())
        }
    }
}
