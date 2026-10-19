//! The build's own safety (task 052): a rebuild beside an open index, two
//! builds of one mbox at once, and a message over the size limit.
//!
//! Each test copies a fixture into a temporary directory and keeps the index
//! home in another one, so nothing is written next to the fixtures or under
//! the user's data folder.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use mboxer_core::{
    BuildOptions, BuildReport, Error, IndexStatus, MboxIndex, Opened, SearchQuery, build,
    index_dir, status_in,
};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Copies `name` into a fresh temp dir and returns the dir and the copy's path.
fn copy_fixture(name: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join(name);
    fs::copy(fixture(name), &mbox).unwrap();
    (dir, mbox)
}

fn options(home: &Path) -> BuildOptions {
    BuildOptions {
        threads: 1,
        memory_per_thread_bytes: 20 << 20,
        home: Some(home.to_path_buf()),
        ..BuildOptions::default()
    }
}

fn build_in(mbox: &Path, home: &Path) -> mboxer_core::Result<BuildReport> {
    build(mbox, &options(home), |_| {}, &AtomicBool::new(false))
}

fn open_ready(mbox: &Path, home: &Path) -> MboxIndex {
    match MboxIndex::open_in(mbox, home).unwrap() {
        Opened::Ready(index) => *index,
        other => panic!("expected a ready index, got {other:?}"),
    }
}

/// The names in the `.mboxer` folder beside `mbox`, sorted.
fn sidecar_entries(mbox: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(mbox.parent().unwrap().join(".mboxer"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn total(index: &MboxIndex, text: &str) -> u64 {
    index.search(&SearchQuery::text(text)).unwrap().total as u64
}

#[test]
fn a_rebuild_replaces_an_index_that_is_still_open() {
    let (_dir, mbox) = copy_fixture("simple-lf.mbox");
    let home = tempfile::tempdir().unwrap();
    let name = mbox.file_name().unwrap().to_string_lossy().into_owned();
    build_in(&mbox, home.path()).unwrap();

    // A viewer holds the old index: its segment files stay open (and mapped).
    let old = open_ready(&mbox, home.path());
    let listing = old.list(&SearchQuery::default()).unwrap();
    assert_eq!(total(&old, "three"), 1);

    let report = build_in(&mbox, home.path()).unwrap();
    assert_eq!(report.index_dir, index_dir(&mbox));
    assert_eq!(report.messages, 3);
    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));
    assert_eq!(total(&open_ready(&mbox, home.path()), "three"), 1);
    // The old handle keeps answering from the files it already holds.
    assert_eq!(total(&old, "three"), 1);
    assert_eq!(listing.rows(0, 10).unwrap().len(), 3);
    assert!(
        !sidecar_entries(&mbox)
            .iter()
            .any(|n| n.ends_with(".building")),
        "{:?}",
        sidecar_entries(&mbox)
    );

    // Once the viewer lets go, the next build clears whatever was set aside.
    drop(listing);
    drop(old);
    build_in(&mbox, home.path()).unwrap();
    assert_eq!(sidecar_entries(&mbox), vec![name]);
}

#[test]
fn a_second_build_of_the_same_mbox_is_refused_while_the_first_runs() {
    let (_dir, mbox) = copy_fixture("threads.mbox");
    let home = tempfile::tempdir().unwrap();
    let (inside_tx, inside_rx) = mpsc::channel::<()>();
    let (done_tx, done_rx) = mpsc::channel::<()>();

    // The first build stops in its progress callback (after the documents,
    // before the commit) until the second one has returned.
    let first = {
        let mbox = mbox.clone();
        let home = home.path().to_path_buf();
        thread::spawn(move || {
            let mut waited = false;
            build(
                &mbox,
                &options(&home),
                |_| {
                    if !waited {
                        waited = true;
                        inside_tx.send(()).unwrap();
                        let _ = done_rx.recv_timeout(Duration::from_secs(60));
                    }
                },
                &AtomicBool::new(false),
            )
        })
    };
    inside_rx.recv_timeout(Duration::from_secs(60)).unwrap();
    let second = build_in(&mbox, home.path());
    done_tx.send(()).unwrap();
    let first = first.join().unwrap();

    let err = second.expect_err("the second build must not run beside the first");
    assert!(matches!(err, Error::BuildLocked { .. }), "{err}");
    assert!(err.to_string().contains("another build holds"), "{err}");
    let report = first.expect("the first build must finish");
    assert_eq!(report.index_dir, index_dir(&mbox));
    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));
    let name = mbox.file_name().unwrap().to_string_lossy().into_owned();
    assert_eq!(sidecar_entries(&mbox), vec![name]);

    // The lock went with the first build: a third one runs.
    build_in(&mbox, home.path()).unwrap();
}

#[test]
fn an_oversized_message_is_skipped_and_the_build_goes_on() {
    let (_dir, mbox) = copy_fixture("one-oversized.mbox");
    let home = tempfile::tempdir().unwrap();
    let bytes = fs::read(&mbox).unwrap();
    let oversized_at = bytes
        .windows(b"\nFrom bob@".len())
        .position(|w| w == b"\nFrom bob@")
        .unwrap() as u64
        + 1;

    let report = build(
        &mbox,
        &BuildOptions {
            max_message_bytes: 1024,
            ..options(home.path())
        },
        |_| {},
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.messages, 2);
    assert_eq!(report.parse_failures, 0);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.skipped_offsets, vec![oversized_at]);
    match status_in(&mbox, home.path()).unwrap() {
        IndexStatus::Fresh(meta) => {
            assert_eq!(meta.message_count, 2);
            assert_eq!(meta.skipped, 1);
        }
        other => panic!("{other:?}"),
    }

    let index = open_ready(&mbox, home.path());
    assert_eq!(total(&index, "before"), 1);
    assert_eq!(total(&index, "after"), 1);
    assert_eq!(total(&index, "filler"), 0);
}

#[test]
fn a_file_of_nothing_but_oversized_text_is_still_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("endless.mbox");
    fs::write(&mbox, vec![b'x'; 4096]).unwrap();
    let err = build(
        &mbox,
        &BuildOptions {
            max_message_bytes: 1024,
            ..options(home.path())
        },
        |_| {},
        &AtomicBool::new(false),
    )
    .unwrap_err();
    assert!(
        matches!(err, Error::MessageTooLarge { offset: 0, .. }),
        "{err}"
    );
    assert_eq!(status_in(&mbox, home.path()).unwrap(), IndexStatus::Missing);
    assert!(sidecar_entries(&mbox).is_empty());
}

#[test]
fn folders_a_crashed_build_left_behind_are_cleared() {
    let (_dir, mbox) = copy_fixture("simple-lf.mbox");
    let home = tempfile::tempdir().unwrap();
    let name = mbox.file_name().unwrap().to_string_lossy().into_owned();
    let sidecar = mbox.parent().unwrap().join(".mboxer");
    for leftover in [".building", ".old", ".old.1"] {
        let folder = sidecar.join(format!("{name}{leftover}"));
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("stray"), b"left by a crash").unwrap();
    }
    build_in(&mbox, home.path()).unwrap();
    assert_eq!(sidecar_entries(&mbox), vec![name]);
}

/// A process that holds a file of the old index without sharing its deletion
/// (a virus scanner, a backup tool) blocks every rename of the folder on
/// Windows: the build gives up, and the old index stays whole and in place.
#[cfg(windows)]
#[test]
fn a_held_old_index_stays_whole_when_the_swap_cannot_run() {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ: u32 = 1;

    let (_dir, mbox) = copy_fixture("simple-lf.mbox");
    let home = tempfile::tempdir().unwrap();
    let name = mbox.file_name().unwrap().to_string_lossy().into_owned();
    build_in(&mbox, home.path()).unwrap();
    let before = sidecar_files(&index_dir(&mbox));

    let holder = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(index_dir(&mbox).join("mboxer.json"))
        .unwrap();
    let err = build_in(&mbox, home.path()).unwrap_err();
    assert!(matches!(err, Error::Io(_)), "{err}");
    drop(holder);

    assert_eq!(sidecar_entries(&mbox), vec![name]);
    assert_eq!(sidecar_files(&index_dir(&mbox)), before);
    assert_eq!(total(&open_ready(&mbox, home.path()), "three"), 1);
}

/// The file names inside `dir`, sorted.
#[cfg(windows)]
fn sidecar_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
