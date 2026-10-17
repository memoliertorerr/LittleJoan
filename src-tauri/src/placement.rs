//! Where a new page window opens: centred in the work area of the monitor under the cursor.

use tauri::{AppHandle, PhysicalPosition, Runtime, Window};

/// Centres the new window in the work area of the monitor under the mouse cursor (falling back
/// to the primary monitor, then to the window manager's centre).
pub fn place<R: Runtime>(app: &AppHandle<R>, window: &Window<R>, width: f64, height: f64) {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        let _ = window.center();
        return;
    };
    let scale = monitor.scale_factor();
    let work = monitor.work_area();
    let area = Rect {
        x: work.position.x,
        y: work.position.y,
        width: work.size.width,
        height: work.size.height,
    };
    let (x, y) = centered_in(
        area,
        (to_physical(width, scale), to_physical(height, scale)),
    );
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn to_physical(logical: f64, scale: f64) -> u32 {
    // Window sizes are clamped to MAX_SIZE in settings, so this never exceeds u32.
    (logical * scale).round().clamp(0.0, f64::from(u32::MAX)) as u32
}

/// A monitor work area in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Top-left position that centres a window of `size` in `area`; a window larger than the area
/// is aligned to its top-left corner so the title bar stays reachable.
pub fn centered_in(area: Rect, size: (u32, u32)) -> (i32, i32) {
    let offset = |available: u32, length: u32| {
        i32::try_from(available.saturating_sub(length) / 2).unwrap_or(0)
    };
    (
        area.x.saturating_add(offset(area.width, size.0)),
        area.y.saturating_add(offset(area.height, size.1)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centres_inside_the_work_area() {
        let area = Rect {
            x: 0,
            y: 25,
            width: 1440,
            height: 875,
        };
        assert_eq!(centered_in(area, (480, 640)), (480, 142));
    }

    #[test]
    fn works_on_secondary_monitors_left_of_and_above_the_primary() {
        let area = Rect {
            x: -1920,
            y: -200,
            width: 1920,
            height: 1040,
        };
        assert_eq!(centered_in(area, (960, 640)), (-1440, 0));
    }

    #[test]
    fn oversized_windows_align_to_the_top_left() {
        let area = Rect {
            x: 100,
            y: 50,
            width: 800,
            height: 600,
        };
        assert_eq!(centered_in(area, (1000, 700)), (100, 50));
        assert_eq!(centered_in(area, (500, 900)), (250, 50));
    }

    #[test]
    fn logical_to_physical_rounds() {
        assert_eq!(to_physical(480.0, 1.25), 600);
        assert_eq!(to_physical(333.0, 1.5), 500);
        assert_eq!(to_physical(-3.0, 2.0), 0);
    }
}
