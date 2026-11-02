//! Liquid Glass at run time (LJ19.9): what the bar is handed as its theme — the theme and the glass
//! it shows (`BarTheme`; Off unless this Mac shows glass now: macOS 26+, and neither Reduce
//! Transparency nor Increase Contrast) —, and the open window following it: the native glass behind
//! the bar (platform/mac/glass.rs), the bar's and the window's backgrounds (clear over glass), and
//! the bar's own CSS (`window.littleJoan.setTheme`). It follows the setting (Save), its preview
//! (Settings → Appearance), the theme the bar painted (the tint), and the accessibility settings
//! while Little Joan runs. Off — the default, and always on Windows, Linux and older Macs — changes
//! nothing: the bar gets the theme exactly as before, and no window is touched.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::window::Color;
use tauri::{AppHandle, Runtime, Webview};

use crate::bar;
use crate::platform;
use crate::settings::glass::{NativeGlass, effective};
use crate::settings::{Glass, Theme};
use crate::smoke;
use crate::state;

/// What the bar is handed as its theme: the theme and the glass it shows. Serialized as the theme's
/// own fields plus `"glass"` — left out for Off, so an Off bar gets exactly what it got before.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BarTheme {
    #[serde(flatten)]
    pub theme: Theme,
    #[serde(skip_serializing_if = "Glass::is_off")]
    pub glass: Glass,
}

impl BarTheme {
    /// The smoke log's view: `<mode> colors=<n>`, then ` glass=<clear|tinted>` when it shows.
    pub fn summary(&self) -> String {
        let theme = crate::commands::theme_summary(&self.theme);
        if self.glass.is_off() {
            theme
        } else {
            format!("{theme} glass={}", self.glass.name())
        }
    }
}

/// The native glass for the bar's window: the style, and the theme's colour that tints it (its
/// `--lj-bg-0`) in the scheme the bar shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassLook {
    pub native: NativeGlass,
    pub tint: [u8; 3],
    pub dark: bool,
}

impl GlassLook {
    /// `None` for Off.
    pub fn new(glass: Glass, tint: [u8; 3], dark: bool) -> Option<GlassLook> {
        glass
            .native()
            .map(|native| GlassLook { native, tint, dark })
    }
}

/// A background colour over glass: clear where glass shows, else the colour itself.
pub fn background(rgb: [u8; 3], glass: Glass) -> Color {
    let [r, g, b] = rgb;
    Color(r, g, b, if glass.is_off() { 255 } else { 0 })
}

/// The glass the window shows now: the setting (or its preview), where this system can.
pub fn shown<R: Runtime>(app: &AppHandle<R>) -> Glass {
    let setting = state::with(app, |s| s.shown_glass());
    effective(setting, platform::glass_support())
}

/// What the bar is handed now: the shown theme and glass.
pub fn bar_theme<R: Runtime>(app: &AppHandle<R>) -> BarTheme {
    let (theme, setting) = state::with(app, |s| (s.shown_theme(), s.shown_glass()));
    BarTheme {
        theme,
        glass: effective(setting, platform::glass_support()),
    }
}

/// Liquid Glass for Settings → Appearance (`get_settings`): `available` (macOS 26+; the setting
/// shows only then) and `blockedBy` (`reduceTransparency`, `increaseContrast` or `null`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsSupport {
    available: bool,
    blocked_by: Option<&'static str>,
}

impl SettingsSupport {
    pub fn now() -> SettingsSupport {
        let support = platform::glass_support();
        SettingsSupport {
            available: support.available,
            blocked_by: support.blocked_by(),
        }
    }
}

/// Whether the window has the native glass now (put there, and not taken out since).
static ON_WINDOW: AtomicBool = AtomicBool::new(false);

/// A new window's bar: with glass, the glass behind it from the start (before the window shows),
/// tinted with `base` (the theme's colour) in `dark`.
pub fn new_window<R: Runtime>(bar: &Webview<R>, glass: Glass, base: [u8; 3], dark: bool) {
    ON_WINDOW.store(false, Ordering::SeqCst);
    if let Some(look) = GlassLook::new(glass, base, dark) {
        platform::set_glass(bar, Some(look), base);
        ON_WINDOW.store(true, Ordering::SeqCst);
    }
}

/// The bar painted a theme (bar/background.rs): the glass takes its colour and scheme.
pub fn painted<R: Runtime>(bar: &Webview<R>, glass: Glass, base: [u8; 3], dark: bool) {
    if let Some(look) = GlassLook::new(glass, base, dark) {
        platform::set_glass(bar, Some(look), base);
        ON_WINDOW.store(true, Ordering::SeqCst);
    }
}

/// The open window takes `look`'s glass: the native glass and the bar's background, and the window
/// opaque again without it. Only when glass shows or showed: an Off window is left alone.
fn apply<R: Runtime>(bar: &Webview<R>, look: &BarTheme) {
    if look.glass.is_off() && !ON_WINDOW.load(Ordering::SeqCst) {
        return;
    }
    let (base, dark) = bar::base_background(&bar.window(), &look.theme);
    let native = GlassLook::new(look.glass, base, dark);
    // Over glass the bar's webview is clear; without it, the theme's colour (LJ39) again.
    if let Err(err) = bar.set_background_color(Some(background(base, look.glass))) {
        eprintln!("little-joan: could not set the bar's background: {err}");
    }
    platform::set_glass(bar, native, base);
    ON_WINDOW.store(native.is_some(), Ordering::SeqCst);
}

/// The glass may have changed — the setting, its preview, the accessibility settings: the open (or
/// resting) window follows, and its bar gets the theme with the glass it shows now.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let look = bar_theme(app);
    if let Some(bar) = bar::bar(app) {
        apply(&bar, &look);
    }
    bar::set_theme(app, &look);
}

/// At launch: follows Reduce Transparency and Increase Contrast (macOS 26+), and in smoke mode logs
/// `glass-support available=<bool> os=<major|-> reduce=<bool> contrast=<bool>`. Main thread (setup).
pub fn start<R: Runtime>(app: &AppHandle<R>) {
    let handle = app.clone();
    platform::watch_accessibility(Box::new(move || accessibility_changed(&handle)));
    if smoke::enabled() {
        let support = platform::glass_support();
        let os = platform::glass_os().map_or_else(|| "-".to_owned(), |major| major.to_string());
        smoke::log(
            "glass-support",
            &format!(
                "available={} os={os} reduce={} contrast={}",
                support.available, support.reduce_transparency, support.increase_contrast
            ),
        );
    }
}

/// Reduce Transparency or Increase Contrast changed: the window shows the solid theme, or its glass
/// again (smoke log `glass <off|clear|tinted>`, what it shows now).
fn accessibility_changed<R: Runtime>(app: &AppHandle<R>) {
    smoke::log("glass", shown(app).name());
    refresh(app);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::ThemeMode;

    fn theme() -> Theme {
        Theme {
            mode: ThemeMode::Dark,
            colors: vec!["#9d8fd0".into()],
            intensity: 0.6,
            grain: 0.4,
        }
    }

    #[test]
    fn off_hands_the_bar_the_theme_as_before() {
        let off = BarTheme {
            theme: theme(),
            glass: Glass::Off,
        };
        assert_eq!(
            serde_json::to_value(&off).ok(),
            serde_json::to_value(theme()).ok(),
            "no glass field"
        );
        assert_eq!(off.summary(), "dark colors=1");
        let clear = BarTheme {
            glass: Glass::Clear,
            ..off
        };
        let json = serde_json::to_value(&clear).expect("json");
        assert_eq!(json["glass"], "clear");
        assert_eq!(json["mode"], "dark");
        assert_eq!(clear.summary(), "dark colors=1 glass=clear");
    }

    #[test]
    fn glass_clears_the_backgrounds_off_keeps_them() {
        assert_eq!(background([1, 2, 3], Glass::Off), Color(1, 2, 3, 255));
        assert_eq!(background([1, 2, 3], Glass::Clear), Color(1, 2, 3, 0));
        assert_eq!(background([1, 2, 3], Glass::Tinted), Color(1, 2, 3, 0));
        assert_eq!(GlassLook::new(Glass::Off, [1, 2, 3], true), None);
        let look = GlassLook::new(Glass::Tinted, [1, 2, 3], true).expect("tinted");
        assert!(!look.native.clear && look.dark && look.tint == [1, 2, 3]);
    }
}
