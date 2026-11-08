//! What `--extension <id>` opens (LJ56): the extension's first command — or, for an extension
//! that has none, its first page action for the web page that is open, on the same path as the
//! command bar's (`Target::PageAction` with the page as `open_extension` builds it; the panel gets
//! it only when its manifest asks for `pageInfo`, as there). With no web page open there is
//! nothing for a page action to act on: nothing opens, and the smoke log says
//! `extension-missing <id> no-page`.

use super::{PageInfo, Target};
use crate::extensions::manifest::Manifest;

/// Why `--extension` opens nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoTarget {
    /// The extension has only page actions, and no web page is open.
    NoPage,
}

impl NoTarget {
    /// The smoke log's detail: `<id> <reason>` (`extension-missing qr no-page`).
    pub fn log_detail(self, id: &str) -> String {
        let reason = match self {
            NoTarget::NoPage => "no-page",
        };
        format!("{id} {reason}")
    }
}

/// What `--extension` opens of the extension `manifest` describes: its first command; without
/// commands, its first page action for `page()` — the web page that is open, asked only then;
/// with neither, `Target::First` (the opening then fails as it always did: `extension-missing`).
pub fn target(
    manifest: &Manifest,
    page: impl FnOnce() -> Option<PageInfo>,
) -> Result<Target, NoTarget> {
    if !manifest.commands.is_empty() {
        return Ok(Target::First);
    }
    let Some(action) = manifest.page_actions.first() else {
        return Ok(Target::First);
    };
    page()
        .map(|page| Target::PageAction(action.id.clone(), page))
        .ok_or(NoTarget::NoPage)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::extensions::manifest;

    fn manifest(contributes: serde_json::Value) -> Manifest {
        manifest::parse(
            json!({
                "id": "probe",
                "name": "Probe",
                "version": "1.0.0",
                "apiVersion": 1,
                "permissions": ["pageInfo"],
                "contributes": contributes,
            })
            .to_string()
            .as_bytes(),
        )
        .expect("a valid manifest")
    }

    fn contribution(id: &str) -> serde_json::Value {
        json!({ "id": id, "title": id, "panel": "index.html" })
    }

    fn page() -> PageInfo {
        PageInfo {
            url: "https://example.com/a".to_owned(),
            title: "Example".to_owned(),
        }
    }

    #[test]
    fn a_command_comes_first_and_the_page_isnt_asked() {
        let both = manifest(json!({
            "commands": [contribution("open")],
            "pageActions": [contribution("show")],
        }));
        let target = target(&both, || unreachable!("a command needs no page"));
        assert!(matches!(target, Ok(Target::First)), "{target:?}");
    }

    #[test]
    fn without_commands_the_first_page_action_opens_for_the_open_page() {
        let actions = manifest(json!({
            "pageActions": [contribution("show"), contribution("other")],
        }));
        match target(&actions, || Some(page())) {
            Ok(Target::PageAction(action, for_page)) => {
                assert_eq!(action, "show");
                assert_eq!(for_page, page());
            }
            other => panic!("{other:?}"),
        }
        // No web page open (the start page, a panel, no window): nothing to act on.
        let none = target(&actions, || None);
        assert!(matches!(none, Err(NoTarget::NoPage)), "{none:?}");
        assert_eq!(NoTarget::NoPage.log_detail("qr"), "qr no-page");
    }

    #[test]
    fn with_neither_the_opening_fails_as_before() {
        let empty = manifest(json!({}));
        let target = target(&empty, || unreachable!("nothing to open for a page"));
        assert!(matches!(target, Ok(Target::First)), "{target:?}");
    }

    #[test]
    fn the_bundled_page_actions_open_this_way() {
        // QR code, Clean link, Wayback Machine and Copy as Markdown link have page actions only.
        for id in ["qr", "clean-link", "wayback", "markdown-link"] {
            let bytes = std::fs::read(format!(
                "{}/../extensions/{id}/extension.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .expect("the bundled manifest");
            let bundled = manifest::parse(&bytes).expect("valid");
            assert!(bundled.commands.is_empty(), "{id}");
            let target = target(&bundled, || Some(page()));
            assert!(
                matches!(&target, Ok(Target::PageAction(action, _)) if Some(action) == bundled.page_actions.first().map(|a| &a.id)),
                "{id}: {target:?}"
            );
        }
    }
}
