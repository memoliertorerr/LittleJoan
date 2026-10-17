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

#[cfg(test)]
mod tests {
    use super::*;

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
