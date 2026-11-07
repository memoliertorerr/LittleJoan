//! Linux: a sign-in popup's WebKitGTK view (LJ47). WebKitGTK asks for a new window before it
//! knows the size the page wants (wry passes none), so a popup that isn't a known sign-in flow is
//! built hidden and decided at `ready-to-show`, once the engine has set its window properties
//! (`bar::popup::settle`). wry answers the page's `window.close()` by destroying the web view and
//! leaving its window, so the view's end closes the popup. Its element fullscreen is off where
//! WebKitGTK aborts on it (LJ14.1; the popup has settings of its own), and its downloads are
//! refused (linux/downloads.rs).

use gtk::prelude::*;
use tauri::{AppHandle, Runtime, WebviewWindow};
use webkit2gtk::{WebViewExt, WindowPropertiesExt};

/// A popup was built (`pending`: not decided yet).
pub fn install<R: Runtime>(app: &AppHandle<R>, popup: &WebviewWindow<R>, pending: bool) {
    let app = app.clone();
    let label = popup.label().to_owned();
    let result = popup.with_webview(move |webview| {
        let view = webview.inner();
        super::fullscreen_off_where_it_aborts(&view);
        super::downloads::popup(&view);
        {
            let (app, label) = (app.clone(), label.clone());
            view.connect_destroy(move |_| crate::bar::popup::view_gone(&app, &label));
        }
        if pending {
            view.connect_ready_to_show(move |view| {
                let asked = view
                    .window_properties()
                    .map(|properties| properties.geometry())
                    .filter(|geometry| geometry.width() > 0 && geometry.height() > 0)
                    .map(|geometry| (f64::from(geometry.width()), f64::from(geometry.height())));
                crate::bar::popup::settle(&app, &label, asked);
            });
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the popup: {err}");
    }
}
