//! The once-a-second check of the page's address, for in-page changes that raise no load or
//! title event (history.pushState, #anchors): the only place the engine's address is read
//! (page_address.rs says when it may be).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime};

use super::{PAGE, address, publish_url};

/// Bumped for every new window, so an old address watcher stops.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Checks the page's address once a second while this window exists, for in-page changes that
/// raise no load or title event — the only place the engine's address is read, and only while
/// the last load event was `Finished` (page_address.rs). Stops when the window goes or a newer
/// one replaces it.
pub fn watch_url<R: Runtime>(app: &AppHandle<R>) {
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed) + 1;
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("little-joan-url-watch".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_secs(1));
                if GENERATION.load(Ordering::Relaxed) != generation
                    || app.get_window(crate::joan::LABEL).is_none()
                {
                    break;
                }
                let Some(token) = address().read_token() else {
                    continue;
                };
                let Some(page) = app.get_webview(PAGE) else {
                    break;
                };
                let Ok(url) = page.url() else {
                    continue;
                };
                // The lock is released before publish_url takes it again.
                let changed = address().observed(token, url);
                if changed {
                    publish_url(&app);
                }
            }
        });
    if let Err(err) = spawned {
        eprintln!("little-joan: could not start the address watcher: {err}");
    }
}
