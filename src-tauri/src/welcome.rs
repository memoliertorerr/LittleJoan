//! What the one-time welcome on the first launch says about this desktop (pure, tested). The
//! command bar shows it (ui/command/welcome.ts); commands::start_info hands it over.

/// Where the tray / menu-bar icon is on `os` (`std::env::consts::OS`).
pub fn tray_hint(os: &str) -> &'static str {
    match os {
        "macos" => "Little Joan's icon is in the menu bar at the top of the screen.",
        "windows" => {
            "Little Joan's icon is in the system tray at the bottom right of the taskbar; it may be \
             hidden under the ^ arrow."
        }
        _ => {
            "Little Joan's icon is in the system tray of your panel. On GNOME the icon needs the \
             AppIndicator extension. Opening Little Joan from your app list also summons it."
        }
    }
}

/// A Linux Wayland session, where the X11 global shortcut can't fire (it only sees keys while
/// an X11 window has the focus), so the portal is asked (shortcut/backend.rs). `session_type` is
/// `XDG_SESSION_TYPE`, `wayland_display` `WAYLAND_DISPLAY`. An app forced onto XWayland is still
/// in a Wayland session.
pub fn wayland_session(
    os: &str,
    session_type: Option<&str>,
    wayland_display: Option<&str>,
) -> bool {
    os == "linux"
        && (session_type.is_some_and(|kind| kind.eq_ignore_ascii_case("wayland"))
            || wayland_display.is_some_and(|display| !display.is_empty()))
}

/// [`wayland_session`] for this process.
pub fn wayland_session_here() -> bool {
    let var = |name: &str| std::env::var(name).ok();
    wayland_session(
        std::env::consts::OS,
        var("XDG_SESSION_TYPE").as_deref(),
        var("WAYLAND_DISPLAY").as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icon_is_in_the_menu_bar_on_macos() {
        assert!(tray_hint("macos").contains("menu bar"));
        assert!(!tray_hint("macos").contains("GNOME"));
    }

    #[test]
    fn the_icon_may_hide_in_the_overflow_on_windows() {
        let hint = tray_hint("windows");
        assert!(hint.contains("system tray"));
        assert!(hint.contains("^"));
        assert!(!hint.contains("GNOME"));
    }

    #[test]
    fn linux_mentions_the_gnome_extension_and_the_app_list() {
        let hint = tray_hint("linux");
        assert!(hint.contains("system tray"));
        assert!(hint.contains("On GNOME the icon needs the AppIndicator extension."));
        assert!(hint.contains("Opening Little Joan from your app list also summons it."));
    }

    #[test]
    fn wayland_is_a_linux_session_with_wayland() {
        assert!(wayland_session("linux", Some("wayland"), None));
        assert!(wayland_session("linux", Some("Wayland"), None));
        assert!(wayland_session("linux", None, Some("wayland-0")));
        assert!(wayland_session("linux", Some("x11"), Some("wayland-0")));
        assert!(!wayland_session("linux", Some("x11"), None));
        assert!(!wayland_session("linux", None, Some("")));
        assert!(!wayland_session("linux", None, None));
        assert!(!wayland_session(
            "macos",
            Some("wayland"),
            Some("wayland-0")
        ));
        assert!(!wayland_session("windows", None, Some("wayland-0")));
    }
}
