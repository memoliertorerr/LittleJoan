//! Windows' phone agent with client hints (LJ40, pure). A bare custom agent
//! (`ICoreWebView2Settings2::SetUserAgent`) turns Chromium's user-agent client hints off or leaves
//! them saying desktop, so a site saw an Android agent with `Sec-CH-UA-Mobile: ?0` (or none).
//! The DevTools protocol's `Emulation.setUserAgentOverride` takes the agent together with its
//! metadata — Android, mobile, the engine's own brands and versions — so the header, the hints
//! and `navigator.userAgentData` agree. Going back to desktop clears the override (an empty
//! `userAgent`, as Playwright does): WebView2's own agent with its own metadata again, exactly.
//! platform/win.rs sends these; this module builds them and reads `Page.getLayoutMetrics`.

use serde_json::{Value, json};

use super::{FALLBACK_CHROME_MAJOR, chrome_major, mobile_user_agent};
use crate::keys::Platform;

/// The brand Chromium adds to confuse brand sniffing ("GREASE"); any such name works.
const GREASE: (&str, &str, &str) = ("Not_A Brand", "99", "99.0.0.0");

/// The engine's full version: the leading `<digits>(.<digits>)*` of WebView2's version string
/// (`ICoreWebView2Environment::BrowserVersionString`, e.g. "142.0.3595.80"), else
/// `<major>.0.0.0`.
pub fn full_version(version: Option<&str>, major: u32) -> String {
    let leading: String = version
        .unwrap_or_default()
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let leading = leading.trim_end_matches('.');
    let fits = leading
        .split('.')
        .next()
        .and_then(|first| first.parse::<u32>().ok())
        == Some(major);
    if fits && leading.split('.').all(|part| !part.is_empty()) {
        leading.to_owned()
    } else {
        format!("{major}.0.0.0")
    }
}

/// `Emulation.setUserAgentOverride`'s parameters. Phone (`mobile`): Chrome on Android with the
/// engine's Chrome version (from `engine_ua`, WebView2's own agent, and `version`, its full
/// version) and metadata to match — platform Android, mobile, the brands and full versions.
/// Desktop: `{ "userAgent": "" }`, which clears the override.
pub fn override_params(mobile: bool, engine_ua: Option<&str>, version: Option<&str>) -> Value {
    if !mobile {
        return json!({ "userAgent": "" });
    }
    let major = engine_ua
        .and_then(chrome_major)
        .unwrap_or(FALLBACK_CHROME_MAJOR);
    let full = full_version(version, major);
    let major = major.to_string();
    let brands = json!([
        { "brand": "Chromium", "version": major },
        { "brand": "Google Chrome", "version": major },
        { "brand": GREASE.0, "version": GREASE.1 },
    ]);
    let full_versions = json!([
        { "brand": "Chromium", "version": full },
        { "brand": "Google Chrome", "version": full },
        { "brand": GREASE.0, "version": GREASE.2 },
    ]);
    json!({
        "userAgent": mobile_user_agent(Platform::Windows, engine_ua),
        // navigator.platform, as Chrome on an Android phone says it.
        "platform": "Linux armv8l",
        "userAgentMetadata": {
            "brands": brands,
            "fullVersionList": full_versions,
            "fullVersion": full,
            "platform": "Android",
            "platformVersion": "10.0.0",
            "architecture": "",
            "model": "",
            "mobile": true,
            "bitness": "",
            "wow64": false,
        },
    })
}

/// Windows: whether a reload for the phone agent can be skipped because the page already fits
/// the narrower window — its content (`cssContentSize` width) is no wider than the viewport
/// (`cssLayoutViewport` clientWidth), so it is responsive and keeps working as it is. Only asked
/// when switching to the phone agent: a phone page always "fits" a wider window, and going wide
/// must bring the desktop version back.
pub fn skip_reload(css_content_width: f64, viewport_width: f64) -> bool {
    css_content_width.is_finite()
        && viewport_width.is_finite()
        && css_content_width > 0.0
        && viewport_width > 0.0
        && css_content_width <= viewport_width + 1.0
}

/// `Page.getLayoutMetrics`' answer → (the content's CSS width, the layout viewport's CSS width).
pub fn layout_widths(answer: &str) -> Option<(f64, f64)> {
    let value: Value = serde_json::from_str(answer).ok()?;
    let content = value.get("cssContentSize")?.get("width")?.as_f64()?;
    let viewport = value
        .get("cssLayoutViewport")?
        .get("clientWidth")?
        .as_f64()?;
    Some((content, viewport))
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBVIEW2: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) \
                            Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0";

    #[test]
    fn the_phone_agent_carries_android_mobile_metadata_from_the_engine() {
        let params = override_params(true, Some(WEBVIEW2), Some("142.0.3595.80"));
        assert_eq!(
            params["userAgent"],
            "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) \
             Chrome/142.0.0.0 Mobile Safari/537.36"
        );
        let meta = &params["userAgentMetadata"];
        assert_eq!(meta["platform"], "Android");
        assert_eq!(meta["mobile"], true);
        assert_eq!(meta["fullVersion"], "142.0.3595.80");
        let brands = meta["brands"].as_array().expect("brands");
        assert!(
            brands
                .iter()
                .any(|b| b["brand"] == "Chromium" && b["version"] == "142")
        );
        assert!(
            brands
                .iter()
                .any(|b| b["brand"] == "Google Chrome" && b["version"] == "142")
        );
        let full = meta["fullVersionList"].as_array().expect("full versions");
        assert!(
            full.iter()
                .any(|b| b["brand"] == "Chromium" && b["version"] == "142.0.3595.80")
        );
        assert_eq!(brands.len(), full.len());
        assert_eq!(params["platform"], "Linux armv8l");
    }

    #[test]
    fn without_the_engines_versions_the_fallback_major() {
        let params = override_params(true, None, None);
        let major = FALLBACK_CHROME_MAJOR.to_string();
        assert!(
            params["userAgent"]
                .as_str()
                .is_some_and(|ua| ua.contains(&format!("Chrome/{major}.0.0.0 Mobile")))
        );
        assert_eq!(
            params["userAgentMetadata"]["fullVersion"],
            format!("{major}.0.0.0")
        );
    }

    #[test]
    fn desktop_clears_the_override() {
        for engine in [Some(WEBVIEW2), None] {
            assert_eq!(
                override_params(false, engine, Some("142.0.1.2")),
                json!({ "userAgent": "" })
            );
        }
    }

    #[test]
    fn the_full_version_must_match_the_major() {
        assert_eq!(full_version(Some("142.0.3595.80"), 142), "142.0.3595.80");
        assert_eq!(
            full_version(Some(" 142.0.3595.80 beta"), 142),
            "142.0.3595.80"
        );
        assert_eq!(full_version(Some("141.0.1.1"), 142), "142.0.0.0");
        assert_eq!(full_version(Some("garbage"), 142), "142.0.0.0");
        assert_eq!(full_version(Some("142..1"), 142), "142.0.0.0");
        assert_eq!(full_version(None, 99), "99.0.0.0");
    }

    #[test]
    fn a_page_that_fits_needs_no_reload() {
        assert!(skip_reload(390.0, 390.0));
        assert!(skip_reload(389.5, 390.0));
        assert!(skip_reload(390.8, 390.0), "rounding");
        assert!(
            !skip_reload(600.0, 390.0),
            "a desktop page wider than the window"
        );
        assert!(!skip_reload(0.0, 390.0));
        assert!(!skip_reload(f64::NAN, 390.0));
        assert!(!skip_reload(390.0, 0.0));
    }

    #[test]
    fn layout_metrics_are_read() {
        let answer = r#"{"layoutViewport":{"pageX":0,"pageY":0,"clientWidth":390,"clientHeight":560},
            "cssLayoutViewport":{"pageX":0,"pageY":0,"clientWidth":390,"clientHeight":560},
            "cssContentSize":{"x":0,"y":0,"width":600,"height":560},"contentSize":{"x":0,"y":0,"width":600,"height":560}}"#;
        assert_eq!(layout_widths(answer), Some((600.0, 390.0)));
        assert_eq!(layout_widths("{}"), None);
        assert_eq!(layout_widths("not json"), None);
    }
}
