//! A popup's title (LJ47; the security review of phase 22): where it is, never the page's own — so
//! you always see whose page you sign in on. Pure, tested in popups/tests.rs.
//!
//! **Only what committed:** a popup is built titled by its opener (`opener_title`) — before its
//! first page commits, the only script that can run in it is the opener's (it can `stop()` the
//! load and `document.write` into the blank page) —, and its title changes only when a page
//! commits there (`title_at`): never to the host it was asked to open, which may never load. A
//! blank or `srcdoc` document is the opener's too (it inherits the opener's origin).
//!
//! **Long hosts:** a popup can be 320 px wide, and Windows and GTK cut a long title at its end —
//! where a host says whose it is (`accounts.google.com.verify-session-0123.evil.example`). So a
//! host longer than `HOST_MAX` keeps its end: whole labels from the right, at least the last two,
//! and `…` before them (`…evil.example`). Whole labels, not characters: a cut inside a label could
//! show a label that isn't there. No public-suffix list: the registrable domain is always at the
//! end of the host, so keeping the end keeps it (a cut never shows less than the last two labels,
//! and a single label longer than the room keeps its own end).

use url::{Origin, Url};

/// The longest host a title shows whole, in characters.
pub const HOST_MAX: usize = 30;

/// The title for a page at `url`: the host of `https` pages (with a port that isn't 443), the
/// scheme too for anything else (`http://127.0.0.1:8123`) — a long domain keeps its end
/// (`short_host`).
pub fn title_for(url: &Url) -> String {
    let host = url.host_str().map(|host| {
        let host = if url.domain().is_some() {
            short_host(host)
        } else {
            host.to_owned()
        };
        match url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host,
        }
    });
    match (url.scheme(), host) {
        ("https", Some(host)) => host,
        (scheme, Some(host)) => format!("{scheme}://{host}"),
        ("about", None) => format!("about:{}", url.path()),
        (scheme, None) => format!("{scheme}:"),
    }
}

/// The title a popup is built with: its opener's place.
pub fn opener_title(opener: &Origin) -> String {
    match Url::parse(&opener.ascii_serialization()) {
        Ok(url) => title_for(&url),
        Err(_) => opener.ascii_serialization(),
    }
}

/// The title once a document committed at `url` in a popup opened by `opener`: a blank or
/// `srcdoc` document is the opener's place, anything else its own.
pub fn title_at(url: &Url, opener: &Origin) -> String {
    if url.scheme() == "about" {
        opener_title(opener)
    } else {
        title_for(url)
    }
}

/// `host` as a title shows it: whole up to `HOST_MAX` characters, else its end (the module's
/// docs).
pub fn short_host(host: &str) -> String {
    if host.chars().count() <= HOST_MAX {
        return host.to_owned();
    }
    // Room for the labels after the "…".
    let room = HOST_MAX - 1;
    let labels: Vec<&str> = host.split('.').collect();
    let mut start = labels.len();
    let mut length = 0;
    while start > 0 {
        let label = labels[start - 1].chars().count();
        let with = if start == labels.len() {
            label
        } else {
            length + 1 + label
        };
        // The last two labels always stay (cut below if even they don't fit).
        if with > room && labels.len() - start >= 2 {
            break;
        }
        length = with;
        start -= 1;
    }
    let tail = labels[start..].join(".");
    let count = tail.chars().count();
    if count > room {
        let end: String = tail.chars().skip(count - room).collect();
        format!("…{end}")
    } else {
        format!("…{tail}")
    }
}
