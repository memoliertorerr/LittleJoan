//! macOS: Spaces and other apps' full-screen windows (LJ19.2). Before a Little Joan window is
//! first shown, its NSWindow gets `spaces::summoned`'s collection behaviour — `MoveToActiveSpace`
//! and `FullScreenAuxiliary`, at the normal level —, so a summon opens it on the Space the user is
//! on, a full-screen app's Space included (an accessory app's window with that behaviour may show
//! there), instead of switching Spaces. The summon then orders it front and activates the app as
//! before (tao's `set_focus`). Smoke log: `window-spaces <label> behavior= was= level=`, and for
//! CI's check `window-front … space=` (`front`).

use std::sync::mpsc;
use std::time::Duration;

use objc2::encode::{Encode, Encoding, RefEncode};
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use tauri::{Runtime, Window};

use crate::smoke;
use crate::spaces::{self, Front};

/// Sets the window's collection behaviour (`spaces::summoned`) on the main thread. Queued before
/// the window's `show()`, which tauri delivers to the main thread after it, so the window is
/// first ordered front with it; the caller doesn't wait.
pub fn join_active_space<R: Runtime>(window: &Window<R>) {
    let target = window.clone();
    let queued = window.run_on_main_thread(move || {
        let Ok(ns_window) = target.ns_window() else {
            return;
        };
        let ns_window = ns_window.cast::<AnyObject>();
        if ns_window.is_null() {
            return;
        }
        // SAFETY: `ns_window` is the live NSWindow of this tauri window (asked for on the main
        // thread, where nothing can close it meanwhile); -collectionBehavior returns an
        // NSWindowCollectionBehavior (NSUInteger), -setCollectionBehavior: takes one, and -level
        // returns an NSWindowLevel (NSInteger). `spaces::summoned` keeps AppKit's exclusive
        // groups to one member each, so the setter doesn't raise.
        let (was, behavior, level) = unsafe {
            let ns_window = &*ns_window;
            let was: usize = msg_send![ns_window, collectionBehavior];
            let wanted = smoke_override().unwrap_or_else(|| spaces::summoned(was));
            let _: () = msg_send![ns_window, setCollectionBehavior: wanted];
            let behavior: usize = msg_send![ns_window, collectionBehavior];
            let level: isize = msg_send![ns_window, level];
            (was, behavior, level)
        };
        smoke::log(
            "window-spaces",
            &spaces::spaces_line(target.label(), was, behavior, level),
        );
    });
    if let Err(err) = queued {
        eprintln!("little-joan: could not set where the window shows: {err}");
    }
}

/// A summon while WebKit shows the page in element fullscreen (LJ19.4-fix4): the app comes to the
/// front with the keyboard (`activateIgnoringOtherApps:`, as tao's `set_focus` does) and no window
/// is ordered. The page is on screen already, in WebKit's own full-screen window and Space; ordering
/// the page window front would move it into that Space (`summoned`'s `MoveToActiveSpace`), and
/// when the Space then closes with the window inside it — the page leaving fullscreen — macOS's
/// Dock can refuse every app's full-screen Space for minutes. Queued on the main thread.
pub fn activate_app<R: Runtime>(window: &Window<R>) {
    let queued = window.run_on_main_thread(|| {
        let Some(application) = AnyClass::get(c"NSApplication") else {
            return;
        };
        // SAFETY: main thread; +sharedApplication returns the app (or nil), and
        // -activateIgnoringOtherApps: takes a BOOL.
        unsafe {
            let app: Option<Retained<AnyObject>> = msg_send![application, sharedApplication];
            if let Some(app) = app {
                let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
            }
        }
    });
    if let Err(err) = queued {
        eprintln!("little-joan: could not bring Little Joan to the front: {err}");
    }
}

/// CI experiments only (smoke mode): `LITTLEJOAN_SMOKE_SPACES_BITS=<n>` sets these collection
/// behaviour bits instead of `spaces::summoned`'s, to compare AppKit's default and the parts of
/// the setting on one runner (ci.yml's `extra_env`). Ignored without the smoke log.
fn smoke_override() -> Option<usize> {
    if !smoke::enabled() {
        return None;
    }
    std::env::var("LITTLEJOAN_SMOKE_SPACES_BITS")
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Smoke mode, on the main thread, while WebKit's element fullscreen is watched (mac.rs): the app's
/// state as `active=<bool> menubar=<bool> key=<bool> space=<bool> front=<own|other>` — whether the
/// app is active, AppKit shows the menu bar, the Little Joan window is key and on the active Space,
/// and which app is frontmost. `None` when the window is gone.
pub fn app_state<R: Runtime>(window: &Window<R>) -> Option<String> {
    let ns_window = window.ns_window().ok()?.cast::<AnyObject>();
    let application = AnyClass::get(c"NSApplication")?;
    let menu = AnyClass::get(c"NSMenu")?;
    let workspace = AnyClass::get(c"NSWorkspace")?;
    if ns_window.is_null() {
        return None;
    }
    // SAFETY: main thread (the caller's with_webview closure); `ns_window` is this tauri window's
    // live NSWindow; +sharedApplication / +sharedWorkspace return the shared objects (or nil),
    // -isActive / +menuBarVisible / -isKeyWindow / -isOnActiveSpace a BOOL,
    // -frontmostApplication an NSRunningApplication or nil, -processIdentifier a pid_t.
    unsafe {
        let ns_window = &*ns_window;
        let app: Option<Retained<AnyObject>> = msg_send![application, sharedApplication];
        let active = app.is_some_and(|app| {
            let active: bool = msg_send![&*app, isActive];
            active
        });
        let menubar: bool = msg_send![menu, menuBarVisible];
        let key: bool = msg_send![ns_window, isKeyWindow];
        let space: bool = msg_send![ns_window, isOnActiveSpace];
        let shared: Option<Retained<AnyObject>> = msg_send![workspace, sharedWorkspace];
        let front = shared.and_then(|shared| {
            let front: Option<Retained<AnyObject>> = msg_send![&*shared, frontmostApplication];
            front.map(|front| {
                let pid: i32 = msg_send![&*front, processIdentifier];
                pid
            })
        });
        let own = i32::try_from(std::process::id()).ok();
        Some(format!(
            "active={active} menubar={menubar} key={key} space={space} front={}",
            if front.is_some() && front == own {
                "own"
            } else {
                "other"
            }
        ))
    }
}

/// A screen point (CGPoint, which NSPoint is on 64-bit macOS).
#[repr(C)]
#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

// SAFETY: laid out as CGPoint (two CGFloats, f64 on 64-bit macOS), with its encoding.
unsafe impl Encode for Point {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

// SAFETY: a pointer to a CGPoint.
unsafe impl RefEncode for Point {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// A size (CGSize).
#[repr(C)]
#[derive(Clone, Copy)]
struct Size {
    width: f64,
    height: f64,
}

// SAFETY: laid out as CGSize, with its encoding.
unsafe impl Encode for Size {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

// SAFETY: a pointer to a CGSize.
unsafe impl RefEncode for Size {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// A rectangle (CGRect, which NSRect is on 64-bit macOS).
#[repr(C)]
#[derive(Clone, Copy)]
struct Rect {
    origin: Point,
    size: Size,
}

// SAFETY: laid out as CGRect, with its encoding.
unsafe impl Encode for Rect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
}

// SAFETY: a pointer to a CGRect.
unsafe impl RefEncode for Rect {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Self::ENCODING);
}

/// For the smoke log (CI's over-full-screen check): whether the window is the key window of the
/// active app, the topmost window at its centre (`+[NSWindow windowNumberAtPoint:…]`, every app's
/// windows on the active Space), and on the active Space. Asks the main thread; `None` when the
/// window is gone or the main thread didn't answer within 500 ms.
pub fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    let target = window.clone();
    let (sent, answer) = mpsc::channel();
    window
        .run_on_main_thread(move || {
            let Ok(ns_window) = target.ns_window() else {
                return;
            };
            let ns_window = ns_window.cast::<AnyObject>();
            let (Some(windows), Some(application)) =
                (AnyClass::get(c"NSWindow"), AnyClass::get(c"NSApplication"))
            else {
                return;
            };
            if ns_window.is_null() {
                return;
            }
            // SAFETY: `ns_window` is the live NSWindow of this tauri window (main thread); -frame
            // returns an NSRect, -windowNumber and +windowNumberAtPoint:belowWindowWithWindowNumber:
            // (an NSPoint and an NSInteger) an NSInteger, -isKeyWindow / -isOnActiveSpace and
            // NSApplication's -isActive a BOOL; +sharedApplication returns the app (or nil).
            let front = unsafe {
                let ns_window = &*ns_window;
                let frame: Rect = msg_send![ns_window, frame];
                let centre = Point {
                    x: frame.origin.x + frame.size.width / 2.0,
                    y: frame.origin.y + frame.size.height / 2.0,
                };
                let number: isize = msg_send![ns_window, windowNumber];
                let at: isize =
                    msg_send![windows, windowNumberAtPoint: centre, belowWindowWithWindowNumber: 0isize];
                let key: bool = msg_send![ns_window, isKeyWindow];
                let space: bool = msg_send![ns_window, isOnActiveSpace];
                let shared: Option<Retained<AnyObject>> = msg_send![application, sharedApplication];
                let active = shared.is_some_and(|app| {
                    let active: bool = msg_send![&*app, isActive];
                    active
                });
                Front {
                    focused: key && active,
                    top: at == number,
                    space: Some(space),
                }
            };
            let _ = sent.send(front);
        })
        .ok()?;
    answer.recv_timeout(Duration::from_millis(500)).ok()
}
