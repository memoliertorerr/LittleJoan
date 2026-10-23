//! Where Little Joan's windows show up (LJ19.2): on the Space or desktop the user is on, in front
//! and with the keyboard — over another app's full-screen window too — without staying on top.
//! Like Little Arc: it opens where you are, and goes behind other windows once you click them.
//!
//! The pure parts: macOS's collection behaviour for the page and settings windows, the bits' names
//! and the smoke log's lines. The native glue is in platform/: mac/spaces.rs (the NSWindow's
//! collection behaviour, set before the window is first shown), linux/front.rs (the window
//! manager's raise-and-focus with a fresh X server timestamp) and win/front.rs (the smoke log's
//! check of the foreground window).

/// AppKit's `NSWindowCollectionBehavior` bits (NSWindow.h).
pub mod bits {
    pub const CAN_JOIN_ALL_SPACES: usize = 1 << 0;
    pub const MOVE_TO_ACTIVE_SPACE: usize = 1 << 1;
    pub const MANAGED: usize = 1 << 2;
    pub const TRANSIENT: usize = 1 << 3;
    pub const STATIONARY: usize = 1 << 4;
    pub const PARTICIPATES_IN_CYCLE: usize = 1 << 5;
    pub const IGNORES_CYCLE: usize = 1 << 6;
    pub const FULL_SCREEN_PRIMARY: usize = 1 << 7;
    pub const FULL_SCREEN_AUXILIARY: usize = 1 << 8;
    pub const FULL_SCREEN_NONE: usize = 1 << 9;
    pub const FULL_SCREEN_ALLOWS_TILING: usize = 1 << 11;
    pub const FULL_SCREEN_DISALLOWS_TILING: usize = 1 << 12;
    pub const PRIMARY: usize = 1 << 16;
    pub const AUXILIARY: usize = 1 << 17;
    pub const CAN_JOIN_ALL_APPLICATIONS: usize = 1 << 18;
}

use bits::*;

/// Which Space a window goes to: AppKit allows at most one of these.
const SPACE_GROUP: usize = CAN_JOIN_ALL_SPACES | MOVE_TO_ACTIVE_SPACE;
/// What a window does with full screen: AppKit allows at most one of these.
const FULL_SCREEN_GROUP: usize = FULL_SCREEN_PRIMARY | FULL_SCREEN_AUXILIARY | FULL_SCREEN_NONE;

/// The collection behaviour of a Little Joan window (the page window and Settings), from the one
/// it has now:
/// - **`MoveToActiveSpace`**: made active, the window comes to the Space the user is on instead of
///   taking the user to the Space it was on. Not `CanJoinAllSpaces`: that would put the open
///   window on every desktop and full-screen Space at once, following the user around like a
///   sticker — a pop-up browser belongs where it was summoned.
/// - **`FullScreenAuxiliary`**: the window may show on a full-screen window's Space — another
///   app's too, for an accessory app like Little Joan (no Dock icon) — so a summon over a
///   full-screen app opens it right there. AppKit allows one of Primary / Auxiliary / None, so the
///   window can't become a full-screen Space itself (its green button zooms); a page's element
///   fullscreen (a video) is WebKit's own window, which is untouched.
/// - Everything else (cycling, tiling, Stage Manager) stays as tao and AppKit set it.
///
/// The window level stays normal: Little Joan is in front because the summon activates it and
/// orders its window front, and goes behind other windows like any window once the user clicks
/// elsewhere. A raised level would keep it above every other app's windows after that.
pub fn summoned(current: usize) -> usize {
    (current & !(SPACE_GROUP | FULL_SCREEN_GROUP)) | MOVE_TO_ACTIVE_SPACE | FULL_SCREEN_AUXILIARY
}

/// The names of the bits that are set, for the smoke log (`moveToActiveSpace|fullScreenAuxiliary`);
/// `default` for none, an unknown bit in hex.
pub fn names(behavior: usize) -> String {
    const NAMES: [(usize, &str); 15] = [
        (CAN_JOIN_ALL_SPACES, "canJoinAllSpaces"),
        (MOVE_TO_ACTIVE_SPACE, "moveToActiveSpace"),
        (MANAGED, "managed"),
        (TRANSIENT, "transient"),
        (STATIONARY, "stationary"),
        (PARTICIPATES_IN_CYCLE, "participatesInCycle"),
        (IGNORES_CYCLE, "ignoresCycle"),
        (FULL_SCREEN_PRIMARY, "fullScreenPrimary"),
        (FULL_SCREEN_AUXILIARY, "fullScreenAuxiliary"),
        (FULL_SCREEN_NONE, "fullScreenNone"),
        (FULL_SCREEN_ALLOWS_TILING, "fullScreenAllowsTiling"),
        (FULL_SCREEN_DISALLOWS_TILING, "fullScreenDisallowsTiling"),
        (PRIMARY, "primary"),
        (AUXILIARY, "auxiliary"),
        (CAN_JOIN_ALL_APPLICATIONS, "canJoinAllApplications"),
    ];
    if behavior == 0 {
        return "default".to_owned();
    }
    let mut parts: Vec<String> = NAMES
        .iter()
        .filter(|(bit, _)| behavior & bit != 0)
        .map(|(_, name)| (*name).to_owned())
        .collect();
    let known = NAMES.iter().fold(0, |all, (bit, _)| all | bit);
    let unknown = behavior & !known;
    if unknown != 0 {
        parts.push(format!("{unknown:#x}"));
    }
    parts.join("|")
}

/// The smoke log's `window-spaces` detail: `<label> behavior=<names> was=<names> level=<level>`
/// (macOS, each time a window's behaviour was set).
pub fn spaces_line(label: &str, was: usize, behavior: usize, level: isize) -> String {
    format!(
        "{label} behavior={} was={} level={level}",
        names(behavior),
        names(was)
    )
}

/// What the OS says about a window shortly after a summon brought it to the front (smoke mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Front {
    /// It has the keyboard: the key window of the active app (macOS), the foreground window
    /// (Windows), GTK's active window — the window manager gave it the focus — (Linux).
    pub focused: bool,
    /// Nothing covers it: the topmost window at its centre is its own (macOS, Windows), or it is
    /// the top of the window manager's stacking order (Linux).
    pub top: bool,
    /// macOS: it is on the active Space.
    pub space: Option<bool>,
}

impl Front {
    /// The smoke log's `window-front` detail: `<label> focused=<bool> top=<bool>[ space=<bool>]`.
    pub fn line(&self, label: &str) -> String {
        let mut line = format!("{label} focused={} top={}", self.focused, self.top);
        if let Some(space) = self.space {
            line.push_str(&format!(" space={space}"));
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_joins_the_active_space_and_may_show_over_full_screen() {
        assert_eq!(summoned(0), MOVE_TO_ACTIVE_SPACE | FULL_SCREEN_AUXILIARY);
        assert_eq!(summoned(0), 258, "the bits AppKit reads back");
        assert_eq!(names(summoned(0)), "moveToActiveSpace|fullScreenAuxiliary");
    }

    #[test]
    fn exclusive_groups_keep_one_member() {
        // AppKit allows one of CanJoinAllSpaces / MoveToActiveSpace and one of the full-screen
        // trio; anything else raises.
        for current in [
            CAN_JOIN_ALL_SPACES,
            FULL_SCREEN_PRIMARY,
            FULL_SCREEN_NONE,
            CAN_JOIN_ALL_SPACES | FULL_SCREEN_PRIMARY | FULL_SCREEN_NONE,
            usize::MAX,
        ] {
            let set = summoned(current);
            assert_eq!(set & SPACE_GROUP, MOVE_TO_ACTIVE_SPACE, "{current:#x}");
            assert_eq!(
                set & FULL_SCREEN_GROUP,
                FULL_SCREEN_AUXILIARY,
                "{current:#x}"
            );
        }
    }

    #[test]
    fn other_bits_stay_as_they_were() {
        let others = MANAGED
            | PARTICIPATES_IN_CYCLE
            | FULL_SCREEN_ALLOWS_TILING
            | PRIMARY
            | CAN_JOIN_ALL_APPLICATIONS;
        assert_eq!(
            summoned(others | FULL_SCREEN_PRIMARY),
            others | MOVE_TO_ACTIVE_SPACE | FULL_SCREEN_AUXILIARY
        );
        assert_eq!(summoned(TRANSIENT) & TRANSIENT, TRANSIENT);
    }

    #[test]
    fn setting_it_twice_changes_nothing() {
        for current in [
            0,
            CAN_JOIN_ALL_SPACES,
            FULL_SCREEN_PRIMARY | MANAGED,
            usize::MAX,
        ] {
            assert_eq!(summoned(summoned(current)), summoned(current));
        }
    }

    #[test]
    fn names_for_the_log() {
        assert_eq!(names(0), "default");
        assert_eq!(names(FULL_SCREEN_PRIMARY), "fullScreenPrimary");
        assert_eq!(
            names(CAN_JOIN_ALL_SPACES | CAN_JOIN_ALL_APPLICATIONS),
            "canJoinAllSpaces|canJoinAllApplications"
        );
        assert_eq!(names(1 << 10), "0x400", "an unknown bit");
        assert_eq!(names(MANAGED | 1 << 20), "managed|0x100000");
        assert_eq!(
            spaces_line("joan", 0, 258, 0),
            "joan behavior=moveToActiveSpace|fullScreenAuxiliary was=default level=0"
        );
    }

    #[test]
    fn front_lines() {
        let front = Front {
            focused: true,
            top: false,
            space: None,
        };
        assert_eq!(front.line("joan"), "joan focused=true top=false");
        let mac = Front {
            space: Some(true),
            ..front
        };
        assert_eq!(
            mac.line("settings"),
            "settings focused=true top=false space=true"
        );
    }
}
