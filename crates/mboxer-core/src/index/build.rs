//! One-pass index build: stream the mbox, parse each message, add one document.
//!
//! The index is written into a `.building` sibling of the final directory and
//! swapped into place only after the commit and the metadata file succeed, so
//! a crash or a cancel never leaves a half index that looks fresh. The swap
//! moves the old index's files aside (`.old`) before the new folder takes its
//! name and deletes them last, so the index folder is never half deleted, even
//! while a viewer still reads the old one (task 052). Every tantivy handle of
//! the build is dropped before its directory is renamed: Windows refuses that
//! while a file inside is open. When the `.building` directory cannot be
//! created beside the mbox, the build goes to the index home instead (task
//! 014, D-005) and the other location is cleared, so one index exists per mbox.
//!
//! One build of an mbox runs at a time: a lock file in the index home is held
//! for the whole build, and a second build gets [`Error::BuildLocked`]. A
//! message over the size limit is left out and counted, and the build goes on.

use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

use mail_parser::Message;
use tantivy::{Index, IndexWriter, TantivyDocument};

use crate::error::{Error, Result};
use crate::index::fingerprint::{
    FORMAT_VERSION, IndexMeta, META_FILE, default_index_home, fallback_dir, fingerprint, index_dir,
};
use crate::index::schema::{Fields, build_schema, register_tokenizer};
use crate::message::{
    Mailbox, MessageSummary, TextKind, html_to_plain_text, parse, summarize, text_parts,
};
use crate::reader::{DEFAULT_MAX_MESSAGE_BYTES, MboxReader};

/// Knobs of a build; [`BuildOptions::default`] fits a desktop machine.
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Indexing threads; the reader and parser stay on the calling thread.
    pub threads: usize,
    /// tantivy's memory arena per indexing thread (at least 15 MB).
    pub memory_per_thread_bytes: usize,
    /// Passed to [`MboxReader::with_max_message_bytes`]; a larger message is
    /// left out of the index and counted in [`BuildReport::skipped`].
    pub max_message_bytes: u64,
    /// Body text beyond this many bytes is not indexed.
    pub max_body_bytes: usize,
    /// Root of the fallback indexes and the build locks; `None` means
    /// [`default_index_home`].
    pub home: Option<PathBuf>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
        Self {
            threads: cores.clamp(1, 4),
            memory_per_thread_bytes: 128 << 20,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            max_body_bytes: 4 << 20,
            home: None,
        }
    }
}

/// A progress snapshot, reported every few megabytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub bytes: u64,
    pub total_bytes: u64,
    pub messages: u64,
}

/// What a finished build produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub messages: u64,
    pub parse_failures: u64,
    /// Messages over [`BuildOptions::max_message_bytes`], left out of the index.
    pub skipped: u64,
    /// Envelope offsets of the first [`SKIPPED_OFFSETS_KEPT`] skipped messages.
    pub skipped_offsets: Vec<u64>,
    pub elapsed: Duration,
    /// Size of the index directory on disk.
    pub index_bytes: u64,
    pub index_dir: PathBuf,
    /// The index went under the index home because `.mboxer/` beside the mbox
    /// could not be created.
    pub fallback: bool,
}

const PROGRESS_EVERY_BYTES: u64 = 16 << 20;

/// How many offsets of skipped messages a [`BuildReport`] lists.
pub const SKIPPED_OFFSETS_KEPT: usize = 10;

/// Pauses between the attempts of a rename or a removal that Windows refuses
/// while another process (a virus scanner, a viewer letting go) holds a file.
const RETRY_PAUSES_MS: [u64; 5] = [50, 100, 200, 400, 800];

/// Builds (or rebuilds) the sidecar index of `mbox`.
///
/// `progress` is called on the calling thread; setting `cancel` makes the build
/// stop at the next message with [`Error::Cancelled`] and leaves no index behind.
/// While another build of the same mbox runs, this one returns
/// [`Error::BuildLocked`] at once and touches nothing.
pub fn build(
    mbox: &Path,
    options: &BuildOptions,
    progress: impl FnMut(Progress),
    cancel: &AtomicBool,
) -> Result<BuildReport> {
    let start = Instant::now();
    let mbox_fingerprint = fingerprint(mbox)?;
    let home = options.home.clone().unwrap_or_else(default_index_home);
    let _lock = BuildLock::acquire(&lock_path(mbox, &home))?;
    let primary = index_dir(mbox);
    let (final_dir, fallback) = match prepare_building_dir(&primary) {
        Ok(()) => (primary, false),
        Err(_) => {
            let dir = fallback_dir(mbox, &home);
            prepare_building_dir(&dir)?;
            (dir, true)
        }
    };
    let building = building_dir(&final_dir);
    let outcome = index_into(mbox, &building, options, progress, cancel).and_then(|counts| {
        // Every tantivy handle died with `index_into`. The metadata goes in
        // last: a folder without it is never taken for an index.
        let meta = IndexMeta {
            format_version: FORMAT_VERSION,
            mbox: mbox_fingerprint,
            message_count: counts.messages,
            parse_failures: counts.parse_failures,
            skipped: counts.skipped,
            built_at_unix: UNIX_EPOCH.elapsed().map_or(0, |d| d.as_secs() as i64),
            build_secs: start.elapsed().as_secs(),
            tantivy_version: tantivy::version_string().to_owned(),
        };
        meta.write(&building)?;
        Ok(counts)
    });
    let counts = match outcome {
        Ok(counts) => counts,
        Err(err) => {
            let _ = fs::remove_dir_all(&building);
            return Err(err);
        }
    };
    swap_into_place(&building, &final_dir)?;
    // One index per mbox: the location not used this time goes, when it can.
    let other = if fallback {
        index_dir(mbox)
    } else {
        fallback_dir(mbox, &home)
    };
    let _ = remove_dir_if_present(&other);

    Ok(BuildReport {
        messages: counts.messages,
        parse_failures: counts.parse_failures,
        skipped: counts.skipped,
        skipped_offsets: counts.skipped_offsets,
        elapsed: start.elapsed(),
        index_bytes: dir_size(&final_dir)?,
        index_dir: final_dir,
        fallback,
    })
}

/// The build lock of `mbox`: `<home>/<fallback folder name>.lock`, one per mbox
/// location whichever folder its index goes to.
fn lock_path(mbox: &Path, home: &Path) -> PathBuf {
    with_suffix(&fallback_dir(mbox, home), ".lock")
}

/// An exclusive OS lock on the lock file. The operating system releases it
/// when the value drops or the process dies, so a crashed build never leaves a
/// stale lock. The file itself stays: removing it while another build opens it
/// would let two builds lock two different files.
struct BuildLock {
    _file: File,
}

impl BuildLock {
    fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(Error::BuildLocked {
                lock: path.to_path_buf(),
            }),
            Err(TryLockError::Error(err)) => Err(err.into()),
        }
    }
}

/// Creates an empty `.building` sibling of `final_dir` and clears what an
/// earlier build left beside it; an `Err` means the location cannot take an
/// index. The caller holds the build lock, so no other build owns these.
fn prepare_building_dir(final_dir: &Path) -> Result<()> {
    let building = building_dir(final_dir);
    remove_dir_if_present(&building)?;
    remove_set_aside(final_dir);
    fs::create_dir_all(&building)?;
    Ok(())
}

/// Puts the finished `building` folder at `final_dir`.
///
/// The files of the old index are moved aside one by one, its metadata file
/// first, then `building` takes the folder's name and the aside folder is
/// removed last. Windows refuses to rename a folder while a viewer holds a
/// file inside it, but lets each file be moved, so this works beside an open
/// index; a file held without delete sharing (a scanner, a backup tool) stops
/// the move, and what moved goes back. On any failure the old index stays (or
/// is put back) whole and `building` goes.
fn swap_into_place(building: &Path, final_dir: &Path) -> Result<()> {
    let discard = |err: Error| {
        let _ = fs::remove_dir_all(building);
        Err(err)
    };
    if !final_dir.exists() {
        return retry(|| fs::rename(building, final_dir)).or_else(discard);
    }
    let aside = free_aside_dir(final_dir);
    if let Err(err) = move_entries(final_dir, &aside, Meta::First) {
        let _ = fs::remove_dir(&aside);
        return discard(err);
    }
    let renamed = retry(|| fs::remove_dir(final_dir))
        .and_then(|()| retry(|| fs::rename(building, final_dir)));
    if let Err(err) = renamed {
        // Without the old index back in place, both folders stay whole.
        return match move_entries(&aside, final_dir, Meta::Last) {
            Ok(()) => {
                let _ = fs::remove_dir(&aside);
                discard(err)
            }
            Err(_) => Err(err),
        };
    }
    // A viewer may still read the old files; what stays is cleared by the
    // next build.
    let _ = retry(|| fs::remove_dir_all(&aside));
    Ok(())
}

/// Where the metadata file goes in [`move_entries`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Meta {
    /// Out of an index first: a half-moved folder never reads as an index.
    First,
    /// Into an index last, for the same reason.
    Last,
}

/// Moves every entry of `from` into `to` (created when missing). When one
/// entry cannot move, the ones that did go back and the error returns.
fn move_entries(from: &Path, to: &Path, meta: Meta) -> Result<()> {
    fs::create_dir_all(to)?;
    let mut names = fs::read_dir(from)?
        .map(|entry| entry.map(|e| e.file_name()))
        .collect::<std::io::Result<Vec<_>>>()?;
    names.sort_by_key(|name| (name == META_FILE) == (meta == Meta::Last));
    for (done, name) in names.iter().enumerate() {
        if let Err(err) = retry(|| fs::rename(from.join(name), to.join(name))) {
            for back in names[..done].iter().rev() {
                let _ = fs::rename(to.join(back), from.join(back));
            }
            return Err(err);
        }
    }
    Ok(())
}

/// Runs `op` until it succeeds or the pauses of [`RETRY_PAUSES_MS`] run out.
fn retry(mut op: impl FnMut() -> std::io::Result<()>) -> Result<()> {
    let mut pauses = RETRY_PAUSES_MS.iter();
    loop {
        match op() {
            Ok(()) => return Ok(()),
            Err(err) => match pauses.next() {
                Some(&ms) => std::thread::sleep(Duration::from_millis(ms)),
                None => return Err(err.into()),
            },
        }
    }
}

/// The first of `<name>.old`, `<name>.old.1`, `<name>.old.2`, … that does not exist.
fn free_aside_dir(final_dir: &Path) -> PathBuf {
    let base = with_suffix(final_dir, ".old");
    std::iter::once(base.clone())
        .chain((1u64..).map(|n| with_suffix(&base, &format!(".{n}"))))
        .find(|dir| !dir.exists())
        .expect("an endless sequence of names has a free one")
}

/// Whether `name` is one of the aside names of the index folder `index_name`.
fn is_aside_name(name: &str, index_name: &str) -> bool {
    let Some(rest) = name
        .strip_prefix(index_name)
        .and_then(|rest| rest.strip_prefix(".old"))
    else {
        return false;
    };
    rest.is_empty()
        || rest
            .strip_prefix('.')
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// Removes the aside folders of `final_dir` that earlier builds left behind;
/// one that is still held stays for the next build.
fn remove_set_aside(final_dir: &Path) {
    let (Some(parent), Some(index_name)) = (final_dir.parent(), final_dir.file_name()) else {
        return;
    };
    let index_name = index_name.to_string_lossy();
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        if is_aside_name(&entry.file_name().to_string_lossy(), &index_name) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

fn building_dir(final_dir: &Path) -> PathBuf {
    with_suffix(final_dir, ".building")
}

/// `path` with `suffix` appended to its last component.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(suffix);
    path.with_file_name(name)
}

fn remove_dir_if_present(dir: &Path) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

fn dir_size(dir: &Path) -> Result<u64> {
    let mut total = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        total += if metadata.is_dir() {
            dir_size(&entry.path())?
        } else {
            metadata.len()
        };
    }
    Ok(total)
}

#[derive(Default)]
struct Counts {
    messages: u64,
    parse_failures: u64,
    skipped: u64,
    skipped_offsets: Vec<u64>,
}

fn index_into(
    mbox: &Path,
    dir: &Path,
    options: &BuildOptions,
    mut progress: impl FnMut(Progress),
    cancel: &AtomicBool,
) -> Result<Counts> {
    let (schema, fields) = build_schema();
    let index = Index::create_in_dir(dir, schema)?;
    register_tokenizer(&index);
    let threads = options.threads.max(1);
    let writer: IndexWriter =
        index.writer_with_num_threads(threads, threads * options.memory_per_thread_bytes)?;

    let total_bytes = fs::metadata(mbox)?.len();
    let mut reader = MboxReader::open(mbox)?.with_max_message_bytes(options.max_message_bytes);
    let mut counts = Counts::default();
    let mut last_progress = 0;
    while let Some(item) = reader.next() {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let raw = match item {
            Ok(raw) => raw,
            // The reader goes on at the next envelope line; this one is left out.
            Err(Error::MessageTooLarge { offset, .. }) => {
                counts.skipped += 1;
                if counts.skipped_offsets.len() < SKIPPED_OFFSETS_KEPT {
                    counts.skipped_offsets.push(offset);
                }
                continue;
            }
            Err(err) => return Err(err),
        };
        counts.messages += 1;
        let mut doc = TantivyDocument::new();
        doc.add_u64(fields.offset, raw.offset);
        doc.add_u64(fields.len, raw.len);
        match parse(&raw) {
            Some(message) => {
                let summary = summarize(&raw, &message);
                add_summary(&mut doc, &fields, &summary);
                doc.add_text(fields.body, body_text(&message, options.max_body_bytes));
            }
            None => {
                counts.parse_failures += 1;
                doc.add_bool(fields.has_attachment, false);
            }
        }
        writer.add_document(doc)?;
        let position = reader.position();
        if position - last_progress >= PROGRESS_EVERY_BYTES {
            last_progress = position;
            progress(Progress {
                bytes: position,
                total_bytes,
                messages: counts.messages,
            });
        }
    }
    // Nothing but oversized messages: the file is most likely not an mbox at
    // all (one endless line), which stays an error as before task 052.
    if counts.messages == 0
        && let Some(&offset) = counts.skipped_offsets.first()
    {
        return Err(Error::MessageTooLarge {
            offset,
            limit: options.max_message_bytes,
        });
    }
    progress(Progress {
        bytes: reader.position(),
        total_bytes,
        messages: counts.messages,
    });

    let mut writer = writer;
    writer.commit()?;
    writer.wait_merging_threads()?;
    Ok(counts)
}

fn add_summary(doc: &mut TantivyDocument, fields: &Fields, summary: &MessageSummary) {
    if let Some(date) = summary.date_unix {
        doc.add_i64(fields.date, date);
    }
    if let Some(thread_id) = summary.gmail_thread_id {
        doc.add_u64(fields.thread_id, thread_id);
    }
    if let Some(message_id) = &summary.message_id {
        doc.add_text(fields.message_id, message_id);
    }
    add_mailboxes(doc, fields.from, Some(fields.from_addr), &summary.from);
    add_mailboxes(doc, fields.to, Some(fields.to_addr), &summary.to);
    add_mailboxes(doc, fields.cc, None, &summary.cc);
    if let Some(subject) = &summary.subject {
        doc.add_text(fields.subject, subject);
    }
    for label in &summary.labels {
        doc.add_text(fields.labels, label);
    }
    doc.add_bool(fields.has_attachment, summary.has_attachment);
    for name in &summary.attachment_names {
        doc.add_text(fields.attachment_names, name);
    }
}

fn add_mailboxes(
    doc: &mut TantivyDocument,
    display: tantivy::schema::Field,
    address: Option<tantivy::schema::Field>,
    mailboxes: &[Mailbox],
) {
    for mailbox in mailboxes {
        doc.add_text(display, mailbox.display());
        if let (Some(field), Some(addr)) = (address, &mailbox.address) {
            doc.add_text(field, addr.to_lowercase());
        }
    }
}

/// The searchable text of a message: its plain-text parts, or the text of its
/// HTML parts when it has no plain part at all, capped at `max_bytes`.
pub fn body_text(message: &Message<'_>, max_bytes: usize) -> String {
    let parts = text_parts(message);
    let has_plain = parts.iter().any(|p| p.kind == TextKind::Plain);
    let mut out = String::new();
    for part in &parts {
        let text = match part.kind {
            TextKind::Plain => part.text.clone(),
            TextKind::Html if !has_plain => html_to_plain_text(&part.text),
            TextKind::Html => continue,
        };
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&text);
        if out.len() >= max_bytes {
            break;
        }
    }
    truncate_at_char_boundary(&mut out, max_bytes);
    out
}

fn truncate_at_char_boundary(text: &mut String, max_bytes: usize) {
    if text.len() <= max_bytes {
        return;
    }
    let mut cut = max_bytes;
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    text.truncate(cut);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_respects_char_boundaries() {
        let mut s = "árvíz".to_owned();
        truncate_at_char_boundary(&mut s, 2);
        assert_eq!(s, "á");
        let mut s = "abc".to_owned();
        truncate_at_char_boundary(&mut s, 10);
        assert_eq!(s, "abc");
    }

    #[test]
    fn building_dir_is_a_sibling() {
        let dir = building_dir(Path::new("D:/mail/.mboxer/all.mbox"));
        assert_eq!(dir, Path::new("D:/mail/.mboxer/all.mbox.building"));
    }

    #[test]
    fn aside_names_belong_to_their_index_only() {
        assert!(is_aside_name("all.mbox.old", "all.mbox"));
        assert!(is_aside_name("all.mbox.old.12", "all.mbox"));
        assert!(!is_aside_name("all.mbox", "all.mbox"));
        assert!(!is_aside_name("all.mbox.building", "all.mbox"));
        assert!(!is_aside_name("all.mbox.old.", "all.mbox"));
        assert!(!is_aside_name("all.mbox.old.x", "all.mbox"));
        assert!(!is_aside_name("all.mbox.older", "all.mbox"));
        assert!(!is_aside_name("other.mbox.old", "all.mbox"));
    }

    #[test]
    fn the_aside_name_skips_the_ones_in_use() {
        let dir = tempfile::tempdir().unwrap();
        let index = dir.path().join("all.mbox");
        assert_eq!(free_aside_dir(&index), dir.path().join("all.mbox.old"));
        fs::create_dir(dir.path().join("all.mbox.old")).unwrap();
        fs::create_dir(dir.path().join("all.mbox.old.1")).unwrap();
        assert_eq!(free_aside_dir(&index), dir.path().join("all.mbox.old.2"));
        remove_set_aside(&index);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn the_lock_is_one_per_mbox_beside_its_fallback_folder() {
        let home = Path::new("C:/index-home");
        let lock = lock_path(Path::new("D:/mail/All mail.mbox"), home);
        let fallback = fallback_dir(Path::new("D:/mail/All mail.mbox"), home);
        assert_eq!(lock, with_suffix(&fallback, ".lock"));
        assert_eq!(lock.parent(), Some(home));
    }
}
