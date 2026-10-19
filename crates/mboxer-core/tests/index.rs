//! End-to-end tests of the sidecar index over the synthetic fixtures.
//!
//! Each test copies a fixture into a temporary directory first: the index is
//! written next to the mbox, and the fixtures directory must stay clean.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use mboxer_core::index::fingerprint::{FORMAT_VERSION, META_FILE};
use mboxer_core::{
    BuildOptions, Error, Hit, IndexStatus, ListRow, Listing, MboxIndex, Opened, RowKind,
    SearchQuery, Sort, StaleReason, build, fallback_dir, index_dir, read_at, status, status_in,
};
use tempfile::TempDir;

/// The hits of a page of list rows.
fn hits(rows: Vec<ListRow>) -> Vec<Hit> {
    rows.into_iter().map(|row| row.hit).collect()
}

/// `(subject, kind)` of every row of `listing`.
fn outline(listing: &Listing) -> Vec<(String, RowKind)> {
    listing
        .rows(0, 100)
        .unwrap()
        .into_iter()
        .map(|row| (row.hit.subject.unwrap_or_default(), row.kind))
        .collect()
}

fn message() -> RowKind {
    RowKind::Message
}

fn thread(count: u32, expanded: bool) -> RowKind {
    RowKind::Thread { count, expanded }
}

fn member(index: u32, count: u32) -> RowKind {
    RowKind::Member { index, count }
}

/// Offset of the message with `subject` in the fixture's index.
fn offset_of(index: &MboxIndex, subject: &str) -> u64 {
    index
        .search(&SearchQuery {
            limit: 100,
            ..SearchQuery::default()
        })
        .unwrap()
        .hits
        .into_iter()
        .find(|hit| hit.subject.as_deref() == Some(subject))
        .unwrap_or_else(|| panic!("no message with subject {subject:?}"))
        .offset
}

fn grouped(index: &MboxIndex, query: SearchQuery) -> Listing {
    index
        .list(&SearchQuery {
            group_threads: true,
            ..query
        })
        .unwrap()
}

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

fn build_quiet(mbox: &Path) -> mboxer_core::BuildReport {
    build(
        mbox,
        &BuildOptions {
            threads: 1,
            memory_per_thread_bytes: 20 << 20,
            ..BuildOptions::default()
        },
        |_| {},
        &AtomicBool::new(false),
    )
    .unwrap()
}

fn open_ready(mbox: &Path) -> MboxIndex {
    match MboxIndex::open(mbox).unwrap() {
        Opened::Ready(index) => *index,
        other => panic!("expected a ready index, got {other:?}"),
    }
}

fn built(name: &str) -> (TempDir, PathBuf, MboxIndex) {
    let (dir, mbox) = copy_fixture(name);
    build_quiet(&mbox);
    let index = open_ready(&mbox);
    (dir, mbox, index)
}

fn total(index: &MboxIndex, text: &str) -> usize {
    index.search(&SearchQuery::text(text)).unwrap().total
}

#[test]
fn accent_and_case_insensitive_body_search() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    assert_eq!(index.message_count(), 2);
    for query in ["szamla", "számla", "SZÁMLA", "Szamla"] {
        let result = index.search(&SearchQuery::text(query)).unwrap();
        assert_eq!(result.total, 1, "query {query:?}");
        assert_eq!(result.hits[0].offset, 0);
        assert!(result.warnings.is_empty());
    }
    // Only in the body of message one (accented in the fixture).
    assert_eq!(total(&index, "konyveles"), 1);
    assert_eq!(total(&index, "KÖNYVELÉS"), 1);
    // Only in message two, plain ASCII query against an accented body word.
    assert_eq!(total(&index, "koszonom"), 1);
    assert_eq!(total(&index, "nincsilyenszo"), 0);
}

#[test]
fn phrase_queries_need_the_words_in_order() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    assert_eq!(total(&index, "\"könyvelés felől\""), 1);
    assert_eq!(total(&index, "\"konyveles felol\""), 1);
    assert_eq!(total(&index, "\"felől könyvelés\""), 0);
    // Terms are AND-ed by default: one from each message matches nothing.
    assert_eq!(total(&index, "számla köszönöm"), 0);
    assert_eq!(total(&index, "számla OR köszönöm"), 2);
}

#[test]
fn hits_carry_stored_metadata_and_seek_back_to_the_mbox() {
    let (_dir, mbox, index) = built("gmail-crlf.mbox");
    let result = index.search(&SearchQuery::text("számla")).unwrap();
    let hit = &result.hits[0];
    assert_eq!(hit.subject.as_deref(), Some("Számla január"));
    assert_eq!(hit.from, ["Anna Példa <anna@example.com>"]);
    assert_eq!(hit.to, ["Bela <bela@example.com>"]);
    assert_eq!(
        hit.labels,
        ["Inbox", "Important", "Category personal", "Opened"]
    );
    assert!(hit.has_attachment);
    assert_eq!(hit.attachment_names, ["számla-2026-01.pdf"]);
    assert_eq!(hit.date, Some(1_768_285_804));
    assert_eq!(hit.score, None);

    let all = index.search(&SearchQuery::default()).unwrap();
    assert_eq!(all.total, 2);
    for hit in &all.hits {
        let raw = read_at(&mbox, hit.offset, hit.len).unwrap();
        assert_eq!(raw.offset, hit.offset);
        assert_eq!(raw.len, hit.len);
        let looked_up = index.lookup(hit.offset).unwrap().unwrap();
        assert_eq!(&looked_up, hit);
    }
    assert!(index.lookup(7).unwrap().is_none());
}

#[test]
fn structured_filters_and_paging() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    let all = index.search(&SearchQuery::default()).unwrap();
    assert_eq!(all.total, 2);
    // Newest first.
    let dates: Vec<_> = all.hits.iter().map(|h| h.date.unwrap()).collect();
    assert!(dates[0] > dates[1], "{dates:?}");

    let from_anna = index
        .search(&SearchQuery {
            from: Some("anna".into()),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(from_anna.total, 1);
    assert_eq!(from_anna.hits[0].offset, 0);

    let to_konyveles = index
        .search(&SearchQuery {
            to: Some("konyveles".into()),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(to_konyveles.total, 1, "cc counts as a recipient");

    let subject = |words: &str| {
        index
            .search(&SearchQuery {
                subject: Some(words.into()),
                ..SearchQuery::default()
            })
            .unwrap()
            .total
    };
    assert_eq!(subject("szamla"), 1, "folded, subject only");
    assert_eq!(subject("valasz"), 1);
    assert_eq!(subject("köszönöm"), 0, "a body word is not a subject word");
    assert_eq!(total(&index, "köszönöm"), 1);
    let subject_prefix = index
        .search(&SearchQuery {
            subject: Some("janu".into()),
            prefix_last_word: true,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(
        subject_prefix.total, 1,
        "the subject word being typed is a prefix"
    );

    let personal = index
        .search(&SearchQuery {
            labels: vec!["Category personal".into()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(personal.total, 1);
    let sent_and_opened = index
        .search(&SearchQuery {
            labels: vec!["Sent".into(), "Opened".into()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(sent_and_opened.total, 1);
    let impossible = index
        .search(&SearchQuery {
            labels: vec!["Sent".into(), "Inbox".into()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(impossible.total, 0);

    let with_attachment = index
        .search(&SearchQuery {
            has_attachment: Some(true),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(with_attachment.total, 1);
    assert_eq!(with_attachment.hits[0].offset, 0);
    let without = index
        .search(&SearchQuery {
            has_attachment: Some(false),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(without.total, 1);

    // Date range: [newest, ∞) keeps one; (-∞, newest) keeps the other.
    let newest = dates[0];
    let after = index
        .search(&SearchQuery {
            after: Some(newest),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(after.total, 1);
    assert_eq!(after.hits[0].date, Some(newest));
    let before = index
        .search(&SearchQuery {
            before: Some(newest),
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(before.total, 1);
    assert_eq!(before.hits[0].date, Some(dates[1]));

    // Paging keeps the total and shifts the page.
    let page = index
        .search(&SearchQuery {
            limit: 1,
            offset: 1,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.hits.len(), 1);
    assert_eq!(page.hits[0].offset, all.hits[1].offset);
    let count_only = index
        .search(&SearchQuery {
            limit: 0,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!((count_only.total, count_only.hits.len()), (2, 0));

    let scored = index
        .search(&SearchQuery {
            text: Some("számla".into()),
            sort: Sort::Relevance,
            ..SearchQuery::default()
        })
        .unwrap();
    assert!(scored.hits[0].score.is_some_and(|s| s > 0.0));
}

#[test]
fn legacy_charset_body_is_searchable_folded() {
    let (_dir, _mbox, index) = built("hungarian-charsets.mbox");
    assert_eq!(total(&index, "arvizturo tukorfurogep"), 1);
    assert_eq!(total(&index, "árvíztűrő"), 1);
    assert_eq!(total(&index, "\"tükörfúrógép árvíztűrő\""), 0);
}

#[test]
fn html_only_message_is_indexed_as_text() {
    let (_dir, _mbox, index) = built("html-only.mbox");
    assert_eq!(total(&index, "arajanlat"), 1);
    assert_eq!(total(&index, "tükörfúrógépre"), 1);
    for tag_or_hidden in ["html", "body", "style", "color", "red", "javascriptword"] {
        assert_eq!(total(&index, tag_or_hidden), 0, "{tag_or_hidden:?} leaked");
    }
    let hit = &index.search(&SearchQuery::text("arajanlat")).unwrap().hits[0];
    assert_eq!(hit.subject.as_deref(), Some("Ajánlat a gépre"));
    assert_eq!(hit.from, ["Iroda Kft <iroda@example.com>"]);
    assert!(!hit.has_attachment);
}

#[test]
fn freshness_follows_the_mbox_and_the_format() {
    let (_dir, mbox) = copy_fixture("gmail-crlf.mbox");
    assert_eq!(status(&mbox).unwrap(), IndexStatus::Missing);
    assert!(matches!(MboxIndex::open(&mbox).unwrap(), Opened::Missing));

    let report = build_quiet(&mbox);
    assert_eq!(report.messages, 2);
    assert_eq!(report.parse_failures, 0);
    assert!(report.index_bytes > 0);
    assert_eq!(report.index_dir, index_dir(&mbox));
    let meta = match status(&mbox).unwrap() {
        IndexStatus::Fresh(meta) => meta,
        other => panic!("{other:?}"),
    };
    assert_eq!(meta.format_version, FORMAT_VERSION);
    assert_eq!(meta.message_count, 2);

    // A different format version means rebuild.
    let meta_path = index_dir(&mbox).join(META_FILE);
    let json = fs::read_to_string(&meta_path).unwrap();
    fs::write(
        &meta_path,
        json.replace(
            &format!("\"format_version\": {FORMAT_VERSION}"),
            "\"format_version\": 0",
        ),
    )
    .unwrap();
    assert_eq!(
        status(&mbox).unwrap(),
        IndexStatus::Stale(StaleReason::Format {
            found: 0,
            expected: FORMAT_VERSION
        })
    );
    fs::write(&meta_path, "{not json").unwrap();
    assert!(matches!(
        status(&mbox).unwrap(),
        IndexStatus::Stale(StaleReason::Unreadable(_))
    ));
    fs::write(&meta_path, json).unwrap();
    assert!(matches!(status(&mbox).unwrap(), IndexStatus::Fresh(_)));

    // A changed mbox means rebuild; rebuilding over the old index works.
    let mut bytes = fs::read(&mbox).unwrap();
    bytes.push(b'\n');
    fs::write(&mbox, bytes).unwrap();
    assert_eq!(
        status(&mbox).unwrap(),
        IndexStatus::Stale(StaleReason::Fingerprint)
    );
    assert!(matches!(
        MboxIndex::open(&mbox).unwrap(),
        Opened::Stale(StaleReason::Fingerprint)
    ));
    build_quiet(&mbox);
    let reopened = open_ready(&mbox);
    assert_eq!(
        reopened.search(&SearchQuery::text("számla")).unwrap().total,
        1
    );
}

/// Builds with the fallback root set to `home`, one thread.
fn build_with_home(mbox: &Path, home: &Path) -> mboxer_core::Result<mboxer_core::BuildReport> {
    build(
        mbox,
        &BuildOptions {
            threads: 1,
            memory_per_thread_bytes: 20 << 20,
            home: Some(home.to_path_buf()),
            ..BuildOptions::default()
        },
        |_| {},
        &AtomicBool::new(false),
    )
}

/// A file named `.mboxer` beside the mbox: `.mboxer/<name>.building` cannot
/// be created, which is what a read-only share or a denied folder looks like
/// to the build (the Windows read-only folder attribute does not block it).
fn block_sidecar(mbox: &Path) {
    fs::write(mbox.parent().unwrap().join(".mboxer"), b"not a directory").unwrap();
}

#[test]
fn a_blocked_sidecar_falls_back_to_the_index_home() {
    let (_dir, mbox) = copy_fixture("gmail-crlf.mbox");
    let home = tempfile::tempdir().unwrap();
    block_sidecar(&mbox);

    let report = build_with_home(&mbox, home.path()).unwrap();
    assert!(report.fallback);
    assert_eq!(report.index_dir, fallback_dir(&mbox, home.path()));
    assert!(report.index_dir.join(META_FILE).is_file());
    assert!(report.index_dir.starts_with(home.path()));
    assert_eq!(report.messages, 2);

    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));
    let index = match MboxIndex::open_in(&mbox, home.path()).unwrap() {
        Opened::Ready(index) => *index,
        other => panic!("{other:?}"),
    };
    assert_eq!(index.dir(), fallback_dir(&mbox, home.path()));
    assert_eq!(index.search(&SearchQuery::text("számla")).unwrap().total, 1);

    // Another home knows nothing about it.
    let elsewhere = tempfile::tempdir().unwrap();
    assert_eq!(
        status_in(&mbox, elsewhere.path()).unwrap(),
        IndexStatus::Missing
    );

    // A changed mbox is stale in the fallback too, and rebuilds in place.
    let mut bytes = fs::read(&mbox).unwrap();
    bytes.push(b'\n');
    fs::write(&mbox, bytes).unwrap();
    assert_eq!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Stale(StaleReason::Fingerprint)
    );
    let again = build_with_home(&mbox, home.path()).unwrap();
    assert!(again.fallback);
    assert_eq!(again.index_dir, report.index_dir);
    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));
}

#[test]
fn the_primary_location_wins_when_it_is_writable() {
    let (_dir, mbox) = copy_fixture("gmail-crlf.mbox");
    let home = tempfile::tempdir().unwrap();
    let primary = index_dir(&mbox);
    let fallback = fallback_dir(&mbox, home.path());

    // Writable folder: the index goes beside the mbox, the home stays empty.
    let report = build_with_home(&mbox, home.path()).unwrap();
    assert!(!report.fallback);
    assert_eq!(report.index_dir, primary);
    assert!(!fallback.exists());
    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));

    // The mbox changes and the folder is now blocked: the rebuild goes to the
    // home and the stale primary is cleared, so one index exists.
    let mut bytes = fs::read(&mbox).unwrap();
    bytes.push(b'\n');
    fs::write(&mbox, bytes).unwrap();
    fs::remove_dir_all(mbox.parent().unwrap().join(".mboxer")).unwrap();
    block_sidecar(&mbox);
    let moved = build_with_home(&mbox, home.path()).unwrap();
    assert!(moved.fallback);
    assert_eq!(moved.index_dir, fallback);
    assert!(!primary.exists());
    let index = match MboxIndex::open_in(&mbox, home.path()).unwrap() {
        Opened::Ready(index) => *index,
        other => panic!("{other:?}"),
    };
    assert_eq!(index.dir(), fallback);

    // The folder is writable again and the mbox changes once more: the build
    // returns beside the mbox and the fallback directory goes.
    fs::remove_file(mbox.parent().unwrap().join(".mboxer")).unwrap();
    let mut bytes = fs::read(&mbox).unwrap();
    bytes.push(b'\n');
    fs::write(&mbox, bytes).unwrap();
    let back = build_with_home(&mbox, home.path()).unwrap();
    assert!(!back.fallback);
    assert_eq!(back.index_dir, primary);
    assert!(!fallback.exists());
    assert!(matches!(
        status_in(&mbox, home.path()).unwrap(),
        IndexStatus::Fresh(_)
    ));
}

#[test]
fn cancelled_build_leaves_nothing_behind() {
    let (dir, mbox) = copy_fixture("gmail-crlf.mbox");
    let err = build(
        &mbox,
        &BuildOptions {
            threads: 1,
            memory_per_thread_bytes: 20 << 20,
            ..BuildOptions::default()
        },
        |_| {},
        &AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(matches!(err, Error::Cancelled), "{err}");
    assert_eq!(status(&mbox).unwrap(), IndexStatus::Missing);
    let leftovers: Vec<_> = fs::read_dir(dir.path().join(".mboxer"))
        .map(|entries| entries.map(|e| e.unwrap().file_name()).collect())
        .unwrap_or_default();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn every_message_is_reachable_even_when_it_does_not_parse() {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("odd.mbox");
    // The second message has an envelope line and nothing else.
    let data = b"From a@example.com Mon Jan  5 09:00:00 2026\nDate: Mon, 5 Jan 2026 09:00:00 +0000\nSubject: one\n\nbody one\nFrom b@example.com Tue Jan  6 10:00:00 2026\n";
    fs::write(&mbox, data).unwrap();
    let report = build_quiet(&mbox);
    assert_eq!(report.messages, 2);
    let index = open_ready(&mbox);
    let all = index.search(&SearchQuery::default()).unwrap();
    assert_eq!(all.total, 2);
    let (dated, undated): (Vec<_>, Vec<_>) = all.hits.iter().partition(|h| h.date.is_some());
    assert_eq!(dated.len(), 1);
    assert_eq!(undated.len(), 1);
    // Undated hits sort last and still carry their identity.
    assert_eq!(all.hits[1].date, None);
    assert!(all.hits[1].len > 0);
    read_at(&mbox, all.hits[1].offset, all.hits[1].len).unwrap();

    // The listing agrees: the undated message is the last row.
    let listing = index.list(&SearchQuery::default()).unwrap();
    let rows = hits(listing.rows(0, 10).unwrap());
    assert_eq!(rows.len(), 2);
    assert!(rows[0].date.is_some());
    assert_eq!(rows[1].date, None);
    assert_eq!(rows, all.hits);
}

#[test]
fn listing_holds_every_match_in_date_order_and_pages_by_slice() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    let mut listing = index.list(&SearchQuery::default()).unwrap();
    assert_eq!(listing.len() as u64, index.message_count());
    assert_eq!(listing.message_count(), listing.len());
    assert_eq!(listing.group_count(), listing.len());
    assert!(listing.warnings().is_empty());
    let count_only = index
        .search(&SearchQuery {
            limit: 0,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(listing.len(), count_only.total);

    let rows = listing.rows(0, 10).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows[0].hit.date.unwrap() > rows[1].hit.date.unwrap(),
        "newest first"
    );
    assert!(rows.iter().all(|row| row.kind == RowKind::Message));
    let first_page = index
        .search(&SearchQuery {
            limit: 1,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(rows[0].hit, first_page.hits[0]);

    // Slices are clamped to the end and empty past it.
    assert_eq!(listing.rows(1, 10).unwrap().len(), 1);
    assert_eq!(listing.rows(1, 10).unwrap()[0], rows[1]);
    assert!(listing.rows(2, 10).unwrap().is_empty());
    assert!(listing.rows(usize::MAX, 10).unwrap().is_empty());
    assert!(listing.rows(0, 0).unwrap().is_empty());

    // Every row can be found again by its identity; an unknown offset cannot.
    for (position, row) in rows.iter().enumerate() {
        assert_eq!(listing.position_of(row.hit.offset).unwrap(), Some(position));
    }
    assert_eq!(listing.position_of(12_345).unwrap(), None);
    assert_eq!(
        listing.toggle(0),
        None,
        "a single message has no thread row"
    );

    // Filters narrow the listing like they narrow a search.
    let mut inbox = index
        .list(&SearchQuery {
            labels: vec!["Inbox".into()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox.rows(0, 1).unwrap()[0].hit.offset, 0);
    assert_eq!(inbox.position_of(rows[0].hit.offset).unwrap(), None);

    // Relevance order lists the same matches.
    let relevance = index
        .list(&SearchQuery {
            text: Some("számla".into()),
            sort: Sort::Relevance,
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(relevance.len(), total(&index, "számla"));
    assert!(!relevance.is_empty());
}

#[test]
fn encoded_label_names_are_decoded_before_they_are_stored() {
    let (_dir, _mbox, index) = built("labels-encoded.mbox");
    assert_eq!(index.message_count(), 3);
    // A Q-encoded, a B-encoded (comma and slash inside the encoded word) and a
    // raw UTF-8 label name: decoded first, split on commas second, so the two
    // spellings of `Számlák` land on one term.
    let counts = index.label_counts().unwrap();
    let counts: Vec<(&str, u64)> = counts
        .iter()
        .map(|(name, count)| (name.as_str(), *count))
        .collect();
    assert_eq!(counts, [("Inbox", 3), ("Munka/Számlák", 1), ("Számlák", 2)]);

    let exact = index
        .search(&SearchQuery {
            labels: vec!["Számlák".to_owned()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(exact.total, 2);
    assert!(
        exact
            .hits
            .iter()
            .all(|hit| hit.labels.contains(&"Számlák".to_owned()))
    );
    assert!(
        exact
            .hits
            .iter()
            .all(|hit| !hit.labels.iter().any(|l| l.contains("=?"))),
        "an encoded form leaked into the stored labels: {:?}",
        exact.hits.iter().map(|h| &h.labels).collect::<Vec<_>>()
    );

    let under_munka = index
        .search(&SearchQuery {
            any_labels: vec!["Munka".to_owned(), "Munka/Számlák".to_owned()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(under_munka.total, 1);
    assert_eq!(
        under_munka.hits[0].subject.as_deref(),
        Some("B-encoded label with a comma inside")
    );
    assert_eq!(under_munka.hits[0].labels, ["Munka/Számlák", "Inbox"]);

    let encoded_form = index
        .search(&SearchQuery {
            labels: vec!["=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=".to_owned()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(encoded_form.total, 0);
}

#[test]
fn label_counts_follow_the_headers() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    let counts = index.label_counts().unwrap();
    // `Opened` sits on both messages (folded onto a continuation line in the first).
    let expected: Vec<(String, u64)> = [
        ("Category personal", 1),
        ("Important", 1),
        ("Inbox", 1),
        ("Opened", 2),
        ("Sent", 1),
    ]
    .into_iter()
    .map(|(name, count)| (name.to_owned(), count))
    .collect();
    assert_eq!(counts, expected);

    let (_dir, _mbox, empty) = built("empty.mbox");
    assert!(empty.label_counts().unwrap().is_empty());
    let mut listing = empty.list(&SearchQuery::default()).unwrap();
    assert!(listing.is_empty());
    assert!(listing.rows(0, 10).unwrap().is_empty());
    assert_eq!(listing.position_of(0).unwrap(), None);
    let mut grouped = grouped(&empty, SearchQuery::default());
    assert_eq!((grouped.len(), grouped.message_count()), (0, 0));
    assert_eq!(grouped.toggle(0), None);
    assert_eq!(grouped.position_of(0).unwrap(), None);
}

#[test]
fn the_word_being_typed_matches_as_a_prefix_when_asked() {
    let (_dir, _mbox, index) = built("gmail-crlf.mbox");
    let typing = |text: &str| SearchQuery {
        text: Some(text.to_owned()),
        prefix_last_word: true,
        ..SearchQuery::default()
    };
    let hits = |query: &SearchQuery| index.search(query).unwrap().total;

    assert_eq!(
        total(&index, "szaml"),
        0,
        "whole words only without the flag"
    );
    assert_eq!(hits(&typing("szaml")), 1);
    assert_eq!(hits(&typing("Száml")), 1, "the prefix is folded too");
    assert_eq!(hits(&typing("szamla")), 1, "a whole word still matches");
    assert_eq!(
        hits(&typing("szaml ")),
        0,
        "a trailing space closes the word"
    );
    assert_eq!(
        hits(&typing("konyveles szaml")),
        1,
        "the rest is parsed as before"
    );
    assert_eq!(hits(&typing("nincsilyen szaml")), 0);
    assert_eq!(hits(&typing("s")), 0, "one letter is not a prefix");
    assert!(index.search(&typing("szaml")).unwrap().warnings.is_empty());

    // The same for the sender filter, which also refreshes while typing.
    let from = |from: &str, prefix: bool| SearchQuery {
        from: Some(from.to_owned()),
        prefix_last_word: prefix,
        ..SearchQuery::default()
    };
    assert_eq!(hits(&from("ann", false)), 0);
    assert_eq!(hits(&from("ann", true)), 1);
    assert_eq!(hits(&from("gab", true)), 1);
    assert_eq!(hits(&from("zzz", true)), 0);

    // The listing behind the viewer goes through the same clause.
    let listing = index.list(&typing("szaml")).unwrap();
    assert_eq!(listing.len(), 1);
    assert_eq!(listing.rows(0, 1).unwrap()[0].hit.offset, 0);
}

#[test]
fn an_ungrouped_listing_of_threads_is_one_row_per_message() {
    let (_dir, _mbox, index) = built("threads.mbox");
    assert_eq!(index.message_count(), 6);
    let mut listing = index.list(&SearchQuery::default()).unwrap();
    assert_eq!(listing.len(), 6);
    assert_eq!(listing.message_count(), 6);
    assert_eq!(listing.group_count(), 6);
    let rows = listing.rows(0, 10).unwrap();
    let subjects: Vec<_> = rows
        .iter()
        .map(|row| row.hit.subject.as_deref().unwrap())
        .collect();
    assert_eq!(
        subjects,
        [
            "Re: Számla",
            "Hírlevél",
            "Re: Ajánlat",
            "Számla",
            "Ajánlat",
            "Számla kérdés"
        ]
    );
    assert!(rows.iter().all(|row| row.kind == RowKind::Message));
    let threads: Vec<_> = rows.iter().map(|row| row.hit.thread_id).collect();
    assert_eq!(
        threads,
        [Some(222), None, Some(111), Some(222), Some(111), Some(222)]
    );
    for (position, row) in rows.iter().enumerate() {
        assert_eq!(listing.position_of(row.hit.offset).unwrap(), Some(position));
    }
    assert_eq!(listing.toggle(0), None);
}

#[test]
fn a_grouped_listing_orders_conversations_by_their_newest_message() {
    let (_dir, _mbox, index) = built("threads.mbox");
    let listing = grouped(&index, SearchQuery::default());
    assert_eq!(listing.len(), 3);
    assert_eq!(listing.message_count(), 6);
    assert_eq!(listing.group_count(), 3);
    assert_eq!(
        outline(&listing),
        [
            ("Re: Számla".to_owned(), thread(3, false)),
            ("Hírlevél".to_owned(), message()),
            ("Re: Ajánlat".to_owned(), thread(2, false)),
        ]
    );
    let rows = listing.rows(0, 3).unwrap();
    assert_eq!(rows[0].hit.thread_id, Some(222));
    assert_eq!(rows[1].hit.thread_id, None);
    assert_eq!(rows[2].hit.thread_id, Some(111));
    assert!(listing.warnings().is_empty());
}

#[test]
fn toggling_a_thread_row_inserts_its_members_oldest_first() {
    let (_dir, _mbox, index) = built("threads.mbox");
    let mut listing = grouped(&index, SearchQuery::default());
    assert_eq!(listing.toggle(0), Some(6));
    assert_eq!(
        outline(&listing),
        [
            ("Re: Számla".to_owned(), thread(3, true)),
            ("Számla kérdés".to_owned(), member(0, 3)),
            ("Számla".to_owned(), member(1, 3)),
            ("Re: Számla".to_owned(), member(2, 3)),
            ("Hírlevél".to_owned(), message()),
            ("Re: Ajánlat".to_owned(), thread(2, false)),
        ]
    );
    assert_eq!(listing.toggle(5), Some(8));
    assert_eq!(listing.toggle(0), Some(5), "collapsing B leaves A expanded");
    assert_eq!(
        outline(&listing),
        [
            ("Re: Számla".to_owned(), thread(3, false)),
            ("Hírlevél".to_owned(), message()),
            ("Re: Ajánlat".to_owned(), thread(2, true)),
            ("Ajánlat".to_owned(), member(0, 2)),
            ("Re: Ajánlat".to_owned(), member(1, 2)),
        ]
    );
    assert_eq!(listing.toggle(1), None, "a single message");
    assert_eq!(listing.toggle(4), None, "a member row");
    assert_eq!(listing.toggle(99), None, "past the end");
    assert_eq!(listing.len(), 5);
    // A page in the middle of an expanded thread is sliced like any other.
    let page = listing.rows(3, 10).unwrap();
    assert_eq!(page.len(), 2);
    assert_eq!(page[0].kind, member(0, 2));
}

#[test]
fn position_of_expands_the_conversation_around_a_hidden_member() {
    let (_dir, _mbox, index) = built("threads.mbox");
    let mut listing = grouped(&index, SearchQuery::default());
    let a2 = offset_of(&index, "Re: Ajánlat");
    let b1 = offset_of(&index, "Számla");
    let b2 = offset_of(&index, "Re: Számla");
    let n = offset_of(&index, "Hírlevél");

    assert_eq!(
        listing.position_of(a2).unwrap(),
        Some(2),
        "the newest is the thread row"
    );
    assert_eq!(listing.len(), 3);
    assert_eq!(
        listing.position_of(b1).unwrap(),
        Some(2),
        "a hidden member expands B"
    );
    assert_eq!(listing.len(), 6);
    assert_eq!(
        listing.position_of(b2).unwrap(),
        Some(3),
        "expanded: the newest is a member row"
    );
    assert_eq!(listing.position_of(n).unwrap(), Some(4));
    assert_eq!(listing.position_of(12_345).unwrap(), None);
    assert_eq!(listing.len(), 6);
}

#[test]
fn a_conversation_counts_only_the_matching_messages() {
    let (_dir, _mbox, index) = built("threads.mbox");
    let listing = grouped(&index, SearchQuery::text("kérdés"));
    assert_eq!(listing.len(), 1);
    let row = &listing.rows(0, 1).unwrap()[0];
    assert_eq!(row.kind, RowKind::Message);
    assert_eq!(row.hit.thread_id, Some(222));

    let mut listing = grouped(
        &index,
        SearchQuery {
            text: Some("számla".into()),
            sort: Sort::Relevance,
            ..SearchQuery::default()
        },
    );
    assert_eq!(listing.len(), 1);
    assert_eq!(listing.message_count(), 3);
    assert_eq!(listing.toggle(0), Some(4));
    assert_eq!(
        outline(&listing),
        [
            ("Re: Számla".to_owned(), thread(3, true)),
            ("Számla kérdés".to_owned(), member(0, 3)),
            ("Számla".to_owned(), member(1, 3)),
            ("Re: Számla".to_owned(), member(2, 3)),
        ]
    );
}

#[test]
fn any_labels_matches_a_label_or_its_sub_labels() {
    let (_dir, _mbox, index) = built("threads.mbox");
    let count = |query: SearchQuery| index.list(&query).unwrap().len();
    let any = |labels: &[&str]| SearchQuery {
        any_labels: labels.iter().map(|l| (*l).to_owned()).collect(),
        ..SearchQuery::default()
    };
    assert_eq!(
        count(any(&["Work", "Work/Invoices", "Work/Invoices/2026"])),
        4
    );
    assert_eq!(count(any(&["Work/Invoices", "Work/Invoices/2026"])), 2);
    assert_eq!(
        count(any(&["Work"])),
        2,
        "exact: not Workshop, not Work-archive"
    );
    assert_eq!(count(any(&["Nothing"])), 0);
    assert_eq!(count(any(&["Category updates"])), 1);
    assert_eq!(
        count(SearchQuery {
            labels: vec!["Sent".into()],
            ..any(&["Work", "Work/Invoices", "Work/Invoices/2026"])
        }),
        1,
        "AND-ed with the exact labels"
    );
    assert_eq!(
        count(SearchQuery {
            labels: vec!["Work".into()],
            ..SearchQuery::default()
        }),
        2
    );
    let counts = index.label_counts().unwrap();
    assert!(
        counts
            .iter()
            .any(|(label, n)| label == "Work/Invoices/2026" && *n == 1)
    );
    assert!(counts.iter().any(|(label, n)| label == "Bin" && *n == 1));
}

#[test]
fn an_undated_member_sorts_oldest_and_a_dated_one_stands_for_the_thread() {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("thread.mbox");
    let data = b"From a@example.com Mon Jan  5 09:00:00 2026\nX-GM-THRID: 5\nDate: Mon, 5 Jan 2026 09:00:00 +0000\nSubject: dated\n\nbody one\nFrom b@example.com Tue Jan  6 10:00:00 2026\nX-GM-THRID: 5\nSubject: undated\n\nbody two\n";
    fs::write(&mbox, data).unwrap();
    build_quiet(&mbox);
    let index = open_ready(&mbox);
    let mut listing = grouped(&index, SearchQuery::default());
    assert_eq!(outline(&listing), [("dated".to_owned(), thread(2, false))]);
    assert_eq!(listing.toggle(0), Some(3));
    assert_eq!(
        outline(&listing),
        [
            ("dated".to_owned(), thread(2, true)),
            ("undated".to_owned(), member(0, 2)),
            ("dated".to_owned(), member(1, 2)),
        ]
    );
}
