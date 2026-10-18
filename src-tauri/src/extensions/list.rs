//! What `extensions_list()` tells the bar (the command bar's extension rows) and Settings → Extensions
//! (the list, with switches and details): every loaded extension with its commands, page actions
//! and permissions, whether it is switched on, and what was skipped (pure, tested).

use serde::Serialize;

use super::manifest::{Contribution, Permission};
use super::registry::{Extension, Kind, Registry};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub title: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// "bundled" or "developer".
    pub kind: &'static str,
    pub enabled: bool,
    /// A data: URI of its icon, or null.
    pub icon: Option<String>,
    /// As the manifest says them: `storage`, `clipboard`, `pageInfo`, `network:<host>`
    /// (Settings words them).
    pub permissions: Vec<String>,
    pub commands: Vec<Item>,
    pub page_actions: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedInfo {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionsView {
    pub developer_mode: bool,
    /// Where developer extensions go.
    pub folder: String,
    /// The built-in quick answers switched off (`builtin:<name>`): the command bar skips them.
    pub disabled_built_ins: Vec<String>,
    pub extensions: Vec<ExtensionInfo>,
    pub skipped: Vec<SkippedInfo>,
}

fn items(list: &[Contribution]) -> Vec<Item> {
    list.iter()
        .map(|c| Item {
            id: c.id.clone(),
            title: c.title.clone(),
            keywords: c.keywords.clone(),
        })
        .collect()
}

fn permission_name(permission: &Permission) -> String {
    match permission {
        Permission::Storage => "storage".to_owned(),
        Permission::Clipboard => "clipboard".to_owned(),
        Permission::PageInfo => "pageInfo".to_owned(),
        Permission::Network(host) => format!("network:{host}"),
    }
}

pub fn info(extension: &Extension) -> ExtensionInfo {
    let manifest = &extension.manifest;
    ExtensionInfo {
        id: manifest.id.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        author: manifest.author.clone(),
        kind: match extension.kind {
            Kind::Bundled => "bundled",
            Kind::Developer => "developer",
        },
        enabled: extension.enabled,
        icon: extension.icon_data_uri(),
        permissions: manifest.permissions.iter().map(permission_name).collect(),
        commands: items(&manifest.commands),
        page_actions: items(&manifest.page_actions),
    }
}

/// The list for the bar and Settings; `disabled` is settings' `disabledExtensions`.
pub fn view(
    registry: &Registry,
    developer_mode: bool,
    folder: String,
    disabled: &[String],
) -> ExtensionsView {
    ExtensionsView {
        developer_mode,
        folder,
        disabled_built_ins: disabled
            .iter()
            .filter(|id| id.starts_with("builtin:"))
            .cloned()
            .collect(),
        extensions: registry.all().iter().map(|e| info(e)).collect(),
        skipped: registry
            .skipped
            .iter()
            .map(|s| SkippedInfo {
                name: s.name.clone(),
                reason: s.reason.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::registry::{Bundled, build};

    const NOTES: &[u8] = br#"{ "id": "notes", "name": "Notes", "version": "2.1", "apiVersion": 1,
        "description": "Quick notes.", "author": "Someone",
        "permissions": ["storage", "network:api.example.com"],
        "contributes": {
            "commands": [{ "id": "open", "title": "Open notes", "keywords": ["memo"], "panel": "index.html" }],
            "pageActions": [{ "id": "clip", "title": "Clip this page", "panel": "clip.html" }]
        } }"#;
    static FIXTURE: Bundled = &[(
        "notes",
        &[
            ("extension.json", NOTES),
            ("index.html", b"x"),
            ("clip.html", b"x"),
        ],
    )];

    #[test]
    fn the_list_carries_what_the_bar_and_settings_show() {
        let registry = build(FIXTURE, None, &["notes".to_owned()]);
        let view = view(
            &registry,
            true,
            "/config/extensions".to_owned(),
            &["notes".to_owned(), "builtin:calc".to_owned()],
        );
        assert!(view.developer_mode);
        let notes = &view.extensions[0];
        assert_eq!(notes.kind, "bundled");
        assert!(!notes.enabled);
        assert_eq!(
            notes.permissions,
            vec!["storage", "network:api.example.com"]
        );
        assert_eq!(notes.commands[0].keywords, vec!["memo"]);
        assert_eq!(notes.page_actions[0].title, "Clip this page");
        assert_eq!(notes.icon, None);
        let json = serde_json::to_value(&view).expect("json");
        assert_eq!(json["extensions"][0]["pageActions"][0]["id"], "clip");
        assert_eq!(json["developerMode"], true);
        assert_eq!(
            json["disabledBuiltIns"],
            serde_json::json!(["builtin:calc"])
        );
    }
}
