---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 051 — `set_query` takes its generation before the files lock, and the sync commands block on that lock

## Context

What: in `crates/mboxer-app/src/commands.rs` `set_query` increments the generation counter and only then takes the
files mutex; when two queries wait on a third, the later one can win the lock first, so the backend ends on an older
listing than the UI waits for, every `list_rows` page is discarded and the list stays at `…`. Take the generation
under the lock, or return early from a query older than the current generation. In the same change, `list_rows`,
`toggle_thread`, `position_of`, `activate_file` and `close_file` are sync commands blocking the webview on the same
mutex while a relevance query holds it, so scrolling during a slow query freezes the window; split the listing from
the search state or make the readers async, and make the comment above `set_query` true.
Why: fast typing over a large archive is the load the viewer is built for.
Done when: a test over a slow query and two queued ones ends on the newest listing, and the window scrolls during a
relevance query over a large archive.
Decided 2026-10-19: filed by the post-release review; the UI's generation protocol (`queryToken` in `ui/main.js`)
stays, the backend is made to honour it.

## Steps

1. [x] a failing test in `crates/mboxer-app/src/state.rs` (or `commands.rs`): three queries where the first holds
   the lock, the third takes it before the second; the listing left behind must belong to the newest generation
2. [x] the fix: the generation is taken under the lock, or a query whose generation is older than the counter
   returns without touching the listing; `list_info` reports the generation the listing belongs to
3. [x] the readers: `list_rows`, `toggle_thread`, `position_of`, `activate_file`, `close_file` no longer block the
   webview for the length of a search (the search runs on a snapshot or the listing sits behind its own lock, or the
   commands become async with `spawn_blocking`); the comment above `set_query` describes what is true
4. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --check ui/main.js`; a
   CLI or test-level timing note in the Log; the manual scroll-during-search check is left to the maintainer's
   round (064) and said so in Verification

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — tests first. The body of the `set_query` command moved unchanged into `AppState::set_query` in
  `state.rs` (no Tauri types, so a test drives it), with a test-only pause hook called at two `QueryStage`s:
  `Queued` (the generation is taken, the query waits for its turn) and `Searching` (it has its turn). Two tests
  over `gmail-crlf.mbox`: `the_newest_query_leaves_its_listing` (A pauses while searching, B pauses while queued,
  C queues behind A; A is let go, C finishes, then B) and `a_running_search_leaves_the_files_lock_free` (a reader
  takes the files lock while a query is paused in its search; the pause gives up after 2 s so the old code does
  not hang the test). Against the old logic both failed: the listing ended on generation 2 (B) while the newest
  was 3 (C), and the reader waited 2.006 s for the paused search and then saw the new listing instead of the one
  in place.
- 2026-10-19 — the race. Decided: the generation is taken under the files lock and recorded on the file as its
  newest query (`OpenMbox::newest_query`); a query swaps its listing in only while it is still that newest
  query. An overtaken query answers with the listing in place and the new `ListInfo.superseded` flag; the
  webview's `queryToken` already drops that answer, so `ui/main.js` is unchanged. A reload of the file (a new
  `OpenMbox` with its own generation) overtakes a query on the old index the same way.
- 2026-10-19 — the readers. Decided: option (a), the search runs on a snapshot outside the files lock. `OpenMbox`
  holds its index as an `Arc<MboxIndex>`; the query takes the generation and a handle on the index under the
  files lock, releases it, searches, and takes the lock again only for the swap. Reason: the readers then wait
  only for those two short holds, whatever the search costs, and stay sync commands; making them async alone
  (option c) would only move the wait off the webview thread, the rows would still come after the search.
  Searches take turns on a separate `search` lock (always taken before the files lock, never while holding it),
  so two relevance queries do not compete for the CPU, and a query that a newer one overtook while it waited for
  its turn skips its search. `activate_file` and `close_file` now also take their generation under the lock. The
  comment above the `set_query` command says this.
- 2026-10-19 — timing: with the fix the reader during a paused search waited 1.3–1.6 µs over five runs (2.006 s,
  the whole pause, before). The whole `state` test module runs in about 0.2 s.
- 2026-10-19 — checks in the worktree: `cargo fmt --all --check` clean, `cargo clippy --all-targets --locked --
  -D warnings` clean, the macOS-target clippy (`DOCS_RS=1`, `aarch64-apple-darwin`) and `cargo +1.90 check` on
  that target clean, `cargo test --locked` green (mboxer-app 46 unit + 1 Node runner; mboxer-cli 4 + 6;
  mboxer-core 38 unit, 10 fixtures, 25 index, 6 index_build), the 51 Node UI tests pass. `ui/main.js` is not
  touched, so no `node --check` was needed; `Cargo.toml` and `Cargo.lock` unchanged.
- 2026-10-19 — follow-up, in the pull request description: a query overtaken in the middle of its search runs
  to its end before the newest one gets its turn; stopping it early needs a collector that checks the file's
  newest query.

## Deviations from the plan

- `OpenMbox::set_query` (searching under the caller's lock) is now test-only; the app goes through
  `AppState::set_query`.
- Searches are serialized on their own lock, which the task did not ask for: it keeps an overtaken query from
  searching at all and keeps two searches from competing.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo test --locked -p mboxer-app --lib` — 46 passed, among
  them `the_newest_query_leaves_its_listing` (the file ends on generation 3 with the Sent listing of one row; C
  answers generation 3, not superseded; A and B answer `superseded`, B with generation 3) and
  `a_running_search_leaves_the_files_lock_free` (the reader sees generation 0 and two rows while the search is
  paused, then the query lands with generation 1 and one row); the two race tests passed five runs in a row.
  `ListInfo` serializes with `"superseded": false`.
- Not run here: the manual check that the window scrolls during a relevance query over a large archive; it
  belongs to the maintainer's round (task 064).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
