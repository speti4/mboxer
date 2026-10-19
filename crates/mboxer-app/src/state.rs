//! The application state: the open mbox files, each with its index and its
//! own listing, and which of them is active (D-006).
//!
//! No Tauri types here; [`OpenMbox::load`] is the whole open-or-build flow and
//! runs against the fixtures in the tests, as does [`OpenFiles`].

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[cfg(test)]
use mboxer_core::index_dir;
use mboxer_core::{
    BuildOptions, Error, IndexStatus, Listing, MboxIndex, Opened, Progress, SearchQuery, build,
    status,
};

use crate::folders::label_tree;
use crate::print::PrintJobs;
use crate::types::{FileInfo, IndexState, LabelNode, OpenSummary};

/// The error every command gets for a file id the set does not hold.
pub const NOT_OPEN: &str = "that file is no longer open";

/// Managed by Tauri; one per process.
#[derive(Default)]
pub struct AppState {
    pub files: Mutex<OpenFiles>,
    /// Set by `cancel_open`, read by the running index build.
    pub cancel: Arc<AtomicBool>,
    /// True while an open is in flight; a second open is refused.
    pub busy: AtomicBool,
    /// Bumped on every open and query change; stamps every page of rows.
    pub generation: AtomicU64,
    /// Where the last attachment or message was saved; the next save dialog starts there.
    pub last_save_dir: Mutex<Option<PathBuf>>,
    /// Files the system asked the app to open before the webview was ready for them.
    pub system_opens: Mutex<SystemOpens>,
    /// The documents of the open print windows (macOS, task 020).
    pub print_jobs: Mutex<PrintJobs>,
}

/// The files macOS hands the app through Finder "Open with", a Dock drop or
/// `open -a` (task 019). They arrive as an event on the main thread, possibly
/// before the webview has loaded: until the webview asks for its start-up
/// paths they wait here, from then on they go straight to it as an event.
#[derive(Debug, Default)]
pub struct SystemOpens {
    pub paths: Vec<String>,
    /// The webview has asked for the start-up paths and listens for the rest.
    pub webview_ready: bool,
}

impl AppState {
    /// The open files; a poisoned lock is recovered, the data is plain values.
    pub fn files(&self) -> MutexGuard<'_, OpenFiles> {
        self.files.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn system_opens(&self) -> MutexGuard<'_, SystemOpens> {
        self.system_opens
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub fn print_jobs(&self) -> MutexGuard<'_, PrintJobs> {
        self.print_jobs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
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

/// The open files in the order of the folder pane, and the active one. A
/// non-empty set always has an active file; ids are handed out once per
/// process and never reused, so a command that names a closed file fails
/// instead of touching another one.
#[derive(Debug, Default)]
pub struct OpenFiles {
    files: Vec<OpenMbox>,
    active: Option<u32>,
    next_id: u32,
}

impl OpenFiles {
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub fn active_id(&self) -> Option<u32> {
        self.active
    }

    pub fn by_id(&self, id: u32) -> Result<&OpenMbox, String> {
        self.files
            .iter()
            .find(|open| open.id == id)
            .ok_or_else(|| NOT_OPEN.to_owned())
    }

    pub fn by_id_mut(&mut self, id: u32) -> Result<&mut OpenMbox, String> {
        self.files
            .iter_mut()
            .find(|open| open.id == id)
            .ok_or_else(|| NOT_OPEN.to_owned())
    }

    /// The id of the open file at `path` (compared canonically), if any.
    pub fn id_of(&self, path: &Path) -> Option<u32> {
        let canonical = canonical(path);
        self.files
            .iter()
            .find(|open| open.canonical == canonical)
            .map(|open| open.id)
    }

    /// Adds `open` at the end and returns its id. A file already in the set
    /// is replaced in place and keeps its id. The first file of an empty set
    /// becomes active; otherwise the active file does not change.
    pub fn insert(&mut self, mut open: OpenMbox) -> u32 {
        if let Some(id) = self.id_of(&open.path) {
            open.id = id;
            let slot = self
                .files
                .iter_mut()
                .find(|existing| existing.id == id)
                .expect("id_of found it");
            *slot = open;
            return id;
        }
        self.next_id += 1;
        open.id = self.next_id;
        let id = open.id;
        self.files.push(open);
        if self.active.is_none() {
            self.active = Some(id);
        }
        id
    }

    /// Makes `id` the active file and stamps its listing with `generation`,
    /// so the pages of the previously active file are told apart.
    pub fn activate(&mut self, id: u32, generation: u64) -> Result<(), String> {
        let open = self.by_id_mut(id)?;
        open.generation = generation;
        self.active = Some(id);
        Ok(())
    }

    /// Removes `id`; when it was active, its neighbour below (else above)
    /// takes over. Returns the active id afterwards, `None` for an empty set.
    pub fn close(&mut self, id: u32) -> Result<Option<u32>, String> {
        let at = self
            .files
            .iter()
            .position(|open| open.id == id)
            .ok_or_else(|| NOT_OPEN.to_owned())?;
        self.files.remove(at);
        if self.active == Some(id) {
            self.active = self
                .files
                .get(at)
                .or_else(|| self.files.last())
                .map(|open| open.id);
        }
        Ok(self.active)
    }

    /// The pane's rows: every open file, the active one marked.
    pub fn infos(&self) -> Vec<FileInfo> {
        self.files
            .iter()
            .map(|open| FileInfo {
                id: open.id,
                path: open.path.display().to_string(),
                file_name: file_name(&open.path),
                message_count: open.index.message_count(),
                index: open.index_state.clone(),
                active: self.active == Some(open.id),
            })
            .collect()
    }

    /// The summary of file `id`, with the whole set for the pane.
    pub fn summary(&self, id: u32) -> Result<OpenSummary, String> {
        let mut summary = self.by_id(id)?.summary();
        summary.files = self.infos();
        Ok(summary)
    }
}

/// The canonical form for telling two spellings of one path apart; a path
/// that cannot be canonicalized (gone, no rights) is compared as written.
fn canonical(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// An mbox with a fresh index and the listing behind the message list.
#[derive(Debug)]
pub struct OpenMbox {
    /// The id the webview addresses this file by; 0 until [`OpenFiles::insert`].
    pub id: u32,
    pub path: PathBuf,
    canonical: PathBuf,
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
            id: 0,
            path: path.to_path_buf(),
            canonical: canonical(path),
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

    /// This file alone; [`OpenFiles::summary`] adds the set.
    pub fn summary(&self) -> OpenSummary {
        OpenSummary {
            id: self.id,
            files: Vec::new(),
            path: self.path.display().to_string(),
            file_name: file_name(&self.path),
            message_count: self.index.message_count(),
            labels: self.labels.clone(),
            index: self.index_state.clone(),
            index_dir: self.index.dir().display().to_string(),
            generation: self.generation,
            total: self.listing.len(),
            messages: self.listing.message_count(),
            conversations: self.listing.group_count(),
        }
    }
}

/// A fresh index is opened; a missing or stale one is rebuilt first (D-004).
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
                skipped: report.skipped,
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
        assert_eq!(summary.index_dir, index_dir(&mbox).display().to_string());
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

        // Body word (accent-folded) + sender word + a two-hour window around its date.
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

        // The same words with a sender that is not there: nothing.
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
    fn accented_labels_on_the_category_prefix_boundary_open() {
        // Task 049: a two-byte character on byte 9 of a label panicked the
        // label tree after the index was built.
        let dir = tempfile::tempdir().unwrap();
        let mbox = dir.path().join("accented-labels.mbox");
        let message = |n: u32, labels: &str| {
            format!(
                "From {n}@xxx Tue Jan 13 06:30:04 +0000 2026\n\
                 X-Gmail-Labels: {labels}\n\
                 From: Anna <anna@example.com>\n\
                 Subject: Label {n}\n\
                 Date: Tue, 13 Jan 2026 07:30:04 +0100\n\
                 Message-ID: <label{n}@mail.example.com>\n\
                 \n\
                 Body {n}.\n\n"
            )
        };
        let text = message(1, "Különböző,Inbox")
            + &message(2, "Beérkező levelek,Category Promotions")
            + &message(3, "=?UTF-8?Q?K=C3=BCl=C3=B6nb=C3=B6z=C5=91?=");
        fs::write(&mbox, text).unwrap();

        let open = load(&mbox, 1);
        assert_eq!(open.listing.len(), 3);
        let names: Vec<&str> = open.labels.iter().map(|node| node.name.as_str()).collect();
        assert_eq!(
            names,
            ["Inbox", "Categories", "Beérkező levelek", "Különböző"]
        );
        assert_eq!(open.labels[3].count, 2);
        assert_eq!(open.labels[1].children[0].name, "Promotions");
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

    fn load(mbox: &Path, generation: u64) -> OpenMbox {
        OpenMbox::load(mbox, &AtomicBool::new(false), generation, false, |_| {}).unwrap()
    }

    #[test]
    fn the_set_hands_out_ids_and_the_first_file_is_active() {
        let (_dir, a) = fixture_copy("gmail-crlf.mbox");
        let (_dir2, b) = fixture_copy("threads.mbox");
        let mut files = OpenFiles::default();
        assert!(files.is_empty());
        assert_eq!(files.active_id(), None);
        assert_eq!(files.by_id(1).unwrap_err(), NOT_OPEN);

        let id_a = files.insert(load(&a, 1));
        let id_b = files.insert(load(&b, 2));
        assert_eq!((id_a, id_b), (1, 2));
        assert_eq!(
            files.active_id(),
            Some(id_a),
            "the first file of an empty set is active"
        );
        assert_eq!(files.id_of(&a), Some(id_a));
        assert_eq!(files.id_of(&b), Some(id_b));
        assert_eq!(files.id_of(Path::new("nowhere.mbox")), None);

        let infos = files.infos();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].file_name, "gmail-crlf.mbox");
        assert!(infos[0].active);
        assert!(!infos[1].active);
        assert_eq!(infos[1].message_count, 6);

        let summary = files.summary(id_b).unwrap();
        assert_eq!(summary.id, id_b);
        assert_eq!(summary.file_name, "threads.mbox");
        assert_eq!(summary.files.len(), 2);
        assert_eq!(files.summary(9).unwrap_err(), NOT_OPEN);
    }

    #[test]
    fn opening_a_file_again_keeps_its_id_and_place() {
        let (_dir, a) = fixture_copy("gmail-crlf.mbox");
        let (_dir2, b) = fixture_copy("threads.mbox");
        let mut files = OpenFiles::default();
        let id_a = files.insert(load(&a, 1));
        let id_b = files.insert(load(&b, 2));
        files.activate(id_b, 3).unwrap();

        // The same file, spelt through its parent: one entry, the old id, first in the pane.
        let spelt = a.parent().unwrap().join(".").join(a.file_name().unwrap());
        let again = files.insert(load(&spelt, 4));
        assert_eq!(again, id_a);
        let infos = files.infos();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].id, id_a);
        assert_eq!(
            files.active_id(),
            Some(id_b),
            "a re-open does not steal the active file"
        );
        assert_eq!(files.by_id(id_a).unwrap().generation, 4);
    }

    #[test]
    fn activation_restamps_the_generation() {
        let (_dir, a) = fixture_copy("gmail-crlf.mbox");
        let (_dir2, b) = fixture_copy("threads.mbox");
        let mut files = OpenFiles::default();
        let id_a = files.insert(load(&a, 1));
        let id_b = files.insert(load(&b, 2));
        files.activate(id_b, 7).unwrap();
        assert_eq!(files.active_id(), Some(id_b));
        assert_eq!(files.by_id(id_b).unwrap().generation, 7);
        assert_eq!(files.by_id(id_a).unwrap().generation, 1);
        assert_eq!(files.activate(42, 8).unwrap_err(), NOT_OPEN);
        assert_eq!(files.active_id(), Some(id_b));
    }

    #[test]
    fn closing_hands_the_active_role_to_the_neighbour() {
        let (_dir, a) = fixture_copy("gmail-crlf.mbox");
        let (_dir2, b) = fixture_copy("threads.mbox");
        let (_dir3, c) = fixture_copy("attachments.mbox");
        let mut files = OpenFiles::default();
        let id_a = files.insert(load(&a, 1));
        let id_b = files.insert(load(&b, 2));
        let id_c = files.insert(load(&c, 3));

        // Closing a file that is not active leaves the active one alone.
        assert_eq!(files.close(id_c).unwrap(), Some(id_a));
        assert_eq!(files.infos().len(), 2);
        assert_eq!(files.close(id_c).unwrap_err(), NOT_OPEN);

        // Closing the active first file: the one below takes over.
        assert_eq!(files.close(id_a).unwrap(), Some(id_b));
        assert_eq!(files.by_id(id_a).unwrap_err(), NOT_OPEN);

        // A new file gets a fresh id, never a reused one.
        let id_d = files.insert(load(&c, 4));
        assert_eq!(id_d, 4);
        files.activate(id_d, 5).unwrap();
        // Closing the active last file: the one above takes over.
        assert_eq!(files.close(id_d).unwrap(), Some(id_b));
        assert_eq!(files.close(id_b).unwrap(), None);
        assert!(files.is_empty());
        assert_eq!(files.active_id(), None);
    }
}
