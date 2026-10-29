//! Smoke mode only (CI, macOS; LJ19.4-fix4): AppKit's own notifications about windows and the app —
//! which window became or stopped being key, native full-screen transitions (WebKit shows a page's
//! element fullscreen in a window of its own that goes full screen), the app's activation —, with
//! the call stack for the ones that say who caused them. CI saw WebKit leave a page's fullscreen by
//! itself while the Little Joan window took the key back; these lines say what did it. Nothing is
//! installed without the smoke log, and nothing here changes what the app does.

use std::sync::Once;

use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, NSObject, Sel};
use objc2::{ClassType, msg_send, sel};
use objc2_foundation::NSString;

use crate::smoke;

static INSTALL: Once = Once::new();

/// What is observed (AppKit's notification names are their own values).
const NAMES: [&str; 9] = [
    "NSWindowDidBecomeKeyNotification",
    "NSWindowDidResignKeyNotification",
    "NSWindowWillEnterFullScreenNotification",
    "NSWindowDidEnterFullScreenNotification",
    "NSWindowWillExitFullScreenNotification",
    "NSWindowDidExitFullScreenNotification",
    "NSApplicationDidBecomeActiveNotification",
    "NSApplicationDidResignActiveNotification",
    "NSApplicationDidHideNotification",
];

/// The ones logged with the call stack that posted them.
const WITH_STACK: [&str; 4] = [
    "NSWindowDidBecomeKeyNotification",
    "NSWindowWillExitFullScreenNotification",
    "NSWindowDidExitFullScreenNotification",
    "NSApplicationDidResignActiveNotification",
];

/// How many frames of a stack are logged (after the observer's own).
const FRAMES: usize = 24;

/// Installs the observer once (smoke mode only). Main thread (setup).
pub fn install() {
    if !smoke::enabled() {
        return;
    }
    INSTALL.call_once(|| {
        let Some(observer_class) = observer_class() else {
            return;
        };
        let Some(center_class) = AnyClass::get(c"NSNotificationCenter") else {
            return;
        };
        // SAFETY: +defaultCenter returns the shared center; +new an instance of our NSObject
        // subclass, kept for the process's lifetime (the center doesn't retain observers); each
        // -addObserver:selector:name:object: names a method the class has (`ljNote:`, one object
        // argument) and a nil object (any sender).
        unsafe {
            let center: Option<Retained<AnyObject>> = msg_send![center_class, defaultCenter];
            let observer: Option<Retained<AnyObject>> = msg_send![observer_class, new];
            let (Some(center), Some(observer)) = (center, observer) else {
                return;
            };
            for name in NAMES {
                let name = NSString::from_str(name);
                let _: () = msg_send![
                    &*center,
                    addObserver: &*observer,
                    selector: sel!(ljNote:),
                    name: &*name,
                    object: None::<&AnyObject>
                ];
            }
            std::mem::forget(observer);
        }
        smoke::log("wk-notes", "installed");
    });
}

/// The observer's class: an NSObject subclass with one method, `ljNote:`.
fn observer_class() -> Option<&'static AnyClass> {
    let mut builder = ClassBuilder::new(c"LittleJoanSmokeNotes", NSObject::class())?;
    // SAFETY: `note` has the signature of a method taking one object argument and returning
    // nothing.
    unsafe {
        builder.add_method(sel!(ljNote:), note as unsafe extern "C-unwind" fn(_, _, _));
    }
    Some(builder.register())
}

/// One notification: `wk-note <name> <class of its object> [key=<bool>] [stack=…]`.
unsafe extern "C-unwind" fn note(_this: &NSObject, _cmd: Sel, notification: *mut AnyObject) {
    if notification.is_null() {
        return;
    }
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: AppKit hands the observer a live NSNotification: -name returns an NSString,
        // -object the sender (a window, the app) or nil; -isKeyWindow is asked of NSWindows only.
        let (name, object, key) = unsafe {
            let notification = &*notification;
            let name: Option<Retained<NSString>> = msg_send![notification, name];
            let object: Option<Retained<AnyObject>> = msg_send![notification, object];
            let class = object
                .as_deref()
                .map(|object| object.class().name().to_string_lossy().into_owned());
            let window = AnyClass::get(c"NSWindow");
            let key = match (object.as_deref(), window) {
                (Some(object), Some(window)) => {
                    let is_window: bool = msg_send![object, isKindOfClass: window];
                    is_window.then(|| {
                        let key: bool = msg_send![object, isKeyWindow];
                        key
                    })
                }
                _ => None,
            };
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                class.unwrap_or_else(|| "nil".to_owned()),
                key,
            )
        };
        let short = name
            .trim_start_matches("NS")
            .trim_end_matches("Notification")
            .to_owned();
        let mut line = format!("{short} {object}");
        if let Some(key) = key {
            line.push_str(&format!(" key={key}"));
        }
        if WITH_STACK.contains(&name.as_str()) {
            line.push_str(" stack=");
            line.push_str(&stack());
        }
        smoke::log("wk-note", &line);
    });
}

/// The calling thread's stack (`+[NSThread callStackSymbols]`), each frame as `module:symbol`,
/// after the observer's own frames.
fn stack() -> String {
    let Some(thread) = AnyClass::get(c"NSThread") else {
        return String::new();
    };
    // SAFETY: +callStackSymbols returns an NSArray of NSStrings; -count and -objectAtIndex: are
    // asked within its bounds.
    let frames: Vec<String> = unsafe {
        let symbols: Option<Retained<AnyObject>> = msg_send![thread, callStackSymbols];
        let Some(symbols) = symbols else {
            return String::new();
        };
        let count: usize = msg_send![&*symbols, count];
        (0..count)
            .filter_map(|index| {
                let line: Option<Retained<NSString>> = msg_send![&*symbols, objectAtIndex: index];
                line.map(|line| line.to_string())
            })
            .collect()
    };
    frames
        .iter()
        .filter_map(|line| frame(line))
        .skip_while(|frame| frame.contains("ljNote") || frame.contains("notes::"))
        .filter(|frame| !frame.starts_with("Foundation:") || frame.contains("Notification"))
        .take(FRAMES)
        .collect::<Vec<_>>()
        .join(" < ")
}

/// `3   AppKit   0x000000018b1c0a4c -[NSWindow makeKeyWindow] + 88` → `AppKit:-[NSWindow makeKeyWindow]`.
fn frame(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    let _index = parts.next()?;
    let module = parts.next()?;
    let _address = parts.next()?;
    let rest: Vec<&str> = parts.collect();
    let symbol = rest
        .iter()
        .position(|part| *part == "+")
        .map_or(rest.join(" "), |plus| rest[..plus].join(" "));
    Some(format!("{module}:{symbol}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_keeps_its_module_and_symbol() {
        assert_eq!(
            frame("3   AppKit                              0x000000018b1c0a4c -[NSWindow makeKeyWindow] + 88")
                .as_deref(),
            Some("AppKit:-[NSWindow makeKeyWindow]")
        );
        assert_eq!(
            frame("12  little-joan   0x0000000100abc123 0x100000000 + 11255075").as_deref(),
            Some("little-joan:0x100000000")
        );
        assert_eq!(frame("garbage"), None);
    }
}
