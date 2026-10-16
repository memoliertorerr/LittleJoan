//! Windows: page keys through WebView2's `AcceleratorKeyPressed`, which sees key presses before
//! the page does and lets us mark them handled.

use tauri::{Manager, Runtime, WebviewWindow};
use webview2_com::AcceleratorKeyPressedEventHandler;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
    COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};

use crate::keys::{self, Key, Mods, Platform};
use crate::page_actions;

pub fn install<R: Runtime>(window: &WebviewWindow<R>) {
    let app = window.app_handle().clone();
    let result = window.with_webview(move |webview| {
        let controller = webview.controller();
        let handler = AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else {
                return Ok(());
            };
            let mut kind = COREWEBVIEW2_KEY_EVENT_KIND::default();
            let mut virtual_key = 0u32;
            // SAFETY: plain COM getters on the event args WebView2 passed to this handler.
            unsafe {
                args.KeyEventKind(&mut kind)?;
                args.VirtualKey(&mut virtual_key)?;
            }
            if kind != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                && kind != COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
            {
                return Ok(());
            }
            let mods = Mods {
                ctrl: is_down(VK_CONTROL),
                shift: is_down(VK_SHIFT),
                alt: is_down(VK_MENU),
                meta: is_down(VK_LWIN) || is_down(VK_RWIN),
            };
            if let Some(action) =
                keys::action_for(Key::from_windows_vk(virtual_key), mods, Platform::Windows)
            {
                // SAFETY: as above; marks the key as consumed so the page never sees it.
                unsafe { args.SetHandled(true)? };
                page_actions::perform(&app, action);
            }
            Ok(())
        }));
        let mut token = 0i64;
        // SAFETY: registers a handler on the live controller of this webview.
        if let Err(err) = unsafe { controller.add_AcceleratorKeyPressed(&handler, &mut token) } {
            eprintln!("little-joan: could not install the key handler: {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the webview: {err}");
    }
}

fn is_down(key: VIRTUAL_KEY) -> bool {
    // SAFETY: GetKeyState only reads the calling thread's keyboard state; the high bit is set
    // while the key is down. This runs on the UI thread that received the key event.
    unsafe { GetKeyState(i32::from(key.0)) < 0 }
}
