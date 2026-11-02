//! macOS 26+: Liquid Glass behind the bar (LJ19.9). AppKit's public `NSGlassEffectView`, looked up
//! at run time — one build serves macOS 12.3 → 26: before 26 the class isn't there and nothing here
//! runs. One glass view goes into the window's content view **at the back**, the content view's size
//! (width and height autoresizing), under both webviews: the bar's transparent page shows it — all of
//! it in the command bar, the 40 px strip above the opaque page in the toolbar view. The window is
//! made non-opaque with a clear background meanwhile (public NSWindow calls), so the glass shows
//! what is behind the window rather than the window's own colour; without glass it is opaque again,
//! with the theme's colour. Reduce Transparency and Increase Contrast (NSWorkspace's display
//! options, followed through its notification) turn it off (glass.rs decides). Smoke log:
//! `glass-view …` after each change (what AppKit reports back).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Once, OnceLock};
use std::time::Duration;

use objc2::encode::{Encode, Encoding};
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, NSObject, Sel};
use objc2::{ClassType, msg_send, sel};
use objc2_foundation::NSString;
use tauri::{Runtime, Webview};

use super::spaces::Rect;
use crate::glass::GlassLook;
use crate::settings::glass::{Support, version_has_glass};
use crate::smoke;

/// The glass's corners: a title-bar-only window's on macOS 26 (no public API says it; checked
/// against CI's screenshots, LJ19.9's notes).
const CORNER_RADIUS: f64 = 16.0;
/// NSViewWidthSizable | NSViewHeightSizable: the glass follows the content view's size.
const FILL_MASK: usize = 2 | 16;
/// NSWindowBelow: `addSubview:positioned:relativeTo:nil` puts it under every sibling.
const BELOW: isize = -1;

/// NSOperatingSystemVersion (three NSIntegers).
#[repr(C)]
#[derive(Clone, Copy)]
struct OsVersion {
    major: isize,
    minor: isize,
    patch: isize,
}

// SAFETY: laid out as NSOperatingSystemVersion (an anonymous struct of three NSIntegers), with its
// encoding (objc2-foundation's is the same).
unsafe impl Encode for OsVersion {
    const ENCODING: Encoding =
        Encoding::Struct("?", &[isize::ENCODING, isize::ENCODING, isize::ENCODING]);
}

/// The system's major version and whether AppKit has NSGlassEffectView, read once.
static SYSTEM: OnceLock<(i64, bool)> = OnceLock::new();

fn system() -> (i64, bool) {
    *SYSTEM.get_or_init(|| {
        let major = AnyClass::get(c"NSProcessInfo").map_or(0, |class| {
            // SAFETY: +processInfo returns the shared NSProcessInfo; -operatingSystemVersion returns
            // an NSOperatingSystemVersion by value. Any thread.
            unsafe {
                let info: Option<Retained<AnyObject>> = msg_send![class, processInfo];
                info.map_or(0, |info| {
                    let version: OsVersion = msg_send![&*info, operatingSystemVersion];
                    let _ = (version.minor, version.patch);
                    i64::try_from(version.major).unwrap_or(0)
                })
            }
        });
        (major, AnyClass::get(c"NSGlassEffectView").is_some())
    })
}

/// The system's major version (the smoke log's `glass-support`).
pub fn os_major() -> i64 {
    system().0
}

/// macOS 26 or later with NSGlassEffectView.
pub fn available() -> bool {
    let (major, class) = system();
    class && version_has_glass(major)
}

/// Reduce Transparency and Increase Contrast as last read (the watcher keeps them current).
static REDUCE: AtomicBool = AtomicBool::new(false);
static CONTRAST: AtomicBool = AtomicBool::new(false);

/// Whether glass can show now.
pub fn support() -> Support {
    Support {
        available: available(),
        reduce_transparency: REDUCE.load(Ordering::SeqCst),
        increase_contrast: CONTRAST.load(Ordering::SeqCst),
    }
}

/// NSWorkspace's two display options. Main thread.
fn display_options() -> Option<(bool, bool)> {
    let class = AnyClass::get(c"NSWorkspace")?;
    // SAFETY: +sharedWorkspace returns the shared NSWorkspace; the two properties are BOOLs (macOS
    // 10.10+).
    unsafe {
        let workspace: Option<Retained<AnyObject>> = msg_send![class, sharedWorkspace];
        let workspace = workspace?;
        let reduce: bool = msg_send![&*workspace, accessibilityDisplayShouldReduceTransparency];
        let contrast: bool = msg_send![&*workspace, accessibilityDisplayShouldIncreaseContrast];
        Some((reduce, contrast))
    }
}

/// Smoke mode only: a file standing in for System Settings (`LITTLEJOAN_SMOKE_A11Y`): `reduce`,
/// `contrast`, `both`, or anything else for neither. CI can't switch the runner's own settings.
const SMOKE_A11Y_ENV: &str = "LITTLEJOAN_SMOKE_A11Y";

fn smoke_options() -> Option<(bool, bool)> {
    if !smoke::enabled() {
        return None;
    }
    let path = std::env::var_os(SMOKE_A11Y_ENV)?;
    let text = std::fs::read_to_string(path).ok()?;
    Some(match text.trim() {
        "reduce" => (true, false),
        "contrast" => (false, true),
        "both" => (true, true),
        _ => (false, false),
    })
}

/// What runs when the display options changed (glass.rs re-applies the glass).
static ON_CHANGE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static WATCH: Once = Once::new();

/// Reads the display options now and follows their changes (NSWorkspace's
/// `NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification`), calling `on_change` after one.
/// macOS 26+ only (nothing else needs them). Main thread (setup).
pub fn watch_accessibility(on_change: Box<dyn Fn() + Send + Sync>) {
    if !available() {
        return;
    }
    let _ = ON_CHANGE.set(on_change);
    if smoke::enabled()
        && let Some((reduce, contrast)) = display_options()
    {
        // What the system itself says (the stand-in file, if any, wins below).
        smoke::log(
            "glass-system",
            &format!("reduce={reduce} contrast={contrast}"),
        );
    }
    if let Some((reduce, contrast)) = smoke_options().or_else(display_options) {
        REDUCE.store(reduce, Ordering::SeqCst);
        CONTRAST.store(contrast, Ordering::SeqCst);
    }
    WATCH.call_once(|| {
        observe();
        if smoke::enabled() && std::env::var_os(SMOKE_A11Y_ENV).is_some() {
            poll_smoke_file();
        }
    });
}

/// The display options again; `on_change` when they differ. From the notification (main thread)
/// or the smoke file's poll (its own thread; it reads only the file).
fn options_changed(options: Option<(bool, bool)>) {
    let Some((reduce, contrast)) = options else {
        return;
    };
    let was_reduce = REDUCE.swap(reduce, Ordering::SeqCst);
    let was_contrast = CONTRAST.swap(contrast, Ordering::SeqCst);
    if was_reduce != reduce || was_contrast != contrast {
        smoke::log(
            "glass-accessibility",
            &format!("reduce={reduce} contrast={contrast}"),
        );
        if let Some(on_change) = ON_CHANGE.get() {
            on_change();
        }
    }
}

/// Observes NSWorkspace's notification with an NSObject subclass of one method.
fn observe() {
    let Some(class) = observer_class() else {
        return;
    };
    let Some(workspace_class) = AnyClass::get(c"NSWorkspace") else {
        return;
    };
    // The notification's name is its own value (a test below checks it against AppKit's).
    let name = NSString::from_str(DISPLAY_OPTIONS_CHANGED);
    // SAFETY: +sharedWorkspace returns the shared workspace and -notificationCenter its center;
    // +new an instance of our NSObject subclass, kept for the process's lifetime (centers don't
    // retain observers); -addObserver:selector:name:object: names the class's one method
    // (`ljDisplayOptionsChanged:`, one object argument) and a nil object (any sender).
    unsafe {
        let workspace: Option<Retained<AnyObject>> = msg_send![workspace_class, sharedWorkspace];
        let center: Option<Retained<AnyObject>> =
            workspace.and_then(|workspace| msg_send![&*workspace, notificationCenter]);
        let observer: Option<Retained<AnyObject>> = msg_send![class, new];
        let (Some(center), Some(observer)) = (center, observer) else {
            return;
        };
        let _: () = msg_send![
            &*center,
            addObserver: &*observer,
            selector: sel!(ljDisplayOptionsChanged:),
            name: &*name,
            object: None::<&AnyObject>
        ];
        std::mem::forget(observer);
    }
}

/// NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification's value.
const DISPLAY_OPTIONS_CHANGED: &str = "NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification";

fn observer_class() -> Option<&'static AnyClass> {
    let mut builder = ClassBuilder::new(c"LittleJoanGlassAccessibility", NSObject::class())?;
    // SAFETY: `changed` has the signature of a method taking one object argument and returning
    // nothing.
    unsafe {
        builder.add_method(
            sel!(ljDisplayOptionsChanged:),
            changed as unsafe extern "C-unwind" fn(_, _, _),
        );
    }
    Some(builder.register())
}

/// The notification (main thread): the options as System Settings has them now (the smoke file
/// wins while it exists).
unsafe extern "C-unwind" fn changed(_this: &NSObject, _cmd: Sel, _notification: *mut AnyObject) {
    options_changed(smoke_options().or_else(display_options));
}

/// Smoke mode: reads the stand-in file every 250 ms (CI only; never in normal use).
fn poll_smoke_file() {
    let _ = std::thread::Builder::new()
        .name("little-joan-smoke-a11y".into())
        .spawn(|| {
            loop {
                std::thread::sleep(Duration::from_millis(250));
                if let Some(options) = smoke_options() {
                    options_changed(Some(options));
                }
            }
        });
}

/// Puts the glass behind the bar's window's webviews (`Some`), or takes it out and makes the
/// window opaque again with `solid` behind (`None`: only where glass was — a window that never had
/// it isn't touched). Queued on the main thread.
pub fn set<R: Runtime>(bar: &Webview<R>, look: Option<GlassLook>, solid: [u8; 3]) {
    if !available() {
        return;
    }
    let result = bar.with_webview(move |webview| {
        let view = webview.inner().cast::<AnyObject>();
        let window = webview.ns_window().cast::<AnyObject>();
        if view.is_null() || window.is_null() {
            return;
        }
        // SAFETY: `view` is the bar's live WKWebView and `window` its NSWindow (with_webview runs
        // on the main thread while they exist).
        unsafe { apply(&*view, &*window, look, solid) };
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not change the glass: {err}");
    }
}

/// An NSColor in sRGB.
fn color(rgb: [u8; 3], alpha: f64) -> Option<Retained<AnyObject>> {
    let class = AnyClass::get(c"NSColor")?;
    let [r, g, b] = rgb.map(|c| f64::from(c) / 255.0);
    // SAFETY: +colorWithSRGBRed:green:blue:alpha: takes four CGFloats and returns an NSColor.
    unsafe { msg_send![class, colorWithSRGBRed: r, green: g, blue: b, alpha: alpha] }
}

/// The glass view among `superview`'s subviews, if there is one.
unsafe fn find_glass(superview: &AnyObject, class: &AnyClass) -> Option<Retained<AnyObject>> {
    // SAFETY (caller: main thread, a live view): -subviews returns an NSArray of NSViews; -count an
    // NSUInteger, -objectAtIndex: a view for an index below it; -isKindOfClass: a BOOL.
    unsafe {
        let subviews: Option<Retained<AnyObject>> = msg_send![superview, subviews];
        let subviews = subviews?;
        let count: usize = msg_send![&*subviews, count];
        (0..count).find_map(|index| {
            let view: Option<Retained<AnyObject>> = msg_send![&*subviews, objectAtIndex: index];
            view.filter(|view| {
                let glass: bool = msg_send![&**view, isKindOfClass: class];
                glass
            })
        })
    }
}

/// Main thread, `view` and `window` live (`set`).
unsafe fn apply(view: &AnyObject, window: &AnyObject, look: Option<GlassLook>, solid: [u8; 3]) {
    let (Some(class), Some(view_class)) = (
        AnyClass::get(c"NSGlassEffectView"),
        AnyClass::get(c"NSView"),
    ) else {
        return;
    };
    // SAFETY (each call): AppKit on the main thread with live objects — -superview returns an
    // NSView or nil, -bounds an NSRect; +new a new view (retained); -setFrame: takes an NSRect,
    // -setAutoresizingMask: an NSUInteger, -setContentView: a view, -addSubview:positioned:
    // relativeTo: a view, an NSWindowOrderingMode (NSInteger) and nil; NSGlassEffectView's -setStyle:
    // takes an NSGlassEffectViewStyle (NSInteger), -setTintColor: an NSColor, -setCornerRadius: a
    // CGFloat; -setAppearance: an NSAppearance or nil (+appearanceNamed: takes an NSString);
    // NSWindow's -setOpaque: a BOOL, -setBackgroundColor: an NSColor, -invalidateShadow nothing;
    // -removeFromSuperview nothing.
    unsafe {
        let superview: Option<Retained<AnyObject>> = msg_send![view, superview];
        let Some(superview) = superview else {
            return;
        };
        let existing = find_glass(&superview, class);
        match look {
            Some(look) => {
                let glass = match existing {
                    Some(glass) => glass,
                    None => {
                        let glass: Option<Retained<AnyObject>> = msg_send![class, new];
                        let Some(glass) = glass else {
                            return;
                        };
                        let bounds: Rect = msg_send![&*superview, bounds];
                        let _: () = msg_send![&*glass, setFrame: bounds];
                        let _: () = msg_send![&*glass, setAutoresizingMask: FILL_MASK];
                        // An empty content view: the glass is all there is to it.
                        let content: Option<Retained<AnyObject>> = msg_send![view_class, new];
                        if let Some(content) = content {
                            let _: () = msg_send![&*glass, setContentView: &*content];
                        }
                        let _: () = msg_send![
                            &*superview,
                            addSubview: &*glass,
                            positioned: BELOW,
                            relativeTo: None::<&AnyObject>
                        ];
                        glass
                    }
                };
                let style: isize = if look.native.clear { 1 } else { 0 };
                let _: () = msg_send![&*glass, setStyle: style];
                if let Some(tint) = color(look.tint, look.native.tint_alpha) {
                    let _: () = msg_send![&*glass, setTintColor: &*tint];
                }
                let _: () = msg_send![&*glass, setCornerRadius: CORNER_RADIUS];
                if let Some(appearances) = AnyClass::get(c"NSAppearance") {
                    let name = NSString::from_str(if look.dark { DARK_AQUA } else { AQUA });
                    let appearance: Option<Retained<AnyObject>> =
                        msg_send![appearances, appearanceNamed: &*name];
                    let _: () = msg_send![&*glass, setAppearance: appearance.as_deref()];
                }
                let _: () = msg_send![window, setOpaque: false];
                if let Some(clear) = color([0, 0, 0], 0.0) {
                    let _: () = msg_send![window, setBackgroundColor: &*clear];
                }
                let _: () = msg_send![window, invalidateShadow];
            }
            None => {
                let Some(glass) = existing else {
                    // Never had glass: left exactly as it is.
                    return;
                };
                let _: () = msg_send![&*glass, removeFromSuperview];
                let _: () = msg_send![window, setOpaque: true];
                if let Some(solid) = color(solid, 1.0) {
                    let _: () = msg_send![window, setBackgroundColor: &*solid];
                }
                let _: () = msg_send![window, invalidateShadow];
            }
        }
        if smoke::enabled() {
            smoke::log("glass-view", &readback(&superview, window, class));
        }
    }
}

/// NSAppearanceNameAqua / NSAppearanceNameDarkAqua's values (a test checks them against AppKit's).
const AQUA: &str = "NSAppearanceNameAqua";
const DARK_AQUA: &str = "NSAppearanceNameDarkAqua";

/// Smoke mode: what AppKit says now — `<clear|regular|none> back=<the content view's first
/// subview> fills=<its frame is the content view's bounds> corner=<r> tint=<alpha> dark=<bool>
/// opaque=<the window's isOpaque> window-alpha=<its background's alpha> views=<subviews>`.
unsafe fn readback(superview: &AnyObject, window: &AnyObject, class: &AnyClass) -> String {
    // SAFETY (main thread, live objects): -subviews returns an NSArray, -firstObject a view or nil;
    // NSGlassEffectView's -style an NSInteger, -cornerRadius a CGFloat, -tintColor an NSColor or
    // nil, -effectiveAppearance an NSAppearance whose -name is an NSString; NSColor's
    // -alphaComponent a CGFloat; -frame / -bounds NSRects; NSWindow's -isOpaque a BOOL and
    // -backgroundColor an NSColor.
    unsafe {
        let opaque: bool = msg_send![window, isOpaque];
        let background: Option<Retained<AnyObject>> = msg_send![window, backgroundColor];
        let window_alpha: f64 = background.map_or(1.0, |c| msg_send![&*c, alphaComponent]);
        let subviews: Option<Retained<AnyObject>> = msg_send![superview, subviews];
        let views: usize = subviews.as_deref().map_or(0, |s| msg_send![s, count]);
        let window_line = format!("opaque={opaque} window-alpha={window_alpha:.2} views={views}");
        let Some(glass) = find_glass(superview, class) else {
            return format!("none {window_line}");
        };
        let first: Option<Retained<AnyObject>> =
            subviews.as_deref().and_then(|s| msg_send![s, firstObject]);
        let back = first.is_some_and(|first| std::ptr::eq(&*first, &*glass));
        let frame: Rect = msg_send![&*glass, frame];
        let bounds: Rect = msg_send![superview, bounds];
        let fills = (frame.size.width - bounds.size.width).abs() < 0.5
            && (frame.size.height - bounds.size.height).abs() < 0.5
            && frame.origin.x.abs() < 0.5
            && frame.origin.y.abs() < 0.5;
        let style: isize = msg_send![&*glass, style];
        let corner: f64 = msg_send![&*glass, cornerRadius];
        let tint: Option<Retained<AnyObject>> = msg_send![&*glass, tintColor];
        let tint_alpha: f64 = tint.map_or(0.0, |c| msg_send![&*c, alphaComponent]);
        let appearance: Option<Retained<AnyObject>> = msg_send![&*glass, effectiveAppearance];
        let appearance: Option<Retained<NSString>> = appearance.and_then(|a| msg_send![&*a, name]);
        let dark = appearance.is_some_and(|name| name.to_string().contains("Dark"));
        format!(
            "{} back={back} fills={fills} corner={corner} tint={tint_alpha:.2} dark={dark} {window_line}",
            if style == 1 { "clear" } else { "regular" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AppKit's own constants (linked here only), compared with the values this file uses.
    #[link(name = "AppKit", kind = "framework")]
    unsafe extern "C" {
        static NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification: &'static NSString;
        static NSAppearanceNameAqua: &'static NSString;
        static NSAppearanceNameDarkAqua: &'static NSString;
    }

    #[test]
    fn the_names_are_appkits() {
        // SAFETY: AppKit's exported NSString constants, alive for the process's lifetime.
        let (note, aqua, dark) = unsafe {
            (
                NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification.to_string(),
                NSAppearanceNameAqua.to_string(),
                NSAppearanceNameDarkAqua.to_string(),
            )
        };
        assert_eq!(note, DISPLAY_OPTIONS_CHANGED);
        assert_eq!(aqua, AQUA);
        assert_eq!(dark, DARK_AQUA);
    }

    #[test]
    fn glass_is_there_exactly_on_macos_26_and_later() {
        // This Mac (and CI's macOS runner) says its own version; the class must agree with it.
        let (major, class) = system();
        assert!(major >= 12, "macOS {major}");
        assert_eq!(available(), class && version_has_glass(major));
        if major >= 26 {
            assert!(class, "macOS {major} has NSGlassEffectView");
            assert!(available());
        }
        if (12..=15).contains(&major) {
            assert!(!available(), "macOS {major} has no Liquid Glass");
        }
    }
}
