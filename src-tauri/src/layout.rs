//! Where the URL bar and the page go inside the page window (logical pixels).
//!
//! The window holds two webviews: the trusted local URL bar pinned to the top, and the page
//! below it. During element fullscreen (a video's fullscreen button) the bar is hidden and the
//! page fills the window.

/// Height of the URL bar in logical pixels.
pub const BAR_HEIGHT: f64 = 40.0;

/// A rectangle in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// `None` while the bar is hidden.
    pub bar: Option<Rect>,
    pub page: Rect,
}

/// Lays out a window of `width` × `height` logical pixels. A window shorter than the bar gives
/// the page zero height rather than a negative one.
pub fn layout(width: f64, height: f64, bar_visible: bool) -> Layout {
    let width = width.max(0.0);
    let height = height.max(0.0);
    if !bar_visible {
        return Layout {
            bar: None,
            page: Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
        };
    }
    let bar_height = BAR_HEIGHT.min(height);
    Layout {
        bar: Some(Rect {
            x: 0.0,
            y: 0.0,
            width,
            height: bar_height,
        }),
        page: Rect {
            x: 0.0,
            y: bar_height,
            width,
            height: height - bar_height,
        },
    }
}

/// Lays out a window whose inner size is given in **physical** pixels (what the OS reports) at
/// `scale` physical pixels per logical pixel. Everything after this is logical, which is what
/// the webviews' bounds take and what a page's CSS viewport (`innerWidth`) measures, so a
/// Retina window (scale 2) gets the same layout as a scale-1 window of the same logical size.
pub fn for_physical_size(width: u32, height: u32, scale: f64, bar_visible: bool) -> Layout {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    layout(
        f64::from(width) / scale,
        f64::from(height) / scale,
        bar_visible,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page's logical size must not depend on the display's scale factor: a phone-shaped
    /// window is phone-sized for the page on a Retina screen too.
    #[test]
    fn same_logical_layout_at_every_scale_factor() {
        for (logical_w, logical_h) in [(390.0, 844.0), (1280.0, 800.0), (480.0, 640.0)] {
            for scale in [1.0, 1.5, 2.0] {
                let physical = |v: f64| (v * scale).round() as u32;
                let got = for_physical_size(physical(logical_w), physical(logical_h), scale, true);
                let bar = got.bar.expect("visible");
                assert!(
                    (got.page.width - logical_w).abs() < 0.01,
                    "{logical_w} at {scale}: {got:?}"
                );
                assert!(
                    (got.page.height - (logical_h - BAR_HEIGHT)).abs() < 0.01,
                    "{logical_h} at {scale}: {got:?}"
                );
                assert_eq!(got.page.y, BAR_HEIGHT);
                assert_eq!(bar.height, BAR_HEIGHT);
                assert!((bar.width - logical_w).abs() < 0.01);
            }
        }
    }

    #[test]
    fn narrow_phone_window_at_scale_two() {
        // 390 × 844 logical on a Retina display: the OS reports 780 × 1688.
        let got = for_physical_size(780, 1688, 2.0, true);
        assert_eq!(
            got.page,
            Rect {
                x: 0.0,
                y: BAR_HEIGHT,
                width: 390.0,
                height: 804.0
            }
        );
    }

    #[test]
    fn fullscreen_at_scale_one_and_a_half_hides_the_bar() {
        let got = for_physical_size(2880, 1620, 1.5, false);
        assert_eq!(got.bar, None);
        assert_eq!((got.page.width, got.page.height), (1920.0, 1080.0));
    }

    #[test]
    fn a_bogus_scale_factor_counts_as_one() {
        assert_eq!(
            for_physical_size(480, 640, 0.0, true),
            layout(480.0, 640.0, true)
        );
        assert_eq!(
            for_physical_size(480, 640, f64::NAN, true),
            layout(480.0, 640.0, true)
        );
    }

    #[test]
    fn bar_on_top_page_below() {
        let got = layout(480.0, 640.0, true);
        assert_eq!(
            got.bar,
            Some(Rect {
                x: 0.0,
                y: 0.0,
                width: 480.0,
                height: BAR_HEIGHT
            })
        );
        assert_eq!(
            got.page,
            Rect {
                x: 0.0,
                y: BAR_HEIGHT,
                width: 480.0,
                height: 640.0 - BAR_HEIGHT
            }
        );
    }

    #[test]
    fn hidden_bar_gives_the_page_the_whole_window() {
        let got = layout(1920.0, 1080.0, false);
        assert_eq!(got.bar, None);
        assert_eq!(
            got.page,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0
            }
        );
    }

    #[test]
    fn tiny_or_odd_sizes_never_go_negative() {
        let got = layout(100.0, 10.0, true);
        assert_eq!(got.bar.map(|b| b.height), Some(10.0));
        assert_eq!(got.page.height, 0.0);
        let got = layout(-5.0, -5.0, true);
        assert_eq!(got.page.width, 0.0);
        assert_eq!(got.page.height, 0.0);
    }

    #[test]
    fn bar_and_page_tile_the_window_exactly() {
        for (w, h) in [(320.0, 300.0), (481.5, 777.25), (4096.0, 4096.0)] {
            let got = layout(w, h, true);
            let bar = got.bar.expect("visible");
            assert_eq!(bar.height + got.page.height, h);
            assert_eq!(got.page.y, bar.height);
            assert_eq!(bar.width, w);
            assert_eq!(got.page.width, w);
        }
    }
}
