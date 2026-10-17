//! Where the sidecar lives and whether it still matches the mbox.
//!
//! The index of `<dir>/<name>.mbox` is the directory `<dir>/.mboxer/<name>.mbox/`.
//! Its `mboxer.json` records the index format version and a fingerprint of the
//! mbox: size, modification time and a hash over a few sampled regions. Any
//! mismatch makes the index stale; the caller rebuilds it from scratch.

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Bumped whenever the schema, the tokenizer or a stored value changes; an
/// older index is rebuilt. 2: label names are RFC 2047 decoded (task 015).
pub const FORMAT_VERSION: u32 = 2;

/// The metadata file inside the index directory (tantivy owns `meta.json`).
pub const META_FILE: &str = "mboxer.json";

/// Name of the sidecar directory created next to the mbox.
pub const SIDECAR_DIR: &str = ".mboxer";

const HEAD_BYTES: u64 = 1 << 20;
const TAIL_BYTES: u64 = 1 << 20;
const SAMPLE_BLOCKS: u64 = 16;
const SAMPLE_BLOCK_BYTES: u64 = 64 << 10;

/// The index directory for `mbox`: `<parent>/.mboxer/<file name>/`.
pub fn index_dir(mbox: &Path) -> PathBuf {
    let parent = mbox.parent().unwrap_or_else(|| Path::new("."));
    let name = mbox
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_else(|| "mbox".into());
    parent.join(SIDECAR_DIR).join(name)
}

/// What identifies the mbox content the index was built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fingerprint {
    /// File size in bytes.
    pub len: u64,
    /// Modification time as Unix seconds.
    pub mtime_unix: i64,
    /// FNV-1a 64 over the sampled regions, see [`sample_ranges`].
    pub sample_hash: u64,
}

/// Computes the fingerprint of `mbox`; reads at most a few megabytes.
pub fn fingerprint(mbox: &Path) -> Result<Fingerprint> {
    let mut file = File::open(mbox)?;
    let metadata = file.metadata()?;
    let len = metadata.len();
    let mtime_unix = metadata
        .modified()
        .ok()
        .map(|t| match t.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs() as i64,
            Err(e) => -(e.duration().as_secs() as i64),
        })
        .unwrap_or(0);
    let sample_hash = sample_hash(&mut file, len)?;
    Ok(Fingerprint {
        len,
        mtime_unix,
        sample_hash,
    })
}

/// The `(offset, size)` regions hashed for a file of `len` bytes: the first and
/// last megabyte plus 16 blocks of 64 KiB spread evenly between them. A small
/// file is hashed whole.
pub fn sample_ranges(len: u64) -> Vec<(u64, u64)> {
    if len <= HEAD_BYTES + TAIL_BYTES {
        return if len == 0 { vec![] } else { vec![(0, len)] };
    }
    let mut ranges = vec![(0, HEAD_BYTES)];
    let middle_start = HEAD_BYTES;
    let middle_end = len - TAIL_BYTES;
    let step = (middle_end - middle_start) / SAMPLE_BLOCKS;
    for i in 0..SAMPLE_BLOCKS {
        let start = middle_start + i * step;
        let size = SAMPLE_BLOCK_BYTES.min(middle_end - start);
        if size > 0 {
            ranges.push((start, size));
        }
    }
    ranges.push((middle_end, TAIL_BYTES));
    ranges
}

fn sample_hash<R: Read + Seek>(source: &mut R, len: u64) -> io::Result<u64> {
    let mut hash = FNV_OFFSET;
    let mut buf = vec![0u8; SAMPLE_BLOCK_BYTES as usize];
    for (offset, size) in sample_ranges(len) {
        source.seek(SeekFrom::Start(offset))?;
        let mut remaining = size;
        while remaining > 0 {
            let want = (buf.len() as u64).min(remaining) as usize;
            let n = source.read(&mut buf[..want])?;
            if n == 0 {
                break;
            }
            hash = fnv1a64(hash, &buf[..n]);
            remaining -= n as u64;
        }
    }
    Ok(hash)
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FNV-1a 64, continued from `state`. Stable across Rust releases, unlike `DefaultHasher`.
pub fn fnv1a64(mut state: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        state ^= u64::from(b);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

/// Contents of `mboxer.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexMeta {
    pub format_version: u32,
    pub mbox: Fingerprint,
    pub message_count: u64,
    pub parse_failures: u64,
    /// Unix seconds.
    pub built_at_unix: i64,
    pub build_secs: u64,
    pub tantivy_version: String,
}

impl IndexMeta {
    pub(crate) fn read(dir: &Path) -> Result<Self> {
        let text = fs::read_to_string(dir.join(META_FILE))?;
        Ok(serde_json::from_str(&text)?)
    }

    pub(crate) fn write(&self, dir: &Path) -> Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        fs::write(dir.join(META_FILE), text)?;
        Ok(())
    }
}

/// Why an existing index cannot be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleReason {
    /// The mbox differs from the one the index was built from.
    Fingerprint,
    /// The index was written by a different format version.
    Format { found: u32, expected: u32 },
    /// `mboxer.json` is unreadable or malformed.
    Unreadable(String),
}

impl std::fmt::Display for StaleReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StaleReason::Fingerprint => write!(f, "the mbox changed since the index was built"),
            StaleReason::Format { found, expected } => {
                write!(f, "index format {found}, this build expects {expected}")
            }
            StaleReason::Unreadable(msg) => write!(f, "index metadata unreadable: {msg}"),
        }
    }
}

/// The state of the sidecar for one mbox.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexStatus {
    /// No index directory (or no `mboxer.json` in it).
    Missing,
    /// An index exists but must be rebuilt.
    Stale(StaleReason),
    /// The index matches the mbox and this build.
    Fresh(IndexMeta),
}

/// Checks the sidecar of `mbox` without opening the index itself.
pub fn status(mbox: &Path) -> Result<IndexStatus> {
    let dir = index_dir(mbox);
    if !dir.join(META_FILE).is_file() {
        return Ok(IndexStatus::Missing);
    }
    let meta = match IndexMeta::read(&dir) {
        Ok(meta) => meta,
        Err(Error::Json(err)) => {
            return Ok(IndexStatus::Stale(StaleReason::Unreadable(err.to_string())));
        }
        Err(err) => return Err(err),
    };
    if meta.format_version != FORMAT_VERSION {
        return Ok(IndexStatus::Stale(StaleReason::Format {
            found: meta.format_version,
            expected: FORMAT_VERSION,
        }));
    }
    if fingerprint(mbox)? != meta.mbox {
        return Ok(IndexStatus::Stale(StaleReason::Fingerprint));
    }
    Ok(IndexStatus::Fresh(meta))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn fnv1a64_known_vectors() {
        assert_eq!(fnv1a64(FNV_OFFSET, b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(FNV_OFFSET, b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(FNV_OFFSET, b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn small_files_are_hashed_whole() {
        assert!(sample_ranges(0).is_empty());
        assert_eq!(sample_ranges(10), vec![(0, 10)]);
        assert_eq!(sample_ranges(2 << 20), vec![(0, 2 << 20)]);
    }

    #[test]
    fn large_files_sample_head_middle_and_tail() {
        let len = 100 << 20;
        let ranges = sample_ranges(len);
        assert_eq!(ranges.len(), 18);
        assert_eq!(ranges[0], (0, HEAD_BYTES));
        assert_eq!(ranges[17], (len - TAIL_BYTES, TAIL_BYTES));
        let mut last_end = HEAD_BYTES;
        for &(start, size) in &ranges[1..17] {
            assert!(start >= last_end);
            assert_eq!(size, SAMPLE_BLOCK_BYTES);
            last_end = start + size;
        }
        assert!(last_end <= len - TAIL_BYTES);
    }

    #[test]
    fn sample_hash_sees_a_change_in_the_middle() {
        let len = 100 << 20;
        let mut data = vec![0u8; len as usize];
        let base = sample_hash(&mut Cursor::new(&data), len).unwrap();
        // The first sampled middle block starts exactly at HEAD_BYTES.
        data[HEAD_BYTES as usize] = 1;
        let changed = sample_hash(&mut Cursor::new(&data), len).unwrap();
        assert_ne!(base, changed);
    }

    #[test]
    fn index_dir_is_a_sidecar_folder_next_to_the_mbox() {
        let dir = index_dir(Path::new("D:/mail/All mail.mbox"));
        assert_eq!(dir, Path::new("D:/mail/.mboxer/All mail.mbox"));
    }
}
