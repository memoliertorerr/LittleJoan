//! HTML5 element fullscreen (a video's fullscreen button) → the window goes fullscreen.
//!
//! Per engine:
//! - **WebView2 (Windows):** the engine only fills the webview's bounds; the app must make the
//!   window fullscreen when `ContainsFullScreenElementChanged` fires (platform/win.rs uses this
//!   tracker). wry 0.57 does not do it.
//! - **WKWebView (macOS):** wry enables `elementFullscreenEnabled`; WebKit moves the element into
//!   its own fullscreen window. The app only watches it (`WebKitFullscreen`, platform/mac.rs).
//! - **WebKitGTK (Linux):** the default `enter-fullscreen` handler fullscreens the toplevel GTK
//!   window (`enable-fullscreen` defaults to true). Older than 2.52 (Ubuntu 22.04's 2.50.4) the
//!   process aborts as a page enters element fullscreen — even when `enter-fullscreen` declines it
//!   (LJ14) —, so there platform/linux.rs switches `enable-fullscreen` off (`needs_fullscreen_off`):
//!   a page's request is refused and nothing crashes (LJ14.1).

/// Remembers whether the window went fullscreen because of the page, so leaving element
/// fullscreen never undoes a fullscreen the user chose.
#[derive(Debug, Default)]
pub struct FullscreenTracker {
    entered_by_element: bool,
}

impl FullscreenTracker {
    /// Called when the page gains or loses its fullscreen element. Returns the fullscreen state
    /// to apply to the window, or `None` to leave it alone.
    pub fn on_element_change(
        &mut self,
        contains: bool,
        window_is_fullscreen: bool,
    ) -> Option<bool> {
        if contains {
            if window_is_fullscreen {
                None
            } else {
                self.entered_by_element = true;
                Some(true)
            }
        } else if self.entered_by_element {
            self.entered_by_element = false;
            Some(false)
        } else {
            None
        }
    }
}

/// WebKitGTK older than 2.52 aborts the process when a page enters element fullscreen (2.50.4 on
/// Ubuntu 22.04; the plan's Landmines), whatever the app answers to `enter-fullscreen`, so element
/// fullscreen is switched off there: the page's request is refused, nothing goes fullscreen.
pub fn needs_fullscreen_off(major: u32, minor: u32) -> bool {
    (major, minor) < (2, 52)
}

/// WKWebView's `fullscreenState` (macOS 13+): where WebKit's element fullscreen is. During it
/// WebKit has moved the page's view into a fullscreen window of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebKitFullscreen {
    Not,
    Entering,
    In,
    Exiting,
}

impl WebKitFullscreen {
    /// From the `WKFullscreenState` value (an NSInteger: 0–3). A value this code doesn't know
    /// counts as fullscreen.
    pub fn from_raw(raw: isize) -> Self {
        match raw {
            0 => WebKitFullscreen::Not,
            1 => WebKitFullscreen::Entering,
            3 => WebKitFullscreen::Exiting,
            _ => WebKitFullscreen::In,
        }
    }

    /// The smoke log's name (`wk-fullscreen <name>`).
    pub fn name(self) -> &'static str {
        match self {
            WebKitFullscreen::Not => "not",
            WebKitFullscreen::Entering => "entering",
            WebKitFullscreen::In => "in",
            WebKitFullscreen::Exiting => "exiting",
        }
    }
}

/// Whether the app may lay out the page's WKWebView (set its bounds, show or hide it): only while
/// WebKit's element fullscreen is off — `webkit_state` is `fullscreenState` (`None` on macOS 12,
/// which lacks it) — and the view sits in the Little Joan window, where WebKit's fullscreen takes
/// it from (`in_own_window`, the only sign on macOS 12).
pub fn page_layout_allowed(webkit_state: Option<WebKitFullscreen>, in_own_window: bool) -> bool {
    in_own_window && webkit_state.is_none_or(|state| state == WebKitFullscreen::Not)
}

/// What macOS says while a summon waits for WebKit's element fullscreen to end (LJ19.5):
/// WKWebView's `fullscreenState` (`None` on macOS 12), whether the page's view is back in the
/// Little Joan window, and whether that window is on the Space on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebKitExit {
    pub state: Option<WebKitFullscreen>,
    pub in_window: bool,
    pub on_screen: bool,
}

/// A summon waiting for WebKit's element fullscreen to end (LJ19.5): whether it has — the page
/// back in the Little Joan window (`page_layout_allowed`) and that window on the Space on screen
/// (`on_screen`: WebKit's full-screen Space has closed), so ordering the window front can't put it
/// into WebKit's Space (LJ19.4-fix4).
pub fn webkit_fullscreen_over(
    webkit_state: Option<WebKitFullscreen>,
    in_own_window: bool,
    on_screen: bool,
) -> bool {
    page_layout_allowed(webkit_state, in_own_window) && on_screen
}

/// Whether WebKit hasn't begun leaving its element fullscreen — still entering or in it (on macOS
/// 12, which lacks `fullscreenState`: the page still outside the window). A summon that asked the
/// page to leave asks WebKit itself after a while (joan/fullscreen.rs).
pub fn webkit_still_in(webkit_state: Option<WebKitFullscreen>, in_own_window: bool) -> bool {
    match webkit_state {
        Some(WebKitFullscreen::Entering | WebKitFullscreen::In) => true,
        Some(WebKitFullscreen::Exiting | WebKitFullscreen::Not) => false,
        None => !in_own_window,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_summon_waits_for_the_page_back_in_the_window_on_screen() {
        use WebKitFullscreen::{Entering, Exiting, In, Not};
        assert!(webkit_fullscreen_over(Some(Not), true, true));
        // WebKit still has the page, or its full-screen Space is still the one on screen.
        for state in [Entering, In, Exiting] {
            assert!(
                !webkit_fullscreen_over(Some(state), true, true),
                "{state:?}"
            );
        }
        assert!(!webkit_fullscreen_over(Some(Not), false, true));
        assert!(!webkit_fullscreen_over(Some(Not), true, false));
        // macOS 12: the view's window tells.
        assert!(webkit_fullscreen_over(None, true, true));
        assert!(!webkit_fullscreen_over(None, false, true));
    }

    #[test]
    fn webkit_is_asked_itself_only_while_it_hasnt_begun_leaving() {
        use WebKitFullscreen::{Entering, Exiting, In, Not};
        assert!(webkit_still_in(Some(Entering), false));
        assert!(webkit_still_in(Some(In), false));
        assert!(!webkit_still_in(Some(Exiting), false));
        assert!(!webkit_still_in(Some(Not), true));
        assert!(webkit_still_in(None, false));
        assert!(!webkit_still_in(None, true));
    }

    #[test]
    fn element_fullscreen_round_trip() {
        let mut tracker = FullscreenTracker::default();
        assert_eq!(tracker.on_element_change(true, false), Some(true));
        assert_eq!(tracker.on_element_change(false, true), Some(false));
        // A stray exit afterwards changes nothing.
        assert_eq!(tracker.on_element_change(false, false), None);
    }

    #[test]
    fn user_fullscreen_survives_an_element_exit() {
        let mut tracker = FullscreenTracker::default();
        assert_eq!(tracker.on_element_change(true, true), None);
        assert_eq!(tracker.on_element_change(false, true), None);
    }

    #[test]
    fn repeated_enters_need_one_exit() {
        let mut tracker = FullscreenTracker::default();
        assert_eq!(tracker.on_element_change(true, false), Some(true));
        // The page swaps its fullscreen element; the window is already fullscreen.
        assert_eq!(tracker.on_element_change(true, true), None);
        assert_eq!(tracker.on_element_change(false, true), Some(false));
    }

    #[test]
    fn webkitgtk_older_than_2_52_has_fullscreen_off() {
        assert!(needs_fullscreen_off(2, 44));
        assert!(needs_fullscreen_off(2, 50));
        assert!(needs_fullscreen_off(2, 51));
        assert!(needs_fullscreen_off(1, 99));
        assert!(!needs_fullscreen_off(2, 52));
        assert!(!needs_fullscreen_off(2, 54));
        assert!(!needs_fullscreen_off(3, 0));
    }

    #[test]
    fn webkit_states_by_value() {
        assert_eq!(WebKitFullscreen::from_raw(0), WebKitFullscreen::Not);
        assert_eq!(WebKitFullscreen::from_raw(1), WebKitFullscreen::Entering);
        assert_eq!(WebKitFullscreen::from_raw(2), WebKitFullscreen::In);
        assert_eq!(WebKitFullscreen::from_raw(3), WebKitFullscreen::Exiting);
        // Unknown values: fullscreen, the careful answer.
        assert_eq!(WebKitFullscreen::from_raw(7), WebKitFullscreen::In);
        assert_eq!(WebKitFullscreen::from_raw(-1), WebKitFullscreen::In);
        let names: Vec<_> = [0, 1, 2, 3]
            .map(|raw| WebKitFullscreen::from_raw(raw).name())
            .into();
        assert_eq!(names, ["not", "entering", "in", "exiting"]);
    }

    #[test]
    fn the_page_is_laid_out_only_outside_webkits_fullscreen() {
        use WebKitFullscreen::{Entering, Exiting, In, Not};
        assert!(page_layout_allowed(Some(Not), true));
        for state in [Entering, In, Exiting] {
            assert!(!page_layout_allowed(Some(state), true), "{state:?}");
        }
        // macOS 12: no fullscreenState; the view's window tells.
        assert!(page_layout_allowed(None, true));
        assert!(!page_layout_allowed(None, false));
        // Outside the window even with the state saying "not": leave it alone.
        assert!(!page_layout_allowed(Some(Not), false));
    }
}
