//! Index quality (task 060): prefix completions, nested HTML-only messages,
//! capped paging, the read-back size limit, tie order of a multi-thread build,
//! a corrupt metadata file and labels with a comma.
//!
//! Each test writes or copies its mbox into a temporary directory and keeps
//! the index home in another one, so nothing is written next to the fixtures
//! or under the user's data folder.

use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use mboxer_core::{
    BuildOptions, DEFAULT_MAX_MESSAGE_BYTES, Error, IndexMeta, IndexStatus, MboxIndex, MboxReader,
    Opened, SearchQuery, Sort, StaleReason, build, index_dir, parse, read_at, read_at_from,
    read_at_from_with_limit, status_in, summarize,
};
use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A temporary folder with the mbox and a separate index home.
struct Place {
    _dir: TempDir,
    home: TempDir,
    mbox: PathBuf,
}

/// Copies the fixture `name` into a fresh temp dir.
fn copy_fixture(name: &str) -> Place {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join(name);
    fs::copy(fixture(name), &mbox).unwrap();
    Place {
        _dir: dir,
        home: tempfile::tempdir().unwrap(),
        mbox,
    }
}

/// Writes `messages` (header and body lines, no envelope) as one mbox.
fn write_mbox(messages: &[String]) -> Place {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("generated.mbox");
    let mut data = String::new();
    for message in messages {
        data.push_str("From sender@example.com Mon Jan  5 09:00:00 2026\n");
        data.push_str(message);
        data.push('\n');
    }
    fs::write(&mbox, data).unwrap();
    Place {
        _dir: dir,
        home: tempfile::tempdir().unwrap(),
        mbox,
    }
}

fn build_with(place: &Place, threads: usize, max_message_bytes: Option<u64>) {
    let mut options = BuildOptions {
        threads,
        memory_per_thread_bytes: 20 << 20,
        home: Some(place.home.path().to_path_buf()),
        ..BuildOptions::default()
    };
    if let Some(limit) = max_message_bytes {
        options.max_message_bytes = limit;
    }
    build(&place.mbox, &options, |_| {}, &AtomicBool::new(false)).unwrap();
}

fn open_ready(place: &Place) -> MboxIndex {
    match MboxIndex::open_in(&place.mbox, place.home.path()).unwrap() {
        Opened::Ready(index) => *index,
        other => panic!("expected a ready index, got {other:?}"),
    }
}

fn built(place: Place) -> (Place, MboxIndex) {
    build_with(&place, 1, None);
    let index = open_ready(&place);
    (place, index)
}

fn total(index: &MboxIndex, text: &str) -> usize {
    index.search(&SearchQuery::text(text)).unwrap().total
}

fn subjects_of(index: &MboxIndex, query: SearchQuery) -> Vec<String> {
    index
        .search(&SearchQuery {
            limit: 10_000,
            ..query
        })
        .unwrap()
        .hits
        .into_iter()
        .map(|hit| hit.subject.unwrap_or_default())
        .collect()
}

#[test]
fn a_prefix_completes_to_frequent_words_before_dictionary_order() {
    // One message holds 300 one-off tokens that sort before `abrakadabra`
    // (digits before letters), two others hold the real word.
    let junk: Vec<String> = (0..300).map(|n| format!("ab{n:08x}")).collect();
    let place = write_mbox(&[
        format!("Subject: junk\n\n{}\n", junk.join(" ")),
        "Subject: real one\n\nabrakadabra egy\n".to_owned(),
        "Subject: real two\n\nmegint abrakadabra\n".to_owned(),
    ]);
    let (_place, index) = built(place);
    let typing = |text: &str| SearchQuery {
        text: Some(text.to_owned()),
        prefix_last_word: true,
        ..SearchQuery::default()
    };
    let mut found = subjects_of(&index, typing("ab"));
    found.sort();
    assert_eq!(found, ["junk", "real one", "real two"]);
    // The same through a phrase that ends in the word being typed.
    assert_eq!(subjects_of(&index, typing("megint ab")), ["real two"]);
    // A prefix that names one junk token still finds it.
    assert_eq!(subjects_of(&index, typing("ab0000012")), ["junk"]);
    // Relevance order and a count-only search take the same expansion.
    let scored = index
        .search(&SearchQuery {
            sort: Sort::Relevance,
            ..typing("abr")
        })
        .unwrap();
    assert_eq!(scored.total, 2);
    let counted = index
        .search(&SearchQuery {
            limit: 0,
            ..typing("ab")
        })
        .unwrap();
    assert_eq!(counted.total, 3);
    // A prefix of nothing in the index matches nothing.
    assert_eq!(subjects_of(&index, typing("abx")), Vec::<String>::new());
}

#[test]
fn a_nested_html_only_message_is_indexed_beside_a_plain_outer_one() {
    let (_place, index) = built(copy_fixture("nested-html-only.mbox"));
    assert_eq!(index.message_count(), 1);
    // The outer plain part and the nested message's only (HTML) part.
    assert_eq!(total(&index, "tovabbitom"), 1);
    assert_eq!(total(&index, "berszamfejtes"), 1);
    assert_eq!(total(&index, "\"határideje változik\""), 1);
    // The outer HTML alternative stays out: the outer message has a plain part.
    assert_eq!(total(&index, "htmlalternativaszo"), 0);
    // Markup and style rules of the nested HTML are not words.
    for tag_or_style in ["html", "body", "style", "color", "red"] {
        assert_eq!(total(&index, tag_or_style), 0, "{tag_or_style:?} leaked");
    }
}

#[test]
fn limit_and_offset_far_above_the_message_count_are_capped() {
    let (_place, index) = built(copy_fixture("threads.mbox"));
    let all = index.message_count() as usize;
    assert!(all > 2);
    let page = |limit: usize, offset: usize| {
        index
            .search(&SearchQuery {
                limit,
                offset,
                ..SearchQuery::default()
            })
            .unwrap()
    };
    for sort in [Sort::DateDesc, Sort::Relevance] {
        let everything = index
            .search(&SearchQuery {
                limit: usize::MAX,
                sort,
                ..SearchQuery::default()
            })
            .unwrap();
        assert_eq!((everything.total, everything.hits.len()), (all, all));
    }
    let tail = page(usize::MAX, all - 1);
    assert_eq!((tail.total, tail.hits.len()), (all, 1));
    assert_eq!(tail.hits[0], page(all, 0).hits[all - 1]);
    for offset in [all, usize::MAX / 2, usize::MAX] {
        let past = page(usize::MAX, offset);
        assert_eq!((past.total, past.hits.len()), (all, 0), "offset {offset}");
    }
    let counted = page(0, usize::MAX);
    assert_eq!((counted.total, counted.hits.len()), (all, 0));
}

#[test]
fn a_message_is_read_back_under_the_size_limit_of_its_build() {
    let place = copy_fixture("gmail-crlf.mbox");
    build_with(&place, 1, Some(DEFAULT_MAX_MESSAGE_BYTES * 2));
    let index = open_ready(&place);
    assert_eq!(
        index.meta().max_message_bytes,
        DEFAULT_MAX_MESSAGE_BYTES * 2
    );
    let hit = index.search(&SearchQuery::text("számla")).unwrap().hits[0].clone();
    let raw = index.read_message(hit.offset, hit.len).unwrap();
    assert_eq!((raw.offset, raw.len), (hit.offset, hit.len));
    assert_eq!(raw, read_at(&place.mbox, hit.offset, hit.len).unwrap());

    // A length the build could not have produced is refused with its limit.
    let small = copy_fixture("gmail-crlf.mbox");
    build_with(&small, 1, Some(1 << 20));
    let index = open_ready(&small);
    match index.read_message(0, (1 << 20) + 1) {
        Err(Error::MessageTooLarge { offset: 0, limit }) => assert_eq!(limit, 1 << 20),
        other => panic!("expected MessageTooLarge, got {other:?}"),
    }
    // The reader takes the limit it is given.
    let bytes = fs::read(&small.mbox).unwrap();
    let first = read_at_from(Cursor::new(&bytes), 0, hit.len).unwrap();
    assert_eq!(
        read_at_from_with_limit(Cursor::new(&bytes), 0, hit.len, hit.len).unwrap(),
        first
    );
    assert!(matches!(
        read_at_from_with_limit(Cursor::new(&bytes), 0, hit.len, hit.len - 1),
        Err(Error::MessageTooLarge { .. })
    ));
}

#[test]
fn ties_list_in_mbox_order_after_a_four_thread_build() {
    // 400 messages with one date: the four indexing threads spread them over
    // segments in whatever order they pick them up.
    let messages: Vec<String> = (0..400)
        .map(|n| {
            let thread = if n % 50 == 7 { "X-GM-THRID: 77\n" } else { "" };
            format!("{thread}Date: Mon, 5 Jan 2026 09:00:00 +0000\nSubject: m{n:03}\n\nszo\n")
        })
        .collect();
    let place = write_mbox(&messages);
    build_with(&place, 4, None);
    let index = open_ready(&place);
    assert_eq!(index.message_count(), 400);
    let offsets = |listing: &mboxer_core::Listing| -> Vec<u64> {
        listing
            .rows(0, listing.len())
            .unwrap()
            .into_iter()
            .map(|row| row.hit.offset)
            .collect()
    };
    let ascending = |offsets: &[u64]| offsets.windows(2).all(|w| w[0] < w[1]);

    for sort in [Sort::DateDesc, Sort::Relevance] {
        let query = SearchQuery {
            text: Some("szo".to_owned()),
            sort,
            ..SearchQuery::default()
        };
        let plain = index.list(&query).unwrap();
        assert_eq!(plain.len(), 400);
        assert!(
            ascending(&offsets(&plain)),
            "{sort:?}: ties not in mbox order"
        );

        // Grouped: the eight messages of thread 77 are one row at the place of
        // its newest member (the last of them in the mbox), the rest in mbox order.
        let mut grouped = index
            .list(&SearchQuery {
                group_threads: true,
                ..query
            })
            .unwrap();
        assert_eq!(grouped.len(), 393);
        let rows = offsets(&grouped);
        assert!(ascending(&rows), "{sort:?}: grouped ties not in mbox order");
        let thread_row = grouped
            .rows(0, grouped.len())
            .unwrap()
            .iter()
            .position(|row| row.hit.thread_id == Some(77))
            .unwrap();
        assert_eq!(grouped.toggle(thread_row), Some(401));
        let members: Vec<u64> = grouped
            .rows(thread_row + 1, 8)
            .unwrap()
            .into_iter()
            .map(|row| row.hit.offset)
            .collect();
        assert!(ascending(&members), "{sort:?}: members not in mbox order");
        assert_eq!(members.last(), Some(&rows[thread_row]));
    }
}

#[test]
fn a_corrupt_metadata_file_reads_as_stale_and_a_rebuild_replaces_it() {
    let place = copy_fixture("gmail-crlf.mbox");
    build_with(&place, 1, None);
    let meta_path = index_dir(&place.mbox).join("mboxer.json");
    let json = fs::read(&meta_path).unwrap();
    let status = || status_in(&place.mbox, place.home.path()).unwrap();
    assert!(matches!(status(), IndexStatus::Fresh(_)));

    let corruptions: [(&str, Vec<u8>); 4] = [
        ("truncated", json[..json.len() / 2].to_vec()),
        ("empty", Vec::new()),
        ("zero-filled", vec![0; json.len()]),
        (
            "not UTF-8",
            [&json[..10], &[0xff, 0xfe][..], &json[10..]].concat(),
        ),
    ];
    for (what, bytes) in corruptions {
        fs::write(&meta_path, &bytes).unwrap();
        assert!(
            matches!(status(), IndexStatus::Stale(StaleReason::Unreadable(_))),
            "{what}: {:?}",
            status_in(&place.mbox, place.home.path())
        );
        assert!(
            matches!(
                MboxIndex::open_in(&place.mbox, place.home.path()).unwrap(),
                Opened::Stale(StaleReason::Unreadable(_))
            ),
            "{what}"
        );
        build_with(&place, 1, None);
        assert!(matches!(status(), IndexStatus::Fresh(_)), "{what}");
    }
}

#[test]
fn a_quoted_label_keeps_its_comma() {
    // Gmail quotes a label name that holds a comma: `"Work, Home",Inbox`.
    let labels: Vec<Vec<String>> = MboxReader::open(fixture("labels-quoted-comma.mbox"))
        .unwrap()
        .map(|raw| {
            let raw = raw.unwrap();
            summarize(&raw, &parse(&raw).unwrap()).labels
        })
        .collect();
    assert_eq!(
        labels,
        [
            vec!["Work, Home", "Inbox"],
            vec!["Inbox", "Projekt, 2026/Q1", "Opened", "Say \"hi\", Bob"],
            vec!["Work", "Home", "Inbox"],
        ]
    );

    let (_place, index) = built(copy_fixture("labels-quoted-comma.mbox"));
    let counts = index.label_counts().unwrap();
    let names: Vec<&str> = counts.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Home",
            "Inbox",
            "Opened",
            "Projekt, 2026/Q1",
            "Say \"hi\", Bob",
            "Work",
            "Work, Home"
        ]
    );
    let exact = index
        .search(&SearchQuery {
            labels: vec!["Work, Home".to_owned()],
            ..SearchQuery::default()
        })
        .unwrap();
    assert_eq!(exact.total, 1);
    assert_eq!(exact.hits[0].labels, ["Work, Home", "Inbox"]);
}

#[test]
fn metadata_of_an_older_build_reads_with_the_default_size_limit() {
    let json = r#"{"format_version":3,"mbox":{"len":1,"mtime_unix":2,"sample_hash":3},
        "message_count":4,"parse_failures":0,"skipped":0,"built_at_unix":5,"build_secs":6,
        "tantivy_version":"x"}"#;
    let meta: IndexMeta = serde_json::from_str(json).unwrap();
    assert_eq!(meta.max_message_bytes, DEFAULT_MAX_MESSAGE_BYTES);
}
