//! Phone mode: in a narrow window the page identifies as a phone browser, so sites that pick
//! their mobile version by user agent (Google, YouTube, …) serve it. Pure decisions; the glue
//! is phone_mode.rs and the per-OS setters in platform/.
//!
//! - **Threshold: 600 logical px of page width.** Phone layouts are built for ~360–430 CSS px;
//!   600 is where "compact" width ends in Material Design and where Android itself stops
//!   treating a screen as a phone, so anything narrower is clearly phone-sized, while Little
//!   Joan's default 480 px window counts as a phone and a half-screen window does not.
//! - **Which phone:** the one whose browser shares the page's engine, so the mobile pages that
//!   come back are the ones built for that engine. WKWebView (macOS) and WebKitGTK (Linux) are
//!   WebKit → iPhone Safari. WebView2 (Windows) is Chromium → Chrome on Android, with the same
//!   Chrome major version as the installed WebView2.

use std::time::{Duration, Instant};

use crate::keys::Platform;

/// Pages narrower than this many logical pixels get the phone user agent.
pub const PHONE_WIDTH: f64 = 600.0;

/// How long a resize must have stopped before the user agent may switch (and the page reload).
pub const SETTLE: Duration = Duration::from_millis(400);

/// iPhone Safari on iOS 26 (Safari reports a frozen "18_6" OS version from iOS 26 on).
pub const IPHONE_SAFARI: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) \
AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Mobile/15E148 Safari/604.1";

/// Chrome major version used when WebView2's own version can't be read.
pub const FALLBACK_CHROME_MAJOR: u32 = 141;

/// Whether a page `width` logical pixels wide should look like a phone to websites.
pub fn wants_mobile(enabled: bool, width: f64) -> bool {
    enabled && width.is_finite() && width < PHONE_WIDTH
}

/// The phone user agent for `platform`. `engine_ua` is the engine's own (desktop) user agent;
/// on Windows its Chrome version is reused so the phone browser matches the engine.
pub fn mobile_user_agent(platform: Platform, engine_ua: Option<&str>) -> String {
    match platform {
        Platform::Windows => {
            let major = engine_ua
                .and_then(chrome_major)
                .unwrap_or(FALLBACK_CHROME_MAJOR);
            // Chrome's reduced user agent on Android (the device model is always "K").
            format!(
                "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/{major}.0.0.0 Mobile Safari/537.36"
            )
        }
        Platform::MacOs | Platform::Linux => IPHONE_SAFARI.to_owned(),
    }
}

/// The major version in a `Chrome/<major>.…` token, if any.
pub fn chrome_major(user_agent: &str) -> Option<u32> {
    let rest = &user_agent[user_agent.find("Chrome/")? + "Chrome/".len()..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok().filter(|major| *major > 0)
}

/// What to do after a (settled) size change: `Some(mobile)` to switch the user agent, `None`
/// when the page already has the right one.
pub fn switch(applied: Option<bool>, wanted: bool) -> Option<bool> {
    (applied != Some(wanted)).then_some(wanted)
}

/// Waits for a burst of events (a window being dragged to a new size) to settle: `ready`
/// answers true once, when `quiet` has passed since the last `touch`.
#[derive(Debug, Clone)]
pub struct Debounce {
    quiet: Duration,
    last: Option<Instant>,
}

impl Debounce {
    pub const fn new(quiet: Duration) -> Self {
        Debounce { quiet, last: None }
    }

    /// An event happened at `now`.
    pub fn touch(&mut self, now: Instant) {
        self.last = Some(now);
    }

    /// True once the events have been quiet for long enough; then resets.
    pub fn ready(&mut self, now: Instant) -> bool {
        match self.last {
            Some(last) if now.saturating_duration_since(last) >= self.quiet => {
                self.last = None;
                true
            }
            _ => false,
        }
    }

    /// Whether an event is waiting to settle.
    pub fn pending(&self) -> bool {
        self.last.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrow_pages_are_phones_wide_ones_are_not() {
        assert!(wants_mobile(true, 390.0));
        assert!(wants_mobile(true, 480.0));
        assert!(wants_mobile(true, 599.9));
        assert!(!wants_mobile(true, 600.0));
        assert!(!wants_mobile(true, 1280.0));
        assert!(!wants_mobile(true, f64::NAN));
    }

    #[test]
    fn the_setting_turns_phone_mode_off() {
        assert!(!wants_mobile(false, 390.0));
    }

    #[test]
    fn webkit_platforms_look_like_an_iphone() {
        for platform in [Platform::MacOs, Platform::Linux] {
            let ua = mobile_user_agent(
                platform,
                Some("Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15"),
            );
            assert!(
                ua.contains("iPhone") && ua.contains("Mobile/") && ua.contains("Safari/"),
                "{ua}"
            );
        }
    }

    #[test]
    fn windows_looks_like_chrome_on_android_with_the_engines_version() {
        let webview2 = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
                        Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0";
        let ua = mobile_user_agent(Platform::Windows, Some(webview2));
        assert_eq!(
            ua,
            "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/142.0.0.0 Mobile Safari/537.36"
        );
        let fallback = mobile_user_agent(Platform::Windows, None);
        assert!(fallback.contains(&format!("Chrome/{FALLBACK_CHROME_MAJOR}.0.0.0 Mobile")));
    }

    #[test]
    fn every_phone_user_agent_says_mobile() {
        // Sites (and the CI smoke page) recognise phones by the "Mobile" token.
        for platform in [Platform::Windows, Platform::MacOs, Platform::Linux] {
            assert!(mobile_user_agent(platform, None).contains("Mobile"));
        }
    }

    #[test]
    fn chrome_versions() {
        assert_eq!(chrome_major("x Chrome/131.0.6778.86 y"), Some(131));
        assert_eq!(chrome_major("Safari/605.1.15"), None);
        assert_eq!(chrome_major("Chrome/"), None);
        assert_eq!(chrome_major("Chrome/0.1"), None);
    }

    #[test]
    fn switch_only_on_change() {
        assert_eq!(switch(None, true), Some(true));
        assert_eq!(switch(None, false), Some(false));
        assert_eq!(switch(Some(true), true), None);
        assert_eq!(switch(Some(true), false), Some(false));
        assert_eq!(switch(Some(false), true), Some(true));
    }

    #[test]
    fn debounce_waits_for_the_drag_to_stop() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut debounce = Debounce::new(SETTLE);
        assert!(!debounce.ready(t0), "nothing happened yet");
        debounce.touch(t0);
        assert!(debounce.pending());
        assert!(!debounce.ready(t0 + ms(100)));
        // Still dragging: every new event restarts the wait.
        debounce.touch(t0 + ms(300));
        assert!(!debounce.ready(t0 + ms(500)));
        debounce.touch(t0 + ms(600));
        assert!(!debounce.ready(t0 + ms(999)));
        assert!(debounce.ready(t0 + ms(1000)), "400 ms after the last event");
        assert!(!debounce.ready(t0 + ms(2000)), "fires once per burst");
        assert!(!debounce.pending());
    }
}
