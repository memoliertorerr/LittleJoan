//! Which files of an extension a path may name (pure, tested), and finding them on disk.
//!
//! A path is a list of plain names separated by `/`: letters, digits, `.`, `_` and `-`, not
//! starting with `.` (no hidden files) and not ending with `.`, at most 8 deep. That rules out
//! `..`, absolute paths, drive letters, backslashes, percent-encoding (so no encoded traversal
//! like `%2e%2e`), control characters and Windows device names (`CON`, `nul.txt`), without
//! decoding anything. The first name may not be `__lj`, which holds the SDK and the API. On
//! disk (unpacked extensions), the file's real path — symlinks followed — must still be inside
//! the extension's folder.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

/// The deepest path accepted.
pub const MAX_DEPTH: usize = 8;
/// The longest name accepted, in bytes.
pub const MAX_NAME: usize = 100;
/// The first name of the SDK's and the API's paths; no file may live there.
pub const RESERVED: &str = "__lj";

/// A checked path inside an extension: plain names, joined by `/`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelPath(String);

impl RelPath {
    /// The path, names joined by `/` (no leading `/`).
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Its names, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The last name's extension, lower case (`"html"`), if it has one.
    pub fn extension(&self) -> Option<String> {
        let last = self.names().last()?;
        let (_, ext) = last.rsplit_once('.')?;
        Some(ext.to_ascii_lowercase())
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a path was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    Empty,
    TooDeep,
    /// A name is empty (`a//b`), `.` / `..`, too long, has a character outside the allowed set,
    /// starts or ends with `.`, or is a Windows device name.
    BadName,
    /// The first name is `__lj`.
    Reserved,
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            PathError::Empty => "the path is empty",
            PathError::TooDeep => "the path is too deep",
            PathError::BadName => {
                "a name in the path is not a plain file name (letters, digits, . _ -)"
            }
            PathError::Reserved => "paths under __lj/ are Little Joan's",
        })
    }
}

/// Windows device names, which name a device whatever the folder or extension (`nul.txt`).
pub const DEVICES: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

fn is_plain_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_NAME || name.starts_with('.') || name.ends_with('.') {
        return false;
    }
    if !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    {
        return false;
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    !DEVICES.contains(&stem.as_str())
}

/// Checks `path` — a URL path (`/a/b.js`, as the request carries it, still percent-encoded) or a
/// manifest path (`a/b.js`). One leading `/` is allowed; everything else must be plain names.
pub fn clean(path: &str) -> Result<RelPath, PathError> {
    let path = path.strip_prefix('/').unwrap_or(path);
    if path.is_empty() {
        return Err(PathError::Empty);
    }
    let names: Vec<&str> = path.split('/').collect();
    if names.len() > MAX_DEPTH {
        return Err(PathError::TooDeep);
    }
    if !names.iter().all(|name| is_plain_name(name)) {
        return Err(PathError::BadName);
    }
    if names.first() == Some(&RESERVED) {
        return Err(PathError::Reserved);
    }
    Ok(RelPath(path.to_owned()))
}

/// The real file `rel` names inside `root` (an unpacked extension's folder): symlinks are
/// followed, and the result must still be inside the folder's own real path and be a file.
/// A symlink that leads out of the folder is refused (`PermissionDenied`).
pub fn on_disk(root: &Path, rel: &RelPath) -> io::Result<PathBuf> {
    let root = root.canonicalize()?;
    let joined = rel.names().fold(root.clone(), |dir, name| dir.join(name));
    let real = joined.canonicalize()?;
    if !real.starts_with(&root) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the file is outside the extension's folder",
        ));
    }
    if !real.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "not a file"));
    }
    Ok(real)
}

/// The MIME type served for `rel`, from its extension; `None` for anything else (it isn't
/// served at all).
pub fn mime_type(rel: &RelPath) -> Option<&'static str> {
    Some(match rel.extension()?.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "txt" | "md" => "text/plain; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn plain_paths_are_accepted() {
        for (path, want) in [
            ("/index.html", "index.html"),
            ("index.html", "index.html"),
            ("/img/icon-2_x.svg", "img/icon-2_x.svg"),
            ("a/b/c/d/e/f/g/h.js", "a/b/c/d/e/f/g/h.js"),
            ("/__ljx/a.js", "__ljx/a.js"),
            ("/lj/__lj.js", "lj/__lj.js"),
        ] {
            assert_eq!(clean(path).map(|p| p.0), Ok(want.to_owned()), "{path}");
        }
    }

    #[test]
    fn traversal_and_absolute_paths_are_refused() {
        for path in [
            "..",
            "/../secret",
            "a/../../b",
            "./a",
            "a/./b",
            "//etc/passwd",
            "/a//b",
            "a/",
            "C:/Windows/win.ini",
            "C:\\Windows",
            "a\\..\\b",
            "\\\\server\\share",
            "/a/b.",
            "/.git/config",
            "/.hidden",
            "a/.env",
        ] {
            assert_eq!(clean(path), Err(PathError::BadName), "{path}");
        }
        assert_eq!(clean(""), Err(PathError::Empty));
        assert_eq!(clean("/"), Err(PathError::Empty));
        assert_eq!(clean("a/b/c/d/e/f/g/h/i.js"), Err(PathError::TooDeep));
    }

    #[test]
    fn percent_encoding_is_never_decoded_into_traversal() {
        for path in [
            "/%2e%2e/secret",
            "/%2E%2E%2Fsecret",
            "/a%2f..%2fb",
            "/..%2fb",
            "/a%5c..%5cb",
            "/%252e%252e/x",
            "/index%2ehtml",
            "/a%00.js",
            "/a b.js",
            "/caf\u{e9}.js",
        ] {
            assert_eq!(clean(path), Err(PathError::BadName), "{path}");
        }
    }

    #[test]
    fn windows_device_names_and_the_reserved_folder_are_refused() {
        for path in ["con", "NUL.txt", "a/Com1.js", "lpt9.html", "aux"] {
            assert_eq!(clean(path), Err(PathError::BadName), "{path}");
        }
        assert!(clean("console.js").is_ok());
        assert_eq!(clean("/__lj/sdk/v1.js"), Err(PathError::Reserved));
        assert_eq!(clean("__lj"), Err(PathError::Reserved));
    }

    #[test]
    fn only_known_file_types_are_served() {
        let mime = |p: &str| mime_type(&clean(p).expect("clean"));
        assert_eq!(mime("index.html"), Some("text/html; charset=utf-8"));
        assert_eq!(mime("a/App.JS"), Some("text/javascript; charset=utf-8"));
        assert_eq!(mime("icon.svg"), Some("image/svg+xml"));
        assert_eq!(mime("tool.exe"), None);
        assert_eq!(mime("Makefile"), None);
        assert_eq!(mime("archive.tar.gz"), None);
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "little-joan-test-paths-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create");
        dir
    }

    #[test]
    fn files_are_found_inside_the_folder() {
        let dir = temp_dir("inside");
        let root = dir.join("hello");
        fs::create_dir_all(root.join("img")).expect("create");
        fs::write(root.join("img").join("a.svg"), "<svg/>").expect("write");
        let found = on_disk(&root, &clean("img/a.svg").expect("clean")).expect("found");
        assert!(found.ends_with("a.svg"));
        // A folder is not a file; a missing file is missing.
        assert!(on_disk(&root, &clean("img").expect("clean")).is_err());
        assert!(on_disk(&root, &clean("missing.js").expect("clean")).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_out_of_the_folder_are_refused() {
        use std::os::unix::fs::symlink;
        let dir = temp_dir("symlink");
        let root = dir.join("hello");
        fs::create_dir_all(&root).expect("create");
        fs::write(dir.join("secret.txt"), "secret").expect("write");
        fs::create_dir_all(dir.join("outside")).expect("create");
        fs::write(dir.join("outside").join("a.js"), "x").expect("write");
        fs::write(root.join("real.js"), "ok").expect("write");
        symlink(dir.join("secret.txt"), root.join("leak.txt")).expect("symlink");
        symlink(dir.join("outside"), root.join("out")).expect("symlink");
        symlink(root.join("real.js"), root.join("alias.js")).expect("symlink");
        for path in ["leak.txt", "out/a.js"] {
            let err = on_disk(&root, &clean(path).expect("clean")).expect_err(path);
            assert_eq!(err.kind(), io::ErrorKind::PermissionDenied, "{path}");
        }
        // A symlink that stays inside the folder is fine.
        assert!(on_disk(&root, &clean("alias.js").expect("clean")).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }
}
