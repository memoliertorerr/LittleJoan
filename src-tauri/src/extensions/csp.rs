//! The Content Security Policy and the other headers every `ljext` response carries (pure,
//! tested).
//!
//! The policy allows the extension's own files and nothing else: no inline or `eval`'d script,
//! no frames, plugins, forms or `<base>`, and network access only to the `https://` hosts its
//! manifest declares (`network:<host>`). `frame-ancestors 'none'` (with `X-Frame-Options: DENY`)
//! keeps a web page from framing a panel, and `Cross-Origin-Resource-Policy: same-origin` from
//! embedding its files.

/// The CSP for an extension whose manifest declares `network_hosts` (already checked:
/// manifest::is_valid_network_host).
pub fn policy(network_hosts: &[&str]) -> String {
    let mut connect = String::from("'self'");
    for host in network_hosts {
        connect.push_str(" https://");
        connect.push_str(host);
    }
    format!(
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; \
         connect-src {connect}; frame-src 'none'; object-src 'none'; base-uri 'none'; \
         form-action 'none'; frame-ancestors 'none'"
    )
}

/// Browser features an extension panel never gets (Chromium honours this; WebKit ignores the
/// header, and its own prompts still apply).
pub const PERMISSIONS_POLICY: &str = "camera=(), microphone=(), geolocation=(), \
     clipboard-read=(), display-capture=(), usb=(), serial=(), hid=(), bluetooth=(), payment=()";

/// The referrer policy of every response. Not `no-referrer`: under it WebKit (and the Fetch
/// standard) send `Origin: null` with a same-origin POST, which the API must refuse — CI saw every
/// call of the panel's own refused on macOS and Linux. `same-origin` sends nothing to other
/// origins, so it leaks no more, and keeps the panel's own origin on its API calls.
pub const REFERRER_POLICY: &str = "same-origin";

/// The headers of every `ljext` response (files, the SDK, the API, refusals), given its CSP.
pub fn headers(policy: &str) -> [(&'static str, String); 7] {
    [
        ("Content-Security-Policy", policy.to_owned()),
        ("X-Content-Type-Options", "nosniff".to_owned()),
        ("X-Frame-Options", "DENY".to_owned()),
        ("Cross-Origin-Resource-Policy", "same-origin".to_owned()),
        ("Referrer-Policy", REFERRER_POLICY.to_owned()),
        ("Permissions-Policy", PERMISSIONS_POLICY.to_owned()),
        ("Cache-Control", "no-store".to_owned()),
    ]
}

/// The policy for responses that belong to no extension (refusals): nothing at all.
pub const NOTHING: &str = "default-src 'none'; frame-ancestors 'none'";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_network_hosts_only_the_extension_itself_is_reachable() {
        assert_eq!(
            policy(&[]),
            "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; \
             connect-src 'self'; frame-src 'none'; object-src 'none'; base-uri 'none'; \
             form-action 'none'; frame-ancestors 'none'"
        );
    }

    #[test]
    fn declared_hosts_are_added_to_connect_src_only_and_over_https() {
        let csp = policy(&["api.example.com", "*.example.org"]);
        assert!(
            csp.contains("; connect-src 'self' https://api.example.com https://*.example.org; ")
        );
        let directives: Vec<&str> = csp.split("; ").collect();
        for directive in &directives {
            if !directive.starts_with("connect-src") {
                assert!(!directive.contains("example"), "{directive}");
            }
        }
        // No inline script, eval, frames or plugins, whatever the manifest says.
        assert!(directives.contains(&"script-src 'self'"));
        assert!(!csp.contains("unsafe-eval"));
        assert!(directives.contains(&"frame-src 'none'"));
        assert!(directives.contains(&"object-src 'none'"));
        assert!(directives.contains(&"frame-ancestors 'none'"));
    }

    #[test]
    fn every_response_is_nosniff_unframeable_and_uncached() {
        let headers = headers(&policy(&[]));
        let get = |name: &str| {
            headers
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("X-Content-Type-Options"), Some("nosniff"));
        assert_eq!(get("X-Frame-Options"), Some("DENY"));
        assert_eq!(get("Cross-Origin-Resource-Policy"), Some("same-origin"));
        assert_eq!(get("Cache-Control"), Some("no-store"));
        // Never "no-referrer": it would make the engines send `Origin: null` to the API.
        assert_eq!(get("Referrer-Policy"), Some("same-origin"));
        assert!(get("Content-Security-Policy").is_some_and(|v| v.starts_with("default-src")));
        assert!(get("Permissions-Policy").is_some_and(|v| v.contains("camera=()")));
        // Never a CORS header: web pages can't read an extension's answers.
        assert!(
            headers
                .iter()
                .all(|(n, _)| !n.starts_with("Access-Control"))
        );
    }
}
