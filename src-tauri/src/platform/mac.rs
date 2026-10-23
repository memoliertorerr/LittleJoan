//! macOS: WKWebView's `customUserAgent` (nil = the engine's own user agent), its autoresizing
//! mask (LJ39), WebKit's element fullscreen (`fullscreenState`), and the general pasteboard
//! (extensions' clipboard.writeText); the page's downloads are followed to cancel them when the
//! window closes (mac/downloads.rs, LJ15); App Nap stays off while a summon builds the window
//! (mac/activity.rs, LJ19.1); the windows join the active Space and may show over another app's
//! full-screen window (mac/spaces.rs, LJ19.2).

use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::sync::{Mutex, PoisonError, mpsc};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, sel};
use objc2_foundation::NSString;
use tauri::{AppHandle, Runtime, Webview};

use crate::bar;
use crate::fullscreen::WebKitFullscreen;
use crate::keys::Platform;
use crate::layout;
use crate::phone;
use crate::smoke;

/// The page's WKDownloads, cancelled when the window closes (LJ15).
mod downloads;
pub use downloads::{cancel as cancel_downloads, follow as follow_downloads};
/// App Nap and a summon (LJ19.1).
mod activity;
pub use activity::{SummonActivity, nap_state};
/// Spaces and other apps' full-screen windows (LJ19.2).
mod spaces;
pub use spaces::{front, join_active_space};

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

/// Sets the WKWebView's autoresizing mask to `bits` (layout::mask_for, written for AppKit's
/// default unflipped coordinates; swapped for a flipped superview), so AppKit resizes it in its
/// own layout pass during a live resize — wry gives child webviews a fixed size
/// (`NSViewMinYMargin`), and tao's `Resized` arrives a frame after the window changed. Smoke log:
/// `autoresizing <webview> <mask> flipped=<bool> follows=<bool>` (whether the superview resizes
/// its subviews at all).
pub fn set_autoresizing<R: Runtime>(webview: &Webview<R>, bits: usize) {
    let label = webview.label().to_owned();
    let result = webview.with_webview(move |platform| {
        let view = platform.inner().cast::<AnyObject>();
        if view.is_null() {
            return;
        }
        // SAFETY: `view` is the live WKWebView of this webview (with_webview runs on the main
        // thread while it exists); -superview returns an NSView or nil, -isFlipped and
        // -autoresizesSubviews a BOOL, and -setAutoresizingMask: takes an NSUInteger.
        unsafe {
            let view = &*view;
            let superview: Option<Retained<AnyObject>> = msg_send![view, superview];
            let (flipped, follows) = match superview.as_deref() {
                Some(parent) => {
                    let flipped: bool = msg_send![parent, isFlipped];
                    let follows: bool = msg_send![parent, autoresizesSubviews];
                    (flipped, follows)
                }
                None => (false, false),
            };
            let mask = if flipped { layout::flipped(bits) } else { bits };
            let _: () = msg_send![view, setAutoresizingMask: mask];
            smoke::log(
                "autoresizing",
                &format!("{label} {mask} flipped={flipped} follows={follows}"),
            );
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not set the webview's autoresizing: {err}");
    }
}

/// What WebKit says about the page's element fullscreen: its `fullscreenState` (`None` before
/// macOS 13, which lacks the property) and whether the page's WKWebView still sits in the Little
/// Joan window (during element fullscreen WebKit moves it into a window of its own). Read on the
/// main thread (`with_webview`); `None` when the webview couldn't be asked in time.
pub fn webkit_fullscreen<R: Runtime>(
    page: &Webview<R>,
) -> Option<(Option<WebKitFullscreen>, bool)> {
    let ours = page.window().ns_window().ok()? as usize;
    let (sent, answer) = mpsc::channel();
    page.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        if view.is_null() {
            return;
        }
        // The view's window now (wry reads `[webview window]` for this).
        let current = webview.ns_window() as usize;
        let state = fullscreen_state(view);
        // The smoke log follows what this decision saw, before the caller acts on it.
        note_state(state.unwrap_or(WebKitFullscreen::Not));
        let _ = sent.send((state, current == ours));
    })
    .ok()?;
    // On the main thread with_webview has run already; elsewhere the main thread answers soon.
    answer.recv_timeout(Duration::from_millis(500)).ok()
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
                let window = page.window();
                let _ = page.with_webview(move |webview| {
                    let view = webview.inner().cast::<AnyObject>();
                    if view.is_null() {
                        return;
                    }
                    note_state(fullscreen_state(view).unwrap_or(WebKitFullscreen::Not));
                    // LJ19.2: the app, the menu bar and the window around WebKit's fullscreen.
                    if let Some(state) = spaces::app_state(&window) {
                        note_app(state);
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

/// Smoke mode: logs `wk-fullscreen <state>` when it differs from the last line. Called on the main
/// thread by the 100 ms poll and by every read of the guard (`webkit_fullscreen`), so a layout the
/// guard allowed is logged after the state it saw — never before a lagging poll catches up.
fn note_state(state: WebKitFullscreen) {
    if !smoke::enabled() {
        return;
    }
    let code = state as i8;
    if LOGGED.swap(code, Ordering::SeqCst) != code {
        smoke::log("wk-fullscreen", state.name());
    }
}

/// The last `wk-app` line the smoke log got.
static LOGGED_APP: Mutex<String> = Mutex::new(String::new());

/// Smoke mode: logs `wk-app <state>` (spaces::app_state) when it differs from the last line.
fn note_app(state: String) {
    let mut last = LOGGED_APP.lock().unwrap_or_else(PoisonError::into_inner);
    if *last != state {
        smoke::log("wk-app", &state);
        *last = state;
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

/// macOS's own shortcuts (LJ44, read-only): the user's `com.apple.symbolichotkeys` preferences
/// domain through NSUserDefaults, as JSON for system_shortcuts::parse (dictionaries, arrays,
/// strings and numbers; anything else is left out). `None` when the domain doesn't exist. Never
/// written. Any thread: NSUserDefaults is thread-safe, and an autorelease pool drains what the
/// walk leaves.
pub fn symbolic_hotkeys() -> Option<serde_json::Value> {
    let class = AnyClass::get(c"NSUserDefaults")?;
    let name = NSString::from_str("com.apple.symbolichotkeys");
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: +standardUserDefaults returns the shared defaults; -persistentDomainForName:
        // takes an NSString and returns an NSDictionary or nil.
        let domain: Option<Retained<AnyObject>> = unsafe {
            let defaults: Option<Retained<AnyObject>> = msg_send![class, standardUserDefaults];
            let defaults = defaults?;
            msg_send![&*defaults, persistentDomainForName: &*name]
        };
        let domain = domain?;
        to_json(&domain, 0)
    })
}

/// A property-list object as JSON (at most 8 levels and 1,024 items deep / wide).
fn to_json(object: &AnyObject, depth: usize) -> Option<serde_json::Value> {
    use serde_json::Value;
    if depth > 8 {
        return None;
    }
    let is = |name: &std::ffi::CStr| {
        AnyClass::get(name).is_some_and(|class| {
            // SAFETY: -isKindOfClass: takes a class and returns a BOOL.
            unsafe { msg_send![object, isKindOfClass: class] }
        })
    };
    // SAFETY (each branch): the object was checked to be of that class; NSDictionary's -allKeys
    // returns an NSArray and -objectForKey: an object or nil; NSArray's -count an NSUInteger and
    // -objectAtIndex: an object for an index below it; NSNumber's -longLongValue a long long.
    if is(c"NSDictionary") {
        let keys: Option<Retained<AnyObject>> = unsafe { msg_send![object, allKeys] };
        let keys = keys?;
        let count: usize = unsafe { msg_send![&*keys, count] };
        let mut map = serde_json::Map::new();
        for index in 0..count.min(1024) {
            let key: Option<Retained<AnyObject>> =
                unsafe { msg_send![&*keys, objectAtIndex: index] };
            let Some(key) = key else {
                continue;
            };
            let Some(Value::String(name)) = to_json(&key, depth + 1) else {
                continue;
            };
            let value: Option<Retained<AnyObject>> =
                unsafe { msg_send![object, objectForKey: &*key] };
            if let Some(value) = value.as_deref().and_then(|v| to_json(v, depth + 1)) {
                map.insert(name, value);
            }
        }
        Some(Value::Object(map))
    } else if is(c"NSArray") {
        let count: usize = unsafe { msg_send![object, count] };
        let items = (0..count.min(1024))
            .filter_map(|index| {
                let item: Option<Retained<AnyObject>> =
                    unsafe { msg_send![object, objectAtIndex: index] };
                let item = item?;
                to_json(&item, depth + 1)
            })
            .collect();
        Some(Value::Array(items))
    } else if is(c"NSString") {
        // SAFETY: checked to be an NSString above.
        let text: &NSString = unsafe { &*std::ptr::from_ref(object).cast::<NSString>() };
        Some(Value::String(text.to_string()))
    } else if is(c"NSNumber") {
        let number: i64 = unsafe { msg_send![object, longLongValue] };
        Some(Value::from(number))
    } else {
        None
    }
}
