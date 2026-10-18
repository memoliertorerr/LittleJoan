//! littleJoan.theme: the colors the bar shows right now, for panels to match.
//!
//! All theme math is TypeScript (ui/lib/theme.ts), so the bar — which computes the `--lj-*`
//! variables whenever it applies a theme — reports them (`set_theme_tokens`, the bar's
//! capability only) and they are kept here, in memory, while the page window exists. The values
//! come from our own bar, but they are still checked before an extension gets them: known
//! variable names, short CSS values without anything that could end a declaration.

use std::collections::BTreeMap;
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

/// At most this many variables.
pub const MAX_VARS: usize = 40;
/// The longest value (the grain's data: URI is the long one, about 1 KB).
pub const MAX_VALUE: usize = 8 * 1024;
/// How long a panel's request waits for the bar's first report.
pub const WAIT: Duration = Duration::from_secs(3);

/// What littleJoan.theme.get() returns: `scheme` ("light" or "dark", the system's choice
/// already applied) and the CSS variables (`--lj-bg-0` …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tokens {
    pub scheme: String,
    pub vars: BTreeMap<String, String>,
}

fn is_var_name(name: &str) -> bool {
    name.strip_prefix("--lj-").is_some_and(|rest| {
        (1..=24).contains(&rest.len())
            && rest
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    })
}

fn is_css_value(value: &str) -> bool {
    value.len() <= MAX_VALUE
        && !value.is_empty()
        && !value
            .chars()
            .any(|c| c.is_control() || matches!(c, ';' | '{' | '}' | '<' | '>' | '\\'))
}

/// Checks the bar's report: `{ scheme, vars }`. Anything off rejects the whole report.
pub fn parse(value: &Value) -> Option<Tokens> {
    let scheme = value.get("scheme")?.as_str()?;
    if !matches!(scheme, "light" | "dark") {
        return None;
    }
    let vars = value.get("vars")?.as_object()?;
    if vars.len() > MAX_VARS {
        return None;
    }
    let mut checked = BTreeMap::new();
    for (name, value) in vars {
        let value = value.as_str()?;
        if !is_var_name(name) || !is_css_value(value) {
            return None;
        }
        checked.insert(name.clone(), value.to_owned());
    }
    Some(Tokens {
        scheme: scheme.to_owned(),
        vars: checked,
    })
}

static TOKENS: Mutex<Option<Tokens>> = Mutex::new(None);
static REPORTED: Condvar = Condvar::new();

/// The bar applied a theme.
pub fn store(tokens: Tokens) {
    *TOKENS.lock().unwrap_or_else(PoisonError::into_inner) = Some(tokens);
    REPORTED.notify_all();
}

/// The current tokens; waits up to `timeout` for the bar's first report (a panel opened with
/// the window can ask before the bar has loaded).
pub fn current(timeout: Duration) -> Option<Tokens> {
    let guard = TOKENS.lock().unwrap_or_else(PoisonError::into_inner);
    let (guard, _) = REPORTED
        .wait_timeout_while(guard, timeout, |tokens| tokens.is_none())
        .unwrap_or_else(PoisonError::into_inner);
    guard.clone()
}

/// The window closed: nothing kept while hibernating.
pub fn forget() {
    *TOKENS.lock().unwrap_or_else(PoisonError::into_inner) = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_bars_report_is_checked() {
        let good = json!({ "scheme": "dark", "vars": {
            "--lj-bg-0": "#1f2430",
            "--lj-gradient": "radial-gradient(120% 90% at 100% 0%, rgba(1, 2, 3, 0.6) 0%, transparent 70%)",
            "--lj-grain": "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'%3E%3C/svg%3E\")"
        }});
        let tokens = parse(&good).expect("valid");
        assert_eq!(tokens.scheme, "dark");
        assert_eq!(tokens.vars.len(), 3);
        for bad in [
            json!({ "scheme": "dim", "vars": {} }),
            json!({ "vars": {} }),
            json!({ "scheme": "light" }),
            json!({ "scheme": "light", "vars": { "color": "red" } }),
            json!({ "scheme": "light", "vars": { "--lj-Bg": "red" } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": "red; background: url(x)" } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": "</style>" } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": "a\nb" } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": "" } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": 3 } }),
            json!({ "scheme": "light", "vars": { "--lj-bg-0": "x".repeat(MAX_VALUE + 1) } }),
        ] {
            assert_eq!(parse(&bad), None, "{bad}");
        }
        let many: serde_json::Map<String, Value> = (0..=MAX_VARS)
            .map(|i| (format!("--lj-v{i}"), json!("red")))
            .collect();
        assert_eq!(parse(&json!({ "scheme": "light", "vars": many })), None);
    }
}
