//! Phone or desktop per site (LJ40): the bar's toolbar pill and its command-bar rows. Only the
//! bar may call these (capabilities/bar.json, and a test). The site is always the open web
//! page's, taken by Rust (phone_mode.rs) — the bar sends only the mode it wants.

use serde::Serialize;
use tauri::{AppHandle, Runtime};

use crate::keys::Platform;
use crate::phone::{self, SiteInfo};
use crate::phone_mode;
use crate::settings::SiteMode;

/// A site built in on this OS (`get_settings`' `builtInSiteModes`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuiltInSite {
    pub host: &'static str,
    pub mode: SiteMode,
}

/// The sites built in on this OS: phone::WEBKIT_DESKTOP_SITES on macOS and Linux, none on
/// Windows.
pub fn built_in_sites() -> Vec<BuiltInSite> {
    built_in_sites_for(Platform::current())
}

fn built_in_sites_for(platform: Platform) -> Vec<BuiltInSite> {
    phone::WEBKIT_DESKTOP_SITES
        .iter()
        .filter_map(|host| phone::built_in(host, platform).map(|mode| BuiltInSite { host, mode }))
        .collect()
}

/// What the open web page's site is on: `{ host, mode, builtIn, agent }` (phone::SiteInfo), or
/// `null` without a web page (the start page, an extension's panel).
#[tauri::command]
pub async fn site_mode_info<R: Runtime>(app: AppHandle<R>) -> Option<SiteInfo> {
    phone_mode::info(&app)
}

/// Keeps the open web page's site on `mode` (`auto` | `phone` | `desktop`) and applies it at once:
/// the agent changes, and the page loads again only when it really changed. Returns the site's
/// new info; refused without a web page.
#[tauri::command]
pub async fn set_site_mode<R: Runtime>(
    app: AppHandle<R>,
    mode: SiteMode,
) -> Result<SiteInfo, String> {
    phone_mode::set_site_mode(&app, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn youtube_is_built_in_on_webkit_only() {
        for platform in [Platform::MacOs, Platform::Linux] {
            assert_eq!(
                built_in_sites_for(platform),
                [BuiltInSite {
                    host: "youtube.com",
                    mode: SiteMode::Desktop
                }]
            );
        }
        assert!(built_in_sites_for(Platform::Windows).is_empty());
        let json = serde_json::to_value(built_in_sites_for(Platform::MacOs)).expect("json");
        assert_eq!(
            json,
            serde_json::json!([{ "host": "youtube.com", "mode": "desktop" }])
        );
    }
}
