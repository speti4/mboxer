---
size: S
origin: done/051-query-generation-race-and-sync-commands-on-the-files-lock.md
created: 2026-10-19
started: 2026-10-20
closed: 2026-10-20
plan: none
milestone: M6 Hardening
---

# 067 — Stop an overtaken search early instead of letting it run to its end

## Context

What: since task 051 the newest query always wins, but a query overtaken in the middle of its search still runs to
its end before the newest one gets its turn on the search lock. A tantivy collector that checks the file's newest
generation every N documents (or a cancellation flag the search loop reads) would end the stale search within
milliseconds and hand the lock over.
Why: on a large archive a relevance query takes seconds; typing three characters in that window queues three
searches and the list waits for all of them.
Done when: a test over a slow query and a newer one shows the stale search ending early, and the queued-query wait
in the 051 test drops accordingly.
Task 051 left the generation bookkeeping in `AppState::set_query` (`crates/mboxer-app/src/state.rs`): the
generation is taken under the files lock, the search runs on an `Arc<MboxIndex>` snapshot under a separate search
lock, and the result is swapped in only while the query is still the file's newest one.

## Steps

1. [x] a cancellation check the search can read: the file's newest query (an `AtomicU64` or an `Arc<AtomicBool>`
   handed to the search) compared every N documents inside the collector or the listing loop in
   `crates/mboxer-core/src/index/` (`search.rs`, `listing.rs`), ending the search with a "stale" result
2. [x] `AppState::set_query` passes the check and treats a stale search like an overtaken one (`superseded`,
   the listing in place stays)
3. [x] a test in `state.rs` over a slow query (the test-only pause hook or a large enough fixture) and a newer
   one: the stale search ends early, measured; the queued-query wait in the 051 test drops accordingly
4. [x] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   macOS-target clippy and the `rust-version` check (CONTRIBUTING, "Checks before a commit")
5. [x] `docs/reference.md` if the behaviour is user-visible; the Log says the measured before/after

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-20 — the check. Decided 2026-10-20: the search reads the file's newest query itself. `OpenMbox`
  holds it as an `Arc<AtomicU64>` (written under the files lock as before, read by the running search without
  it), and the search asks a `stale` closure (`newest != generation`) through the new
  `MboxIndex::list_unless(query, stale) -> Result<Option<Listing>>`; `list` is `list_unless` with a check that
  never says yes. Reason: one source of truth, the generation bookkeeping of task 051, instead of a second
  per-query flag that every path would have to set; mboxer-core stays free of the app's generations, it only
  sees a closure.
- 2026-10-20 — where the check sits. tantivy's own document loop (`Weight::for_each`) cannot be left half-way,
  so the listing collector (`AllMatches`) overrides `Collector::collect_segment` and walks the scorer itself:
  the check is asked before the first document of each segment and then every `STALE_CHECK_EVERY` (1024)
  documents, the alive bitset is honoured as in tantivy's loop, and a score is read only for a relevance
  sort. A stopped search sets a flag on the collector, the fruits are thrown away and `list_unless` answers
  `None`. Decided 2026-10-20: 1024 documents, a few milliseconds of scoring at most in a debug build; the check
  itself is one atomic load. The page search of the CLI (`MboxIndex::search`) keeps tantivy's collectors: it
  has no newer query to give way to.
- 2026-10-20 — `AppState::set_query`: a `None` from the search, or a listing whose query is no longer the newest
  by the swap, answers `superseded` with the listing in place, as an overtaken query did. Decided 2026-10-20:
  a reload of the file (`OpenFiles::insert` over an open file) takes over the file's `Arc<AtomicU64>` and
  stores its own generation into it, and `OpenFiles::close` stores 0 (query generations start at 1), so a
  search on a reloaded or closed file stops too; the answers are the ones of task 051 (`superseded`, or
  `that file is no longer open`). The comments above `AppState::set_query` and the `set_query` command say it.
- 2026-10-20 — tests. `an_overtaken_search_stops_early` in `state.rs` builds a synthetic mbox of 60 000 short
  messages in a temp dir (about 1 s in a debug build), times a relevance query for `common topic` (a prefix,
  every message matches) alone, then runs it through `set_query_with` and sends a newer query a fifth of the
  way into the search; the overtaken search must end before three fifths of the whole search, and the newer
  query, queued behind it, must wait less than half of it. `a_stale_listing_stops_and_answers_none` in
  `crates/mboxer-core/tests/index.rs` covers `list_unless` itself (asked even on a two-message index; a check
  that says yes ends a date and a relevance search with `None`). The 051 race test is unchanged but for a
  comment: A's search now stops at its first check, C being the newest query by then.
- 2026-10-20 — measured, debug build, Windows 11: before (the same test with the check switched off), the
  whole search took 44–48 ms, the overtaken one ran on for 46–49 ms and the newer query waited 38–41 ms; after,
  over five runs, the whole search took 40–121 ms, the overtaken one ended after 10–29 ms (the fifth of the way
  plus 1–2 ms) and the newer query waited 2.0–3.9 ms, its own search included.
- 2026-10-20 — `docs/reference.md`, Search: one sentence, a search still running when the next character is
  typed stops where it is.
- 2026-10-20 — checks in the worktree: `cargo fmt --all --check` clean, `cargo clippy --all-targets --locked --
  -D warnings` clean, `cargo test --locked` green (mboxer-app 53 unit + 1 Node runner; mboxer-cli 4 + 7;
  mboxer-core 43 unit, 10 fixtures, 26 index, 6 index_build, 8 index_quality, 1 tantivy_excluded_phrase), the
  macOS-target clippy (`DOCS_RS=1`, `aarch64-apple-darwin`) and `cargo +1.90 check` on that target clean.
  `ui/main.js`, `Cargo.toml` and `Cargo.lock` are not touched.

## Deviations from the plan

- The done-when names the queued-query wait of the 051 test; that test holds its searches with the pause hook,
  so its waits are set by the test and cannot drop. The wait is measured in the new test instead, before and
  after, over a search slow enough to be overtaken in its middle.
- A reload or a close of the file also stops a running search, which the steps did not ask for: it comes free
  with the shared newest query.

## Verification

- 2026-10-20, Windows 11, in the task worktree: `cargo test --locked` green; among them
  `state::tests::an_overtaken_search_stops_early` (five runs: the overtaken search ended after 10–29 ms of a
  40–121 ms search, the newer query waited 2.0–3.9 ms; with the check switched off it ran 46–49 ms and the
  newer query waited 38–41 ms), `state::tests::the_newest_query_leaves_its_listing`,
  `state::tests::a_running_search_leaves_the_files_lock_free` (the reader waited 1.2 µs) and
  `a_stale_listing_stops_and_answers_none` in `crates/mboxer-core/tests/index.rs`.
- Not run here: typing over a relevance query on a large archive in the viewer; it belongs to a maintainer's
  manual round.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none found)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-20`
