---
size: S
origin: done/051-query-generation-race-and-sync-commands-on-the-files-lock.md
created: 2026-10-19
started: 2026-10-20
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

1. [ ] a cancellation check the search can read: the file's newest query (an `AtomicU64` or an `Arc<AtomicBool>`
   handed to the search) compared every N documents inside the collector or the listing loop in
   `crates/mboxer-core/src/index/` (`search.rs`, `listing.rs`), ending the search with a "stale" result
2. [ ] `AppState::set_query` passes the check and treats a stale search like an overtaken one (`superseded`,
   the listing in place stays)
3. [ ] a test in `state.rs` over a slow query (the test-only pause hook or a large enough fixture) and a newer
   one: the stale search ends early, measured; the queued-query wait in the 051 test drops accordingly
4. [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   macOS-target clippy and the `rust-version` check (CONTRIBUTING, "Checks before a commit")
5. [ ] `docs/reference.md` if the behaviour is user-visible; the Log says the measured before/after

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (filled at the close: date, the test names, the measured wait before and after)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
