//! What a shortcut press or command-line request does, given whether the page window exists.
//!
//! | State                      | Toggle    | Show(None) | Show(Some(u))        | Hide    |
//! |----------------------------|-----------|------------|----------------------|---------|
//! | Hibernating                | Create    | Create     | Create(u)            | Nothing |
//! | Open, focused              | Destroy   | Focus      | FocusAndNavigate(u)  | Destroy |
//! | Open, not focused          | Focus     | Focus      | FocusAndNavigate(u)  | Destroy |
//!
//! `Extension(id)` (`--extension <id>`) opens that extension's panel: `CreateExtension(id)`
//! while hibernating, `FocusAndExtension(id)` with a window open.

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
    /// `--size <width>x<height>`: resize the open window (logical pixels).
    Resize { width: u32, height: u32 },
    /// `--extension <id>`: open an extension's panel (its first command).
    Extension(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Create the window (start page), then navigate to the URL if one is given.
    Create(Option<Url>),
    Focus,
    FocusAndNavigate(Url),
    /// Create the window, then open the extension's panel.
    CreateExtension(String),
    FocusAndExtension(String),
    /// Close and destroy the window: back to hibernation.
    Destroy,
    Resize {
        width: u32,
        height: u32,
    },
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
        (WindowState::Open { .. }, Command::Resize { width, height }) => {
            Action::Resize { width, height }
        }
        (WindowState::Hibernating, Command::Resize { .. }) => Action::Nothing,
        (WindowState::Hibernating, Command::Extension(id)) => Action::CreateExtension(id),
        (WindowState::Open { .. }, Command::Extension(id)) => Action::FocusAndExtension(id),
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
    fn resize_applies_to_an_open_window_only() {
        let resize = Command::Resize {
            width: 390,
            height: 600,
        };
        assert_eq!(decide(HIBERNATING, resize.clone()), Action::Nothing);
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
        assert_eq!(decide(FOCUSED, Command::Hide), Action::Destroy);
        assert_eq!(decide(BACKGROUND, Command::Hide), Action::Destroy);
    }
}
