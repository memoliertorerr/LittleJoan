//! Linux: a sign-in popup's WebKitGTK view (LJ47). WebKitGTK asks for a new window before it
//! knows the size the page wants (wry passes none), so a popup that isn't a known sign-in flow is
//! built hidden and decided at `ready-to-show` (`bar::popup::settle`). What decides is the geometry
//! the request's features left in the new view's window properties, read at once after the engine's
//! `create` (a high-priority idle: WebKitGTK writes the features' width and height there right
//! after `create` returns, and nothing else touches it until the page process — which waits for
//! that reply — sets the window frame); at `ready-to-show` every request has a geometry and the
//! bars say nothing (`popups::gtk_asked`). A popup with no `ready-to-show` after
//! `popups::GTK_SETTLE_WAIT` is settled as unsized. wry answers the page's `window.close()` by
//! destroying the web view and leaving its window, so the view's end closes the popup. Its element
//! fullscreen is always off (the review: a fullscreen element would cover its title, the host; the
//! popup has settings of its own, so the page keeps its fullscreen), and its downloads are refused
//! (linux/downloads.rs). Little Joan's keys reach the popup's own GTK window before WebKitGTK does
//! (LJ48): only Close acts there, closing the popup; every other key is the page's.

use std::cell::Cell;
use std::rc::Rc;

use gtk::glib::{self, Propagation};
use gtk::prelude::*;
use tauri::{AppHandle, Runtime, WebviewWindow};
use webkit2gtk::{SettingsExt, WebViewExt, WindowPropertiesExt};

use crate::smoke;
use crate::state;

/// A popup was built (`pending`: not decided yet).
pub fn install<R: Runtime>(app: &AppHandle<R>, popup: &WebviewWindow<R>, pending: bool) {
    let app = app.clone();
    let label = popup.label().to_owned();
    let result = popup.with_webview(move |webview| {
        let view = webview.inner();
        match WebViewExt::settings(&view) {
            Some(settings) => settings.set_enable_fullscreen(false),
            None => eprintln!("little-joan: the popup has no settings; its fullscreen stays on"),
        }
        super::downloads::popup(&view);
        {
            let (app, label) = (app.clone(), label.clone());
            view.connect_destroy(move |_| crate::bar::popup::view_gone(&app, &label));
        }
        match view.toplevel() {
            Some(toplevel) => {
                let (app, label) = (app.clone(), label.clone());
                toplevel.connect_key_press_event(move |_, event| {
                    let (key, mods) = super::pressed(event);
                    let taken = state::with(&app, |s| s.keymap.action_for(key, mods))
                        .is_some_and(|action| crate::bar::popup::key_pressed(&app, &label, action));
                    if taken {
                        Propagation::Stop
                    } else {
                        Propagation::Proceed
                    }
                });
            }
            None => {
                eprintln!("little-joan: the popup has no toplevel window; its keys are the page's")
            }
        }
        if pending {
            decide_when_ready(&app, &label, &view);
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the popup: {err}");
    }
}

/// The geometry a window-properties object holds (width × height).
fn size_of(properties: &webkit2gtk::WindowProperties) -> (i32, i32) {
    let geometry = properties.geometry();
    (geometry.width(), geometry.height())
}

/// A pending popup: the features' size read right after `create`, settled at `ready-to-show` (or
/// after `GTK_SETTLE_WAIT` without one).
fn decide_when_ready<R: Runtime>(app: &AppHandle<R>, label: &str, view: &webkit2gtk::WebView) {
    // `None` until the idle below has read it.
    let features: Rc<Cell<Option<(i32, i32)>>> = Rc::new(Cell::new(None));
    {
        let (features, view) = (features.clone(), view.clone());
        glib::idle_add_local_full(glib::Priority::HIGH, move || {
            features.set(Some(
                view.window_properties().map_or((0, 0), |p| size_of(&p)),
            ));
            glib::ControlFlow::Break
        });
    }
    {
        let (app, label) = (app.clone(), label.to_owned());
        view.connect_ready_to_show(move |view| {
            let read = features.get();
            // CI: what WebKitGTK said, to keep popups::gtk_asked's account of it true.
            if let Some(properties) = view.window_properties() {
                let (width, height) = size_of(&properties);
                smoke::log(
                    "popup-gtk",
                    &format!(
                        "features={} toolbar={} location={} menubar={} geometry={width}x{height}",
                        read.map_or("unread".to_owned(), |(w, h)| format!("{w}x{h}")),
                        u8::from(properties.is_toolbar_visible()),
                        u8::from(properties.is_locationbar_visible()),
                        u8::from(properties.is_menubar_visible()),
                    ),
                );
            }
            crate::bar::popup::settle(&app, &label, crate::popups::gtk_asked(read));
        });
    }
    let (app, label) = (app.clone(), label.to_owned());
    glib::timeout_add_local_once(crate::popups::GTK_SETTLE_WAIT, move || {
        crate::bar::popup::settle_late(&app, &label);
    });
}
