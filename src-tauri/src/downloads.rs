//! Downloads (LJ15), the pure part: which downloads Little Joan saves, the name a file gets —
//! cleaned, capped and unique in the Downloads folder — and the downloads under way in the open
//! window. `bar/download.rs` puts it to work for every engine (WebView2 and WKWebView through
//! Tauri's `on_download`, WebKitGTK through platform/linux.rs, see Landmines).
//!
//! - **What is saved:** web addresses (`http`, `https`) and the `blob:` / `data:` documents a web
//!   page makes — never an extension's URL in any form (`ljext:`, Windows' `http://ljext.*`), the
//!   app's own pages, `file:` or any other scheme. Nothing at all while the page shows an
//!   extension's panel: extensions have no permission to drop files into Downloads.
//! - **The name** comes from the engine's suggestion (the server's `Content-Disposition`, a link's
//!   `download` attribute), else the URL's last path segment, else `download`. It is cleaned — no
//!   path separators, none of the characters Windows refuses, no control or format characters
//!   (bidi overrides could disguise an extension), no leading dots (no hidden files, no `..`), no
//!   trailing dots or spaces, never a Windows device name (`nul.txt` becomes `_nul.txt`) — and
//!   capped at 200 bytes, keeping its extension.
//! - **Never overwrites:** a name that is taken — on disk, or by a download still under way —
//!   gets a number before its extension: `report (1).pdf`, `archive (1).tar.gz`.

use std::path::{Path, PathBuf};

use url::Url;

use crate::extensions::manifest::is_hidden_char;
use crate::extensions::paths::DEVICES;
use crate::extensions::urls;
use crate::keys;

/// The longest name a download gets, in bytes (before a ` (n)` that makes it unique).
pub const MAX_NAME_BYTES: usize = 200;
/// The name of a download nothing names.
pub const FALLBACK_NAME: &str = "download";
/// How many numbered names are tried before a download is refused.
pub const MAX_COPIES: u32 = 9_999;
/// The longest extension kept as one (`.torrent`, `.dmg`); longer dot parts are just text.
const MAX_EXTENSION: usize = 10;

/// Why a download wasn't saved (the smoke log's `download-refused <reason>`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The page shows an extension's panel.
    Panel,
    /// Not a web download (an extension's or the app's own URL, `file:`, another scheme).
    Scheme,
    /// No Downloads folder could be found or made.
    Folder,
    /// Every numbered name is taken.
    Name,
}

impl Refusal {
    pub fn as_str(self) -> &'static str {
        match self {
            Refusal::Panel => "panel",
            Refusal::Scheme => "scheme",
            Refusal::Folder => "folder",
            Refusal::Name => "name",
        }
    }
}

/// Whether a download of `url` may be saved: `http` / `https`, and `blob:` documents of such a
/// page, and `data:` documents — never an extension's URL or the app's own (`app_origin`).
pub fn accepts(url: &Url, app_origin: &Url) -> bool {
    match url.scheme() {
        "http" | "https" => is_web(url, app_origin),
        "data" => true,
        // `blob:<origin>/<uuid>`: the blob belongs to the page that made it.
        "blob" => Url::parse(url.path()).is_ok_and(|inner| {
            matches!(inner.scheme(), "http" | "https") && is_web(&inner, app_origin)
        }),
        _ => false,
    }
}

fn is_web(url: &Url, app_origin: &Url) -> bool {
    !urls::is_ext_like(url) && !keys::is_app_url(app_origin, url)
}

/// The name a download of `url` gets before it is made unique: the engine's suggestion, else the
/// URL's last path segment (web addresses only), else `download` — cleaned (`clean`).
pub fn file_name(suggested: Option<&str>, url: &Url) -> String {
    suggested
        .filter(|name| !is_data_payload(name, url))
        .and_then(clean)
        .or_else(|| last_segment(url).as_deref().and_then(clean))
        .unwrap_or_else(|| FALLBACK_NAME.to_owned())
}

/// WebKitGTK suggests a `data:` URL's payload as its name (`plain,hello` for
/// `data:text/plain,hello`): that is no name.
fn is_data_payload(name: &str, url: &Url) -> bool {
    url.scheme() == "data"
        && url
            .path()
            .split_once('/')
            .and_then(|(_, rest)| rest.split_once(','))
            .is_some_and(|(subtype, _)| name.starts_with(&format!("{subtype},")))
}

/// The last non-empty path segment of a web address, percent-decoded.
fn last_segment(url: &Url) -> Option<String> {
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let segment = url.path_segments()?.rev().find(|s| !s.is_empty())?;
    Some(percent_decode(segment))
}

/// `%XX` sequences to bytes; text that isn't UTF-8 afterwards is replaced, never trusted.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|pair| std::str::from_utf8(pair).ok())
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match (bytes[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `name` made safe as a file name in any folder on any OS (see the module docs), or `None` when
/// nothing is left of it.
pub fn clean(name: &str) -> Option<String> {
    let replaced: String = name
        .chars()
        .filter(|c| !is_hidden_char(*c))
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    let mut cleaned = trim(&replaced).to_owned();
    if cleaned.is_empty() {
        return None;
    }
    let stem = cleaned
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_ascii_lowercase();
    if DEVICES.contains(&stem.as_str()) {
        cleaned.insert(0, '_');
    }
    let capped = cap(&cleaned, MAX_NAME_BYTES);
    let capped = trim(&capped);
    (!capped.is_empty()).then(|| capped.to_owned())
}

/// No leading dots or white space (hidden files, `..`), no trailing ones (Windows drops them).
fn trim(name: &str) -> &str {
    name.trim_matches(|c: char| c == '.' || c.is_whitespace())
}

/// `name` shortened to at most `max` bytes at a character boundary, its extension kept.
fn cap(name: &str, max: usize) -> String {
    if name.len() <= max {
        return name.to_owned();
    }
    let (stem, extension) = split_extension(name);
    let (base, extension) = if extension.len() < max / 2 {
        (stem, extension)
    } else {
        (name, "")
    };
    let mut end = (max - extension.len()).min(base.len());
    while !base.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{extension}", &base[..end])
}

/// `name` split before its extension: the last dot part when it is short and plain
/// (`report.pdf` → `report`, `.pdf`), with a `.tar` before it (`archive.tar.gz` → `archive`,
/// `.tar.gz`); no extension otherwise (`notes`, `Chapter 1. Intro`).
pub fn split_extension(name: &str) -> (&str, &str) {
    let Some(dot) = name.rfind('.').filter(|dot| *dot > 0) else {
        return (name, "");
    };
    let extension = &name[dot + 1..];
    if extension.is_empty()
        || extension.len() > MAX_EXTENSION
        || !extension.chars().all(|c| c.is_ascii_alphanumeric())
    {
        return (name, "");
    }
    let stem = &name[..dot];
    let tar = stem.len().checked_sub(4).filter(|at| {
        *at > 0 && stem.is_char_boundary(*at) && stem[*at..].eq_ignore_ascii_case(".tar")
    });
    match tar {
        Some(at) => (&name[..at], &name[at..]),
        None => (stem, &name[dot..]),
    }
}

/// `name`, or the first of `stem (1).ext`, `stem (2).ext` … that isn't `taken`; `None` when
/// every one up to `MAX_COPIES` is.
pub fn unique(name: &str, taken: impl Fn(&str) -> bool) -> Option<String> {
    if !taken(name) {
        return Some(name.to_owned());
    }
    let (stem, extension) = split_extension(name);
    (1..=MAX_COPIES)
        .map(|n| format!("{stem} ({n}){extension}"))
        .find(|candidate| !taken(candidate))
}

/// The Downloads folder: the one the OS reports, else `Downloads` in the home folder (Linux
/// without XDG user dirs).
pub fn folder(reported: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    reported.or_else(|| home.map(|home| home.join("Downloads")))
}

/// A download under way: what it fetches and where it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    pub url: Url,
    pub path: PathBuf,
}

impl Download {
    /// The file's name, as the log and the toolbar show it.
    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

/// The open window's downloads: those under way (keyed by URL — macOS reports a finished
/// download by its URL only, Landmines) and the last one saved ("Show in folder").
#[derive(Debug, Default)]
pub struct Downloads {
    under_way: Vec<Download>,
    last_saved: Option<PathBuf>,
}

impl Downloads {
    pub const fn new() -> Self {
        Downloads {
            under_way: Vec::new(),
            last_saved: None,
        }
    }

    /// Where a download of `url` goes, given the engine's `suggested` name, the Downloads folder
    /// `dir`, whether the page shows an extension's panel, and whether a path `exists` on disk;
    /// or why it isn't saved. Names under way count as taken (they aren't on disk yet).
    pub fn decide(
        &self,
        url: &Url,
        app_origin: &Url,
        panel: bool,
        suggested: Option<&str>,
        dir: &Path,
        exists: impl Fn(&Path) -> bool,
    ) -> Result<PathBuf, Refusal> {
        if panel {
            return Err(Refusal::Panel);
        }
        if !accepts(url, app_origin) {
            return Err(Refusal::Scheme);
        }
        let name = file_name(suggested, url);
        let taken = |candidate: &str| {
            exists(&dir.join(candidate))
                || self.under_way.iter().any(|d| {
                    d.path.parent() == Some(dir) && d.name().eq_ignore_ascii_case(candidate)
                })
        };
        unique(&name, taken)
            .map(|name| dir.join(name))
            .ok_or(Refusal::Name)
    }

    /// A download of `url` to `path` started.
    pub fn started(&mut self, url: Url, path: PathBuf) {
        self.under_way.push(Download { url, path });
    }

    /// A download of `url` ended (`saved` or not): the earliest one under way for that URL, if
    /// any — an engine reporting it twice gets `None` the second time.
    pub fn finished(&mut self, url: &Url, saved: bool) -> Option<Download> {
        let at = self.under_way.iter().position(|d| &d.url == url)?;
        let done = self.under_way.remove(at);
        if saved {
            self.last_saved = Some(done.path.clone());
        }
        Some(done)
    }

    /// The last file saved in this window.
    pub fn last_saved(&self) -> Option<&Path> {
        self.last_saved.as_deref()
    }

    /// The window closed: the downloads still under way (which go with it), and nothing kept.
    pub fn closed(&mut self) -> Vec<Download> {
        self.last_saved = None;
        std::mem::take(&mut self.under_way)
    }
}

#[cfg(test)]
mod tests;
