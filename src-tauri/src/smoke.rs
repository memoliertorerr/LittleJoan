//! An event log for automated smoke tests, written only when `LITTLEJOAN_SMOKE_LOG` names a
//! file. Normal use never sets it, so nothing about browsing is ever logged.
//!
//! Line format: `<unix millis> <event>[ <detail>]`, e.g. `1760620000000 title Example Domain`.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

pub const SMOKE_LOG_ENV: &str = "LITTLEJOAN_SMOKE_LOG";

static PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn path() -> Option<&'static PathBuf> {
    PATH.get_or_init(|| {
        std::env::var_os(SMOKE_LOG_ENV)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    })
    .as_ref()
}

/// Whether smoke logging is on for this process.
pub fn enabled() -> bool {
    path().is_some()
}

/// Appends one event line, or does nothing when smoke logging is off. Failures are ignored:
/// the log must never affect the app.
pub fn log(event: &str, detail: &str) {
    let Some(path) = path() else {
        return;
    };
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(format_line(millis, event, detail).as_bytes());
    }
}

/// One log line. Control characters in `detail` (newlines in page titles) become spaces.
pub fn format_line(millis: u128, event: &str, detail: &str) -> String {
    let detail: String = detail
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    match detail.trim() {
        "" => format!("{millis} {event}\n"),
        detail => format!("{millis} {event} {detail}\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_have_time_event_and_optional_detail() {
        assert_eq!(format_line(5, "started", ""), "5 started\n");
        assert_eq!(
            format_line(7, "title", "Example Domain"),
            "7 title Example Domain\n"
        );
    }

    #[test]
    fn details_stay_on_one_line() {
        assert_eq!(format_line(1, "title", " a\nb\tc\r\n"), "1 title a b c\n");
        assert_eq!(format_line(1, "title", "\n\n"), "1 title\n");
    }
}
