---
size: S
origin: done/060-index-quality-follow-ups-from-the-review.md
created: 2026-10-19
started: 2026-10-20
closed: 2026-10-20
plan: none
milestone: M6 Hardening
---

# 069 — The viewer reads a message through the index's limit, and the CLI's paging breaks ties by offset

## Context

What: two leftovers of task 060. The viewer still reads messages with `read_at` (`commands.rs`, `view.rs`,
`attachments.rs`); `MboxIndex::read_message` would honour the `max_message_bytes` the index was built with, should
the viewer ever build with a non-default limit. `MboxIndex::search`, which the CLI's paging uses, still breaks date
ties by document address, so `mboxer-cli search` lists same-date messages of a multi-thread build out of file
order; the listing behind the viewer already orders by offset.
Why: one read path and one tie rule, so the CLI and the viewer never disagree about the same archive.
Done when: the viewer's three call sites use `read_message`, and a CLI test over a four-thread build lists
same-date messages in file order.
The viewer's call sites are `read_message` and the attachment commands in `crates/mboxer-app/src/commands.rs`,
`read_message_view` in `view.rs` and `read_attachment` in `attachments.rs`; the CLI's paging is
`MboxIndex::search` in `crates/mboxer-core/src/index/search.rs`, the viewer's listing order is in `listing.rs`.

## Steps

1. [x] the viewer's read path: `read_message_view`, the raw read behind *Source* / *Save .eml* and
   `read_attachment` take the message through `MboxIndex::read_message` (or a reader built from the index's
   limit), so the index's `max_message_bytes` holds; `read_at` stays for the places with no index
2. [x] `MboxIndex::search` breaks a date tie by file offset, the same rule as the listing; the tie order
   documented where the listing's is
3. [x] tests: a core or CLI test over a four-thread build of a fixture with same-date messages lists them in
   file order through `mboxer-cli search`; an app test that a message above the limit reads as the index
   says it does
4. [x] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   macOS-target clippy and the `rust-version` check
5. [x] `docs/reference.md` where it describes the CLI's order or the message limit, if the wording changes

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-20 — tie order (step 2). The new CLI test
  `same_date_hits_of_a_four_thread_build_page_in_mbox_order` (400 messages with one date, written by the test,
  `mboxer-cli index --threads 4`, then `search szo --limit 400` and four pages of 100) failed by date
  ("ties not in mbox order") and, with the date order fixed alone, by relevance too: tantivy breaks a tie by
  document address. Decided 2026-10-20: `MboxIndex::search` orders by a two-part sort key, the date (descending,
  a missing date last as before) or the BM25 score, then the `offset` fast field ascending (`by_offset`), built
  from tantivy 0.27's tuple sort keys (`TopDocs::order_by`). Each page is then a slice of one total order, the
  rule of `MboxIndex::list`; the doc comment of `search` says so and points at the listing. By relevance the
  pair key gives up tantivy's Block-WAND pruning for the score-only top-k; `search` serves the CLI only (the
  viewer pages a `Listing`), so the exact order wins.
- 2026-10-20 — read path (step 1). Decided 2026-10-20: the viewer's readers take the open file's index instead
  of its path: `read_message_view`, `read_attachment` and `read_listed_attachments` take a `&MboxIndex` and read
  through `MboxIndex::read_message`; `commands.rs` swaps `file_path` for `file_index`, which shares the
  `Arc<MboxIndex>` of the open file out from under the lock (as a search already does), and the raw reads of
  *Source* and *Save .eml* call `read_message` on it. The app no longer calls `read_at`; it stays in the core
  for readers without an index (`mboxer-cli scan`, the tests). The tests of `view.rs` and `attachments.rs` read
  through an index too, built in a temporary folder by `test_support::indexed_fixture` in `lib.rs` (one
  indexing thread, the index home in that folder).
- 2026-10-20 — limit test (step 3). `a_read_above_the_size_limit_of_the_build_is_refused_as_the_index_refuses_it`
  builds `gmail-crlf.mbox` with the first message's length as the limit: that message reads, while one byte
  more is refused by the view and both attachment readers with the error `MboxIndex::read_message` gives
  (`MessageTooLarge` with the build's limit), where `read_at` with its default limit reads across into the
  next message.
- 2026-10-20 — docs (step 5). `docs/reference.md` says the same-date order holds in the viewer and in
  `mboxer-cli search` alike; the message limit's wording is unchanged (the viewer's builds keep the default).

## Deviations from the plan

- The four-thread test builds its mbox in the test (as the core's tie test does) instead of a committed
  fixture: the threads only spread messages over segments with a few hundred of them.
- The shared test helper `test_support` sits in `lib.rs` (test builds only), so `state.rs` stays untouched.

## Verification

- 2026-10-20, Windows 11, in the task worktree: `cargo test --locked` green across the workspace, with the new
  CLI test `same_date_hits_of_a_four_thread_build_page_in_mbox_order` (400 offsets strictly ascending by date
  and by relevance, every page of 100 the matching slice) and the new app test
  `a_read_above_the_size_limit_of_the_build_is_refused_as_the_index_refuses_it`; the reworked app tests
  (`plain_part_wins_and_headers_are_filled`, `remote_fixture_carries_its_html_and_unquoted_text`,
  `attachments_travel_with_the_view`, `legacy_charsets_are_decoded`, `attachments_are_read_from_the_fixture`)
  pass through an index. Before the fix the CLI test failed by date and by relevance.
- Same day: `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` clean; the
  macOS-target clippy (`DOCS_RS=1`, `aarch64-apple-darwin`) and `cargo +1.90 check` for that target clean;
  `python -I scripts/render-usage-guide.py --check` passes.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none found)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-20`
