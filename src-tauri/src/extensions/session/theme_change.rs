//! A theme change reaches an open panel (LJ56).
//!
//! `littleJoan.theme.get()` is a snapshot (LJ29). When the bar reports new colors
//! (`set_theme_tokens`) and the page shows a panel that a load event **committed**
//! (`NavGate::committed`), the app runs one script in the page webview: it dispatches
//! `littlejoan:themechange` on the panel's window — an event with no data. The panel's SDK
//! (`littleJoan.theme.onChange`) then asks `theme` again, through the checked API like any call.
//!
//! The check is made right before the eval, and only a committed panel passes it: never a web
//! page, never a panel that hasn't committed yet (it asks for the theme as it starts). The eval
//! itself runs later, on the main thread, in whatever document the page shows then; a web page
//! that received the event in that race, or dispatched it on itself, would gain nothing — it
//! carries no data and opens no door. Nothing is added to the page webview: no IPC, no
//! capability, no initialization script (Ground Rule 18).

use tauri::{AppHandle, Runtime};

use crate::bar;
use crate::smoke;

/// What the page runs: the event (`littlejoan:themechange`, the name the SDK listens for —
/// extensions/sdk/v1.js), nothing in it.
pub const SCRIPT: &str = r#"window.dispatchEvent(new Event("littlejoan:themechange"))"#;

/// Tells the panel the page committed (`committed`, `NavGate::committed` read right before) that
/// the theme changed: `eval` runs `SCRIPT` in the page and says whether it could. With no
/// committed panel — a web page, the start page, a panel still loading — `eval` isn't called.
/// Returns the extension told.
pub fn tell(committed: Option<String>, eval: impl FnOnce(&'static str) -> bool) -> Option<String> {
    let id = committed?;
    eval(SCRIPT).then_some(id)
}

/// The bar reported new colors (`set_theme_tokens`): the panel the page committed, if it shows
/// one, is told — `panel-theme-changed <id>` in the smoke log.
pub fn theme_changed<R: Runtime>(app: &AppHandle<R>) {
    let Some(page) = bar::page(app) else {
        return;
    };
    let told = tell(super::committed(), |script| match page.eval(script) {
        Ok(()) => true,
        Err(err) => {
            eprintln!("little-joan: could not tell the panel about the new theme: {err}");
            false
        }
    });
    if let Some(id) = told {
        smoke::log("panel-theme-changed", &id);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::time::Instant;

    use url::Url;

    use super::*;
    use crate::extensions::gate::{NavGate, page_may_navigate};
    use crate::keys::Platform;

    fn url(s: &str) -> Url {
        Url::parse(s).expect("valid")
    }

    /// What `tell` evaluated for the page `gate` describes, and whom it told.
    fn told(gate: &NavGate) -> (Vec<&'static str>, Option<String>) {
        let evals = RefCell::new(Vec::new());
        let id = tell(gate.committed().map(str::to_owned), |script| {
            evals.borrow_mut().push(script);
            true
        });
        (evals.into_inner(), id)
    }

    #[test]
    fn only_a_committed_panel_is_told_and_the_event_carries_nothing() {
        let now = Instant::now();
        let mut gate = NavGate::new();
        let panel = url("ljext://snake.localhost/index.html?lj-open=0123456789abcdef");
        let nothing: (Vec<&str>, Option<String>) = (Vec::new(), None);

        // A web page is never told.
        let web = url("https://example.com/");
        assert!(page_may_navigate(&web, &mut gate, now, Platform::MacOs));
        gate.commit(&web, Platform::MacOs);
        assert_eq!(told(&gate), nothing, "a web page");

        // A panel the guard admitted but the page hasn't committed: not told (it asks as it
        // starts).
        gate.expect(&panel, now);
        assert!(page_may_navigate(&panel, &mut gate, now, Platform::MacOs));
        assert_eq!(gate.showing(), Some("snake"));
        assert_eq!(told(&gate), nothing, "an uncommitted panel");

        // Committed: told, with the fixed script — a bare event, no data in it.
        assert!(gate.commit(&panel, Platform::MacOs));
        assert_eq!(told(&gate), (vec![SCRIPT], Some("snake".to_owned())));
        assert_eq!(
            SCRIPT,
            "window.dispatchEvent(new Event(\"littlejoan:themechange\"))"
        );
        // The SDK served to panels listens for that very name.
        assert!(
            crate::extensions::protocol::SDK
                .contains(r#"const THEME_CHANGE = "littlejoan:themechange";"#)
        );

        // Leaving for a web page: from the guard's decision on, nobody is told.
        let next = url("https://example.org/");
        assert!(page_may_navigate(&next, &mut gate, now, Platform::MacOs));
        gate.commit(&next, Platform::MacOs);
        assert_eq!(told(&gate), nothing, "the web page after the panel");

        // Windows' form of a panel URL, committed: told the same way.
        let mut gate = NavGate::new();
        let windows = url("http://ljext.qr.localhost/index.html?lj-open=00ff");
        gate.expect(&windows, now);
        assert!(page_may_navigate(
            &windows,
            &mut gate,
            now,
            Platform::Windows
        ));
        assert_eq!(told(&gate), nothing, "uncommitted on Windows");
        assert!(gate.commit(&windows, Platform::Windows));
        assert_eq!(told(&gate), (vec![SCRIPT], Some("qr".to_owned())));
    }

    #[test]
    fn an_eval_that_failed_tells_nobody() {
        assert_eq!(tell(Some("snake".to_owned()), |_| false), None);
        assert_eq!(tell(None, |_| unreachable!("no panel, no eval")), None);
    }
}
