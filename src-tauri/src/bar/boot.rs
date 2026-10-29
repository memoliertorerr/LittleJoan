//! The bar's first paint (LJ38), per window:
//! - **its theme before it paints:** the bar's webview — and only it, never the page's (Ground
//!   Rules 10 and 18) — gets an initialization script, `window.__littleJoanTheme = <theme>;`
//!   (`theme_script`), which the engine runs at document start, before the page's own scripts (so
//!   the bar's CSP doesn't apply to it); ui/bar-boot.ts applies it before anything awaits. The
//!   theme is a checked one — `#rrggbb` colors, a mode name and two numbers — so its JSON is a
//!   safe script literal. The bar's first theme report then says `early` (commands.rs logs
//!   `theme-early`);
//! - **whether the bar's script runs yet** (`start_info`, its first call, logs `bar-ready`): a panel
//!   opened from hibernation waits for it (extensions/session.rs), so the toolbar is there to name
//!   it (on macOS a panel committed before the bar's script ran left the toolbar unpainted).
//!
//! Its webview's own background is background.rs's.

use std::sync::{Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager, Runtime};

use super::BAR;
use crate::settings::{self, Theme};
use crate::smoke;

/// The initialization script that hands the bar `theme` before its first paint. The theme is
/// checked again here, so whatever reaches it, the script is one assignment of plain JSON.
pub fn theme_script(theme: &Theme) -> String {
    let checked = serde_json::to_value(theme)
        .map(|value| settings::theme::parse(Some(&value)))
        .unwrap_or_default();
    let json = serde_json::to_string(&checked).unwrap_or_else(|_| "null".to_owned());
    format!("window.__littleJoanTheme = {json};")
}

/// Shows `theme` in the open window's bar at once (after Settings saves one, or previews one).
pub fn set_theme<R: Runtime>(app: &AppHandle<R>, theme: &Theme) {
    let Some(bar) = app.get_webview(BAR) else {
        return;
    };
    // Checked hex colors, a mode name and two numbers: the JSON is a safe script literal.
    match serde_json::to_string(theme) {
        Ok(json) => {
            let _ = bar.eval(format!(
                "window.littleJoan && window.littleJoan.setTheme({json})"
            ));
        }
        Err(err) => eprintln!("little-joan: could not send the theme to the bar: {err}"),
    }
}

/// The theme this window's bar got in its initialization script (for `theme-early`).
static EARLY: Mutex<Option<Theme>> = Mutex::new(None);

/// A new window's bar is built with `theme` in its initialization script.
pub fn set_early_theme(theme: Theme) {
    *EARLY.lock().unwrap_or_else(PoisonError::into_inner) = Some(theme);
}

/// The theme of this window's initialization script.
pub fn early_theme() -> Option<Theme> {
    EARLY.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// Whether something happened yet — here, whether this window's bar has run its script — and a
/// wait for it with a time limit.
#[derive(Debug, Default)]
pub struct Ready {
    ready: Mutex<bool>,
    changed: Condvar,
}

impl Ready {
    pub const fn new() -> Self {
        Ready {
            ready: Mutex::new(false),
            changed: Condvar::new(),
        }
    }

    /// Not yet (a new window).
    pub fn reset(&self) {
        *self.ready.lock().unwrap_or_else(PoisonError::into_inner) = false;
    }

    /// It happened: wakes every waiter.
    pub fn set(&self) {
        *self.ready.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.changed.notify_all();
    }

    pub fn is_set(&self) -> bool {
        *self.ready.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Blocks the calling thread until it happened or `limit` passed; true if it happened. Never
    /// call it on the main thread or inside an engine callback.
    pub fn wait(&self, limit: Duration) -> bool {
        let deadline = Instant::now() + limit;
        let mut ready = self.ready.lock().unwrap_or_else(PoisonError::into_inner);
        while !*ready {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            ready = self
                .changed
                .wait_timeout(ready, left)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        true
    }
}

/// This window's bar: has its script run (`start_info`)?
pub static BAR_READY: Ready = Ready::new();

/// The bar's script called `start_info`: it runs, and panels may open.
pub fn mark_ready() {
    if !BAR_READY.is_set() {
        smoke::log("bar-ready", "");
    }
    BAR_READY.set();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::ThemeMode;

    #[test]
    fn the_theme_script_is_one_assignment_of_a_checked_theme() {
        let theme = Theme {
            mode: ThemeMode::Dark,
            colors: vec!["#9d8fd0".into(), "#e0a2a0".into()],
            intensity: 0.6,
            grain: 0.4,
        };
        assert_eq!(
            theme_script(&theme),
            r##"window.__littleJoanTheme = {"mode":"dark","colors":["#9d8fd0","#e0a2a0"],"intensity":0.6,"grain":0.4};"##
        );
    }

    #[test]
    fn extremes_and_junk_stay_plain_json() {
        let hostile = Theme {
            mode: ThemeMode::Light,
            colors: vec![
                "</script><script>alert(1)</script>".into(),
                "#FFFFFF\n;window.x=1".into(),
                "#000000".into(),
                "#abcdef".into(),
            ],
            intensity: f64::INFINITY,
            grain: -3.0,
        };
        for theme in [
            hostile,
            Theme::default(),
            Theme {
                intensity: 0.0,
                grain: 1.0,
                ..Theme::default()
            },
        ] {
            let script = theme_script(&theme);
            assert!(!script.contains('<'), "{script}");
            assert!(!script.contains('\n') && !script.contains('\r'), "{script}");
            assert_eq!(script.matches(';').count(), 1, "one statement: {script}");
            assert!(script.starts_with("window.__littleJoanTheme = {") && script.ends_with("};"));
        }
        let checked = theme_script(&Theme {
            colors: vec!["</script>".into(), "#ABC".into()],
            intensity: 7.0,
            ..Theme::default()
        });
        assert!(checked.contains(r##""colors":["#aabbcc"]"##), "{checked}");
        assert!(checked.contains(r#""intensity":1.0"#), "{checked}");
    }

    #[test]
    fn only_the_bars_builder_adds_an_initialization_script() {
        // The page webview (joan-page) never gets one (Ground Rules 10 and 18): in bar.rs's code
        // there is exactly one, in the bar's builder; the page's builder (bar/page.rs since
        // LJ19.4) has none — the walk below checks that file with every other.
        let source = include_str!("../bar.rs");
        let code = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert_eq!(code.matches("initialization_script").count(), 1);
        let call = code
            .find(".initialization_script(")
            .expect("the bar's script");
        let bar = code
            .find("WebviewBuilder::new(BAR")
            .expect("the bar's builder");
        assert!(bar < call, "the script belongs to the bar's builder");
        assert!(!code.contains("WebviewBuilder::new(PAGE"));
        let page = include_str!("page.rs");
        assert!(
            page.contains("WebviewBuilder::new(PAGE"),
            "the page's builder lives in bar/page.rs"
        );
        // And nowhere else but the settings window's own (its tab).
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read src") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if matches!(name, "bar.rs" | "settings_window.rs") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                let code = text.split("#[cfg(test)]").next().unwrap_or(&text);
                assert!(
                    !code.contains("initialization_script"),
                    "{}: no other webview gets an initialization script",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn ready_wakes_a_waiter_and_times_out_without() {
        let ready = Ready::new();
        let started = Instant::now();
        assert!(!ready.wait(Duration::from_millis(80)));
        assert!(started.elapsed() >= Duration::from_millis(80));
        ready.set();
        assert!(ready.wait(Duration::from_millis(0)));
        ready.reset();
        assert!(!ready.is_set());
    }
}
