//! The application state: the open mbox, its index and the current listing.
//!
//! No Tauri types here; [`OpenMbox::load`] is the whole open-or-build flow and
//! runs against the fixtures in the tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use mboxer_core::{
    BuildOptions, Error, IndexStatus, Listing, MboxIndex, Opened, Progress, SearchQuery, build,
    status,
};

use crate::folders::label_tree;
use crate::types::{IndexState, LabelNode, OpenSummary};

/// Managed by Tauri; one per process.
#[derive(Default)]
pub struct AppState {
    pub open: Mutex<Option<OpenMbox>>,
    /// Set by `cancel_open`, read by the running index build.
    pub cancel: Arc<AtomicBool>,
    /// True while an open is in flight; a second open is refused.
    pub busy: AtomicBool,
    /// Bumped on every open and query change; stamps every page of rows.
    pub generation: AtomicU64,
    /// Where the last attachment was saved; the next save dialog starts there.
    pub last_save_dir: Mutex<Option<PathBuf>>,
}

impl AppState {
    /// The open file; a poisoned lock is recovered, the data is plain values.
    pub fn open(&self) -> MutexGuard<'_, Option<OpenMbox>> {
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn last_save_dir(&self) -> Option<PathBuf> {
        self.last_save_dir
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn set_last_save_dir(&self, dir: PathBuf) {
        *self
            .last_save_dir
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(dir);
    }
}

/// An mbox with a fresh index and the listing behind the message list.
#[derive(Debug)]
pub struct OpenMbox {
    pub path: PathBuf,
    pub index: MboxIndex,
    pub index_state: IndexState,
    pub labels: Vec<LabelNode>,
    pub query: SearchQuery,
    pub listing: Listing,
    pub generation: u64,
}

impl OpenMbox {
    /// Opens `path`: reuses its index when fresh, builds it otherwise
    /// (reporting `progress`, stopping on `cancel`), then lists every message,
    /// one row per conversation when `group_threads` is set.
    pub fn load(
        path: &Path,
        cancel: &AtomicBool,
        generation: u64,
        group_threads: bool,
        progress: impl FnMut(Progress),
    ) -> mboxer_core::Result<Self> {
        let (index, index_state) = open_or_build(path, cancel, progress)?;
        let labels = label_tree(index.label_counts()?);
        let query = SearchQuery {
            group_threads,
            ..SearchQuery::default()
        };
        let listing = index.list(&query)?;
        Ok(Self {
            path: path.to_path_buf(),
            index,
            index_state,
            labels,
            query,
            listing,
            generation,
        })
    }

    /// Replaces the listing with the matches of `query`.
    pub fn set_query(&mut self, query: SearchQuery, generation: u64) -> mboxer_core::Result<()> {
        self.listing = self.index.list(&query)?;
        self.query = query;
        self.generation = generation;
        Ok(())
    }

    pub fn summary(&self) -> OpenSummary {
        OpenSummary {
            path: self.path.display().to_string(),
            file_name: self
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            message_count: self.index.message_count(),
            labels: self.labels.clone(),
            index: self.index_state.clone(),
            generation: self.generation,
            total: self.listing.len(),
            messages: self.listing.message_count(),
            conversations: self.listing.group_count(),
        }
    }
}

/// A fresh index is opened; a missing or stale one is rebuilt first (D-04).
pub fn open_or_build(
    path: &Path,
    cancel: &AtomicBool,
    progress: impl FnMut(Progress),
) -> mboxer_core::Result<(MboxIndex, IndexState)> {
    if let IndexStatus::Fresh(meta) = status(path)?
        && let Opened::Ready(index) = MboxIndex::open(path)?
    {
        return Ok((
            *index,
            IndexState::Reused {
                built_at_unix: meta.built_at_unix,
            },
        ));
    }
    let report = build(path, &BuildOptions::default(), progress, cancel)?;
    match MboxIndex::open(path)? {
        Opened::Ready(index) => Ok((
            *index,
            IndexState::Built {
                build_secs: report.elapsed.as_secs(),
                parse_failures: report.parse_failures,
            },
        )),
        Opened::Missing => Err(Error::IndexMeta(
            "the index was built but cannot be found".to_owned(),
        )),
        Opened::Stale(reason) => Err(Error::IndexMeta(format!(
            "the index was built but is already stale: {reason}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::Ordering;

    fn fixture_copy(name: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let mbox = dir.path().join(name);
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../mboxer-core/tests/fixtures")
            .join(name);
        fs::copy(source, &mbox).unwrap();
        (dir, mbox)
    }

    #[test]
    fn first_open_builds_second_reuses() {
        let (_dir, mbox) = fixture_copy("gmail-crlf.mbox");
        let cancel = AtomicBool::new(false);
        let mut progress_calls = 0;
        let open = OpenMbox::load(&mbox, &cancel, 1, false, |_| progress_calls += 1).unwrap();
        assert!(progress_calls > 0);
        assert!(matches!(open.index_state, IndexState::Built { .. }));
        assert_eq!(open.listing.len(), 2);
        assert_eq!(open.labels[0].name, "Inbox");
        let summary = open.summary();
        assert_eq!(summary.file_name, "gmail-crlf.mbox");
        assert_eq!(summary.total, 2);
        assert_eq!(summary.message_count, 2);
        assert_eq!((summary.messages, summary.conversations), (2, 2));

        let again = OpenMbox::load(&mbox, &cancel, 2, false, |_| {}).unwrap();
        assert!(matches!(again.index_state, IndexState::Reused { .. }));
        assert_eq!(again.generation, 2);
    }

    #[test]
    fn query_changes_replace_the_listing() {
        let (_dir, mbox) = fixture_copy("gmail-crlf.mbox");
        let mut open = OpenMbox::load(&mbox, &AtomicBool::new(false), 1, false, |_| {}).unwrap();
        let sent = SearchQuery {
            labels: vec!["Sent".to_owned()],
            ..SearchQuery::default()
        };
        open.set_query(sent.clone(), 5).unwrap();
        assert_eq!(open.listing.len(), 1);
        assert_eq!(open.query, sent);
        assert_eq!(open.summary().generation, 5);
    }

    #[test]
    fn text_and_filters_combine_in_one_listing() {
        let (_dir, mbox) = fixture_copy("gmail-crlf.mbox");
        let mut open = OpenMbox::load(&mbox, &AtomicBool::new(false), 1, false, |_| {}).unwrap();
        let all = open.listing.rows(0, 10).unwrap();
        assert_eq!(all.len(), 2);
        let first = all
            .iter()
            .find(|row| row.hit.offset == 0)
            .expect("the first message is listed");
        let date = first
            .hit
            .date
            .expect("the fixture's first message has a date");

        // Body word (accent-folded) + sender word + a one-day window around its date.
        let narrow = SearchQuery {
            text: Some("szamla".to_owned()),
            from: Some("anna".to_owned()),
            after: Some(date - 3600),
            before: Some(date + 3600),
            ..SearchQuery::default()
        };
        open.set_query(narrow, 2).unwrap();
        assert_eq!(open.listing.len(), 1);
        assert_eq!(open.listing.rows(0, 1).unwrap()[0].hit.offset, 0);

        // The same window with a sender that is not there: nothing.
        let wrong_sender = SearchQuery {
            text: Some("szamla".to_owned()),
            from: Some("nobody".to_owned()),
            ..SearchQuery::default()
        };
        open.set_query(wrong_sender, 3).unwrap();
        assert!(open.listing.is_empty());

        // No text at all: a structured filter alone still narrows the list.
        let attachments = SearchQuery {
            text: Some("   ".to_owned()),
            has_attachment: Some(true),
            ..SearchQuery::default()
        };
        open.set_query(attachments, 4).unwrap();
        let with = open.listing.len();
        assert!(with < 2, "the fixture has a message without attachments");
        assert!(
            open.listing
                .rows(0, 10)
                .unwrap()
                .iter()
                .all(|row| row.hit.has_attachment),
            "every row has an attachment"
        );
    }

    #[test]
    fn conversations_are_grouped_from_the_first_listing() {
        let (_dir, mbox) = fixture_copy("threads.mbox");
        let mut open = OpenMbox::load(&mbox, &AtomicBool::new(false), 1, true, |_| {}).unwrap();
        assert_eq!(open.listing.len(), 3);
        assert_eq!(open.listing.message_count(), 6);
        let summary = open.summary();
        assert_eq!(
            (summary.total, summary.messages, summary.conversations),
            (3, 6, 3)
        );
        assert_eq!(summary.message_count, 6);
        // The label tree: Work carries its sub-labels, the Takeout category sits under Categories.
        let work = summary
            .labels
            .iter()
            .find(|node| node.name == "Work")
            .expect("a Work node");
        assert_eq!(work.labels, ["Work", "Work/Invoices", "Work/Invoices/2026"]);
        assert!(summary.labels.iter().any(|node| node.name == "Categories"));

        assert_eq!(open.listing.toggle(0), Some(6));
        let any_work = SearchQuery {
            any_labels: work.labels.clone(),
            group_threads: true,
            ..SearchQuery::default()
        };
        open.set_query(any_work, 2).unwrap();
        assert_eq!(open.listing.len(), 2, "a new query starts collapsed");
        assert_eq!(open.listing.message_count(), 4);
        assert_eq!(open.summary().conversations, 2);

        let flat = SearchQuery::default();
        open.set_query(flat, 3).unwrap();
        assert_eq!(open.listing.len(), 6);
    }

    #[test]
    fn a_cancelled_build_leaves_no_index() {
        let (_dir, mbox) = fixture_copy("gmail-crlf.mbox");
        let cancel = AtomicBool::new(true);
        let err = OpenMbox::load(&mbox, &cancel, 1, false, |_| {}).unwrap_err();
        assert!(matches!(err, Error::Cancelled), "{err}");
        assert!(matches!(status(&mbox).unwrap(), IndexStatus::Missing));
        cancel.store(false, Ordering::Relaxed);
        OpenMbox::load(&mbox, &cancel, 2, false, |_| {}).unwrap();
    }

    #[test]
    fn a_text_file_is_not_an_mbox() {
        let (_dir, path) = fixture_copy("not-an-mbox.txt");
        let err = OpenMbox::load(&path, &AtomicBool::new(false), 1, false, |_| {}).unwrap_err();
        assert!(matches!(err, Error::NotAnMbox { offset: 0 }), "{err}");
    }
}
