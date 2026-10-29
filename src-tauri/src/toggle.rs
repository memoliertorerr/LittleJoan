//! What a shortcut press, a command-line request or Little Joan itself does, given the state of the
//! page window.
//!
//! | State                  | Toggle      | Show(None) | Show(Some(u))       | Hide    | Prewarm | Release |
//! |------------------------|-------------|------------|---------------------|---------|---------|---------|
//! | Hibernating            | Create      | Create     | Create(u)           | Nothing | Prewarm | Nothing |
//! | Ready (bar alive)      | Summon      | Summon     | Summon(u)           | Nothing | Nothing | Release |
//! | Ready (bar gone)       | Rebuild     | Rebuild    | Rebuild(u)          | Nothing | Nothing | Release |
//! | Open, focused          | Close       | Focus      | FocusAndNavigate(u) | Close   | Nothing | Nothing |
//! | Open, not focused      | Focus       | Focus      | FocusAndNavigate(u) | Close   | Nothing | Nothing |
//!
//! `Extension(id)` (`--extension <id>`) opens that extension's panel: `CreateExtension(id)` while
//! hibernating, `SummonExtension(id)` from ready (`RebuildExtension(id)` when its bar is gone),
//! `FocusAndExtension(id)` with a window open. `Resize` applies to an open window only.
//!
//! **Ready** (LJ19.4, "After closing: Stay ready", the owner's default): the window exists,
//! hidden, with the trusted bar alive and no page webview. A close always discards the page; what
//! stays depends on the setting (joan.rs reads it): `Close` leaves Ready or Hibernating behind.

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// No window and no webview exist.
    Hibernating,
    /// The window exists, hidden, with the bar and no page. `usable`: the bar's web process is
    /// alive (false once the engine said it ended: the next summon builds everything anew).
    Ready {
        usable: bool,
    },
    Open {
        focused: bool,
    },
}

/// Why a resting window is released (the smoke log's `released <why>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// Stay ready for 15 minutes ran out.
    Timer,
    /// "After closing" switched to Hibernate.
    Setting,
    /// Little Joan quits.
    Quit,
}

impl Why {
    pub fn name(self) -> &'static str {
        match self {
            Why::Timer => "timer",
            Why::Setting => "setting",
            Why::Quit => "quit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// The global shortcut, or `--toggle`.
    Toggle,
    /// `--show [url]` (or a second launch without flags).
    Show(Option<Url>),
    /// `--hide`.
    Hide,
    /// `--size <width>x<height>`: resize the open window (logical pixels).
    Resize { width: u32, height: u32 },
    /// `--extension <id>`: open an extension's panel (its first command).
    Extension(String),
    /// Build the hidden window and the bar ahead of a summon (Stay ready: after the launch, or
    /// when the setting switches to it). Nothing shows.
    Prewarm,
    /// Drop what stays ready: no window, no webview (Stay ready for 15 minutes' timer, the
    /// setting switching to Hibernate, Quit). An open window stays.
    Release(Why),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Create the window (start page), then navigate to the URL if one is given.
    Create(Option<Url>),
    /// Show the ready window — its bar is loaded already —, then navigate to the URL if given.
    Summon(Option<Url>),
    Focus,
    FocusAndNavigate(Url),
    /// Create the window, then open the extension's panel.
    CreateExtension(String),
    /// Show the ready window, then open the extension's panel.
    SummonExtension(String),
    FocusAndExtension(String),
    /// Close the window: the page is discarded; the hidden window and the bar stay or go as the
    /// "After closing" setting says.
    Close,
    /// The ready window's bar is gone: destroy everything, then create (and navigate) anew.
    Rebuild(Option<Url>),
    RebuildExtension(String),
    /// Build the hidden window and the bar.
    Prewarm,
    /// Destroy the ready window: back to hibernation.
    Release(Why),
    Resize {
        width: u32,
        height: u32,
    },
    Nothing,
}

pub fn decide(state: WindowState, command: Command) -> Action {
    use WindowState::{Hibernating, Open, Ready};
    match (state, command) {
        (Hibernating, Command::Toggle) => Action::Create(None),
        (Hibernating, Command::Show(url)) => Action::Create(url),
        (Hibernating, Command::Extension(id)) => Action::CreateExtension(id),
        (Hibernating, Command::Prewarm) => Action::Prewarm,
        (Ready { usable: true }, Command::Toggle) => Action::Summon(None),
        (Ready { usable: true }, Command::Show(url)) => Action::Summon(url),
        (Ready { usable: true }, Command::Extension(id)) => Action::SummonExtension(id),
        (Ready { usable: false }, Command::Toggle) => Action::Rebuild(None),
        (Ready { usable: false }, Command::Show(url)) => Action::Rebuild(url),
        (Ready { usable: false }, Command::Extension(id)) => Action::RebuildExtension(id),
        (Ready { .. }, Command::Release(why)) => Action::Release(why),
        (Open { focused: true }, Command::Toggle) => Action::Close,
        (Open { focused: false }, Command::Toggle) => Action::Focus,
        (Open { .. }, Command::Show(None)) => Action::Focus,
        (Open { .. }, Command::Show(Some(url))) => Action::FocusAndNavigate(url),
        (Open { .. }, Command::Hide) => Action::Close,
        (Open { .. }, Command::Resize { width, height }) => Action::Resize { width, height },
        (Open { .. }, Command::Extension(id)) => Action::FocusAndExtension(id),
        (Hibernating | Ready { .. }, Command::Hide | Command::Resize { .. })
        | (Hibernating, Command::Release(_))
        | (Ready { .. } | Open { .. }, Command::Prewarm)
        | (Open { .. }, Command::Release(_)) => Action::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIBERNATING: WindowState = WindowState::Hibernating;
    const READY: WindowState = WindowState::Ready { usable: true };
    const BAR_GONE: WindowState = WindowState::Ready { usable: false };
    const FOCUSED: WindowState = WindowState::Open { focused: true };
    const BACKGROUND: WindowState = WindowState::Open { focused: false };
    const ALL: [WindowState; 5] = [HIBERNATING, READY, BAR_GONE, FOCUSED, BACKGROUND];

    fn url() -> Url {
        Url::parse("https://example.com/").expect("valid")
    }

    #[test]
    fn toggle() {
        assert_eq!(decide(HIBERNATING, Command::Toggle), Action::Create(None));
        assert_eq!(decide(READY, Command::Toggle), Action::Summon(None));
        assert_eq!(decide(BAR_GONE, Command::Toggle), Action::Rebuild(None));
        assert_eq!(decide(FOCUSED, Command::Toggle), Action::Close);
        assert_eq!(decide(BACKGROUND, Command::Toggle), Action::Focus);
    }

    #[test]
    fn show() {
        assert_eq!(
            decide(HIBERNATING, Command::Show(None)),
            Action::Create(None)
        );
        assert_eq!(
            decide(HIBERNATING, Command::Show(Some(url()))),
            Action::Create(Some(url()))
        );
        assert_eq!(decide(READY, Command::Show(None)), Action::Summon(None));
        assert_eq!(
            decide(READY, Command::Show(Some(url()))),
            Action::Summon(Some(url()))
        );
        assert_eq!(
            decide(BAR_GONE, Command::Show(Some(url()))),
            Action::Rebuild(Some(url()))
        );
        assert_eq!(decide(FOCUSED, Command::Show(None)), Action::Focus);
        assert_eq!(decide(BACKGROUND, Command::Show(None)), Action::Focus);
        assert_eq!(
            decide(FOCUSED, Command::Show(Some(url()))),
            Action::FocusAndNavigate(url())
        );
        assert_eq!(
            decide(BACKGROUND, Command::Show(Some(url()))),
            Action::FocusAndNavigate(url())
        );
    }

    #[test]
    fn resize_applies_to_an_open_window_only() {
        let resize = Command::Resize {
            width: 390,
            height: 600,
        };
        for state in [HIBERNATING, READY, BAR_GONE] {
            assert_eq!(decide(state, resize.clone()), Action::Nothing, "{state:?}");
        }
        for state in [FOCUSED, BACKGROUND] {
            assert_eq!(
                decide(state, resize.clone()),
                Action::Resize {
                    width: 390,
                    height: 600
                }
            );
        }
    }

    #[test]
    fn extension_opens_the_window_if_needed() {
        let open = || Command::Extension("hello".into());
        assert_eq!(
            decide(HIBERNATING, open()),
            Action::CreateExtension("hello".into())
        );
        assert_eq!(
            decide(READY, open()),
            Action::SummonExtension("hello".into())
        );
        assert_eq!(
            decide(BAR_GONE, open()),
            Action::RebuildExtension("hello".into())
        );
        for state in [FOCUSED, BACKGROUND] {
            assert_eq!(
                decide(state, open()),
                Action::FocusAndExtension("hello".into())
            );
        }
    }

    #[test]
    fn hide() {
        assert_eq!(decide(HIBERNATING, Command::Hide), Action::Nothing);
        assert_eq!(decide(READY, Command::Hide), Action::Nothing);
        assert_eq!(decide(BAR_GONE, Command::Hide), Action::Nothing);
        assert_eq!(decide(FOCUSED, Command::Hide), Action::Close);
        assert_eq!(decide(BACKGROUND, Command::Hide), Action::Close);
    }

    #[test]
    fn prewarm_builds_only_from_hibernation() {
        assert_eq!(decide(HIBERNATING, Command::Prewarm), Action::Prewarm);
        for state in [READY, BAR_GONE, FOCUSED, BACKGROUND] {
            assert_eq!(
                decide(state, Command::Prewarm),
                Action::Nothing,
                "{state:?}"
            );
        }
    }

    #[test]
    fn release_drops_a_ready_window_never_an_open_one() {
        for why in [Why::Timer, Why::Setting, Why::Quit] {
            assert_eq!(decide(READY, Command::Release(why)), Action::Release(why));
            assert_eq!(
                decide(BAR_GONE, Command::Release(why)),
                Action::Release(why)
            );
            for state in [HIBERNATING, FOCUSED, BACKGROUND] {
                assert_eq!(
                    decide(state, Command::Release(why)),
                    Action::Nothing,
                    "{state:?}"
                );
            }
            assert!(why.name().chars().all(|c| c.is_ascii_lowercase()));
        }
    }

    #[test]
    fn a_summon_from_ready_never_creates_and_a_gone_bar_never_shows() {
        let summons = || {
            [
                Command::Toggle,
                Command::Show(None),
                Command::Show(Some(url())),
                Command::Extension("hello".into()),
            ]
        };
        for command in summons() {
            let ready = decide(READY, command.clone());
            assert!(
                matches!(ready, Action::Summon(_) | Action::SummonExtension(_)),
                "{command:?}: {ready:?}"
            );
            let gone = decide(BAR_GONE, command.clone());
            assert!(
                matches!(gone, Action::Rebuild(_) | Action::RebuildExtension(_)),
                "{command:?}: {gone:?}"
            );
        }
        // Nothing but a summon, Prewarm or Release ever acts on a window at rest.
        for state in ALL {
            assert_eq!(
                matches!(state, WindowState::Open { .. }),
                decide(state, Command::Hide) == Action::Close
            );
        }
    }
}
