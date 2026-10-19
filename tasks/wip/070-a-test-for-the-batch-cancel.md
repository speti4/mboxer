---
size: S
origin: done/061-small-hygiene-follow-ups-from-the-review.md
created: 2026-10-20
started: 2026-10-20
plan: none
milestone: M6 Hardening
---

# 070 — A test for the Cancel between two files of a batch

## Context

What: task 061 made the cancel flag clear once per batch and checked before each file, so a Cancel between two
files stops the next one, but nothing tests it: the file loop sits inside the `open_mbox` Tauri command. Move the
loop into a function of `state.rs` that takes the paths, the cancel flag and a progress hook, and drive it from a
unit test that cancels after the first file.
Why: a behaviour fix without a test is the next regression.
Done when: the test opens two fixture files, cancels after the first, and sees the second untouched.
The loop is `open_mbox_inner` in `crates/mboxer-app/src/commands.rs` (the cancel flag is cleared once at its
start and read before each file; each file loads through `OpenMbox::load` with the flag and a progress closure);
`AppState` and `OpenMbox` live in `state.rs`, where task 051 already moved `set_query` for the same reason.

## Steps

1. [ ] the file loop of `open_mbox_inner` moves into a function of `state.rs` without Tauri types: it takes the
   paths, the cancel flag, the thread-grouping flag and a progress hook, returns what the command needs to emit;
   `open_mbox_inner` becomes the Tauri glue around it
2. [ ] a unit test in `state.rs` over two fixture files whose progress hook sets the cancel flag after the first
   file: the first file is open, the second untouched (no `OpenMbox`, no sidecar built), the result says
   cancelled
3. [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   macOS-target clippy and the `rust-version` check; the viewer's open flow unchanged in behaviour
4. [ ] no docs change expected; the Log says so if that holds

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (filled at the close: date, the test name, what it asserted)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
