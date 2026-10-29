//! The page window itself (joan/rest.rs builds it): the macOS traffic lights inside the toolbar,
//! the size a summon opens at, and the size it rests at.

use tauri::window::WindowBuilder;
use tauri::{AppHandle, Runtime, Window};

use super::LABEL;
use crate::bar;
use crate::settings;
use crate::state;

/// The window's logical size for a summon: the remembered one.
pub(super) fn size_of<R: Runtime>(app: &AppHandle<R>) -> (f64, f64) {
    state::with(app, |s| {
        (
            f64::from(s.settings.window_width),
            f64::from(s.settings.window_height),
        )
    })
}

/// The window back to its remembered size, not fullscreen, maximized or minimized (it rests so).
pub(super) fn normal_size<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
    if bar::element_fullscreen() || window.is_fullscreen().unwrap_or(false) {
        let _ = window.set_fullscreen(false);
    }
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    }
    if window.is_minimized().unwrap_or(false) {
        let _ = window.unminimize();
    }
    let (width, height) = size_of(app);
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
}

/// Where the traffic lights sit on macOS (logical px: the close button's left edge, and the
/// title bar area's height below the buttons — tao's inset): inside the 40 px toolbar, which
/// leaves them about 78 px on its left.
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHTS: (f64, f64) = (14.0, 20.0);

/// macOS: the traffic lights sit inside the toolbar. `TitleBarStyle::Overlay` with a hidden
/// title extends the content view under a transparent title bar, so the bar webview starts at
/// y = 0 beneath the buttons (LJ10.1 used `Transparent` to keep content below the title bar;
/// here that is the point). Tauri's `window::WindowBuilder` has no setter for the buttons'
/// position, but its config does, so the builder starts from a `WindowConfig`.
#[cfg(target_os = "macos")]
pub(super) fn window_builder<R: Runtime>(
    app: &AppHandle<R>,
) -> tauri::Result<WindowBuilder<'_, R, AppHandle<R>>> {
    let mut config = tauri::utils::config::WindowConfig::default();
    config.label = LABEL.to_owned();
    config.title_bar_style = tauri::TitleBarStyle::Overlay;
    config.hidden_title = true;
    config.traffic_light_position = Some(tauri::utils::config::LogicalPosition {
        x: TRAFFIC_LIGHTS.0,
        y: TRAFFIC_LIGHTS.1,
    });
    WindowBuilder::from_config(app, &config)
}

/// Windows and Linux keep their native title bar (custom chrome is Later).
#[cfg(not(target_os = "macos"))]
pub(super) fn window_builder<R: Runtime>(
    app: &AppHandle<R>,
) -> tauri::Result<WindowBuilder<'_, R, AppHandle<R>>> {
    Ok(WindowBuilder::new(app, LABEL))
}

/// Saves the window's logical size as the size for the next summon. A fullscreen, maximized,
/// or minimized window says nothing about the size the user wants, so it is skipped.
pub(super) fn remember_size<R: Runtime>(app: &AppHandle<R>, window: &Window<R>) {
    let unusual = window.is_fullscreen().unwrap_or(true)
        || window.is_maximized().unwrap_or(true)
        || window.is_minimized().unwrap_or(true);
    if unusual {
        return;
    }
    let (Ok(size), Ok(scale)) = (window.inner_size(), window.scale_factor()) else {
        return;
    };
    let logical = size.to_logical::<f64>(scale);
    let changed = state::with(app, |s| {
        let updated = s
            .settings
            .clone()
            .with_window_size(logical.width, logical.height);
        if updated == s.settings {
            return None;
        }
        s.settings = updated.clone();
        Some((s.data_dir.clone(), updated))
    });
    if let Some((dir, updated)) = changed
        && let Err(err) = settings::save(&dir, &updated)
    {
        eprintln!("little-joan: could not save the window size: {err}");
    }
}
