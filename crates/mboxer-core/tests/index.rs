//! End-to-end tests of the sidecar index over the synthetic fixtures.
//!
//! Each test copies a fixture into a temporary directory first: the index is
//! written next to the mbox, and the fixtures directory must stay clean.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use mboxer_core::index::fingerprint::{FORMAT_VERSION, META_FILE};
use mboxer_core::{
    BuildOptions, Error, IndexStatus, MboxIndex, Opened, SearchQuery, Sort, StaleReason, build,
    index_dir, read_at, status,
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
}
