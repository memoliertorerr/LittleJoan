//! littleJoan.storage: each extension's own key–value store, `<config dir>/extension-data/<id>.json`
//! (pure operations, tested, plus the file).
//!
//! Keys are 1 to 128 characters; values are any JSON. The whole store, as JSON, may be at most
//! 1 MB; a write that would pass that is refused and changes nothing. The file is read for every
//! call and written atomically (temp file + rename), so nothing is kept in memory, and one lock
//! serializes every read-modify-write. A missing or unreadable file is an empty store.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde_json::{Value, json};

/// The folder, inside the config folder.
pub const DATA_DIR: &str = "extension-data";
/// The most a store may hold: its JSON, in bytes.
pub const QUOTA_BYTES: usize = 1024 * 1024;
/// The longest key, in characters.
pub const MAX_KEY_CHARS: usize = 128;

/// One lock for every extension's file: no two calls change a store at once.
static FILES: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageError {
    /// The key is empty or longer than 128 characters.
    BadKey,
    /// The store would pass 1 MB.
    Quota,
    Io(String),
}

impl StorageError {
    pub fn code(&self) -> &'static str {
        match self {
            StorageError::BadKey => "bad-key",
            StorageError::Quota => "quota",
            StorageError::Io(_) => "io",
        }
    }

    pub fn message(&self) -> String {
        match self {
            StorageError::BadKey => format!("Keys are 1 to {MAX_KEY_CHARS} characters."),
            StorageError::Quota => "The extension's storage is full (1 MB).".to_owned(),
            StorageError::Io(err) => format!("The storage couldn't be saved: {err}"),
        }
    }
}

/// An extension's items, as stored.
pub type Items = BTreeMap<String, Value>;

pub fn check_key(key: &str) -> Result<(), StorageError> {
    let chars = key.chars().count();
    if chars == 0 || chars > MAX_KEY_CHARS {
        Err(StorageError::BadKey)
    } else {
        Ok(())
    }
}

/// The size the store's file would have.
pub fn size(items: &Items) -> usize {
    serde_json::to_vec(items).map_or(usize::MAX, |bytes| bytes.len())
}

/// Sets `key` in `items` if the result stays within the quota; otherwise leaves them alone.
pub fn set(items: &mut Items, key: &str, value: Value) -> Result<(), StorageError> {
    check_key(key)?;
    let old = items.insert(key.to_owned(), value);
    if size(items) > QUOTA_BYTES {
        match old {
            Some(old) => items.insert(key.to_owned(), old),
            None => items.remove(key),
        };
        return Err(StorageError::Quota);
    }
    Ok(())
}

fn file(dir: &Path, id: &str) -> PathBuf {
    // `id` passed manifest::is_valid_id: a-z, 0-9 and -, so it is a plain file name.
    dir.join(DATA_DIR).join(format!("{id}.json"))
}

/// Reads an extension's items; a missing or bad file is empty.
pub fn load(dir: &Path, id: &str) -> Items {
    fs::read(file(dir, id))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|value| match value.get("items") {
            Some(Value::Object(map)) => Some(map.clone().into_iter().collect()),
            _ => None,
        })
        .unwrap_or_default()
}

fn save(dir: &Path, id: &str, items: &Items) -> io::Result<()> {
    let path = file(dir, id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json =
        serde_json::to_vec(&json!({ "version": 1, "items": items })).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)
}

/// Reads, changes and (when `change` says so) saves an extension's store under the lock.
pub fn with_store<T>(
    dir: &Path,
    id: &str,
    change: impl FnOnce(&mut Items) -> Result<(T, bool), StorageError>,
) -> Result<T, StorageError> {
    let _lock = FILES.lock().unwrap_or_else(PoisonError::into_inner);
    let mut items = load(dir, id);
    let (result, changed) = change(&mut items)?;
    if changed {
        save(dir, id, &items).map_err(|err| StorageError::Io(err.to_string()))?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "little-joan-test-storage-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn keys_are_1_to_128_characters() {
        assert_eq!(check_key(""), Err(StorageError::BadKey));
        assert!(check_key("a").is_ok());
        assert!(check_key(&"k".repeat(128)).is_ok());
        assert!(check_key(&"é".repeat(128)).is_ok(), "characters, not bytes");
        assert_eq!(check_key(&"k".repeat(129)), Err(StorageError::BadKey));
    }

    #[test]
    fn a_write_past_1_mb_is_refused_and_changes_nothing() {
        let mut items = Items::new();
        set(&mut items, "a", json!("x".repeat(QUOTA_BYTES / 2))).expect("half fits");
        let before = items.clone();
        assert_eq!(
            set(&mut items, "b", json!("y".repeat(QUOTA_BYTES / 2))),
            Err(StorageError::Quota)
        );
        assert_eq!(items, before, "a refused new key is not kept");
        assert_eq!(
            set(&mut items, "a", json!("z".repeat(QUOTA_BYTES))),
            Err(StorageError::Quota)
        );
        assert_eq!(items, before, "a refused overwrite keeps the old value");
        set(&mut items, "a", json!(1)).expect("smaller fits");
        assert!(size(&items) <= QUOTA_BYTES);
    }

    #[test]
    fn stores_are_per_extension_and_saved_atomically() {
        let dir = temp_dir("files");
        with_store(&dir, "hello", |items| {
            set(items, "count", json!(3))?;
            Ok(((), true))
        })
        .expect("saved");
        assert_eq!(load(&dir, "hello").get("count"), Some(&json!(3)));
        assert!(
            load(&dir, "other").is_empty(),
            "another extension sees nothing"
        );
        assert!(dir.join(DATA_DIR).join("hello.json").is_file());
        assert!(!dir.join(DATA_DIR).join("hello.json.tmp").exists());
        // A failed change saves nothing.
        let err = with_store(&dir, "hello", |items| {
            items.clear();
            set(items, "", json!(1))?;
            Ok(((), true))
        });
        assert_eq!(err, Err(StorageError::BadKey));
        assert_eq!(load(&dir, "hello").get("count"), Some(&json!(3)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bad_file_is_an_empty_store() {
        let dir = temp_dir("bad");
        fs::create_dir_all(dir.join(DATA_DIR)).expect("create");
        for contents in ["not json", "[]", r#"{"items": 3}"#] {
            fs::write(dir.join(DATA_DIR).join("hello.json"), contents).expect("write");
            assert!(load(&dir, "hello").is_empty(), "{contents}");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
