//! Linux. Tauri packs a window's webviews into a vertical GtkBox (so bounds don't apply; GTK
//! lays them out), which this module uses for the URL bar: the bar is packed at a fixed height
//! and the page takes the rest. Page keys go through the GTK window's `key-press-event`, which
//! runs before WebKitGTK receives the key (handlers connected with `connect_*` run before the
//! class handler). Element fullscreen is followed through WebKitGTK's `enter-fullscreen` and
//! `leave-fullscreen` signals, to hide the bar meanwhile.

use gtk::glib::Propagation;
use gtk::glib::prelude::*;
use gtk::prelude::*;
use tauri::{Manager, Runtime, Webview, Window};

use crate::bar;
use crate::keys::{self, Key, Mods, Platform};
use crate::layout::BAR_HEIGHT;
use crate::page_actions;
use crate::phone;

/// The URL bar: a fixed height at the top of the window's box instead of an equal share.
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    // with_webview runs on the GTK main thread, where GTK objects may be touched.
    let result = bar.with_webview(|webview| {
        let view = webview.inner();
        // BAR_HEIGHT is a small constant (40), so the cast is exact.
        view.set_size_request(-1, BAR_HEIGHT as i32);
        match view
            .parent()
            .and_then(|parent| parent.downcast::<gtk::Box>().ok())
        {
            Some(container) => {
                container.set_child_packing(&view, false, false, 0, gtk::PackType::Start);
            }
            None => eprintln!("little-joan: the URL bar is not in a GtkBox; layout is GTK's"),
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the URL bar: {err}");
    }
}

/// The page: page keys for the whole window, and element fullscreen.
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    let app = page.app_handle().clone();
    let result = page.with_webview(move |webview| {
        let view = webview.inner();
        for (signal, on) in [("enter-fullscreen", true), ("leave-fullscreen", false)] {
            let app = app.clone();
            view.connect_local(signal, false, move |_| {
                // Hide or show the bar after WebKitGTK has handled the signal.
                let app = app.clone();
                tauri::async_runtime::spawn(async move { bar::set_element_fullscreen(&app, on) });
                // FALSE: let WebKitGTK go on with its own fullscreen handling.
                Some(false.to_value())
            });
        }
        let Some(toplevel) = view.toplevel() else {
            eprintln!("little-joan: the webview has no toplevel window; page keys are off");
            return;
        };
        toplevel.connect_key_press_event(move |_, event| {
            let state = event.state();
            let mods = Mods {
                ctrl: state.contains(gtk::gdk::ModifierType::CONTROL_MASK),
                shift: state.contains(gtk::gdk::ModifierType::SHIFT_MASK),
                alt: state.contains(gtk::gdk::ModifierType::MOD1_MASK),
                meta: state.contains(gtk::gdk::ModifierType::SUPER_MASK)
                    || state.contains(gtk::gdk::ModifierType::META_MASK),
            };
            let key = event
                .keyval()
                .name()
                .map_or(Key::Other, |name| Key::from_gdk_name(name.as_str()));
            match keys::action_for(key, mods, Platform::Linux) {
                Some(action) => {
                    page_actions::perform(&app, action);
                    Propagation::Stop
                }
                None => Propagation::Proceed,
            }
        });
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the webview: {err}");
    }
}

/// The GDK window state's fullscreen bit (set by WebKitGTK's own fullscreen, by tao, or by the
/// window manager). Main thread only.
pub fn is_fullscreen<R: Runtime>(window: &Window<R>) -> bool {
    window
        .gtk_window()
        .ok()
        .and_then(|gtk_window| gtk_window.window())
        .is_some_and(|gdk_window| {
            gdk_window
                .state()
                .contains(gtk::gdk::WindowState::FULLSCREEN)
        })
}

/// WebKitGTK: the `user-agent` property of the webview's WebKitSettings (unset = the engine's
/// own). Set through GObject properties, so no webkit2gtk crate is needed.
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    let result = page.with_webview(move |webview| {
        let settings = webview.inner().property::<gtk::glib::Object>("settings");
        let user_agent = mobile.then(|| phone::mobile_user_agent(Platform::Linux, None));
        settings.set_property("user-agent", user_agent.as_deref());
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not change the user agent: {err}");
    }
}
