//! How much an extension may ask of the app at once (pure, tested): at most 8 API calls in
//! flight, at most 20 `storage.set` calls a second, at most one `openUrl` to the browser every
//! 2 s. Over a limit the call is answered 429 and does nothing.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// API calls one extension may have in flight at once.
pub const MAX_IN_FLIGHT: usize = 8;

/// At most `max` events per key within any `window`.
#[derive(Debug)]
pub struct RateLimit {
    window: Duration,
    max: usize,
    hits: HashMap<String, VecDeque<Instant>>,
}

impl RateLimit {
    pub fn new(window: Duration, max: usize) -> Self {
        RateLimit {
            window,
            max,
            hits: HashMap::new(),
        }
    }

    /// Counts an event for `key` at `now` if it is within the limit; returns whether it was.
    pub fn allow(&mut self, key: &str, now: Instant) -> bool {
        let hits = self.hits.entry(key.to_owned()).or_default();
        while hits
            .front()
            .is_some_and(|at| now.saturating_duration_since(*at) >= self.window)
        {
            hits.pop_front();
        }
        if hits.len() >= self.max {
            return false;
        }
        hits.push_back(now);
        true
    }
}

/// Calls in flight per key, at most `max`.
#[derive(Debug, Default)]
pub struct InFlight {
    counts: HashMap<String, usize>,
}

impl InFlight {
    /// Starts a call for `key` if fewer than `max` are running; returns whether it did.
    pub fn start(&mut self, key: &str, max: usize) -> bool {
        let count = self.counts.entry(key.to_owned()).or_default();
        if *count >= max {
            return false;
        }
        *count += 1;
        true
    }

    /// A call for `key` ended.
    pub fn end(&mut self, key: &str) {
        if let Some(count) = self.counts.get_mut(key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.counts.remove(key);
            }
        }
    }
}

static IN_FLIGHT: Mutex<Option<InFlight>> = Mutex::new(None);
static STORAGE_SETS: Mutex<Option<RateLimit>> = Mutex::new(None);
static BROWSER_OPENS: Mutex<Option<RateLimit>> = Mutex::new(None);

/// One API call of an extension, counted while it lives (`Call::start` gives `None` over the limit).
pub struct Call {
    id: String,
}

impl Call {
    pub fn start(id: &str) -> Option<Call> {
        let mut in_flight = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
        in_flight
            .get_or_insert_with(InFlight::default)
            .start(id, MAX_IN_FLIGHT)
            .then(|| Call { id: id.to_owned() })
    }
}

impl Drop for Call {
    fn drop(&mut self) {
        let mut in_flight = IN_FLIGHT.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(in_flight) = in_flight.as_mut() {
            in_flight.end(&self.id);
        }
    }
}

/// Whether the extension `id` may call `storage.set` now (20 a second).
pub fn storage_set(id: &str) -> bool {
    STORAGE_SETS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert_with(|| RateLimit::new(Duration::from_secs(1), 20))
        .allow(id, Instant::now())
}

/// Whether the extension `id` may hand a page to the default browser now (one every 2 s).
pub fn browser_open(id: &str) -> bool {
    BROWSER_OPENS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert_with(|| RateLimit::new(Duration::from_secs(2), 1))
        .allow(id, Instant::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rate_limit_counts_per_key_within_its_window() {
        let start = Instant::now();
        let mut limit = RateLimit::new(Duration::from_secs(1), 20);
        for _ in 0..20 {
            assert!(limit.allow("hello", start));
        }
        assert!(!limit.allow("hello", start), "the 21st within the second");
        assert!(limit.allow("other", start), "each extension has its own");
        assert!(!limit.allow("hello", start + Duration::from_millis(999)));
        assert!(
            limit.allow("hello", start + Duration::from_secs(1)),
            "the window moved on"
        );
        let mut browser = RateLimit::new(Duration::from_secs(2), 1);
        assert!(browser.allow("hello", start));
        assert!(!browser.allow("hello", start + Duration::from_millis(1999)));
        assert!(browser.allow("hello", start + Duration::from_secs(2)));
    }

    #[test]
    fn calls_in_flight_are_capped_per_extension() {
        let mut in_flight = InFlight::default();
        for _ in 0..MAX_IN_FLIGHT {
            assert!(in_flight.start("hello", MAX_IN_FLIGHT));
        }
        assert!(!in_flight.start("hello", MAX_IN_FLIGHT));
        assert!(in_flight.start("other", MAX_IN_FLIGHT));
        in_flight.end("hello");
        assert!(in_flight.start("hello", MAX_IN_FLIGHT));
        in_flight.end("nobody");
    }

    #[test]
    fn a_call_ends_when_dropped() {
        let calls: Vec<Call> = (0..MAX_IN_FLIGHT)
            .map(|_| Call::start("limits-test").expect("under the cap"))
            .collect();
        assert!(Call::start("limits-test").is_none());
        drop(calls);
        assert!(Call::start("limits-test").is_some());
    }
}
