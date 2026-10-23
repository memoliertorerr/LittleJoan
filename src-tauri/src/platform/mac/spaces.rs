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
            let _: () = msg_send![ns_window, setCollectionBehavior: spaces::summoned(was)];
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
