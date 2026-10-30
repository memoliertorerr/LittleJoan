//! Windows: page keys through WebView2's `AcceleratorKeyPressed`, which sees key presses before
//! the page does and lets us mark them handled; element fullscreen through
//! `ContainsFullScreenElementChanged`, which WebView2 leaves to the host app; and phone mode's
//! agent through the DevTools protocol (LJ40): `Emulation.setUserAgentOverride` with client-hint
//! metadata, `SetUserAgent` only as the fallback, and `Page.getLayoutMetrics` to skip a reload for
//! a page that already fits. The protocol is the engine's own interface, called from Rust: no
//! script goes into the page, and DevTools stay off. Whether a summoned window is in front, for
//! the smoke log: win/front.rs (LJ19.2). LJ19.4: the bar's `ProcessFailed` (a ready window whose
//! bar is gone is built anew), the bar's memory target and visibility while the window rests, and
//! the page's downloads noted (`DownloadStarting`) so closing the page cancels them — the window's
//! WebView2 environment, which used to take them along, now outlives the page.

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, mpsc};
use std::thread::ThreadId;
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, Webview, Window};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
    COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN, ICoreWebView2, ICoreWebView2_2,
    ICoreWebView2CallDevToolsProtocolMethodCompletedHandler, ICoreWebView2Controller,
    ICoreWebView2Settings2,
};
use webview2_com::{
    AcceleratorKeyPressedEventHandler, CallDevToolsProtocolMethodCompletedHandler,
    ContainsFullScreenElementChangedEventHandler,
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
use crate::keymap::KeyAction;
use crate::keys::{Key, Mods, Platform};
use crate::page_actions;
use crate::phone::{self, agent};
use crate::smoke;
use crate::state;

/// A summoned window in front, for the smoke log (LJ19.2).
mod front;
pub use front::{foreground, front};
/// The bar's processes failing, its rest, the page's downloads (LJ19.4).
mod rest;
pub use rest::{bar_at_rest, cancel_downloads};
use rest::{install_process_failed, note_downloads};

/// The bar: page keys, and its processes failing (LJ19.4).
pub fn install_bar<R: Runtime>(bar: &Webview<R>) {
    install(bar, false);
}

/// The page: page keys, element fullscreen, and its downloads noted.
pub fn install_page<R: Runtime>(page: &Webview<R>) {
    install(page, true);
}

fn install<R: Runtime>(webview: &Webview<R>, page: bool) {
    let app = webview.app_handle().clone();
    let window = webview.window();
    let result = webview.with_webview(move |platform_webview| {
        let controller = platform_webview.controller();
        if page {
            install_fullscreen(&controller, app.clone(), window);
            note_downloads(&controller);
        } else {
            install_process_failed(&controller, app.clone());
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
            let key = Key::from_windows_vk(virtual_key);
            if let Some(action) =
                state::with(&app, |s| s.keymap.action_for(key, mods)).map(KeyAction::page_action)
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
    controller: &ICoreWebView2Controller,
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
/// WebView2's full version (`ICoreWebView2Environment::BrowserVersionString`), read once.
static ENGINE_VERSION: OnceLock<Option<String>> = OnceLock::new();

/// Whether the DevTools protocol's agent override works here: not tried yet, yes, or refused (the
/// bare `SetUserAgent` from then on).
static OVERRIDE: AtomicU8 = AtomicU8::new(UNTRIED);
const UNTRIED: u8 = 0;
const WORKS: u8 = 1;
const REFUSED: u8 = 2;

/// How long a caller waits for the engine's answer: the page reloads right after its agent
/// changes, and must go out with the new one.
const ANSWER_WAIT: Duration = Duration::from_secs(2);

/// The main thread (set up in lib.rs): never wait there for an answer the main thread delivers.
static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

pub fn note_main_thread() {
    let _ = MAIN_THREAD.set(std::thread::current().id());
}

fn on_main_thread() -> bool {
    MAIN_THREAD.get() == Some(&std::thread::current().id())
}

/// WebView2's own agent, from the settings (never changed while the override works).
///
/// SAFETY: COM calls on a live core webview's settings, on the UI thread.
unsafe fn engine_user_agent(settings: &ICoreWebView2Settings2) -> windows::core::Result<String> {
    if let Some(engine) = ENGINE_USER_AGENT.get() {
        return Ok(engine.clone());
    }
    let mut raw = PWSTR::null();
    // SAFETY: as above; UserAgent hands back a string WebView2 allocated, which take_pwstr frees.
    unsafe { settings.UserAgent(&mut raw)? };
    Ok(ENGINE_USER_AGENT
        .get_or_init(|| webview2_com::take_pwstr(raw))
        .clone())
}

/// WebView2's full version, for the client hints' full version list (`None` if unreadable).
///
/// SAFETY: COM calls on a live core webview, on the UI thread.
unsafe fn engine_version(core: &ICoreWebView2) -> Option<String> {
    ENGINE_VERSION
        .get_or_init(|| {
            let mut raw = PWSTR::null();
            // SAFETY: as above; BrowserVersionString allocates the string, take_pwstr frees it.
            let read = unsafe {
                core.cast::<ICoreWebView2_2>()
                    .and_then(|core| core.Environment())
                    .and_then(|environment| environment.BrowserVersionString(&mut raw))
            };
            read.ok().map(|()| webview2_com::take_pwstr(raw))
        })
        .clone()
}

/// The bare agent (`SetUserAgent`): the fallback, and the desktop agent before any override.
///
/// SAFETY: COM calls on a live core webview's settings, on the UI thread.
unsafe fn bare_agent(
    settings: &ICoreWebView2Settings2,
    mobile: bool,
    engine: &str,
) -> windows::core::Result<()> {
    let user_agent = if mobile {
        phone::mobile_user_agent(Platform::Windows, Some(engine))
    } else {
        engine.to_owned()
    };
    // SAFETY: as above.
    unsafe { settings.SetUserAgent(&HSTRING::from(user_agent.as_str())) }
}

/// How a change of agent went.
enum Outcome {
    /// Through the DevTools protocol, with client hints.
    Override,
    /// The protocol refused it; the bare agent instead (reason).
    Fallback(String),
    /// The bare agent, on purpose (the protocol refused before, or nothing to clear).
    Bare,
    /// Neither worked (reason).
    Failed(String),
}

/// The handler for the override's answer: on success the protocol works here; on a refusal the
/// bare agent instead, and the protocol isn't tried again.
fn override_answer(
    fallback: ICoreWebView2Settings2,
    mobile: bool,
    engine: String,
    reply: mpsc::Sender<Outcome>,
) -> ICoreWebView2CallDevToolsProtocolMethodCompletedHandler {
    CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |result, _answer| {
        let outcome = match result {
            Ok(()) => {
                OVERRIDE.store(WORKS, Ordering::SeqCst);
                Outcome::Override
            }
            Err(err) => {
                OVERRIDE.store(REFUSED, Ordering::SeqCst);
                // SAFETY: the settings of a live core webview; the handler runs on the UI thread.
                match unsafe { bare_agent(&fallback, mobile, &engine) } {
                    Ok(()) => Outcome::Fallback(err.message()),
                    Err(bare) => Outcome::Failed(bare.message()),
                }
            }
        };
        let _ = reply.send(outcome);
        Ok(())
    }))
}

/// Starts the change of agent on the UI thread: `Some(outcome)` when it is already done, `None`
/// when the protocol's handler will answer on `reply`.
///
/// SAFETY: `controller` is the live controller of a webview, and this runs on the UI thread.
unsafe fn start_agent_change(
    controller: &ICoreWebView2Controller,
    mobile: bool,
    reply: mpsc::Sender<Outcome>,
) -> windows::core::Result<Option<Outcome>> {
    // SAFETY: as above: COM calls on the live controller, its core webview and its settings.
    unsafe {
        let core = controller.CoreWebView2()?;
        let settings: ICoreWebView2Settings2 = core.Settings()?.cast()?;
        let engine = engine_user_agent(&settings)?;
        let state = OVERRIDE.load(Ordering::SeqCst);
        // Refused before: the bare agent. Desktop with no override ever set: nothing to clear.
        if state == REFUSED || (!mobile && state == UNTRIED) {
            bare_agent(&settings, mobile, &engine)?;
            return Ok(Some(Outcome::Bare));
        }
        let version = engine_version(&core);
        let params = agent::override_params(mobile, Some(&engine), version.as_deref());
        let handler = override_answer(settings.clone(), mobile, engine.clone(), reply);
        if let Err(err) = core.CallDevToolsProtocolMethod(
            &HSTRING::from("Emulation.setUserAgentOverride"),
            &HSTRING::from(params.to_string()),
            &handler,
        ) {
            OVERRIDE.store(REFUSED, Ordering::SeqCst);
            bare_agent(&settings, mobile, &engine)?;
            return Ok(Some(Outcome::Fallback(err.message())));
        }
        Ok(None)
    }
}

/// Phone mode's agent. The phone agent is Chrome on Android with the installed WebView2's Chrome
/// version and, through `Emulation.setUserAgentOverride`, client-hint metadata to match (Android,
/// mobile: phone::agent); the desktop agent clears that override (WebView2's own agent and
/// metadata). If the protocol refuses, `ICoreWebView2Settings2::SetUserAgent` as before, logged
/// `ua-override fallback <reason>`. Waits (off the main thread) for the engine, so a reload right
/// after goes out with the new agent; smoke log `ua-override <phone|desktop>`.
pub fn set_mobile<R: Runtime>(page: &Webview<R>, mobile: bool) {
    let (sent, answer) = mpsc::channel::<Outcome>();
    let result = page.with_webview(move |webview| {
        let controller = webview.controller();
        // SAFETY: with_webview runs on the UI thread while the webview (and its controller) lives.
        match unsafe { start_agent_change(&controller, mobile, sent.clone()) } {
            Ok(Some(done)) => {
                let _ = sent.send(done);
            }
            // The protocol's handler answers.
            Ok(None) => {}
            Err(err) => {
                let _ = sent.send(Outcome::Failed(err.message()));
            }
        }
    });
    if let Err(err) = result {
        eprintln!("little-joan: could not reach the webview: {err}");
        return;
    }
    if on_main_thread() {
        return;
    }
    match answer.recv_timeout(ANSWER_WAIT) {
        Ok(Outcome::Override) => {
            smoke::log("ua-override", if mobile { "phone" } else { "desktop" });
        }
        Ok(Outcome::Fallback(reason)) => {
            eprintln!(
                "little-joan: the agent override was refused ({reason}); the bare agent instead"
            );
            smoke::log("ua-override", &format!("fallback {}", one_line(&reason)));
        }
        Ok(Outcome::Bare) => {}
        Ok(Outcome::Failed(reason)) => {
            eprintln!("little-joan: could not change the user agent: {reason}");
        }
        Err(_) => smoke::log("ua-override", "no answer"),
    }
}

/// A reason on one line, for the smoke log.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Calls a DevTools protocol method on the page's engine and waits (never on the main thread)
/// for its answer: the result's JSON, or why not.
fn devtools<R: Runtime>(
    page: &Webview<R>,
    method: &'static str,
    params: &str,
) -> Result<String, String> {
    if on_main_thread() {
        return Err("on the main thread".to_owned());
    }
    let params = params.to_owned();
    let (sent, answer) = mpsc::channel::<Result<String, String>>();
    let reached = page.with_webview(move |webview| {
        let failed = sent.clone();
        // SAFETY: COM calls on the live controller of this webview (UI thread); the handler runs
        // there with WebView2's answer.
        let started = unsafe {
            webview.controller().CoreWebView2().and_then(|core| {
                let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(
                    move |result, answer| {
                        let _ = sent.send(result.map(|()| answer).map_err(|err| err.message()));
                        Ok(())
                    },
                ));
                core.CallDevToolsProtocolMethod(
                    &HSTRING::from(method),
                    &HSTRING::from(params.as_str()),
                    &handler,
                )
            })
        };
        if let Err(err) = started {
            let _ = failed.send(Err(err.message()));
        }
    });
    reached.map_err(|err| err.to_string())?;
    answer
        .recv_timeout(ANSWER_WAIT)
        .map_err(|_| "no answer".to_owned())?
}

/// Whether the page already fits its viewport (`Page.getLayoutMetrics`: its content no wider
/// than the layout viewport — phone::agent::skip_reload); `None` when the engine didn't say.
/// Smoke log: `layout-metrics content=<w> viewport=<w>`.
pub fn page_fits<R: Runtime>(page: &Webview<R>) -> Option<bool> {
    let answer = devtools(page, "Page.getLayoutMetrics", "{}")
        .map_err(|err| eprintln!("little-joan: no layout metrics: {err}"))
        .ok()?;
    let (content, viewport) = agent::layout_widths(&answer)?;
    smoke::log(
        "layout-metrics",
        &format!("content={content} viewport={viewport}"),
    );
    Some(agent::skip_reload(content, viewport))
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
