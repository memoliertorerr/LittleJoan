//! Linux: page keys through the GTK window's `key-press-event`, which runs before WebKitGTK
//! receives the key (handlers connected with `connect_*` run before the class handler).

use gtk::glib::Propagation;
use gtk::prelude::*;
use tauri::{Manager, Runtime, WebviewWindow, Window};

use crate::keys::{self, Key, Mods, Platform};
use crate::page_actions;

pub fn install<R: Runtime>(window: &WebviewWindow<R>) {
    let app = window.app_handle().clone();
    // with_webview runs on the GTK main thread, where GTK objects may be touched.
    let result = window.with_webview(move |webview| {
        let Some(toplevel) = webview.inner().toplevel() else {
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
