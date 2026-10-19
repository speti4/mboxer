---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
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

1. [ ] a failing test in `crates/mboxer-app/src/state.rs` (or `commands.rs`): three queries where the first holds
   the lock, the third takes it before the second; the listing left behind must belong to the newest generation
2. [ ] the fix: the generation is taken under the lock, or a query whose generation is older than the counter
   returns without touching the listing; `list_info` reports the generation the listing belongs to
3. [ ] the readers: `list_rows`, `toggle_thread`, `position_of`, `activate_file`, `close_file` no longer block the
   webview for the length of a search (the search runs on a snapshot or the listing sits behind its own lock, or the
   commands become async with `spawn_blocking`); the comment above `set_query` describes what is true
4. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --check ui/main.js`; a
   CLI or test-level timing note in the Log; the manual scroll-during-search check is left to the maintainer's
   round (064) and said so in Verification

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
