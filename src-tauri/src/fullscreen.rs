//! HTML5 element fullscreen (a video's fullscreen button) → the window goes fullscreen.
//!
//! Per engine:
//! - **WebView2 (Windows):** the engine only fills the webview's bounds; the app must make the
//!   window fullscreen when `ContainsFullScreenElementChanged` fires (platform/win.rs uses this
//!   tracker). wry 0.57 does not do it.
//! - **WKWebView (macOS):** wry enables `elementFullscreenEnabled`; WebKit moves the element into
//!   its own fullscreen window. Nothing to do here.
//! - **WebKitGTK (Linux):** the default `enter-fullscreen` handler fullscreens the toplevel GTK
//!   window (`enable-fullscreen` defaults to true). Nothing to do here.

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
