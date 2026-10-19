//! End-to-end tests of the `mboxer-cli` executable over a synthetic fixture.
//!
//! The fixture is copied into a temporary directory first: `index` writes the
//! sidecar next to the mbox, and the fixtures directory must stay clean.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../mboxer-core/tests/fixtures")
        .join(name)
}

/// Copies the Gmail CRLF fixture (two messages) into a fresh temp dir.
fn copy_fixture() -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("gmail-crlf.mbox");
    fs::copy(fixture("gmail-crlf.mbox"), &mbox).unwrap();
    (dir, mbox)
}

fn mboxer(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mboxer-cli"))
        .args(args)
        .output()
        .expect("mboxer-cli runs")
}

/// Runs `mboxer-cli` with the index home set to `home`.
fn mboxer_with_home(args: &[&str], home: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mboxer-cli"))
        .args(args)
        .env("MBOXER_INDEX_HOME", home)
        .output()
        .expect("mboxer-cli runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The value of a `key:   value` summary line.
fn field<'a>(text: &'a str, key: &str) -> &'a str {
    text.lines()
        .find_map(|line| line.strip_prefix(key).map(str::trim))
        .unwrap_or_else(|| panic!("no `{key}` line in:\n{text}"))
}

fn indexed() -> (TempDir, PathBuf) {
    let (dir, mbox) = copy_fixture();
    let output = mboxer(&["index", mbox.to_str().unwrap(), "--quiet"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(field(&stdout(&output), "messages:"), "2");
    assert_eq!(field(&stdout(&output), "skipped:"), "0");
    (dir, mbox)
}

fn search(mbox: &Path, args: &[&str]) -> String {
    let mut full = vec!["search", mbox.to_str().unwrap()];
    full.extend_from_slice(args);
    let output = mboxer(&full);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout(&output)
}

fn hits(mbox: &Path, args: &[&str]) -> String {
    field(&search(mbox, args), "hits:").to_owned()
}

#[test]
fn a_blocked_mbox_folder_indexes_into_the_home_and_search_finds_it() {
    let (dir, mbox) = copy_fixture();
    let home = tempfile::tempdir().unwrap();
    // A file named `.mboxer` blocks the sidecar folder, like a read-only share would.
    fs::write(dir.path().join(".mboxer"), b"blocked").unwrap();

    let output = mboxer_with_home(&["index", mbox.to_str().unwrap(), "--quiet"], home.path());
    assert!(output.status.success(), "{output:?}");
    let text = stdout(&output);
    let index_line = field(&text, "index:");
    assert!(
        index_line.ends_with("(fallback: the mbox folder is not writable)"),
        "{index_line}"
    );
    assert!(
        Path::new(index_line.split(" (fallback").next().unwrap()).starts_with(home.path()),
        "{index_line}"
    );
    assert_eq!(field(&text, "messages:"), "2");

    let output = mboxer_with_home(&["search", mbox.to_str().unwrap(), "számla"], home.path());
    assert!(output.status.success(), "{output:?}");
    assert_eq!(field(&stdout(&output), "hits:"), "1");

    // Without the home the index is not found.
    let output = mboxer(&["search", mbox.to_str().unwrap(), "számla"]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
}

#[test]
fn search_without_an_index_fails_with_a_hint() {
    let (_dir, mbox) = copy_fixture();
    let output = mboxer(&["search", mbox.to_str().unwrap(), "számla"]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("run `mboxer-cli index`"), "{stderr}");
}

#[test]
fn free_text_folds_accents_and_reports_timings() {
    let (_dir, mbox) = indexed();
    let text = search(&mbox, &["SZAMLA"]);
    assert_eq!(field(&text, "query:"), "SZAMLA");
    assert_eq!(field(&text, "filters:"), "-");
    assert_eq!(field(&text, "indexed messages:"), "2");
    assert_eq!(field(&text, "hits:"), "1");
    assert!(field(&text, "open:").ends_with(" ms"));
    assert!(field(&text, "search:").ends_with(" ms"));
    assert!(text.contains("Számla január"), "{text}");
    assert!(text.contains("| attachment"), "{text}");
}

#[test]
fn filters_narrow_the_result_and_work_without_text() {
    let (_dir, mbox) = indexed();
    assert_eq!(hits(&mbox, &[]), "2");
    assert_eq!(hits(&mbox, &["--has-attachment"]), "1");
    assert_eq!(hits(&mbox, &["--no-attachment"]), "1");
    assert_eq!(hits(&mbox, &["--from", "anna"]), "1");
    assert_eq!(hits(&mbox, &["--from", "anna", "--no-attachment"]), "0");
    assert_eq!(hits(&mbox, &["--to", "konyveles"]), "1");
    assert_eq!(hits(&mbox, &["--subject", "szamla"]), "1");
    assert_eq!(hits(&mbox, &["--subject", "köszönöm"]), "0");
    assert_eq!(hits(&mbox, &["--label", "Inbox"]), "1");
    assert_eq!(hits(&mbox, &["--label", "Sent", "--label", "Opened"]), "1");
    assert_eq!(hits(&mbox, &["--label", "Sent", "--label", "Inbox"]), "0");
    // 2026-01-14 09:00 +01:00 is 08:00 UTC: inside [14th, 15th), outside [13th, 14th).
    assert_eq!(
        hits(&mbox, &["--after", "2026-01-14", "--before", "2026-01-15"]),
        "1"
    );
    assert_eq!(
        hits(&mbox, &["--after", "2026-01-13", "--before", "2026-01-14"]),
        "1"
    );
    assert_eq!(hits(&mbox, &["--after", "2026-01-15"]), "0");
    assert_eq!(hits(&mbox, &["számla", "--after", "2026-01-14"]), "0");

    let text = search(&mbox, &["--from", "anna", "--after", "2026-01-13"]);
    assert_eq!(field(&text, "query:"), "-");
    assert_eq!(
        field(&text, "filters:"),
        "from \"anna\", after 2026-01-13T00:00:00Z"
    );
}

#[test]
fn count_and_paging() {
    let (_dir, mbox) = indexed();
    let text = search(&mbox, &["--count"]);
    assert_eq!(field(&text, "hits:"), "2");
    assert!(!text.contains(" | offset "), "count printed hits:\n{text}");

    let page = search(&mbox, &["--limit", "1", "--offset", "1"]);
    assert_eq!(field(&page, "hits:"), "2");
    assert_eq!(page.matches(" | offset ").count(), 1);
    // The second page by date is the older message.
    assert!(page.contains("2026-01-13T"), "{page}");

    // A limit or an offset far above the message count is capped, not an abort (task 060).
    let max = usize::MAX.to_string();
    let everything = search(&mbox, &["--limit", &max]);
    assert_eq!(field(&everything, "hits:"), "2");
    assert_eq!(everything.matches(" | offset ").count(), 2);
    let past = search(&mbox, &["--limit", &max, "--offset", &max]);
    assert_eq!(field(&past, "hits:"), "2");
    assert_eq!(past.matches(" | offset ").count(), 0);
}

#[test]
fn a_corrupt_index_metadata_file_is_rebuilt() {
    let (dir, mbox) = indexed();
    let meta = dir
        .path()
        .join(".mboxer")
        .join("gmail-crlf.mbox")
        .join("mboxer.json");
    assert!(meta.is_file());
    for args in [vec!["--force"], vec![]] {
        fs::write(&meta, b"{\"format_version\": \xff\xfe").unwrap();
        let mut full = vec!["index", mbox.to_str().unwrap()];
        full.extend(args);
        let output = mboxer(&full);
        assert!(output.status.success(), "{output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("index metadata unreadable"), "{stderr}");
        assert_eq!(field(&stdout(&output), "messages:"), "2");
        assert_eq!(hits(&mbox, &["számla"]), "1");
    }
}

#[test]
fn a_bad_date_is_a_usage_error() {
    let (_dir, mbox) = copy_fixture();
    let output = mboxer(&["search", mbox.to_str().unwrap(), "--after", "yesterday"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a date"), "{stderr}");
    let both = mboxer(&[
        "search",
        mbox.to_str().unwrap(),
        "--has-attachment",
        "--no-attachment",
    ]);
    assert_eq!(both.status.code(), Some(2));
}

/// The offsets of the hits a `search` printed, in the order printed.
fn offsets(text: &str) -> Vec<u64> {
    text.lines()
        .filter_map(|line| line.split(" | offset ").nth(1))
        .map(|rest| rest.split(' ').next().unwrap().parse().unwrap())
        .collect()
}

#[test]
fn same_date_hits_of_a_four_thread_build_page_in_mbox_order() {
    // 400 messages with one date: the four indexing threads spread them over
    // segments in whatever order they pick them up (task 069).
    let dir = tempfile::tempdir().unwrap();
    let mbox = dir.path().join("generated.mbox");
    let mut data = String::new();
    for n in 0..400 {
        data.push_str("From sender@example.com Mon Jan  5 09:00:00 2026\n");
        data.push_str(&format!(
            "From: sender@example.com\nDate: Mon, 5 Jan 2026 09:00:00 +0000\nSubject: m{n:03}\n\nszo\n\n"
        ));
    }
    fs::write(&mbox, data).unwrap();
    let path = mbox.to_str().unwrap();
    let output = mboxer(&["index", path, "--threads", "4", "--quiet"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(field(&stdout(&output), "messages:"), "400");

    for sort in [&[][..], &["--relevance"][..]] {
        let mut args = vec!["szo", "--limit", "400"];
        args.extend_from_slice(sort);
        let all = offsets(&search(&mbox, &args));
        assert_eq!(all.len(), 400, "{sort:?}");
        assert!(
            all.windows(2).all(|w| w[0] < w[1]),
            "{sort:?}: ties not in mbox order"
        );
        // Every page is the matching slice of that order.
        for page in 0..4 {
            let start = (page * 100).to_string();
            let mut args = vec!["szo", "--limit", "100", "--offset", &start];
            args.extend_from_slice(sort);
            assert_eq!(
                offsets(&search(&mbox, &args)),
                all[page * 100..(page + 1) * 100],
                "{sort:?}: page {page}"
            );
        }
    }
}
