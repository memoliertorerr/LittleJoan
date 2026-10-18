//! Unguessable values for the extension runtime: the single-use nonce in the URL the app opens a
//! panel at (session.rs), and the per-session token the SDK sends with every API call (api.rs).
//!
//! 128 bits from the operating system's random source — `/dev/urandom` on macOS and Linux,
//! `BCryptGenRandom` (the system-preferred RNG) on Windows; no new crate. If the OS source ever
//! fails, the value comes from SipHash with std's per-process random keys over a counter and the
//! clock — still unknowable to web content, and the failure is logged.

use std::hash::{BuildHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bytes in a nonce or token (128 bits).
pub const BYTES: usize = 16;

#[cfg(unix)]
fn os_random(buffer: &mut [u8]) -> bool {
    use std::io::Read;
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(buffer))
        .is_ok()
}

#[cfg(windows)]
fn os_random(buffer: &mut [u8]) -> bool {
    use windows::Win32::Security::Cryptography::{
        BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
    };
    // SAFETY: fills `buffer` (its pointer and length come from the slice) from the system RNG;
    // no algorithm handle is needed with BCRYPT_USE_SYSTEM_PREFERRED_RNG.
    let status = unsafe { BCryptGenRandom(None, buffer, BCRYPT_USE_SYSTEM_PREFERRED_RNG) };
    status.0 >= 0
}

#[cfg(not(any(unix, windows)))]
fn os_random(_buffer: &mut [u8]) -> bool {
    false
}

fn fallback(buffer: &mut [u8]) {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let keys = std::collections::hash_map::RandomState::new();
    for chunk in buffer.chunks_mut(8) {
        let mut hasher = keys.build_hasher();
        hasher.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
        hasher.write_u128(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos()),
        );
        let bytes = hasher.finish().to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
}

/// 128 random bits as 32 lower-case hex digits.
pub fn random_hex() -> String {
    let mut buffer = [0u8; BYTES];
    if !os_random(&mut buffer) {
        eprintln!("little-joan: the system's random source failed; using the fallback");
        fallback(&mut buffer);
    }
    buffer.iter().map(|b| format!("{b:02x}")).collect()
}

/// Compares two secrets without stopping at the first difference (the time doesn't tell how
/// much of a guess was right). Different lengths are simply unequal.
pub fn same_secret(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |diff, (x, y)| diff | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn nonces_are_128_bit_hex_and_never_repeat() {
        let seen: HashSet<String> = (0..1000).map(|_| random_hex()).collect();
        assert_eq!(seen.len(), 1000);
        for nonce in &seen {
            assert_eq!(nonce.len(), 2 * BYTES);
            assert!(
                nonce
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            );
        }
    }

    #[test]
    fn the_os_source_works_here_and_the_fallback_differs_every_time() {
        let mut buffer = [0u8; BYTES];
        assert!(os_random(&mut buffer));
        let mut a = [0u8; BYTES];
        let mut b = [0u8; BYTES];
        fallback(&mut a);
        fallback(&mut b);
        assert_ne!(a, b);
    }

    #[test]
    fn secrets_compare_whole() {
        assert!(same_secret("abc123", "abc123"));
        assert!(!same_secret("abc123", "abc124"));
        assert!(!same_secret("abc123", "abc12"));
        assert!(!same_secret("", "x"));
        assert!(same_secret("", ""));
    }
}
