//! One-pass index build: stream the mbox, parse each message, add one document.
//!
//! The index is written into a `.building` sibling of the final directory and
//! renamed into place only after the commit and the metadata file succeed, so
//! a crash or a cancel never leaves a half index that looks fresh. Every
//! tantivy handle is dropped before the directory is removed or renamed:
//! Windows refuses both while a file is memory-mapped. When the `.building`
//! directory cannot be created beside the mbox, the build goes to the index
//! home instead (task 014, D-05) and the other location is cleared, so one
//! index exists per mbox.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, UNIX_EPOCH};

use mail_parser::Message;
use tantivy::{Index, IndexWriter, TantivyDocument};

use crate::error::{Error, Result};
use crate::index::fingerprint::{
    FORMAT_VERSION, IndexMeta, default_index_home, fallback_dir, fingerprint, index_dir,
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
    /// Passed to [`MboxReader::with_max_message_bytes`].
    pub max_message_bytes: u64,
    /// Body text beyond this many bytes is not indexed.
    pub max_body_bytes: usize,
    /// Root of the fallback indexes; `None` means [`default_index_home`].
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
    pub elapsed: Duration,
    /// Size of the index directory on disk.
    pub index_bytes: u64,
    pub index_dir: PathBuf,
    /// The index went under the index home because `.mboxer/` beside the mbox
    /// could not be created.
    pub fallback: bool,
}

const PROGRESS_EVERY_BYTES: u64 = 16 << 20;

/// Builds (or rebuilds) the sidecar index of `mbox`.
///
/// `progress` is called on the calling thread; setting `cancel` makes the build
/// stop at the next message with [`Error::Cancelled`] and leaves no index behind.
pub fn build(
    mbox: &Path,
    options: &BuildOptions,
    progress: impl FnMut(Progress),
    cancel: &AtomicBool,
) -> Result<BuildReport> {
    let start = Instant::now();
    let mbox_fingerprint = fingerprint(mbox)?;
    let home = options.home.clone().unwrap_or_else(default_index_home);
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
    let outcome = index_into(mbox, &building, options, progress, cancel);
    let counts = match outcome {
        Ok(counts) => counts,
        Err(err) => {
            // Every tantivy handle died with `index_into`; the directory can go now.
            let _ = fs::remove_dir_all(&building);
            return Err(err);
        }
    };

    let built_at_unix = UNIX_EPOCH.elapsed().map_or(0, |d| d.as_secs() as i64);
    let meta = IndexMeta {
        format_version: FORMAT_VERSION,
        mbox: mbox_fingerprint,
        message_count: counts.messages,
        parse_failures: counts.parse_failures,
        built_at_unix,
        build_secs: start.elapsed().as_secs(),
        tantivy_version: tantivy::version_string().to_owned(),
    };
    meta.write(&building)?;
    remove_dir_if_present(&final_dir)?;
    fs::rename(&building, &final_dir)?;
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
        elapsed: start.elapsed(),
        index_bytes: dir_size(&final_dir)?,
        index_dir: final_dir,
        fallback,
    })
}

/// Creates an empty `.building` sibling of `final_dir`; an `Err` means the
/// location cannot take an index.
fn prepare_building_dir(final_dir: &Path) -> Result<()> {
    let building = building_dir(final_dir);
    remove_dir_if_present(&building)?;
    fs::create_dir_all(&building)?;
    Ok(())
}

fn building_dir(final_dir: &Path) -> PathBuf {
    let mut name = final_dir
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".building");
    final_dir.with_file_name(name)
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

struct Counts {
    messages: u64,
    parse_failures: u64,
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
    let mut counts = Counts {
        messages: 0,
        parse_failures: 0,
    };
    let mut last_progress = 0;
    while let Some(item) = reader.next() {
        if cancel.load(Ordering::Relaxed) {
            return Err(Error::Cancelled);
        }
        let raw = item?;
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
}
