//! The bar webview's own background (LJ39): the color under its page — what shows before the
//! page paints and, on Windows, in the strip a fast drag uncovers before WebView2 repaints (white
//! by default, a white gutter beside a dark toolbar). It is the `--lj-bg-0` the bar painted last,
//! kept in `AppState` across windows (a few bytes, memory only): set when a window's bar is built
//! and again whenever the bar paints another theme (`Webview::set_background_color`).
//!
//! The page webview keeps the engine's white. A page without a background of its own is drawn
//! over the webview's color on every engine (WebView2's `DefaultBackgroundColor`, WebKitGTK's
//! background color, WKWebView's `drawsBackground` turned off), so a dark color there would make
//! such pages — plain text, many simple sites — dark on dark.

use std::collections::BTreeMap;

use tauri::window::Color;
use tauri::{AppHandle, Manager, Runtime, Window};

use super::BAR;
use crate::extensions::theme::Tokens;
use crate::settings::{Theme, ThemeMode};
use crate::smoke;
use crate::state;

/// What a webview shows before its page paints when no theme was painted yet: the start page's
/// plain colors (ui/style.css's `.blank`), white or near-black.
pub const LIGHT_BACKGROUND: [u8; 3] = [0xff, 0xff, 0xff];
pub const DARK_BACKGROUND: [u8; 3] = [0x1c, 0x1c, 0x1e];

/// The variable the bar's background starts from (ui/lib/theme.ts: the base of the gradient).
const BG_VAR: &str = "--lj-bg-0";

/// `#rrggbb` → its three bytes; anything else (another CSS color form, junk) → `None`.
fn hex_rgb(value: &str) -> Option<[u8; 3]> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some([byte(0)?, byte(2)?, byte(4)?])
}

/// The bar webview's background: the `--lj-bg-0` the bar last painted (`tokens`, a checked
/// `#rrggbb`) if it was painted in this scheme (`dark`), else white or near-black for the scheme.
pub fn background_for(tokens: Option<&Tokens>, dark: bool) -> [u8; 3] {
    let scheme = if dark { "dark" } else { "light" };
    tokens
        .filter(|tokens| tokens.scheme == scheme)
        .and_then(|tokens| tokens.vars.get(BG_VAR))
        .and_then(|value| hex_rgb(value))
        .unwrap_or(if dark {
            DARK_BACKGROUND
        } else {
            LIGHT_BACKGROUND
        })
}

/// Whether a theme shows dark: its mode, or for `system` the system's choice.
pub fn shows_dark(mode: ThemeMode, system_dark: bool) -> bool {
    match mode {
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
        ThemeMode::System => system_dark,
    }
}

/// What `background_for` needs of a report, kept across windows: its scheme and `--lj-bg-0`.
pub fn background_tokens(tokens: &Tokens) -> Tokens {
    Tokens {
        scheme: tokens.scheme.clone(),
        vars: tokens
            .vars
            .get(BG_VAR)
            .map(|value| BTreeMap::from([(BG_VAR.to_owned(), value.clone())]))
            .unwrap_or_default(),
    }
}

fn color([r, g, b]: [u8; 3]) -> Color {
    Color(r, g, b, 255)
}

/// The background a new window's bar is built with, for `theme` (the theme it paints first): the
/// last painted one when it fits, else the scheme's plain color.
pub fn bar_background<R: Runtime>(window: &Window<R>, theme: &Theme) -> Color {
    let system_dark = matches!(window.theme(), Ok(tauri::Theme::Dark));
    let painted = state::with(window.app_handle(), |s| s.painted_background.clone());
    color(background_for(
        painted.as_ref(),
        shows_dark(theme.mode, system_dark),
    ))
}

/// The bar painted a theme (`set_theme_tokens`): its background becomes that theme's — kept for
/// the next window, and set on the open bar at once when it changed. Smoke log: `bar-background
/// <light|dark>`.
pub fn painted<R: Runtime>(app: &AppHandle<R>, tokens: &Tokens) {
    let kept = background_tokens(tokens);
    let changed = state::with(app, |s| {
        let changed = s.painted_background.as_ref() != Some(&kept);
        s.painted_background = Some(kept.clone());
        changed
    });
    if !changed {
        return;
    }
    if let Some(bar) = app.get_webview(BAR) {
        let rgb = background_for(Some(&kept), kept.scheme == "dark");
        if let Err(err) = bar.set_background_color(Some(color(rgb))) {
            eprintln!("little-joan: could not set the bar's background: {err}");
        }
    }
    smoke::log("bar-background", &kept.scheme);
}

/// The theme changed while no bar may paint it yet (Settings saved or previews one): the kept
/// background is stale, so a new window starts from the scheme's plain color until its bar paints.
pub fn forget_painted<R: Runtime>(app: &AppHandle<R>) {
    state::with(app, |s| s.painted_background = None);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(scheme: &str, bg: &str) -> Tokens {
        Tokens {
            scheme: scheme.to_owned(),
            vars: BTreeMap::from([
                (BG_VAR.to_owned(), bg.to_owned()),
                ("--lj-fg".to_owned(), "#101010".to_owned()),
            ]),
        }
    }

    #[test]
    fn the_background_is_the_painted_bg_0_in_its_scheme() {
        let dark = tokens("dark", "#1f2430");
        assert_eq!(background_for(Some(&dark), true), [0x1f, 0x24, 0x30]);
        let light = tokens("light", "#DAE1EE");
        assert_eq!(background_for(Some(&light), false), [0xda, 0xe1, 0xee]);
    }

    #[test]
    fn without_a_fitting_painted_theme_white_or_near_black() {
        assert_eq!(background_for(None, false), LIGHT_BACKGROUND);
        assert_eq!(background_for(None, true), DARK_BACKGROUND);
        // Painted in the other scheme (the system switched since): the scheme's plain color.
        let dark = tokens("dark", "#1f2430");
        assert_eq!(background_for(Some(&dark), false), LIGHT_BACKGROUND);
        // Not a plain #rrggbb: never parsed as something else.
        for junk in ["rgb(1, 2, 3)", "#abc", "#12345g", "#1234567", "red", ""] {
            assert_eq!(
                background_for(Some(&tokens("dark", junk)), true),
                DARK_BACKGROUND,
                "{junk}"
            );
        }
        let missing = Tokens {
            scheme: "dark".to_owned(),
            vars: BTreeMap::new(),
        };
        assert_eq!(background_for(Some(&missing), true), DARK_BACKGROUND);
    }

    #[test]
    fn the_scheme_follows_the_theme_then_the_system() {
        for system in [true, false] {
            assert!(shows_dark(ThemeMode::Dark, system));
            assert!(!shows_dark(ThemeMode::Light, system));
            assert_eq!(shows_dark(ThemeMode::System, system), system);
        }
    }

    #[test]
    fn only_the_background_is_kept_across_windows() {
        let kept = background_tokens(&tokens("dark", "#1f2430"));
        assert_eq!(kept.scheme, "dark");
        assert_eq!(kept.vars.len(), 1);
        assert_eq!(background_for(Some(&kept), true), [0x1f, 0x24, 0x30]);
    }

    #[test]
    fn the_plain_colors_are_the_start_pages() {
        // Whitespace folded: Windows checks the file out with CRLF line ends.
        let css = include_str!("../../../ui/style.css")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(css.contains(".blank { background: #ffffff; }"), "light");
        assert!(css.contains(".blank { background: #1c1c1e; }"), "dark");
    }
}
