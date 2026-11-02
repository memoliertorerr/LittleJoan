//! Liquid Glass's native side per OS (LJ19.9): only macOS 26 and later have it (mac/glass.rs);
//! Windows, Linux and older Macs report no support, and nothing here does anything there.

use tauri::{Runtime, Webview};

use crate::glass::GlassLook;
use crate::settings::glass::Support;

/// Whether glass can show now: macOS 26+ with NSGlassEffectView, and the user's Reduce
/// Transparency / Increase Contrast as last read.
#[cfg(target_os = "macos")]
pub fn glass_support() -> Support {
    super::mac::glass_support()
}

/// Windows and Linux: never.
#[cfg(not(target_os = "macos"))]
pub fn glass_support() -> Support {
    Support::default()
}

/// The system's major version, for the smoke log (`None` off macOS).
#[cfg(target_os = "macos")]
pub fn glass_os() -> Option<i64> {
    Some(super::mac::os_major())
}

#[cfg(not(target_os = "macos"))]
pub fn glass_os() -> Option<i64> {
    None
}

/// Puts the glass behind the bar's window (`Some`), or takes it out — the window opaque again over
/// `solid` (`None`; a window that never had glass is left alone). Queued on the main thread.
#[cfg(target_os = "macos")]
pub fn set_glass<R: Runtime>(bar: &Webview<R>, look: Option<GlassLook>, solid: [u8; 3]) {
    super::mac::set_glass(bar, look, solid);
}

#[cfg(not(target_os = "macos"))]
pub fn set_glass<R: Runtime>(_bar: &Webview<R>, _look: Option<GlassLook>, _solid: [u8; 3]) {}

/// Follows Reduce Transparency and Increase Contrast (macOS 26+): `on_change` runs after either
/// changed. Main thread (setup).
#[cfg(target_os = "macos")]
pub fn watch_accessibility(on_change: Box<dyn Fn() + Send + Sync>) {
    super::mac::watch_accessibility(on_change);
}

#[cfg(not(target_os = "macos"))]
pub fn watch_accessibility(_on_change: Box<dyn Fn() + Send + Sync>) {}
