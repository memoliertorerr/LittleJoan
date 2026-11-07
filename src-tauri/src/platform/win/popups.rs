//! Windows: a sign-in popup's WebView2 (LJ47). wry answers the popup page's `window.close()`
//! (`WindowCloseRequested`) by destroying the webview's own container window only, which would
//! leave an empty popup on screen, so the whole popup closes here too. Little Joan's keys reach the
//! popup's controller (`AcceleratorKeyPressed`, LJ48): only Close acts there, closing the popup;
//! every other key is the popup page's own.

use tauri::{AppHandle, Runtime, WebviewWindow};
use webview2_com::{AcceleratorKeyPressedEventHandler, WindowCloseRequestedEventHandler};

use crate::state;

/// A popup was built: its `window.close()` closes it, and so does Close's key.
pub fn install<R: Runtime>(app: &AppHandle<R>, popup: &WebviewWindow<R>) {
    let app = app.clone();
    let label = popup.label().to_owned();
    let result = popup.with_webview(move |platform_webview| {
        let controller = platform_webview.controller();
        let keys = {
            let (app, label) = (app.clone(), label.clone());
            AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else {
                    return Ok(());
                };
                let Some((key, mods)) = super::pressed(&args)? else {
                    return Ok(());
                };
                if let Some(action) = state::with(&app, |s| s.keymap.action_for(key, mods))
                    && crate::bar::popup::key_pressed(&app, &label, action)
                {
                    // SAFETY: marks the key as consumed so the popup page never sees it.
                    unsafe { args.SetHandled(true)? };
                }
                Ok(())
            }))
        };
        let mut token = 0i64;
        // SAFETY: registers a handler on the live controller of this webview.
        if let Err(err) = unsafe { controller.add_AcceleratorKeyPressed(&keys, &mut token) } {
            eprintln!("little-joan: could not install the popup's key handler: {err}");
        }
        // SAFETY: reads the core webview of a live controller.
        let core = match unsafe { controller.CoreWebView2() } {
            Ok(core) => core,
            Err(err) => {
                eprintln!("little-joan: no core webview for the popup: {err}");
                return;
            }
        };
        let handler = WindowCloseRequestedEventHandler::create(Box::new(move |_, _| {
            crate::bar::popup::closed_by_page(&app, &label);
            Ok(())
        }));
        let mut token = 0i64;
        // SAFETY: registers a handler on the live core webview.
        if let Err(err) = unsafe { core.add_WindowCloseRequested(&handler, &mut token) } {
            eprintln!("little-joan: could not follow the popup's window.close(): {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the popup: {err}");
    }
}
