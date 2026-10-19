---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 060 — Index quality follow-ups: token length, completions, nested parts, limits, multi-thread order

## Context

What: smaller findings in `crates/mboxer-core`: `RemoveLongFilter` in `schema.rs` measures bytes before accent
folding, so a 40-character ASCII or a 20-character accented word is dropped, and `MAX_TOKEN_CHARS` is misnamed; the
prefix completion in `search.rs` takes the first 200 terms in dictionary order, which a short prefix over a large
archive spends on hex and base64 junk; `build.rs` skips an HTML-only nested `message/rfc822` when the outer mail has
a plain part; `limit` and `offset` reach tantivy uncapped (a huge `--limit` aborts the process); `read_at_from`
ignores the build's `max_message_bytes`; the tie order comment in `listing.rs` holds for one indexing thread only
and every test builds with one; `from_addr` and `to_addr` are written and never read; the buffer-reuse comments in
`reader.rs` are stale; a corrupt `mboxer.json` makes `index --force` fail; labels are split on every comma.
Why: each one is a quiet wrong answer or a wasted megabyte on the large archive.
Done when: each point has a test or a measured non-issue noted in the Log, and the comments match the code.
Decided 2026-10-19: the index format may change (`FORMAT_VERSION` bump) only if dropping `from_addr`/`to_addr`
needs it; the label-comma question is settled by a fixture shaped like a Takeout header, not by the real archive.

## Steps

1. [x] `schema.rs`: the long-token filter counts characters after folding (or a custom filter), the constant and
   its comment say so; a test with a 40-character ASCII and a 25-character accented word
2. [x] `search.rs` completions: the prefix expansion prefers short or frequent terms (document frequency from the
   term dictionary) over dictionary order, or raises the cap with a cost note; a test with junk tokens ahead of
   the real word
3. [x] `build.rs`: an HTML-only nested `message/rfc822` is indexed when the outer mail has a plain part; a fixture
4. [x] `limit` and `offset` capped at the message count in `search.rs` and the CLI; a test with `--limit` far above
5. [x] `read_at_from` takes the build's `max_message_bytes` (from `mboxer.json`) instead of the default
6. [x] `listing.rs`: the tie-order comment matches a multi-thread build, and at least one listing test builds with
   `threads: 4`
7. [x] `from_addr` / `to_addr`: removed from the schema or given a reader; the comments in `reader.rs` about buffer
   reuse match the `mem::take` handoff
8. [x] a corrupt `mboxer.json` reads as stale, so `index --force` rebuilds; a test
9. [x] labels: a fixture with a quoted label containing a comma (`X-Gmail-Labels: "Work, Home",Inbox`) decides
   whether the split is wrong; the finding and the fix or the non-issue in the Log
10. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` where
    a behaviour changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — tests first. The new integration tests sit in `crates/mboxer-core/tests/index_quality.rs`, with
  two new fixtures, `nested-html-only.mbox` and `labels-quoted-comma.mbox`; each test keeps its index home in a
  temporary folder. Every point below names what the test showed against the old code.
- 2026-10-19 — token length (step 1). `the_length_limit_counts_folded_characters` failed on the old analyzer: a
  40-character ASCII word came out as `["x", "y"]`. tantivy's `RemoveLongFilter` keeps a token only when its UTF-8
  length is *below* the limit, and it ran before the folding, so a 40-letter ASCII word and any accented word of
  20 or more letters were dropped. Fixed with a small `MaxCharsFilter` in `schema.rs` after `LowerCaser` and
  `AsciiFoldingFilter`: a token of up to 40 characters stays, a longer one goes; the constant is now
  `MAX_FOLDED_TOKEN_CHARS`. The test covers 40 and 41 ASCII letters, a 25-letter accented word (50 bytes before
  folding), `Árvíztűrőtükörfúrógéptükörfúrógép` (33 letters, 47 bytes) and 40 against 41 `ő`.
- 2026-10-19 — completions (step 2). The prefix query is built in `free_text.rs`, not in `search.rs`.
  `a_prefix_completes_to_frequent_words_before_dictionary_order` (one message with 300 one-off tokens `ab0000000`…
  that sort before `abrakadabra`, two messages with the real word) failed: `ab` found only the junk message.
  Decided: a new `RankedPrefixQuery` (`index/prefix.rs`) replaces `PhrasePrefixQuery`. When its weight is built it
  streams the prefix range of the term dictionary of each segment, keeps the best 200 in a bounded heap (more
  documents first, then the shorter term, then byte order), sums the segments' picks and keeps the best 200 of
  those; a lone prefix becomes a `TermSetQuery` (constant score, as tantivy's range expansion was), a prefix after
  words one `PhraseQuery` per completion. Without a searcher (never the case in `search` or `list`) it falls back
  to tantivy's byte-order expansion. A term frequent only across several segments can lose to one frequent in one
  segment; the index is built in one pass, so its segments are few and large.
  Cost, measured on a synthetic index (20 messages, 1 000 000 distinct terms under `zq`, 20 messages holding
  `zqreal`, four indexing threads, release build, 5 runs each): a first version that put every term of the range
  into a hash map took 517 ms per search; the bounded heap takes 28 ms (`zq`, `zq0` and the phrase `x zq` alike);
  the old byte-order expansion took 0.1 ms but found 4 of the 20 `zqreal` messages; a prefix with a short range
  (`zqr`) takes 0.1 ms either way. The cap stays at 200.
- 2026-10-19 — nested HTML (step 3). `a_nested_html_only_message_is_indexed_beside_a_plain_outer_one` failed:
  `berszamfejtes`, a word of the forwarded HTML-only message, had 0 hits. `body_text` now decides per message:
  `message::text_parts_by_message` groups the text parts by their owner (outer first, nested depth first), and
  each group gives its plain parts, or its HTML text when it has none. `text_parts` keeps its signature (the
  flattened groups). The outer message's HTML alternative stays out of the index (the test checks a word only in
  it), and the nested HTML's markup and style rules do not leak.
- 2026-10-19 — paging limits (step 4). `limit_and_offset_far_above_the_message_count_are_capped` failed with
  "attempt to multiply with overflow" in tantivy's `top_score_collector.rs`; the extended CLI test
  `count_and_paging` (`--limit 18446744073709551615`) failed the same way inside `mboxer-cli`. `search` now caps
  `limit` at the searcher's document count and answers an `offset` at or past it with the count alone; the CLI
  needs nothing else, a `--limit` above the count prints every hit.
- 2026-10-19 — read-back limit (step 5). No wrong answer in the shipped programs: the viewer (`state.rs`) and
  `mboxer-cli` both build with `BuildOptions::default()`, so `read_at`'s 256 MB default is their build's limit.
  The gap is a library caller with another limit. `IndexMeta` records `max_message_bytes` (an older
  `mboxer.json` without it reads as 256 MB, so no format change), `read_at_with_limit` and
  `read_at_from_with_limit` take a limit, and `MboxIndex::read_message(offset, len)` reads under the index's own.
  `a_message_is_read_back_under_the_size_limit_of_its_build` and
  `metadata_of_an_older_build_reads_with_the_default_size_limit` failed to compile against the old API.
- 2026-10-19 — tie order (step 6). `ties_list_in_mbox_order_after_a_four_thread_build` (400 messages with one
  date, eight of them in one thread, built with `threads: 4`) failed: the ungrouped listing was not in offset
  order, because document order is the order the indexing threads picked the messages up. `offset` is now a fast
  field too, the listing's collector reads it, and every tie (same date or score, members of a thread with one
  date, groups with equal keys) is broken by the offset; the `arrange` comment says so. The test checks the plain
  and the grouped listing, by date and by relevance, and the thread's members after a toggle; a unit test
  `ties_follow_the_offset_not_the_document_address` covers `arrange` with offsets against the segment order.
- 2026-10-19 — `from_addr`/`to_addr` (step 7). Nothing read them (only `schema.rs` and `build.rs` named them), and
  the `from:` and `to:` filters already match an address: the display string `Name <address>` is folded into
  words, so `from:anna@example.com` matches as a phrase. Wiring the exact fields in would need a new operator for
  the same answer. Decided: dropped. That changes the index layout, so `FORMAT_VERSION` goes from 2 to 3, and the
  same bump carries the other changes to what an index holds (the token rule, the nested HTML text, the `offset`
  fast field, quoted labels); an older index reads as stale and is rebuilt once. The `reader.rs` comments on the
  `line` and `body` buffers now describe the `mem::take` handoff: the line buffer is reused within a message, the
  body moves into the `RawMessage`. Keeping the body reusable would mean copying every message, not a one-line
  change.
- 2026-10-19 — corrupt metadata (step 8). `a_corrupt_metadata_file_reads_as_stale_and_a_rebuild_replaces_it`
  writes four damaged `mboxer.json` files over a built index. A truncated, an empty and a zero-filled file already
  read as stale (a JSON error): no issue there. A file with bytes that are not UTF-8 made `status` return
  `Io(InvalidData)`, and `mboxer-cli index --force` stopped on it before its `--force` was looked at. Now the file
  is read as bytes and parsed with `serde_json::from_slice`; a JSON or an I/O error on an existing metadata file
  is `Stale(Unreadable)`, so the viewer and `index` rebuild. `index --force` also rebuilds when the status check
  itself fails. The CLI test `a_corrupt_index_metadata_file_is_rebuilt` runs `index --force` and plain `index`
  over a non-UTF-8 file.
- 2026-10-19 — labels with a comma (step 9). mail-parser 0.11 parses `X-Gmail-Labels` as unstructured text (the
  parser is configured with `header_text`) and hands the quotes and backslashes over as written.
  `a_quoted_label_keeps_its_comma` failed: `"Work, Home",Inbox` became `"Work`, `Home"` and `Inbox`. Fixed with a
  quote-aware `split_labels`: a label that starts with `"` runs to the closing quote, `\` escapes the next
  character, a quote elsewhere is part of the name, an unclosed quote runs to the end. The RFC 2047 case of
  `labels-encoded.mbox` (a comma inside an encoded word separates two labels) is unchanged.
- 2026-10-19 — docs: `docs/reference.md` (quoted labels, the prefix completion, the token length rule, the text
  of nested messages, tie order, a damaged `mboxer.json`, `index --force`, capped `--limit`/`--offset`) and one
  line in `docs/usage.md` (a new version can rebuild each index once). Follow-ups go into the pull request
  description for the main session to file.

## Deviations from the plan

- Step 2: the expansion lives in a new module `index/prefix.rs`; the query is assembled in `free_text.rs`, which
  changes only its import, the constant and one call.
- Step 7: the format bump carries more than the two dropped fields (see the Log); each of those changes alters
  what an index holds, and one bump rebuilds every index once.
- Step 5: the viewer keeps calling `read_at`, whose default is its builds' limit; `MboxIndex::read_message` is
  there for an index built with another limit.
- The new tests sit in their own file, `tests/index_quality.rs`, beside the existing `tests/index.rs`.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo test --locked` green across the workspace; the new tests
  in `crates/mboxer-core/tests/index_quality.rs` (8 passed:
  `a_prefix_completes_to_frequent_words_before_dictionary_order`,
  `a_nested_html_only_message_is_indexed_beside_a_plain_outer_one`,
  `limit_and_offset_far_above_the_message_count_are_capped`,
  `a_message_is_read_back_under_the_size_limit_of_its_build`, `ties_list_in_mbox_order_after_a_four_thread_build`,
  `a_corrupt_metadata_file_reads_as_stale_and_a_rebuild_replaces_it`, `a_quoted_label_keeps_its_comma`,
  `metadata_of_an_older_build_reads_with_the_default_size_limit`), the unit tests
  `the_length_limit_counts_folded_characters`, `ties_follow_the_offset_not_the_document_address`,
  `labels_split_on_commas_outside_quotes` and `the_prefix_range_ends_after_the_last_completion`, and the CLI tests
  `count_and_paging` (with the `usize::MAX` limit and offset) and `a_corrupt_index_metadata_file_is_rebuilt`.
- Same day: `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` clean; the
  macOS-target clippy (`DOCS_RS=1`, `aarch64-apple-darwin`) and `cargo +1.90 check` for that target clean;
  `cargo deny check`, `uvx zizmor .github`, `python -I scripts/render-usage-guide.py --check` and
  `python -I scripts/changelog.py --check` pass.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
