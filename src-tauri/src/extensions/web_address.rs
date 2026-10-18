//! Web addresses an extension may send the page or the default browser to (`openUrl`, pure,
//! tested in api/tests.rs): absolute http(s) URLs with a public host — never an extension URL,
//! this computer or the local network.

use std::net::Ipv4Addr;

use url::{Host, Url};

use super::urls;

/// A web address an extension may open: an absolute http(s) URL with a public host — never an
/// extension URL in any form, and never this computer or the local network (`is_private_host`).
pub fn openable(text: &str) -> Option<Url> {
    let url = Url::parse(text).ok()?;
    (matches!(url.scheme(), "http" | "https")
        && url.has_host()
        && !urls::is_ext_like(&url)
        && !is_private_host(&url))
    .then_some(url)
}

fn private_v4(ip: Ipv4Addr) -> bool {
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.octets()[0] == 0
}

/// Loopback, private, link-local and unspecified addresses (127/8, 10/8, 172.16/12, 192.168/16,
/// 169.254/16, 0/8, ::1, ::, fc00::/7, fe80::/10, and IPv4 mapped into IPv6), `localhost` and
/// `*.localhost`, `*.local`: places a panel must not send the page (or the default browser) to.
pub fn is_private_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Ipv4(ip)) => private_v4(ip),
        Some(Host::Ipv6(ip)) => {
            let first = ip.segments()[0];
            ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || ip.to_ipv4_mapped().is_some_and(private_v4)
        }
        Some(Host::Domain(name)) => {
            let name = name.trim_end_matches('.').to_ascii_lowercase();
            name == "localhost"
                || name.ends_with(".localhost")
                || name == "local"
                || name.ends_with(".local")
        }
        None => true,
    }
}
