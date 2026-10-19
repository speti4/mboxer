---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
---

# 052 — Swap the index atomically, lock the build, and skip a bad message instead of failing the build

## Context

What: `build()` in `crates/mboxer-core/src/index/build.rs` removes the old index folder and then renames
`.building` into place; on Windows a reader that still holds the old index makes the removal fail half-way, the
old index is partly gone and the new one stays in `.building`, so a full build of a 14 GB file is lost.
`prepare_building_dir` deletes any existing `.building`, so two builds of one mbox destroy each other, and one
`MessageTooLarge` or read error inside the stream (`let raw = item?`) aborts the whole build.
Why: a rebuild after a Takeout refresh is the one moment the viewer touches the index, and it should never leave
the archive without one.
Done when: a swap with the old index open, two concurrent builds, and a fixture with one oversized message each
have a test that passes, and `mboxer-cli index --force` over a large archive survives an open viewer.
Decided 2026-10-19: filed by the post-release review; the index format and `mboxer.json` stay as they are, a lock
file in the index home is acceptable.

## Steps

1. [ ] tests first in `crates/mboxer-core/tests/index.rs`: a rebuild while a `Searcher` / `Listing` holds the old
   index open (Windows file locks), two builds of the same mbox started at once, a fixture with one message over
   a small `max_message_bytes`; each red before the change
2. [ ] the swap: rename the old folder aside, rename `.building` into place, remove the old folder afterwards
   with a retry loop on Windows (`fs::remove_dir_all` fails on an open file), and clean the aside folder on the
   next build if it survived a crash
3. [ ] the lock: a lock file in the index home (`fs4` is already in the dependency graph through tantivy, or
   `File::create_new` with the pid inside), held for the build, a clear `Error` when another build holds it
4. [ ] the skip: an oversized or unreadable message is counted (`parse_failures` or a new `skipped` counter in
   `IndexMeta` / `BuildReport`), its envelope offset logged, and the build goes on; the CLI prints the count
5. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` (the
   index section) says what happens to a skipped message and to a second build

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
