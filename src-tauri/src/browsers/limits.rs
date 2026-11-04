//! How often pages may be handed on, and how long the OS's browser list is reused (phase 21's
//! security review; pure, tested below). A web page can ask for new windows as fast as its script
//! runs — WKWebView lets scripts open them, WebView2 doesn't say whether a click asked — so the
//! page's and Settings' handoffs go through one gate: at most one under way, at most one a second;
//! the rest are dropped (`handoff-throttled`). The OS's list is read again only after LIST_TTL.

use std::time::{Duration, Instant};

use crate::extensions::limits::RateLimit;

/// At most this many new-window handoffs in any second.
pub const PER_SECOND: usize = 1;
/// How long a read of the OS's browser list is reused.
pub const LIST_TTL: Duration = Duration::from_secs(2);

/// The gate for new-window handoffs: one under way, PER_SECOND a second.
#[derive(Debug)]
pub struct HandoffGate {
    rate: RateLimit,
    busy: bool,
}

impl Default for HandoffGate {
    fn default() -> Self {
        HandoffGate {
            rate: RateLimit::new(Duration::from_secs(1), PER_SECOND),
            busy: false,
        }
    }
}

impl HandoffGate {
    /// Starts a handoff at `now` if none is under way and the rate allows it.
    pub fn try_start(&mut self, now: Instant) -> bool {
        if self.busy || !self.rate.allow("new-window", now) {
            return false;
        }
        self.busy = true;
        true
    }

    /// The handoff under way ended (opened, nowhere, refused or failed).
    pub fn end(&mut self) {
        self.busy = false;
    }
}

/// Whether a list read at `read_at` may still be used at `now`.
pub fn fresh(read_at: Instant, now: Instant) -> bool {
    now.saturating_duration_since(read_at) < LIST_TTL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_under_way_and_one_a_second() {
        let start = Instant::now();
        let mut gate = HandoffGate::default();
        assert!(gate.try_start(start));
        assert!(
            !gate.try_start(start + Duration::from_millis(1500)),
            "one under way"
        );
        gate.end();
        assert!(gate.try_start(start + Duration::from_millis(1500)));
        gate.end();
        assert!(
            !gate.try_start(start + Duration::from_millis(1900)),
            "one a second"
        );
        assert!(gate.try_start(start + Duration::from_millis(2600)));
    }

    #[test]
    fn a_flood_gets_one_handoff_a_second() {
        let start = Instant::now();
        let mut gate = HandoffGate::default();
        let mut started = 0;
        // 200 requests over 3 s, each ending at once.
        for n in 0..200u64 {
            if gate.try_start(start + Duration::from_millis(n * 15)) {
                started += 1;
                gate.end();
            }
        }
        assert_eq!(started, 3);
    }

    #[test]
    fn the_list_is_reused_for_two_seconds() {
        let read_at = Instant::now();
        assert!(fresh(read_at, read_at));
        assert!(fresh(read_at, read_at + Duration::from_millis(1999)));
        assert!(!fresh(read_at, read_at + LIST_TTL));
    }
}
