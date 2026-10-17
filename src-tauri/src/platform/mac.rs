//! macOS: WKWebView's `customUserAgent` (nil = the engine's own user agent).

use objc2::msg_send;
use objc2::runtime::AnyObject;
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
