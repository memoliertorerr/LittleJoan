//! The bar's part of a summon's timing (LJ19.1): what only the bar can see — its navigation, its
//! HTML, its script, the first answers it got, its first painted frame, the command bar's first
//! rows on screen and (LJ19.4) when its field had the keyboard. The bar reports them once per
//! summon, in one call, only when Rust says so (`start_info`'s `timing`, and a summon from ready's
//! own) — in smoke tests (`LITTLEJOAN_SMOKE_LOG`) — so normal use never calls this. Only the bar may
//! call it (capabilities/bar.json, and a test).

use serde::Deserialize;

use crate::summon_timing::{self, Mark};

/// What the bar saw, each as milliseconds before this call (the bar's own clock measures the
/// gaps, so the two clocks never need to agree). Unknown names are refused.
#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BarMarks {
    /// Its navigation started (the engine's navigation timing: its time origin).
    navigation: Option<f64>,
    /// Its HTML arrived (the navigation entry's `responseEnd`).
    html: Option<f64>,
    /// ui/bar.ts's top level ran.
    module: Option<f64>,
    /// `set_theme_tokens` (its early theme report) answered.
    theme_answered: Option<f64>,
    /// `start_info` answered.
    start_info: Option<f64>,
    /// The first suggestions were drawn into the command bar.
    suggestions: Option<f64>,
    /// Its first frame was painted.
    painted: Option<f64>,
    /// The command bar's first rows were painted.
    command_bar: Option<f64>,
    /// Its field had the keyboard (its document had focus, the field active).
    focused: Option<f64>,
}

impl BarMarks {
    /// Each mark with its "ago", in the order they normally come.
    fn marks(&self) -> [(Mark, Option<f64>); 9] {
        [
            (Mark::BarNavigation, self.navigation),
            (Mark::BarHtml, self.html),
            (Mark::BarModule, self.module),
            (Mark::BarThemeAnswered, self.theme_answered),
            (Mark::BarStartInfo, self.start_info),
            (Mark::BarSuggestions, self.suggestions),
            (Mark::BarPainted, self.painted),
            (Mark::BarFocused, self.focused),
            (Mark::CommandBar, self.command_bar),
        ]
    }
}

/// Places what the bar saw on the summon's clock. Nothing happens outside a measured summon; a
/// value that isn't a sane duration is left out.
#[tauri::command]
pub async fn report_timing(marks: BarMarks) {
    summon_timing::marks_ago(
        &marks
            .marks()
            .into_iter()
            .filter_map(|(mark, ago)| Some((mark, ago?)))
            .collect::<Vec<_>>(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_known_marks_are_read_and_others_refused() {
        let marks: BarMarks = serde_json::from_str(
            r#"{"navigation": 300, "painted": 40.5, "focused": 12, "commandBar": 0}"#,
        )
        .expect("known marks");
        let given: Vec<Mark> = marks
            .marks()
            .into_iter()
            .filter_map(|(mark, ago)| ago.map(|_| mark))
            .collect();
        assert_eq!(
            given,
            [
                Mark::BarNavigation,
                Mark::BarPainted,
                Mark::BarFocused,
                Mark::CommandBar
            ]
        );
        for other in [
            r#"{"shown": 1}"#,
            r#"{"painted": "soon"}"#,
            r#"{"commandbar": 1}"#,
            r#"{"barFocused": 1}"#,
            "[]",
            "1",
        ] {
            assert!(serde_json::from_str::<BarMarks>(other).is_err(), "{other}");
        }
        let none: BarMarks = serde_json::from_str("{}").expect("no marks");
        assert_eq!(none, BarMarks::default());
    }
}
