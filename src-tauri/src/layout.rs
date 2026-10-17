//! Where the bar and the page go inside the page window (logical pixels).
//!
//! The window holds two webviews: the trusted local bar and the page. Three modes:
//! - `Toolbar`: the bar is a 40 px strip at the top and the page fills the rest;
//! - `Overlay`: the bar takes the whole window (the command bar) and the page is hidden;
//! - `Fullscreen`: element fullscreen (a video's fullscreen button) — the bar is hidden and the
//!   page fills the window. It always wins: starting it closes the overlay.

/// Height of the bar in Toolbar mode, in logical pixels.
pub const BAR_HEIGHT: f64 = 40.0;

/// What the window shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The 40 px bar above the page.
    Toolbar,
    /// The bar covers the whole window; the page is hidden.
    Overlay,
    /// The page alone fills the window (element fullscreen).
    Fullscreen,
}

impl Mode {
    /// The mode for the bar's overlay state and the page's element fullscreen: fullscreen wins.
    pub fn from_state(overlay: bool, element_fullscreen: bool) -> Mode {
        if element_fullscreen {
            Mode::Fullscreen
        } else if overlay {
            Mode::Overlay
        } else {
            Mode::Toolbar
        }
    }
}

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
    /// `None` while the page is hidden.
    pub page: Option<Rect>,
}

/// Lays out a window of `width` × `height` logical pixels. A window shorter than the bar gives
/// the page zero height rather than a negative one.
pub fn layout(width: f64, height: f64, mode: Mode) -> Layout {
    let width = width.max(0.0);
    let height = height.max(0.0);
    let whole = Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    match mode {
        Mode::Fullscreen => Layout {
            bar: None,
            page: Some(whole),
        },
        Mode::Overlay => Layout {
            bar: Some(whole),
            page: None,
        },
        Mode::Toolbar => {
            let bar_height = BAR_HEIGHT.min(height);
            Layout {
                bar: Some(Rect {
                    height: bar_height,
                    ..whole
                }),
                page: Some(Rect {
                    y: bar_height,
                    height: height - bar_height,
                    ..whole
                }),
            }
        }
    }
}

/// Lays out a window whose inner size is given in **physical** pixels (what the OS reports) at
/// `scale` physical pixels per logical pixel. Everything after this is logical, which is what
/// the webviews' bounds take and what a page's CSS viewport (`innerWidth`) measures, so a
/// Retina window (scale 2) gets the same layout as a scale-1 window of the same logical size.
pub fn for_physical_size(width: u32, height: u32, scale: f64, mode: Mode) -> Layout {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    layout(f64::from(width) / scale, f64::from(height) / scale, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALES: [f64; 3] = [1.0, 1.5, 2.0];
    const SIZES: [(f64, f64); 3] = [(390.0, 844.0), (1280.0, 800.0), (480.0, 640.0)];

    fn physical(v: f64, scale: f64) -> u32 {
        (v * scale).round() as u32
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    /// The page's logical size must not depend on the display's scale factor: a phone-shaped
    /// window is phone-sized for the page on a Retina screen too.
    #[test]
    fn toolbar_is_the_same_logical_layout_at_every_scale_factor() {
        for (w, h) in SIZES {
            for scale in SCALES {
                let got =
                    for_physical_size(physical(w, scale), physical(h, scale), scale, Mode::Toolbar);
                let bar = got.bar.expect("visible bar");
                let page = got.page.expect("visible page");
                assert!(close(page.width, w), "{w} at {scale}: {got:?}");
                assert!(
                    close(page.height, h - BAR_HEIGHT),
                    "{h} at {scale}: {got:?}"
                );
                assert_eq!(page.y, BAR_HEIGHT);
                assert_eq!(bar.height, BAR_HEIGHT);
                assert!(close(bar.width, w));
            }
        }
    }

    #[test]
    fn overlay_gives_the_bar_the_whole_window_and_hides_the_page_at_every_scale() {
        for (w, h) in SIZES {
            for scale in SCALES {
                let got =
                    for_physical_size(physical(w, scale), physical(h, scale), scale, Mode::Overlay);
                assert_eq!(got.page, None, "{w}x{h} at {scale}");
                let bar = got.bar.expect("visible bar");
                assert_eq!((bar.x, bar.y), (0.0, 0.0));
                assert!(close(bar.width, w) && close(bar.height, h), "{got:?}");
            }
        }
    }

    #[test]
    fn fullscreen_hides_the_bar_and_gives_the_page_the_window_at_every_scale() {
        for (w, h) in SIZES {
            for scale in SCALES {
                let got = for_physical_size(
                    physical(w, scale),
                    physical(h, scale),
                    scale,
                    Mode::Fullscreen,
                );
                assert_eq!(got.bar, None, "{w}x{h} at {scale}");
                let page = got.page.expect("visible page");
                assert_eq!((page.x, page.y), (0.0, 0.0));
                assert!(close(page.width, w) && close(page.height, h), "{got:?}");
            }
        }
    }

    #[test]
    fn element_fullscreen_always_wins_over_the_overlay() {
        assert_eq!(Mode::from_state(true, true), Mode::Fullscreen);
        assert_eq!(Mode::from_state(false, true), Mode::Fullscreen);
        assert_eq!(Mode::from_state(true, false), Mode::Overlay);
        assert_eq!(Mode::from_state(false, false), Mode::Toolbar);
    }

    #[test]
    fn narrow_phone_window_at_scale_two() {
        // 390 × 844 logical on a Retina display: the OS reports 780 × 1688.
        let got = for_physical_size(780, 1688, 2.0, Mode::Toolbar);
        assert_eq!(
            got.page,
            Some(Rect {
                x: 0.0,
                y: BAR_HEIGHT,
                width: 390.0,
                height: 804.0
            })
        );
    }

    #[test]
    fn fullscreen_at_scale_one_and_a_half() {
        let got = for_physical_size(2880, 1620, 1.5, Mode::Fullscreen);
        assert_eq!(got.bar, None);
        let page = got.page.expect("visible page");
        assert_eq!((page.width, page.height), (1920.0, 1080.0));
    }

    #[test]
    fn a_bogus_scale_factor_counts_as_one() {
        for mode in [Mode::Toolbar, Mode::Overlay, Mode::Fullscreen] {
            assert_eq!(
                for_physical_size(480, 640, 0.0, mode),
                layout(480.0, 640.0, mode)
            );
            assert_eq!(
                for_physical_size(480, 640, f64::NAN, mode),
                layout(480.0, 640.0, mode)
            );
        }
    }

    #[test]
    fn bar_on_top_page_below() {
        let got = layout(480.0, 640.0, Mode::Toolbar);
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
            Some(Rect {
                x: 0.0,
                y: BAR_HEIGHT,
                width: 480.0,
                height: 640.0 - BAR_HEIGHT
            })
        );
    }

    #[test]
    fn tiny_or_odd_sizes_never_go_negative() {
        let got = layout(100.0, 10.0, Mode::Toolbar);
        assert_eq!(got.bar.map(|b| b.height), Some(10.0));
        assert_eq!(got.page.map(|p| p.height), Some(0.0));
        for mode in [Mode::Toolbar, Mode::Overlay, Mode::Fullscreen] {
            let got = layout(-5.0, -5.0, mode);
            for rect in [got.bar, got.page].into_iter().flatten() {
                assert_eq!((rect.width, rect.height), (0.0, 0.0), "{mode:?}");
            }
        }
    }

    #[test]
    fn bar_and_page_tile_the_window_exactly() {
        for (w, h) in [(320.0, 300.0), (481.5, 777.25), (4096.0, 4096.0)] {
            let got = layout(w, h, Mode::Toolbar);
            let bar = got.bar.expect("visible");
            let page = got.page.expect("visible");
            assert_eq!(bar.height + page.height, h);
            assert_eq!(page.y, bar.height);
            assert_eq!(bar.width, w);
            assert_eq!(page.width, w);
        }
    }
}
