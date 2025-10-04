use std::{
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::generator::DEFAULT_ALPHABET;

use super::{GeneratorError, ShortCodeGenerator};

/// Fixed-length base62 encoding (left-pad with '0').
///
/// - `v`: value to encode (u128)
/// - `len`: desired output length
/// - `alphabet`: mapping table (must have length 62)
///
/// Returns `None` when `alphabet.len() != 62` or `v` cannot fit in `len` digits.
fn encode_base62_fixed(mut v: u128, len: usize, alphabet: &[char]) -> Option<String> {
    if alphabet.len() != 62 {
        return None;
    }
    let mut buf = vec![alphabet[0]; len];
    let mut i = len;
    while i > 0 {
        i -= 1;
        let rem = (v % 62) as usize;
        v /= 62;
        buf[i] = alphabet[rem];
    }
    // If v still > 0, `len` is too small to hold the value.
    if v != 0 {
        return None;
    }
    Some(buf.into_iter().collect())
}

/// Simple local block allocator:
/// - `next_global`: global cursor; refill obtains `block_size` ids starting from it.
/// - Thread/instance safe inside same process.
/// - Optional persistence to `state_path` to avoid restarting from zero.
pub struct SequenceEngine {
    len: usize,
    alphabet: Vec<char>,

    block_size: u64,
    persist_every: u64,
    state_path: Option<PathBuf>,

    next_global: AtomicU64,    // next refill start
    inner: Mutex<BlockWindow>, // current window
}

#[derive(Clone, Copy, Debug)]
struct BlockWindow {
    current: u64,
    end: u64,
    issued_since_persist: u64,
}

impl SequenceEngine {
    pub fn new(
        len: usize,
        alphabet: Option<String>,
        block_size: u64,
        persist_every: u64,
        state_path: Option<PathBuf>,
    ) -> Self {
        // Use provided alphabet string or DEFAULT_ALPHABET (keeps original behavior).
        let alpha = alphabet
            .unwrap_or_else(|| DEFAULT_ALPHABET.iter().collect())
            .chars()
            .collect::<Vec<_>>();

        // Load previously persisted next_global (if any), otherwise start at 0.
        let next0 = load_state(&state_path).unwrap_or(0);
        SequenceEngine {
            len,
            alphabet: alpha,
            block_size,
            persist_every,
            state_path,
            next_global: AtomicU64::new(next0),
            inner: Mutex::new(BlockWindow {
                current: 0,
                end: 0,
                issued_since_persist: 0,
            }),
        }
    }

    /// Allocate a new block [start, end).
    ///
    /// Caller must hold the `inner` lock. This atomically increments `next_global`
    /// by `block_size` and sets the window to [start, start + block_size).
    fn refill_locked(&self, w: &mut BlockWindow) -> Result<(), GeneratorError> {
        let start = self
            .next_global
            .fetch_add(self.block_size, Ordering::Relaxed);
        let end = start
            .checked_add(self.block_size)
            .ok_or(GeneratorError::Internal("counter overflow"))?;
        w.current = start; // \u2705 \u5fc5\u987b\u8bbe\u7f6e\u5f53\u524d\u7a97\u53e3\u8d77\u70b9
        w.end = end; // \u7ed3\u675f\u8fb9\u754c
        Ok(())
    }

    /// Maybe persist the state to disk.
    ///
    /// If `state_path` is None, do nothing. Otherwise increment `issued_since_persist`
    /// and when it reaches `persist_every`, write the current `next_global` to disk.
    fn maybe_persist(&self, w: &mut BlockWindow) -> Result<(), GeneratorError> {
        if self.state_path.is_none() {
            return Ok(());
        }
        w.issued_since_persist += 1;
        if w.issued_since_persist >= self.persist_every {
            w.issued_since_persist = 0;
            // Persist the "future next_global" (current global cursor).
            let next = self.next_global.load(Ordering::SeqCst);
            store_state(self.state_path.as_ref().unwrap(), next)?;
        }
        Ok(())
    }
}

impl ShortCodeGenerator for SequenceEngine {
    fn generate(&self) -> Result<String, GeneratorError> {
        // Obtain a new numeric id from the local window (refill if needed).
        let n = {
            let mut win = self.inner.lock().expect("lock poisoned");
            if win.current >= win.end {
                self.refill_locked(&mut win)?;
            }
            let v = win.current;
            win.current += 1;
            self.maybe_persist(&mut win)?;
            v
        };

        // Encode to fixed-length base62. If not enough space, return ExhaustedSpace.
        let s = encode_base62_fixed(n as u128, self.len, &self.alphabet)
            .ok_or(GeneratorError::ExhaustedSpace)?;
        Ok(s)
    }

    fn name(&self) -> &'static str {
        "sequence"
    }
}

/// State file stores `next_global` as little-endian u64.
fn load_state(path: &Option<PathBuf>) -> Result<u64, std::io::Error> {
    if let Some(p) = path {
        if p.exists() {
            let mut f = fs::File::open(p)?;
            let mut buf = [0u8; 8];
            f.read_exact(&mut buf)?;
            return Ok(u64::from_le_bytes(buf));
        }
    }
    Ok(0)
}

fn store_state(path: &PathBuf, next: u64) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(&next.to_le_bytes())?;
    f.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Test alphabet: '0'-'9', 'A'-'Z', 'a'-'z' (62 chars)
    fn test_alphabet_vec() -> Vec<char> {
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect()
    }
    fn test_alphabet_string() -> String {
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".to_string()
    }

    #[test]
    fn test_encode_base62_fixed_basic() {
        let alpha = test_alphabet_vec();

        // v = 0 -> all digits are alphabet[0] ('0'), length 4 => "0000"
        assert_eq!(
            encode_base62_fixed(0u128, 4, &alpha).unwrap(),
            "0000".to_string()
        );

        // v = 1 -> last digit becomes '1'
        assert_eq!(
            encode_base62_fixed(1u128, 4, &alpha).unwrap(),
            "0001".to_string()
        );

        // Value that fits exactly: largest value for len=2 is 62^2 - 1 = 3843
        let max_len2 = 62u128.pow(2) - 1;
        assert!(encode_base62_fixed(max_len2, 2, &alpha).is_some());

        // Overflow: 62^2 cannot fit in len=2
        let overflow = 62u128.pow(2);
        assert!(encode_base62_fixed(overflow, 2, &alpha).is_none());
    }

    #[test]
    fn test_sequence_engine_generate_and_persistence() {
        // Build a unique temp path for state file
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut path = std::env::temp_dir();
        path.push(format!("seq_state_test_{}.bin", t));

        // ensure no leftover
        let _ = fs::remove_file(&path);

        // Use block_size=5, persist_every=1 so state is written after the first generate
        let engine = SequenceEngine::new(6, Some(test_alphabet_string()), 5, 1, Some(path.clone()));

        // Generate one code
        let code1 = engine.generate().expect("generate failed");
        let code2 = engine.generate().expect("generate failed");

        // They should be strings and distinct (since we increment)
        assert_ne!(code1, code2);

        // Read persisted state file: after first refill, next_global should be 5
        // (first refill: fetch_add(5) returns 0 and sets next_global -> 5)
        let mut f = fs::File::open(&path).expect("state file should exist");
        let mut buf = [0u8; 8];
        f.read_exact(&mut buf).expect("read state");
        let stored = u64::from_le_bytes(buf);
        assert_eq!(stored, 5u64);

        // cleanup
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_encode_exhausted_space_case() {
        let alpha = test_alphabet_vec();

        // For len = 3, capacity = 62^3 = 238328, so value == 238328 should be out of range
        let cap = 62u128.pow(3);
        assert!(encode_base62_fixed(cap, 3, &alpha).is_none());
    }
}
