//! The bar's part of a summon's timing (LJ19.1): its first painted frame and the command bar's
//! first rows on screen, which only the bar can see. The bar reports them only when `start_info`
//! says so — in smoke tests (`LITTLEJOAN_SMOKE_LOG`) — so normal use never calls this. Only the
//! bar may call it (capabilities/bar.json, and a test).

use serde::Deserialize;

use crate::summon_timing::{self, Mark};

/// What the bar saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BarMark {
    /// Its first frame was painted.
    Painted,
    /// The command bar's first rows were painted.
    CommandBar,
}

impl BarMark {
    fn mark(self) -> Mark {
        match self {
            BarMark::Painted => Mark::BarPainted,
            BarMark::CommandBar => Mark::CommandBar,
        }
    }
}

/// `mark` happened `ago` milliseconds before this call (the bar's own clock measures the gap, so
/// the two clocks never need to agree). Nothing happens outside a measured summon, or for an
/// `ago` that isn't a sane duration.
#[tauri::command]
pub async fn report_timing(mark: BarMark, ago: f64) {
    summon_timing::mark_ago(mark.mark(), ago);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_two_marks_are_accepted() {
        let painted: BarMark = serde_json::from_str("\"painted\"").expect("painted");
        assert_eq!(painted.mark(), Mark::BarPainted);
        let rows: BarMark = serde_json::from_str("\"commandBar\"").expect("commandBar");
        assert_eq!(rows.mark(), Mark::CommandBar);
        for other in ["\"shown\"", "\"barReady\"", "\"\"", "1"] {
            assert!(serde_json::from_str::<BarMark>(other).is_err(), "{other}");
        }
    }
}
