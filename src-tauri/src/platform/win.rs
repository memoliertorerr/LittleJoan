//! Windows: page keys through WebView2's `AcceleratorKeyPressed`, which sees key presses before
//! the page does and lets us mark them handled; and element fullscreen through
//! `ContainsFullScreenElementChanged`, which WebView2 leaves to the host app.

use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, Webview, Window};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
    COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN, ICoreWebView2Settings2,
};
use webview2_com::{
    AcceleratorKeyPressedEventHandler, ContainsFullScreenElementChangedEventHandler,
};
use windows::Win32::Foundation::{GlobalFree, HANDLE};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::core::{HSTRING, Interface, PWSTR};

use crate::bar;
use crate::fullscreen::FullscreenTracker;
use crate::keys::{self, Key, Mods, Platform};
use crate::page_actions;
use crate::phone;

/// The URL bar: page keys only.
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    install(bar, false);
}

/// The page: page keys, and element fullscreen.
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    install(page, true);
}

fn install<R: Runtime>(webview: &Webview<R>, with_fullscreen: bool) {
    let app = webview.app_handle().clone();
    let window = webview.window();
    let result = webview.with_webview(move |platform_webview| {
        let controller = platform_webview.controller();
        if with_fullscreen {
            install_fullscreen(&controller, app.clone(), window);
        }
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

/// Makes the window fullscreen while the page has a fullscreen element (a video's fullscreen
/// button) and hides the URL bar meanwhile; restores both afterwards.
fn install_fullscreen<R: Runtime>(
    controller: &webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller,
    app: AppHandle<R>,
    window: Window<R>,
) {
    // SAFETY: reads the core webview of a live controller.
    let core = match unsafe { controller.CoreWebView2() } {
        Ok(core) => core,
        Err(err) => {
            eprintln!("little-joan: no core webview, fullscreen video stays in the window: {err}");
            return;
        }
    };
    let tracker = Arc::new(Mutex::new(FullscreenTracker::default()));
    let handler =
        ContainsFullScreenElementChangedEventHandler::create(Box::new(move |sender, _| {
            let Some(sender) = sender else {
                return Ok(());
            };
            let mut contains = windows::core::BOOL::default();
            // SAFETY: plain COM getter on the webview that raised the event.
            unsafe { sender.ContainsFullScreenElement(&mut contains)? };
            let contains = contains.as_bool();
            // Resize outside this callback: changing the window re-enters WebView2 (it is resized
            // to the new bounds), so the change is queued from another thread.
            let window = window.clone();
            let app = app.clone();
            let tracker = Arc::clone(&tracker);
            tauri::async_runtime::spawn(async move {
                bar::set_element_fullscreen(&app, contains);
                let window_is_fullscreen = window.is_fullscreen().unwrap_or(false);
                let change = tracker
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .on_element_change(contains, window_is_fullscreen);
                if let Some(fullscreen) = change
                    && let Err(err) = window.set_fullscreen(fullscreen)
                {
                    eprintln!("little-joan: could not change fullscreen: {err}");
                }
            });
            Ok(())
        }));
    let mut token = 0i64;
    // SAFETY: registers a handler on the live core webview.
    if let Err(err) = unsafe { core.add_ContainsFullScreenElementChanged(&handler, &mut token) } {
        eprintln!("little-joan: could not install the fullscreen handler: {err}");
    }
}

fn is_down(key: VIRTUAL_KEY) -> bool {
    // SAFETY: GetKeyState only reads the calling thread's keyboard state; the high bit is set
    // while the key is down. This runs on the UI thread that received the key event.
    unsafe { GetKeyState(i32::from(key.0)) < 0 }
}

/// WebView2's own user agent, read before the first change (every page webview starts with it).
static ENGINE_USER_AGENT: OnceLock<String> = OnceLock::new();

/// WebView2: `ICoreWebView2Settings2::SetUserAgent`. The phone agent is Chrome on Android with
/// the installed WebView2's Chrome version; the desktop agent is WebView2's own, read once.
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    let result = page.with_webview(move |webview| {
        let controller = webview.controller();
        let outcome = (|| -> windows::core::Result<()> {
            // SAFETY: COM calls on the live controller of this webview and its settings.
            unsafe {
                let settings: ICoreWebView2Settings2 =
                    controller.CoreWebView2()?.Settings()?.cast()?;
                let engine = match ENGINE_USER_AGENT.get() {
                    Some(engine) => engine.clone(),
                    None => {
                        let mut raw = PWSTR::null();
                        settings.UserAgent(&mut raw)?;
                        ENGINE_USER_AGENT
                            .get_or_init(|| webview2_com::take_pwstr(raw))
                            .clone()
                    }
                };
                let user_agent = if mobile {
                    phone::mobile_user_agent(Platform::Windows, Some(&engine))
                } else {
                    engine
                };
                settings.SetUserAgent(&HSTRING::from(user_agent.as_str()))
            }
        })();
        if let Err(err) = outcome {
            eprintln!("little-joan: could not change the user agent: {err}");
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the webview: {err}");
    }
}

/// The clipboard's Unicode text format (Win32's CF_UNICODETEXT, 13), without the Ole bindings.
const CF_UNICODETEXT: u32 = 13;

/// Puts `text` on the clipboard as Unicode text. Main thread only. The clipboard may be held by
/// another program for a moment, so opening it is tried a few times.
pub fn write_clipboard(text: &str) -> Result<(), String> {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: the clipboard is opened (with no owner window, as clipboard libraries do), filled
    // with one movable global block holding the NUL-terminated UTF-16 text — which the system
    // owns once SetClipboardData succeeds, and which is freed here when it fails — and closed
    // again on every path.
    unsafe {
        let mut opened = OpenClipboard(None);
        for _ in 0..5 {
            if opened.is_ok() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
            opened = OpenClipboard(None);
        }
        opened.map_err(|err| format!("the clipboard is busy: {err}"))?;
        let written = (|| -> windows::core::Result<()> {
            EmptyClipboard()?;
            let memory = GlobalAlloc(GMEM_MOVEABLE, wide.len() * std::mem::size_of::<u16>())?;
            let target = GlobalLock(memory).cast::<u16>();
            if target.is_null() {
                let _ = GlobalFree(Some(memory));
                return Err(windows::core::Error::from_thread());
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), target, wide.len());
            // Returns "failed" with no error once the block is unlocked; nothing to handle.
            let _ = GlobalUnlock(memory);
            if let Err(err) = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(memory.0))) {
                let _ = GlobalFree(Some(memory));
                return Err(err);
            }
            Ok(())
        })();
        let _ = CloseClipboard();
        written.map_err(|err| err.to_string())
    }
}
