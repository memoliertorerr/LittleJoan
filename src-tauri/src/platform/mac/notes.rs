//! Smoke mode only (CI, macOS; LJ19.4-fix4): AppKit's own notifications about windows and the app —
//! which window became or stopped being key, native full-screen transitions (WebKit shows a page's
//! element fullscreen in a window of its own that goes full screen), the app's activation —, with
//! the call stack for the ones that say who caused them, each window's collection behaviour, level
//! and Space —, and NSWorkspace's about the active Space and the frontmost app. CI saw WebKit leave
//! a page's fullscreen by itself while the Little Joan window took the key back; these lines say
//! what did it. Nothing is installed without the smoke log, and nothing here changes what the app
//! does.

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

/// What is observed on NSWorkspace's own notification center.
const WORKSPACE_NAMES: [&str; 2] = [
    "NSWorkspaceActiveSpaceDidChangeNotification",
    "NSWorkspaceDidActivateApplicationNotification",
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
        let (Some(center_class), Some(workspace_class)) = (
            AnyClass::get(c"NSNotificationCenter"),
            AnyClass::get(c"NSWorkspace"),
        ) else {
            return;
        };
        // SAFETY: +defaultCenter returns the shared center, +sharedWorkspace the shared workspace
        // and its -notificationCenter that one's own; +new an instance of our NSObject subclass,
        // kept for the process's lifetime (the centers don't retain observers); each
        // -addObserver:selector:name:object: names a method the class has (`ljNote:`, one object
        // argument) and a nil object (any sender).
        unsafe {
            let center: Option<Retained<AnyObject>> = msg_send![center_class, defaultCenter];
            let workspace: Option<Retained<AnyObject>> =
                msg_send![workspace_class, sharedWorkspace];
            let workspace_center: Option<Retained<AnyObject>> =
                workspace.and_then(|workspace| msg_send![&*workspace, notificationCenter]);
            let observer: Option<Retained<AnyObject>> = msg_send![observer_class, new];
            let (Some(center), Some(observer)) = (center, observer) else {
                return;
            };
            let observe = |center: &AnyObject, names: &[&str]| {
                for name in names {
                    let name = NSString::from_str(name);
                    let _: () = msg_send![
                        center,
                        addObserver: &*observer,
                        selector: sel!(ljNote:),
                        name: &*name,
                        object: None::<&AnyObject>
                    ];
                }
            };
            observe(&center, &NAMES);
            if let Some(workspace_center) = workspace_center {
                observe(&workspace_center, &WORKSPACE_NAMES);
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

/// One notification: `wk-note <name> <class of its object> [<window's state>] [app=<pid>]
/// [stack=…]` (`window_state`; `app` for NSWorkspace's activations: the pid, `own` for Little
/// Joan).
unsafe extern "C-unwind" fn note(_this: &NSObject, _cmd: Sel, notification: *mut AnyObject) {
    if notification.is_null() {
        return;
    }
    objc2::rc::autoreleasepool(|_| {
        // SAFETY: AppKit hands the observer a live NSNotification: -name returns an NSString,
        // -object the sender (a window, the app) or nil; -isKeyWindow is asked of NSWindows only.
        let (name, object, state, app) = unsafe {
            let notification = &*notification;
            let name: Option<Retained<NSString>> = msg_send![notification, name];
            let object: Option<Retained<AnyObject>> = msg_send![notification, object];
            let class = object
                .as_deref()
                .map(|object| object.class().name().to_string_lossy().into_owned());
            let state = object.as_deref().and_then(|object| window_state(object));
            let info: Option<Retained<AnyObject>> = msg_send![notification, userInfo];
            let app = info.and_then(|info| {
                let key = NSString::from_str("NSWorkspaceApplicationKey");
                let app: Option<Retained<AnyObject>> = msg_send![&*info, objectForKey: &*key];
                app.map(|app| {
                    let pid: i32 = msg_send![&*app, processIdentifier];
                    pid
                })
            });
            (
                name.map(|name| name.to_string()).unwrap_or_default(),
                class.unwrap_or_else(|| "nil".to_owned()),
                state,
                app,
            )
        };
        let short = name
            .trim_start_matches("NS")
            .trim_end_matches("Notification")
            .to_owned();
        let mut line = format!("{short} {object}");
        if let Some(state) = state {
            line.push(' ');
            line.push_str(&state);
        }
        if let Some(pid) = app {
            let own = i32::try_from(std::process::id()).ok() == Some(pid);
            line.push_str(&if own {
                " app=own".to_owned()
            } else {
                format!(" app={pid}")
            });
        }
        if WITH_STACK.contains(&name.as_str()) {
            line.push_str(" stack=");
            line.push_str(&stack());
        }
        smoke::log("wk-note", &line);
    });
}

/// An NSWindow's state as `key=<bool> visible=<bool> space=<bool> behavior=<n> level=<n>` (its
/// collection behaviour bits, its window level), with the app's `active=<bool>`; `None` for an
/// object that isn't a window. Main thread.
///
/// # Safety
///
/// `object` is a live Objective-C object, asked on the main thread.
unsafe fn window_state(object: &AnyObject) -> Option<String> {
    let window = AnyClass::get(c"NSWindow")?;
    let application = AnyClass::get(c"NSApplication")?;
    // SAFETY: -isKindOfClass: is asked of any object; the rest of NSWindows only: -isKeyWindow,
    // -isVisible and -isOnActiveSpace return a BOOL, -collectionBehavior an NSUInteger, -level an
    // NSInteger; +sharedApplication the shared app, whose -isActive is a BOOL.
    unsafe {
        let is_window: bool = msg_send![object, isKindOfClass: window];
        if !is_window {
            return None;
        }
        let key: bool = msg_send![object, isKeyWindow];
        let visible: bool = msg_send![object, isVisible];
        let space: bool = msg_send![object, isOnActiveSpace];
        let behavior: usize = msg_send![object, collectionBehavior];
        let level: isize = msg_send![object, level];
        let app: Option<Retained<AnyObject>> = msg_send![application, sharedApplication];
        let active = app.is_some_and(|app| {
            let active: bool = msg_send![&*app, isActive];
            active
        });
        Some(format!(
            "key={key} visible={visible} space={space} behavior={behavior} level={level} active={active}"
        ))
    }
}

/// Smoke mode, main thread: `wk-context <what> <window's state>` for a tauri window
/// (`window_state`), e.g. just before the page's webview is added to it.
pub fn context<R: tauri::Runtime>(window: &tauri::Window<R>, what: &str) {
    if !smoke::enabled() {
        return;
    }
    let Ok(ns_window) = window.ns_window() else {
        return;
    };
    let ns_window = ns_window.cast::<AnyObject>();
    if ns_window.is_null() {
        return;
    }
    // SAFETY: `ns_window` is this tauri window's live NSWindow, asked on the main thread (the
    // caller's).
    let state = unsafe { window_state(&*ns_window) };
    smoke::log(
        "wk-context",
        &format!("{what} {}", state.unwrap_or_else(|| "none".to_owned())),
    );
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
