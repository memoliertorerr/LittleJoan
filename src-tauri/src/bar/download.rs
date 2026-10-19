//! Downloads at run time (LJ15; the decisions are downloads.rs's). The page webview's downloads
//! go to the OS Downloads folder under a cleaned name that never overwrites a file, and the
//! toolbar shows what happens (`window.littleJoan.setStatus`): "Downloading <name>…", then
//! "<name> saved" with "Show in folder" (`reveal_download`), or "Download failed". While the page
//! shows an extension's panel nothing is saved (`download-refused panel`). Closing the window
//! ends what is under way: nothing keeps a webview alive for a download (Ground Rule 9).
//!
//! WebView2 and WKWebView report through Tauri's `on_download` (`on_download` below). wry turns
//! WebView2's own download flyout off by marking the download handled. WebKitGTK reports through
//! platform/linux.rs instead: wry registers its handler on the shared web context once per webview
//! built, and the first one registered decides every download's destination (Landmines).

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

#[cfg(not(target_os = "linux"))]
use tauri::Webview;
#[cfg(not(target_os = "linux"))]
use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Manager, Runtime};
use url::Url;

use super::status::{self, Status};
use crate::downloads::{self, Downloads, Refusal};
use crate::extensions;
use crate::keys;
use crate::platform;
use crate::smoke;

/// The open window's downloads (downloads.rs).
static DOWNLOADS: Mutex<Downloads> = Mutex::new(Downloads::new());

fn downloads() -> std::sync::MutexGuard<'static, Downloads> {
    DOWNLOADS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// How a download ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Saved,
    Failed,
}

/// The page webview's download handler (WebView2, WKWebView). `Requested`: where the file goes,
/// or `false` to refuse it. `Finished`: macOS carries no path, so the destination is looked up
/// by URL.
#[cfg(not(target_os = "linux"))]
pub fn on_download<R: Runtime>(webview: Webview<R>, event: DownloadEvent<'_>) -> bool {
    match event {
        DownloadEvent::Requested { url, destination } => {
            let suggested = destination
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned);
            match requested(webview.app_handle(), &url, suggested.as_deref()) {
                Some(path) => {
                    *destination = path;
                    true
                }
                None => false,
            }
        }
        DownloadEvent::Finished { url, success, .. } => {
            let outcome = if success {
                Outcome::Saved
            } else {
                Outcome::Failed
            };
            finished(webview.app_handle(), &url, outcome);
            true
        }
        _ => true,
    }
}

/// A download of `url` asks where to go (`suggested`: the engine's name for it): the file's path,
/// or `None` when it isn't saved — while the page shows an extension's panel, for anything but a
/// web download, or with no Downloads folder.
pub fn requested<R: Runtime>(
    app: &AppHandle<R>,
    url: &Url,
    suggested: Option<&str>,
) -> Option<PathBuf> {
    // A navigation that became a download leaves the page where it was (WKWebView says nothing
    // about the page that stays): the toolbar goes back to its address.
    page_stays(app, url);
    let panel =
        extensions::session::committed().is_some() || extensions::session::showing().is_some();
    let decided = match folder(app) {
        Some(dir) => {
            let mut all = downloads();
            let decided = all.decide(url, &keys::app_origin(), panel, suggested, &dir, |path| {
                path.exists()
            });
            if let Ok(path) = &decided {
                all.started(url.clone(), path.clone());
            }
            decided
        }
        None => Err(Refusal::Folder),
    };
    match decided {
        Ok(path) => {
            let name = file_name(&path);
            if let Some(dir) = path.parent() {
                smoke::log("download-folder", &dir.to_string_lossy());
            }
            smoke::log("download-started", &name);
            tell(app, "downloading", name);
            Some(path)
        }
        Err(refusal) => {
            smoke::log("download-refused", refusal.as_str());
            let state = if refusal == Refusal::Panel {
                "blocked"
            } else {
                "failed"
            };
            tell(app, state, String::new());
            None
        }
    }
}

/// The app's navigation to `url` turned into a download: the page and its address stay as they
/// were (page_address.rs), and the toolbar says so.
fn page_stays<R: Runtime>(app: &AppHandle<R>, url: &Url) {
    if !super::address().download(url) {
        return;
    }
    if super::page_url(app).is_some() {
        super::publish_url(app);
        return;
    }
    // Nothing loaded before it (the window opened for the download): the start page's empty bar.
    *super::LAST_SHOWN
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = Some(None);
    if let Some(bar) = app.get_webview(super::BAR) {
        let _ = bar.eval("window.littleJoan && window.littleJoan.setUrl(null)");
    }
    smoke::log("bar-url", "start");
}

/// A download of `url` ended. Only one that started in this window counts, once (an engine may
/// report twice; one under way when the window closed was ended with it).
pub fn finished<R: Runtime>(app: &AppHandle<R>, url: &Url, outcome: Outcome) {
    let Some(done) = downloads().finished(url, outcome == Outcome::Saved) else {
        return;
    };
    let name = done.name();
    match outcome {
        Outcome::Saved => {
            smoke::log("download-finished", &format!("{name} ok"));
            tell(app, "saved", name);
        }
        Outcome::Failed => {
            smoke::log("download-finished", &format!("{name} failed"));
            tell(app, "failed", name);
        }
    }
}

/// The window is closing: what is under way ends with it, and nothing is kept. WebKitGTK's and
/// WKWebView's downloads are cancelled (WKWebView's would go on in WebKit's network process, CI
/// showed, and a partial file it leaves goes too); WebView2's go with the window's environment.
pub fn window_closed<R: Runtime>(app: &AppHandle<R>) {
    let open = downloads().closed();
    for download in &open {
        smoke::log("download-closed", &download.name());
    }
    status::forget();
    platform::cancel_downloads(app, open.into_iter().map(|d| d.path).collect());
}

/// The last file saved in this window ("Show in folder").
pub fn last_saved() -> Option<PathBuf> {
    downloads().last_saved().map(PathBuf::from)
}

fn file_name(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The OS Downloads folder (created if it doesn't exist yet), else `Downloads` at home.
fn folder<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    let paths = app.path();
    let dir = downloads::folder(paths.download_dir().ok(), paths.home_dir().ok())?;
    match std::fs::create_dir_all(&dir) {
        Ok(()) => Some(dir),
        Err(err) => {
            eprintln!(
                "little-joan: no Downloads folder at {}: {err}",
                dir.display()
            );
            None
        }
    }
}

fn tell<R: Runtime>(app: &AppHandle<R>, state: &'static str, name: String) {
    status::tell(app, &Status::Download { state, name });
}
