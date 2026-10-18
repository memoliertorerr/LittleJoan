//! macOS: WKWebView's `customUserAgent` (nil = the engine's own user agent), and the general
//! pasteboard (extensions' clipboard.writeText).

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::NSString;
use tauri::{Runtime, Webview};

use crate::keys::Platform;
use crate::phone;

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
