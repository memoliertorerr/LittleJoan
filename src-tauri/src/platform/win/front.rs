//! Windows: whether a summoned window is in front with the keyboard (LJ19.2), for the smoke log.
//! The summon itself is tao's: `show()` and `set_focus()`, which calls `SetForegroundWindow` and,
//! when Windows' foreground lock refuses it (a background process may not take the foreground),
//! taps Alt through `SendInput` and tries again — input the system counts as the user's, which
//! lifts the lock. A summon through the global shortcut needs neither: the process that receives
//! a hotkey (`WM_HOTKEY`) may set the foreground window; a second launch (`--show`) passes the
//! right on with `AllowSetForegroundWindow` (tauri-plugin-single-instance). The window is never
//! made topmost, so a full-screen window that isn't topmost (a browser in F11, a borderless game)
//! goes behind it, and Little Joan behind whatever the user clicks next.

use tauri::{Runtime, Window};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetForegroundWindow, GetWindowRect, WindowFromPoint,
};

use crate::spaces::Front;

/// Whether the window is the foreground window and the topmost one at its centre
/// (`WindowFromPoint`, then its top-level ancestor: the webviews are child windows). `None` when
/// the window has no handle any more.
pub fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    let hwnd = window.hwnd().ok()?;
    let mut rect = RECT::default();
    // SAFETY: plain Win32 queries about a window handle, valid on any thread; a handle that died
    // meanwhile makes them fail or answer for nothing, never touch memory of ours.
    unsafe {
        GetWindowRect(hwnd, &mut rect).ok()?;
        let centre = POINT {
            x: rect.left + (rect.right - rect.left) / 2,
            y: rect.top + (rect.bottom - rect.top) / 2,
        };
        let hit = WindowFromPoint(centre);
        Some(Front {
            focused: GetForegroundWindow() == hwnd,
            top: !hit.is_invalid() && GetAncestor(hit, GA_ROOT) == hwnd,
            space: None,
        })
    }
}
