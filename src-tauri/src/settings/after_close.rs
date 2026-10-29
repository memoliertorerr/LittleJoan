//! "After closing" (LJ19.4, the owner's decision of 2026-10-29): what stays at rest once the window
//! closes. Closing always discards the page — no web page's scripts, media, network or process
//! survive a close in any mode —; this says only whether the hidden window and the trusted bar stay.
//! Pure: no Tauri types.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// What stays at rest after the window closes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AfterClose {
    /// The hidden window and the bar stay (the default): the next summon is instant.
    #[default]
    Ready,
    /// As `Ready` for `READY_FOR` after the close, then nothing (hibernation).
    #[serde(rename = "ready15")]
    Ready15,
    /// Nothing stays: no window, no webview (hibernation, as before LJ19.4).
    Hibernate,
}

/// How long `Ready15` keeps the window and the bar after a close.
pub const READY_FOR: Duration = Duration::from_secs(15 * 60);

impl AfterClose {
    pub const ALL: [AfterClose; 3] = [
        AfterClose::Ready,
        AfterClose::Ready15,
        AfterClose::Hibernate,
    ];

    /// Its name in settings.json and in the smoke log.
    pub fn name(self) -> &'static str {
        match self {
            AfterClose::Ready => "ready",
            AfterClose::Ready15 => "ready15",
            AfterClose::Hibernate => "hibernate",
        }
    }

    pub fn from_name(name: &str) -> Option<AfterClose> {
        AfterClose::ALL
            .into_iter()
            .find(|mode| mode.name() == name.trim())
    }

    /// Whether a close keeps the hidden window and the bar (for a while, with `Ready15`).
    pub fn stays_ready(self) -> bool {
        self != AfterClose::Hibernate
    }

    /// How long a close keeps them before hibernating: `None` for as long as Little Joan runs
    /// (`Ready`) or not at all (`Hibernate`, see `stays_ready`).
    pub fn ready_for(self) -> Option<Duration> {
        (self == AfterClose::Ready15).then_some(READY_FOR)
    }
}

/// `afterClose` in settings.json: one of the three names; anything else (missing, another type,
/// another word) is the default, `ready`.
pub fn parse(value: Option<&Value>) -> AfterClose {
    value
        .and_then(Value::as_str)
        .and_then(AfterClose::from_name)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_default_is_stay_ready() {
        assert_eq!(AfterClose::default(), AfterClose::Ready);
        assert_eq!(parse(None), AfterClose::Ready);
        for junk in [
            json!(null),
            json!(15),
            json!("Hibernate!"),
            json!(["ready"]),
            json!(""),
        ] {
            assert_eq!(parse(Some(&junk)), AfterClose::Ready, "{junk}");
        }
    }

    #[test]
    fn each_name_reads_and_writes_the_same() {
        for mode in AfterClose::ALL {
            assert_eq!(parse(Some(&json!(mode.name()))), mode);
            assert_eq!(
                serde_json::to_value(mode).expect("json"),
                json!(mode.name())
            );
        }
        assert_eq!(parse(Some(&json!(" hibernate "))), AfterClose::Hibernate);
    }

    #[test]
    fn what_each_mode_keeps() {
        assert!(AfterClose::Ready.stays_ready());
        assert!(AfterClose::Ready15.stays_ready());
        assert!(!AfterClose::Hibernate.stays_ready());
        assert_eq!(AfterClose::Ready.ready_for(), None);
        assert_eq!(
            AfterClose::Ready15.ready_for(),
            Some(Duration::from_secs(900))
        );
        assert_eq!(AfterClose::Hibernate.ready_for(), None);
    }
}
