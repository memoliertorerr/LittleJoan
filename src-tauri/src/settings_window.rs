//! The settings window: created on demand, destroyed on close, like the page window.

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder, Window, WindowEvent};

use crate::shortcut;

pub const LABEL: &str = "settings";

/// Opens the settings window, or focuses it if it is already open. Runs off the main thread
/// (building a webview window on the main thread can deadlock on Windows).
pub fn open<R: Runtime>(app: &AppHandle<R>) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(window) = app.get_webview_window(LABEL) {
            let _ = window.unminimize();
            let _ = window.set_focus();
            return;
        }
        let built = WebviewWindowBuilder::new(&app, LABEL, WebviewUrl::App("settings.html".into()))
            .title("Little Joan Settings")
            .inner_size(420.0, 430.0)
            .resizable(false)
            .maximizable(false)
            .minimizable(false)
            .center()
            .focused(true)
            .build();
        if let Err(err) = built {
            eprintln!("little-joan: could not open settings: {err}");
        }
    });
}

/// If the window closes while it was recording a shortcut, the paused shortcut comes back.
pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() == LABEL && matches!(event, WindowEvent::Destroyed) {
        let app = window.app_handle().clone();
        tauri::async_runtime::spawn(async move { shortcut::set_suspended(&app, false) });
    }
}
