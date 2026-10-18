//! What a new layout must touch (LJ39): only the bounds and the visibility that changed, so a
//! resize sets no more than the two webviews' bounds and shows or hides nothing — on Windows
//! every `show()` was `ShowWindow` + `SetIsVisible`, on Linux `show_all()`, twice per event.
//! And macOS's autoresizing masks per mode, so AppKit resizes the webviews in its own layout pass
//! during a live resize (tao's `Resized` comes a frame late).

use super::{Layout, Mode, Rect};

/// Bounds that differ by less than this (logical px) are the same: every engine rounds to whole
/// pixels, and a layout recomputed from the same window size is the same anyway.
const SAME: f64 = 0.01;

/// What to do to one webview.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewChange {
    /// New bounds to set; `None` leaves them as they are.
    pub bounds: Option<Rect>,
    /// Show (`true`) or hide (`false`) the webview; `None` leaves it as it is.
    pub visible: Option<bool>,
}

/// What to do to the bar and the page.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Changes {
    pub bar: ViewChange,
    pub page: ViewChange,
}

impl Changes {
    /// How many `set_bounds` calls this makes (the smoke log's `bounds-set <n>`).
    pub fn bounds_set(&self) -> usize {
        usize::from(self.bar.bounds.is_some()) + usize::from(self.page.bounds.is_some())
    }

    /// Nothing to do.
    pub fn is_empty(&self) -> bool {
        *self == Changes::default()
    }
}

fn same(a: Rect, b: Rect) -> bool {
    (a.x - b.x).abs() < SAME
        && (a.y - b.y).abs() < SAME
        && (a.width - b.width).abs() < SAME
        && (a.height - b.height).abs() < SAME
}

/// One webview: `previous` is `None` when nothing is known (a new window: everything is applied),
/// `Some(None)` when it was hidden.
fn view_change(previous: Option<Option<Rect>>, next: Option<Rect>) -> ViewChange {
    match (previous, next) {
        (None | Some(None), Some(rect)) => ViewChange {
            bounds: Some(rect),
            visible: Some(true),
        },
        (None | Some(Some(_)), None) => ViewChange {
            bounds: None,
            visible: Some(false),
        },
        (Some(None), None) => ViewChange::default(),
        (Some(Some(old)), Some(rect)) => ViewChange {
            bounds: (!same(old, rect)).then_some(rect),
            visible: None,
        },
    }
}

/// What going from `previous` (the layout last applied; `None` for a new window, or when it isn't
/// known) to `next` changes: the bar's and the page's bounds, and their visibility. A view that
/// was hidden and is shown again gets its bounds too (they may be stale).
pub fn changes(previous: Option<&Layout>, next: &Layout) -> Changes {
    Changes {
        bar: view_change(previous.map(|p| p.bar), next.bar),
        page: view_change(previous.map(|p| p.page), next.page),
    }
}

/// AppKit's autoresizing mask bits (`NSAutoresizingMaskOptions`) that Little Joan uses.
pub mod mask {
    /// `NSViewWidthSizable`: the view's width follows its superview's.
    pub const WIDTH_SIZABLE: usize = 2;
    /// `NSViewMinYMargin`: the margin at the superview's minimum y is flexible — the bottom one in
    /// AppKit's default (unflipped) coordinates, so the view keeps its place at the top.
    pub const MIN_Y_MARGIN: usize = 8;
    /// `NSViewHeightSizable`: the view's height follows its superview's.
    pub const HEIGHT_SIZABLE: usize = 16;
    /// `NSViewMaxYMargin`: the margin at the superview's maximum y is flexible (the top one,
    /// unflipped).
    pub const MAX_Y_MARGIN: usize = 32;
}

/// The bar's and the page's autoresizing masks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Masks {
    pub bar: usize,
    pub page: usize,
}

/// The macOS autoresizing masks for `mode`, for a superview in AppKit's default (unflipped)
/// coordinates — see `flipped`. Toolbar: the bar follows the width and keeps its 40 px at the top
/// (a flexible bottom margin), the page follows both. Overlay: the bar follows both, and so does
/// the hidden page, so it is the right size when it shows again. Fullscreen: `None`, nothing
/// changes (macOS never uses that mode: WebKit shows the element in a window of its own, and the
/// page is left alone meanwhile).
pub fn mask_for(mode: Mode) -> Option<Masks> {
    let both = mask::WIDTH_SIZABLE | mask::HEIGHT_SIZABLE;
    match mode {
        Mode::Toolbar => Some(Masks {
            bar: mask::WIDTH_SIZABLE | mask::MIN_Y_MARGIN,
            page: both,
        }),
        Mode::Overlay => Some(Masks {
            bar: both,
            page: both,
        }),
        Mode::Fullscreen => None,
    }
}

/// The same behaviour in a flipped superview (y grows downwards): its minimum y is the top, so
/// the two vertical margins swap.
pub fn flipped(bits: usize) -> usize {
    let vertical = mask::MIN_Y_MARGIN | mask::MAX_Y_MARGIN;
    let mut out = bits & !vertical;
    if bits & mask::MIN_Y_MARGIN != 0 {
        out |= mask::MAX_Y_MARGIN;
    }
    if bits & mask::MAX_Y_MARGIN != 0 {
        out |= mask::MIN_Y_MARGIN;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::{for_physical_size, layout};
    use super::*;

    const MODES: [Mode; 3] = [Mode::Toolbar, Mode::Overlay, Mode::Fullscreen];
    const SCALES: [f64; 3] = [1.0, 1.5, 2.0];

    fn at(w: f64, h: f64, scale: f64, mode: Mode) -> Layout {
        let px = |v: f64| (v * scale).round() as u32;
        for_physical_size(px(w), px(h), scale, mode)
    }

    #[test]
    fn nothing_changed_nothing_to_do_at_every_scale_in_every_mode() {
        for mode in MODES {
            for scale in SCALES {
                let now = at(480.0, 640.0, scale, mode);
                let got = changes(Some(&now), &now);
                assert!(got.is_empty(), "{mode:?} at {scale}: {got:?}");
                assert_eq!(got.bounds_set(), 0);
            }
        }
    }

    #[test]
    fn a_new_window_gets_everything() {
        for mode in MODES {
            for scale in SCALES {
                let next = at(480.0, 640.0, scale, mode);
                let got = changes(None, &next);
                for (change, rect) in [(got.bar, next.bar), (got.page, next.page)] {
                    assert_eq!(change.bounds, rect, "{mode:?} at {scale}");
                    assert_eq!(change.visible, Some(rect.is_some()), "{mode:?} at {scale}");
                }
            }
        }
    }

    #[test]
    fn a_resize_sets_only_the_bounds_that_changed_and_shows_nothing() {
        for scale in SCALES {
            // Wider: both follow (the bar's width, the page's width).
            let got = changes(
                Some(&at(480.0, 640.0, scale, Mode::Toolbar)),
                &at(520.0, 640.0, scale, Mode::Toolbar),
            );
            assert_eq!(got.bounds_set(), 2, "at {scale}: {got:?}");
            assert_eq!((got.bar.visible, got.page.visible), (None, None));
            // Taller only: the 40 px bar stays as it is, the page grows.
            let got = changes(
                Some(&at(480.0, 640.0, scale, Mode::Toolbar)),
                &at(480.0, 700.0, scale, Mode::Toolbar),
            );
            assert_eq!(got.bar, ViewChange::default(), "at {scale}");
            assert!(got.page.bounds.is_some() && got.page.visible.is_none());
            // The overlay: only the bar (the page stays hidden, untouched).
            let got = changes(
                Some(&at(480.0, 640.0, scale, Mode::Overlay)),
                &at(500.0, 660.0, scale, Mode::Overlay),
            );
            assert_eq!(got.bounds_set(), 1, "at {scale}");
            assert_eq!(got.page, ViewChange::default());
            // Element fullscreen: only the page; the hidden bar stays hidden.
            let got = changes(
                Some(&at(480.0, 640.0, scale, Mode::Fullscreen)),
                &at(1280.0, 800.0, scale, Mode::Fullscreen),
            );
            assert_eq!(got.bar, ViewChange::default(), "at {scale}");
            assert_eq!(got.page.visible, None);
        }
    }

    #[test]
    fn mode_changes_show_and_hide_what_they_must() {
        let toolbar = layout(480.0, 640.0, Mode::Toolbar);
        let overlay = layout(480.0, 640.0, Mode::Overlay);
        let fullscreen = layout(480.0, 640.0, Mode::Fullscreen);
        // The command bar opens: the bar covers the window, the page hides.
        let got = changes(Some(&toolbar), &overlay);
        assert_eq!(got.bar.bounds, overlay.bar);
        assert_eq!(got.bar.visible, None);
        assert_eq!(
            got.page,
            ViewChange {
                bounds: None,
                visible: Some(false)
            }
        );
        // It closes: the page is shown again, with its bounds.
        let got = changes(Some(&overlay), &toolbar);
        assert_eq!(
            got.page,
            ViewChange {
                bounds: toolbar.page,
                visible: Some(true)
            }
        );
        // Element fullscreen: the bar hides, the page takes the window.
        let got = changes(Some(&toolbar), &fullscreen);
        assert_eq!(got.bar.visible, Some(false));
        assert_eq!(got.page.bounds, fullscreen.page);
        assert_eq!(got.page.visible, None);
        // And back: the bar shows again with its bounds.
        let got = changes(Some(&fullscreen), &toolbar);
        assert_eq!(got.bar.visible, Some(true));
        assert_eq!(got.bar.bounds, toolbar.bar);
    }

    #[test]
    fn rounding_noise_is_no_change() {
        let now = layout(480.0, 640.0, Mode::Toolbar);
        let mut noisy = now;
        if let Some(page) = noisy.page.as_mut() {
            page.width += 0.001;
        }
        assert!(changes(Some(&now), &noisy).is_empty());
        if let Some(page) = noisy.page.as_mut() {
            page.width += 0.5;
        }
        assert_eq!(changes(Some(&now), &noisy).bounds_set(), 1);
    }

    #[test]
    fn a_drag_sets_at_most_two_bounds_per_event() {
        for scale in SCALES {
            let mut previous: Option<Layout> = None;
            for step in 0..=24 {
                let width = 380.0 + f64::from(step) * 20.0;
                let next = at(width, 600.0, scale, Mode::Toolbar);
                let got = changes(previous.as_ref(), &next);
                assert!(got.bounds_set() <= 2, "{got:?}");
                if previous.is_some() {
                    assert_eq!((got.bar.visible, got.page.visible), (None, None));
                }
                previous = Some(next);
            }
        }
    }

    #[test]
    fn the_masks_per_mode() {
        let toolbar = mask_for(Mode::Toolbar).expect("toolbar");
        assert_eq!(toolbar.bar, mask::WIDTH_SIZABLE | mask::MIN_Y_MARGIN);
        assert_eq!(toolbar.page, mask::WIDTH_SIZABLE | mask::HEIGHT_SIZABLE);
        let overlay = mask_for(Mode::Overlay).expect("overlay");
        assert_eq!(overlay.bar, mask::WIDTH_SIZABLE | mask::HEIGHT_SIZABLE);
        assert_eq!(overlay.page, mask::WIDTH_SIZABLE | mask::HEIGHT_SIZABLE);
        assert_eq!(mask_for(Mode::Fullscreen), None, "nothing changes");
        // AppKit's values (NSAutoresizingMaskOptions).
        assert_eq!(
            (
                mask::WIDTH_SIZABLE,
                mask::MIN_Y_MARGIN,
                mask::HEIGHT_SIZABLE,
                mask::MAX_Y_MARGIN
            ),
            (2, 8, 16, 32)
        );
    }

    #[test]
    fn a_flipped_superview_swaps_the_vertical_margins() {
        assert_eq!(
            flipped(mask::WIDTH_SIZABLE | mask::MIN_Y_MARGIN),
            mask::WIDTH_SIZABLE | mask::MAX_Y_MARGIN
        );
        assert_eq!(flipped(mask::MAX_Y_MARGIN), mask::MIN_Y_MARGIN);
        let both = mask::WIDTH_SIZABLE | mask::HEIGHT_SIZABLE;
        assert_eq!(flipped(both), both);
        assert_eq!(
            flipped(flipped(mask::MIN_Y_MARGIN | 1)),
            mask::MIN_Y_MARGIN | 1
        );
    }
}
