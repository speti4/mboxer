---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: folytassuk-a-009-taskkal-peaceful-sunbeam.md
---

# 009 — Gmail labels and conversation threads

## Context

Since 005/006 the folders pane shows a flat label list with counts and a click is an exact `labels` filter;
`thread_id` (X-GM-THRID) is indexed + stored + FAST, but nobody reads it back. 009 adds two things: a
conversation view in the list (one row = one thread with the newest message and the count, expandable, the
grouping in the core `Listing`, because the members are scattered in the date-sorted list) and a label tree
in the folders pane (`/` hierarchy + the Takeout `Category …` labels under one *Categories* node).
Why: Takeout gives the Gmail structure for free, this is what makes the flat archive navigable.
Done when: a click on a label narrows the list; a message with replies shows as one expandable
conversation.

Decided 2026-10-17 (planning round): grouping on by default, a *Conversations* switch on the filter strip;
a click on a parent label = parent + every sub-label (OR, `any_labels`); `Category *` → *Categories*
node; oldest first in an expanded thread. The index format does not change (no rebuild); no new
crate (D-002). The real archive's header sample (600 MB) has no label with `/` and none RFC 2047-encoded.

## Steps

1. [x] Core `search.rs`: `SearchQuery.any_labels` (Should clause), `group_threads`, `Hit.thread_id`
2. [x] Core `listing.rs`: `Entry` collector (`date` + `thread_id` column), `arrange` CSR, `Expanded` prefix sum,
   `locate` / `row_of_group`, `ListRow` / `RowKind`, `rows` / `toggle` / `position_of` / `len` / `message_count` /
   `group_count`; unit tests; exports
3. [x] Fixture `threads.mbox`; `tests/index.rs` cases; `cargo test -p mboxer-core`
4. [x] App `types.rs` (`QueryInput`, `Row` kind, `ListInfo`, `PositionInfo`, `LabelNode`), `folders.rs` (`label_tree`,
   `Bin`), `state.rs` (`load` with grouping), `commands.rs` (`open_mbox`, `toggle_thread`, `position_of`), `lib.rs`; tests
5. [x] UI `index.html` + `style.css`; `main.js` label tree, thread rows, toggle + layout token, keys, status, switch
6. [x] `node --check` + stub-DOM check (`stripReplyPrefix`, `labelsUnder`, tree order, selection shift)
7. [x] README; Log / Deviations / Verification; backlog item for the RFC 2047 label names
8. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; debug exe on the fixture copy and on the
   14 GB file (stderr `query:` timings); manual round; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 009 promoted to wip
- 2026-10-17 — core: `SearchQuery.any_labels` (Should TermQueries inside one Must clause) and `group_threads`,
  `Hit.thread_id`; `listing.rs` rewritten: the `AllMatches` collector gathers `Entry` values (`date` + `thread_id`
  FAST column, `NonZeroU64` niche), `arrange` sorts them into a CSR (the ungrouped branch keeps today's order bit
  for bit), the row arithmetic lives in the `Layout` struct (`Expanded` prefix sum, `partition_point`),
  `ListRow`/`RowKind`, `toggle`, `position_of` with auto-expansion. Fixture `threads.mbox` (6 messages, B3 at the
  end of the file but the oldest, `Workshop`/`Work-archive` as negative cases). First run: one unit test's
  expectation was wrong (thread 2's newest member 14 > the threadless 13 → the thread comes first), the code was
  right. 29 unit + 8 + 19 core green
- 2026-10-17 — app: `LabelNode` tree (`folders.rs` `label_tree`: `/` segments, `Category <x>` → `Categories`
  node, `Bin`/`Chat` in the well-known list), `QueryInput.anyLabels/groupThreads`, `Row.kind/threadCount/
  expanded/threadIndex` (the thread id is not sent out: u64 > 2^53), `ListInfo.messages/conversations`,
  `PositionInfo`, `open_mbox(groupThreads)` → `OpenMbox::load` with grouping, `toggle_thread(generation, row)`;
  the CLI literal got the two new fields. One test expectation fixed (the `Categories` node comes after the
  well-known block, before the other labels). `cargo test` 30 app + 4 + 5 CLI + 29 + 8 + 19 core green; fmt +
  clippy clean
- 2026-10-17 — UI: `Conversations` checkbox (`#f-threads`, localStorage `mboxer.groupThreads`, Clear leaves it
  alone), label tree with a chevron and `--depth` indentation (container: `total` in italics; key = the exact
  label or `<path>#`), `labelsUnder(key)` → `anyLabels`; the list row has 6 columns (`twisty, att, from, subject,
  count, date`), a thread row with `stripReplyPrefix` and a count badge, a member row indented; `toggleThread`
  with the `state.layout` token (`requestBlock` drops the pages of an old layout, `pending` clears only its own
  promise), the selection becomes `max(row, selected + delta)`; ArrowRight opens / ArrowLeft closes in the list
  and in the tree; `position_of` → `PositionInfo.total` taken over. `node --check` clean; stub-DOM check 35
  cases, 0 failures
- 2026-10-17 — `target\debug\mboxer.exe` on the `threads.mbox` copy: `opened threads.mbox: 6 messages, Built
  { build_secs: 0, parse_failures: 0 }, 92 ms`; on the 14 GB file 1352 ms with a cold cache, 88 ms warm (the
  005/006 ungrouped listing was ~60 ms in the same debug build → grouping the 86 085 messages costs ~30 ms). Both
  windows left open for the manual round. Follow-up 015 (RFC 2047-encoded label names) filed in the backlog
- 2026-10-17 — implementation committed as `f7f95ef`; waiting for the manual round
- 2026-10-17 — manual round on the fixture copy and the 14 GB archive passed. Follow-ups: 015 filed earlier
  (RFC 2047 label names); the "only matching members in a conversation" limit stays under Deviations as
  accepted. Closed

## Deviations from the plan

- `setTotal` (a row count change after a toggle or `position_of`) drops every cached page, not only those from the
  thread row's block down: simpler, and the 1–3 pages of the visible window reload, which does not show with
  200-row pages.
- The `Layout` struct (row arithmetic tested without an index) is a type of its own instead of the plan's
  "fields of the listing"; `Listing` delegates to it. The API is as planned.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  (new: `label_tree` 5 cases — ordering with `Bin` and the `Categories` node, `/` hierarchy + `labels` +
  `total`, container parent, Takeout categories, odd labels; `QueryInput` `anyLabels`/`groupThreads`; `Row`
  kind serde, `ListInfo`/`PositionInfo`/`LabelNode` shape; `conversations_are_grouped_from_the_first_listing`) +
  4 + 5 CLI + 29 core unit (new: `arrange` 3 cases, `Layout` row arithmetic with 1 000 groups, member row
  expansion, ungrouped layout) + 8 fixtures + 19 index (7 new: ungrouped 6 rows, grouped 3 rows, toggle,
  `position_of` auto-expansion, only matching members under relevance too, `any_labels`, a member without a date)
  pass.
- 2026-10-17 — `node --check ui/main.js` clean; stub-DOM check (scratchpad `ui-check-009.js`, on functions cut out
  of `main.js`): `stripReplyPrefix` 8 cases, rendering the label tree (visible names, depth, keys, container
  `total` vs own count, chevrons, separator), `labelsUnder` 5 cases, selection by key, collapse/expand by click
  and the twisty's `stopPropagation`, the toggle's selection rule 6 cases: 35 cases, 0 failures.
- 2026-10-17 — `target\debug\mboxer.exe` on the fixture copy: 6 messages, index built in 92 ms; on the 14 GB file
  the index reused, 1352 ms with a cold cache, 88 ms warm (grouping ≈ 30 ms in a debug build, the < 100 ms target
  holds).
- 2026-10-17 — manual round on the fixture copy and on the 14 GB archive: conversations show as one
  row with the count, the chevron / ArrowRight opens them oldest first and ArrowLeft closes them, a label click
  narrows to the label and its sub-labels (Categories node included), the selection survives a narrowing with
  the conversation opened around it, the Conversations switch returns to one row per message, the status bar
  counts conversations and messages. Done-when met: a label click narrows the list;
  a message with replies shows as one expandable conversation.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — 015 filed; one accepted limit under Deviations
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
