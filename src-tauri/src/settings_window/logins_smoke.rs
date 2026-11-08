//! Smoke tests only (LJ51): what the smoke harness may have Settings → Privacy's saved-passwords
//! section do with `LITTLEJOAN_SMOKE_SETTINGS_LOGINS` — `remove` (its "Remove saved passwords and
//! form data…", then the question's Remove) or `toggle-save` (flip "Offer to save passwords", then
//! Save). Either acts without a person: removing deletes every saved password with no question
//! asked. So beyond the smoke log (`settings_window::smoke_scroll`), it needs a throwaway data
//! folder (`LITTLEJOAN_DATA_DIR` set, so the webview data folder is Little Joan's own temporary one,
//! never the installed app's profile) on a GitHub-hosted runner (`GITHUB_ACTIONS=true` and
//! `RUNNER_ENVIRONMENT=github-hosted`, what scripts/ci/smoke-runner.mts asks before it changes a
//! machine) — phase 24's security review. Pure, tested.

/// What the section is driven to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LoginsDrive {
    /// "Remove saved passwords and form data…", then the question's Remove.
    Remove,
    /// "Offer to save passwords" flipped, then Save.
    ToggleSave,
}

impl LoginsDrive {
    /// The clicks, in order: what the smoke log says (`settings-logins-click <what>`) and the script
    /// that clicks (ids of Little Joan's own page; the page's own handlers run, as a click's would).
    pub(super) fn steps(self) -> &'static [(&'static str, &'static str)] {
        match self {
            LoginsDrive::Remove => &[
                (
                    "remove",
                    "document.getElementById(\"logins-remove\")?.click()",
                ),
                ("confirm", "document.getElementById(\"logins-go\")?.click()"),
            ],
            LoginsDrive::ToggleSave => &[
                (
                    "toggle",
                    "document.getElementById(\"save-passwords\")?.click()",
                ),
                ("save", "document.getElementById(\"save\")?.click()"),
            ],
        }
    }
}

/// The drive `asked` for (`LITTLEJOAN_SMOKE_SETTINGS_LOGINS`), if it may run: `Ok(None)` when none
/// is asked (or a name it doesn't know), or this system has no switches (`available`); `Err(why)`
/// when one is asked off a throwaway data folder (`temp_data`) or off a GitHub-hosted runner
/// (`github_actions`, `runner`: those variables' values).
pub(super) fn wanted(
    asked: Option<&str>,
    temp_data: bool,
    github_actions: Option<&str>,
    runner: Option<&str>,
    available: bool,
) -> Result<Option<LoginsDrive>, &'static str> {
    let drive = match asked {
        Some("remove") => LoginsDrive::Remove,
        Some("toggle-save") => LoginsDrive::ToggleSave,
        _ => return Ok(None),
    };
    if !temp_data {
        return Err("no-temporary-data-folder");
    }
    if github_actions != Some("true") || runner != Some("github-hosted") {
        return Err("not-a-github-hosted-runner");
    }
    Ok(available.then_some(drive))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CI: (Option<&str>, Option<&str>) = (Some("true"), Some("github-hosted"));

    #[test]
    fn it_drives_the_section_only_on_a_throwaway_folder_on_a_github_hosted_runner() {
        for (asked, drive) in [
            ("remove", LoginsDrive::Remove),
            ("toggle-save", LoginsDrive::ToggleSave),
        ] {
            assert_eq!(wanted(Some(asked), true, CI.0, CI.1, true), Ok(Some(drive)));
            // An installed build — no LITTLEJOAN_DATA_DIR, the real profile —: never.
            assert_eq!(
                wanted(Some(asked), false, CI.0, CI.1, true),
                Err("no-temporary-data-folder"),
                "{asked}"
            );
            // A developer's own machine, act, a self-hosted runner: never.
            for (actions, runner) in [
                (None, None),
                (Some("true"), None),
                (Some("true"), Some("self-hosted")),
                (Some("1"), Some("github-hosted")),
                (None, Some("github-hosted")),
            ] {
                assert_eq!(
                    wanted(Some(asked), true, actions, runner, true),
                    Err("not-a-github-hosted-runner"),
                    "{asked} {actions:?} {runner:?}"
                );
            }
            // Where there are no switches (macOS, Linux) there is nothing to click.
            assert_eq!(wanted(Some(asked), true, CI.0, CI.1, false), Ok(None));
        }
        for asked in [None, Some(""), Some("REMOVE"), Some("all")] {
            assert_eq!(wanted(asked, true, CI.0, CI.1, true), Ok(None), "{asked:?}");
        }
    }

    #[test]
    fn each_drive_clicks_two_of_the_pages_own_controls() {
        let ids = |drive: LoginsDrive| -> Vec<String> {
            drive
                .steps()
                .iter()
                .map(|(_, script)| {
                    let start = script.find("(\"").expect("an id") + 2;
                    let end = script[start..].find('"').expect("its end") + start;
                    script[start..end].to_owned()
                })
                .collect()
        };
        assert_eq!(ids(LoginsDrive::Remove), ["logins-remove", "logins-go"]);
        assert_eq!(ids(LoginsDrive::ToggleSave), ["save-passwords", "save"]);
        let html = include_str!("../../../ui/settings.html");
        for drive in [LoginsDrive::Remove, LoginsDrive::ToggleSave] {
            for id in ids(drive) {
                assert!(html.contains(&format!("id=\"{id}\"")), "{id}");
            }
        }
    }
}
