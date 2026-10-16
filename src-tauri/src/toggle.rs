//! What a shortcut press or command-line request does, given whether the page window exists.
//!
//! | State                      | Toggle    | Show(None) | Show(Some(u))        | Hide    |
//! |----------------------------|-----------|------------|----------------------|---------|
//! | Hibernating                | Create    | Create     | Create(u)            | Nothing |
//! | Open, focused              | Destroy   | Focus      | FocusAndNavigate(u)  | Destroy |
//! | Open, not focused          | Focus     | Focus      | FocusAndNavigate(u)  | Destroy |

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    /// No window and no webview exist.
    Hibernating,
    Open {
        focused: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// The global shortcut, or `--toggle`.
    Toggle,
    /// `--show [url]` (or a second launch without flags).
    Show(Option<Url>),
    /// `--hide`.
    Hide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Create the window (start page), then navigate to the URL if one is given.
    Create(Option<Url>),
    Focus,
    FocusAndNavigate(Url),
    /// Close and destroy the window: back to hibernation.
    Destroy,
    Nothing,
}

pub fn decide(state: WindowState, command: Command) -> Action {
    match (state, command) {
        (WindowState::Hibernating, Command::Toggle) => Action::Create(None),
        (WindowState::Hibernating, Command::Show(url)) => Action::Create(url),
        (WindowState::Hibernating, Command::Hide) => Action::Nothing,
        (WindowState::Open { focused: true }, Command::Toggle) => Action::Destroy,
        (WindowState::Open { focused: false }, Command::Toggle) => Action::Focus,
        (WindowState::Open { .. }, Command::Show(None)) => Action::Focus,
        (WindowState::Open { .. }, Command::Show(Some(url))) => Action::FocusAndNavigate(url),
        (WindowState::Open { .. }, Command::Hide) => Action::Destroy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HIBERNATING: WindowState = WindowState::Hibernating;
    const FOCUSED: WindowState = WindowState::Open { focused: true };
    const BACKGROUND: WindowState = WindowState::Open { focused: false };

    fn url() -> Url {
        Url::parse("https://example.com/").expect("valid")
    }

    #[test]
    fn toggle() {
        assert_eq!(decide(HIBERNATING, Command::Toggle), Action::Create(None));
        assert_eq!(decide(FOCUSED, Command::Toggle), Action::Destroy);
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
    fn hide() {
        assert_eq!(decide(HIBERNATING, Command::Hide), Action::Nothing);
        assert_eq!(decide(FOCUSED, Command::Hide), Action::Destroy);
        assert_eq!(decide(BACKGROUND, Command::Hide), Action::Destroy);
    }
}
