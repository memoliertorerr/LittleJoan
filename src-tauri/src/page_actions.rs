//! What the page keys (keys.rs) and the macOS menu do to the page window.

use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use url::Url;

use crate::joan;
use crate::keys::{self, PageAction};
use crate::navigation;
use crate::state;
use crate::toggle::Command;
use crate::translate;

/// Performs `action` on the page window, off the main thread (callers are native key and menu
/// handlers running on it).
pub fn perform<R: Runtime>(app: &AppHandle<R>, action: PageAction) {
    if action == PageAction::Close {
        joan::dispatch(app, Command::Hide);
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(window) = app.get_webview_window(joan::LABEL) else {
            return;
        };
        let result = match action {
            PageAction::OpenLocation => open_location(&window),
            PageAction::Reload => window.reload(),
            PageAction::Back => window.eval("history.back()"),
            PageAction::Forward => window.eval("history.forward()"),
            PageAction::Translate => translate_page(&app, &window),
            PageAction::OpenInBrowser => open_in_browser(&app, &window),
            PageAction::Close => Ok(()),
        };
        if let Err(err) = result {
            eprintln!("little-joan: page action {action:?} failed: {err}");
        }
    });
}

/// Shows the start page with the current address pre-filled and selected. On the start page
/// itself there is nothing to do.
fn open_location<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<()> {
    let current = window.url()?;
    match keys::start_page_url(&keys::app_origin(), &current) {
        Some(url) => window.navigate(url),
        None => Ok(()),
    }
}

/// The translated view of `current`, or `None` for the start page and for pages that can't or
/// needn't be translated.
pub fn translate_target(origin: &Url, current: &Url, lang: &str) -> Option<Url> {
    if keys::is_app_url(origin, current) {
        return None;
    }
    translate::translate_url(current, lang)
}

fn translate_page<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) -> tauri::Result<()> {
    let lang = state::with(app, |s| s.settings.translate_to.clone());
    match translate_target(&keys::app_origin(), &window.url()?, &lang) {
        Some(url) => window.navigate(url),
        None => Ok(()),
    }
}

/// Opens the page in the default browser, then hibernates — the page moved to the main browser.
fn open_in_browser<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) -> tauri::Result<()> {
    let current = window.url()?;
    if keys::is_app_url(&keys::app_origin(), &current)
        || !navigation::is_external_openable(&current)
    {
        return Ok(());
    }
    match tauri_plugin_opener::open_url(current.as_str(), None::<&str>) {
        Ok(()) => joan::handle(app, Command::Hide),
        Err(err) => eprintln!("little-joan: could not open the page in the default browser: {err}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn translate_target_skips_the_start_page() {
        for (origin, start) in [
            (
                "http://tauri.localhost/",
                "http://tauri.localhost/index.html",
            ),
            ("tauri://localhost/", "tauri://localhost/index.html?u=x"),
        ] {
            assert_eq!(
                translate_target(&url(origin), &url(start), "en"),
                None,
                "{start}"
            );
        }
    }

    #[test]
    fn translate_target_skips_translated_pages() {
        let origin = url("tauri://localhost/");
        assert_eq!(
            translate_target(&origin, &url("https://example-com.translate.goog/"), "en"),
            None
        );
    }

    #[test]
    fn translate_target_builds_the_proxy_url_for_pages() {
        let origin = url("http://tauri.localhost/");
        let got = translate_target(&origin, &url("https://example.com/"), "de").expect("a page");
        assert_eq!(
            got.as_str(),
            "https://translate.google.com/translate?sl=auto&tl=de&hl=de&u=https%3A%2F%2Fexample.com%2F"
        );
    }
}
