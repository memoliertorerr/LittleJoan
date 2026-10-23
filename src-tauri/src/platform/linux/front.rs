//! Linux: a summoned window in front, with the keyboard (LJ19.2). GTK's `present` asks the window
//! manager to raise and focus the window (`_NET_ACTIVE_WINDOW`), with a timestamp; window managers
//! refuse a request older than the user's last input in another window (focus-stealing
//! prevention: openbox, Mutter and KWin all do) and leave the window behind the focused one —
//! behind a full-screen window, out of sight. tao presents with `GDK_CURRENT_TIME`, which GTK
//! turns into the last input event *this app* saw: none for a new process (fine), but after the
//! user once typed into Little Joan, that old keystroke — older than whatever they did since in
//! the full-screen app — so the next summon was refused. The global shortcut's key never reaches
//! GTK (global-hotkey grabs it on its own X connection), so on X11 the request carries the X
//! server's time now (`gdk_x11_get_server_time`), as hotkey launchers do; on Wayland GTK and the
//! compositor decide (xdg-activation), as before. A window manager may still refuse when the user
//! was typing in another window that very moment.

use std::sync::mpsc;
use std::time::Duration;

use gtk::glib::translate::ToGlibPtr;
use gtk::prelude::*;
use tauri::{Runtime, Window};

use crate::smoke;
use crate::spaces::Front;

unsafe extern "C" {
    /// GDK 3's X11 backend (libgdk-3, which gdk-sys links): the X server's current timestamp,
    /// through a property change on `window` (a round trip). X11 windows only.
    fn gdk_x11_get_server_time(window: *mut gtk::gdk::ffi::GdkWindow) -> u32;
}

/// Raises and focuses a window that `show()` was asked for (the request may still be queued:
/// GTK's present shows it then), with the X server's time on X11. Queued on the main thread; the
/// caller doesn't wait.
pub fn bring_to_front<R: Runtime>(window: &Window<R>) -> tauri::Result<()> {
    let target = window.clone();
    window.run_on_main_thread(move || {
        let Ok(gtk_window) = target.gtk_window() else {
            return;
        };
        if !gtk_window.is_visible() {
            // tao's show request hasn't run yet: show the toplevel now (only the toplevel —
            // `show_all` would show the page webview the command bar's overlay hides), so the
            // timestamp below has a realized window to ask for.
            gtk_window.show();
        }
        let x11 = gtk_window
            .window()
            .filter(|gdk_window| gdk_window.type_().name() == "GdkX11Window");
        let time = match &x11 {
            // SAFETY: a realized X11 GdkWindow of this app, on the GTK main thread.
            Some(gdk_window) => unsafe { gdk_x11_get_server_time(gdk_window.to_glib_none().0) },
            None => gtk::gdk::ffi::GDK_CURRENT_TIME as u32,
        };
        gtk_window.present_with_time(time);
        smoke::log(
            "window-present",
            &format!(
                "{} {}",
                target.label(),
                if x11.is_some() {
                    "x11 server-time"
                } else {
                    "current-time"
                }
            ),
        );
    })
}

/// For the smoke log (CI's over-full-screen check): whether GTK says the window is active (the
/// window manager gave it the focus) and it tops the window manager's stacking order
/// (`_NET_CLIENT_LIST_STACKING`, through GDK's window stack; empty without an EWMH window manager).
/// `None` when the window is gone or the main thread didn't answer within 500 ms.
pub fn front<R: Runtime>(window: &Window<R>) -> Option<Front> {
    let target = window.clone();
    let (sent, answer) = mpsc::channel();
    window
        .run_on_main_thread(move || {
            let Ok(gtk_window) = target.gtk_window() else {
                return;
            };
            let top = gtk_window.window().is_some_and(|gdk_window| {
                gdk_window.screen().window_stack().last() == Some(&gdk_window)
            });
            let _ = sent.send(Front {
                focused: gtk_window.is_active(),
                top,
                space: None,
            });
        })
        .ok()?;
    answer.recv_timeout(Duration::from_millis(500)).ok()
}
