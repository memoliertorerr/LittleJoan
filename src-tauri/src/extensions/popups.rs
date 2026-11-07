//! New windows an extension's panel asks for (phase 21's security review). A panel runs in the page
//! webview, so its `window.open` and `target=_blank` reach the same new-window handler as a web
//! page's — and would skip what `openUrl(url, { where: "browser" })` checks. So while the page shows
//! a panel, its new windows get exactly those checks, in that order: the extension is known and on,
//! has the `openBrowser` permission, the address is public web (never this computer, the local
//! network or an extension), and one every 2 s (shared with `openUrl`). Anything else is refused
//! (`handoff-refused <why>` in the smoke log). Pure decision, tested below.

use tauri::{AppHandle, Runtime};
use url::Url;

use super::manifest::Permission;
use super::{limits, registry, web_address};

/// Why a panel's new window doesn't open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// The extension isn't installed, or is switched off.
    Unknown,
    /// It lacks the `openBrowser` permission.
    Permission,
    /// Not a public web address.
    NotPublic,
    /// Another one less than 2 s ago.
    Busy,
}

impl Refused {
    pub fn name(self) -> &'static str {
        match self {
            Refused::Unknown => "unknown-extension",
            Refused::Permission => "permission",
            Refused::NotPublic => "not-public",
            Refused::Busy => "busy",
        }
    }
}

/// `openUrl(where: "browser")`'s checks, in its order; `rate_ok` is asked last (it counts).
pub fn decide(
    known: bool,
    open_browser: bool,
    url: &Url,
    rate_ok: impl FnOnce() -> bool,
) -> Result<Url, Refused> {
    if !known {
        return Err(Refused::Unknown);
    }
    if !open_browser {
        return Err(Refused::Permission);
    }
    let url = web_address::openable(url.as_str()).ok_or(Refused::NotPublic)?;
    if !rate_ok() {
        return Err(Refused::Busy);
    }
    Ok(url)
}

/// The checks for the extension `id`, whose panel the page shows.
pub fn check<R: Runtime>(app: &AppHandle<R>, id: &str, url: &Url) -> Result<Url, Refused> {
    let extension = registry::current(app).get(id);
    let open_browser = extension
        .as_ref()
        .is_some_and(|e| e.manifest.has(&Permission::OpenBrowser));
    decide(extension.is_some(), open_browser, url, || {
        limits::browser_open(id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    #[test]
    fn a_panel_needs_open_browser_and_a_public_address() {
        let public = url("https://example.com/a");
        assert_eq!(decide(true, true, &public, || true), Ok(public.clone()));
        assert_eq!(decide(false, true, &public, || true), Err(Refused::Unknown));
        assert_eq!(
            decide(true, false, &public, || true),
            Err(Refused::Permission)
        );
        for private in [
            "http://127.0.0.1:8123/smoke.html",
            "http://localhost:3000/",
            "http://192.168.1.1/",
            "http://ljext.hello.localhost/index.html",
            "http://printer.local/",
        ] {
            assert_eq!(
                decide(true, true, &url(private), || true),
                Err(Refused::NotPublic),
                "{private}"
            );
        }
        assert_eq!(decide(true, true, &public, || false), Err(Refused::Busy));
    }

    #[test]
    fn the_rate_is_counted_only_for_a_window_that_would_open() {
        let mut asked = 0;
        let _ = decide(true, false, &url("https://example.com/"), || {
            asked += 1;
            true
        });
        let _ = decide(true, true, &url("http://127.0.0.1/"), || {
            asked += 1;
            true
        });
        assert_eq!(asked, 0);
    }
}
