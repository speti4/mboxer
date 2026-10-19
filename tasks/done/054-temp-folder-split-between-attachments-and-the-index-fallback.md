---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 054 — Keep the attachment copies and the index's last-resort home in separate temp folders

## Context

What: `clean_temp` in `crates/mboxer-app/src/attachments.rs` empties the whole `<temp>/mboxer` folder at start-up,
and the core's last-resort index location (`fingerprint.rs`) is `<temp>/mboxer/index`, so those indexes are deleted
on every launch, possibly while another instance opens them; the per-attachment folder names
(`{file}-{offset}-{index}`) also repeat across processes, so two running instances overwrite each other's copies, and
reopening an attachment that is still open in its application fails on the locked file.
Why: a second instance or a read-only archive in a temp-backed index should not lose data at start-up.
Done when: two instances open the same attachment without an error, the index under `<temp>` survives a restart, and
the clean-up test covers a stale folder from a dead process.
Decided 2026-10-19: filed by the post-release review; the core's temp index path stays where D-005 put it, the app
moves its copies under their own sub-folder.

## Steps

1. [x] tests first in `attachments.rs`: `clean_temp` leaves `<temp>/mboxer/index` alone; a folder of a live process
   is kept, one of a dead pid is removed; a second copy of the same attachment from another "process" folder does
   not collide; reopening an attachment whose copy is held open returns the existing copy when its bytes match
2. [x] the copies go to `<temp>/mboxer/attachments/<pid>/{file}-{offset}-{index}/`; `clean_temp` removes only
   `attachments/<pid>` folders whose process is gone (a pid probe per platform, or a lock file held by the process,
   decide and write the reason in the Log) and never touches `index/`
3. [x] reuse: when the target file exists and its length and bytes match, open it instead of rewriting; a
   mismatch gets a fresh folder name
4. [x] `docs/usage.md` ("Where mboxer writes") and `docs/reference.md` name the new folder; `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test`

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — tests first. `attachments.rs` was split, with the old behaviour kept, into `write_copy` (the copy
  under a given folder), `claim` (a process's folder, at first the shared root itself) and `clean_temp_in` (the
  clean-up under a given root), so the tests run on a `tempfile` root instead of the real temp folder. Six tests
  were added; before the change `cargo test -p mboxer-app --lib attachments` failed 5 of them:
  `clean_up_leaves_the_index_alone` (the index folder was deleted),
  `clean_up_keeps_a_live_process_folder_and_removes_a_dead_one` (the live copy was deleted),
  `copies_of_two_processes_do_not_collide` (both copies had one path), `a_different_copy_gets_a_fresh_folder` (the
  second copy overwrote the first) and `a_matching_copy_is_reused_without_rewriting_while_it_is_open` (os error 32,
  the sharing violation, with the copy held open for reading and denying writes, as Office holds a document).
  `old_layout_leftovers_are_removed` passed, as the old clean-up removed everything.
- 2026-10-19 — liveness: a lock file, not a pid probe. Each viewer creates `attachments/<pid>/.lock` on its first
  copy and holds it with `File::try_lock` (std since Rust 1.89, inside the 1.90 MSRV) for as long as it runs; the
  clean-up tries the same lock and removes the folder when it gets it, or when the lock file is missing. The
  system releases the lock when the process ends, a crash included. Reasons: no new crate and no unsafe FFI
  (`windows-sys` is not a direct dependency of `mboxer-app`, `sysinfo` is not in the graph), one code path for
  Windows and macOS, and no false "alive" when the pid of a dead viewer is reused by another process.
- 2026-10-19 — the change. The copies go to `<temp>/mboxer/attachments/<pid>/{file}-{offset}-{index}/`;
  `write_temp` claims the folder once per process and claims it again when the lock file has gone. `write_copy`
  hands back an existing copy whose length and bytes match without writing it, and moves a different file of the
  same name to `…-2`, `…-3`. `clean_temp` removes the unlocked `attachments/<pid>` folders and, once, the folders of
  the old layout (three decimal numbers joined by dashes directly under `<temp>/mboxer`); `index/` and any other
  name stay. All 16 attachment tests pass.
- 2026-10-19 — docs. `docs/usage.md` ("Where mboxer writes") names `attachments` in the attachment row and when it
  is cleaned; `docs/reference.md` (attachments) names the per-process folder, the reuse of an unchanged copy and
  that the clean-up leaves the index home alone.

## Deviations from the plan

- A folder of copies whose lock file is missing counts as stale, not only one whose lock is free: a viewer that
  crashed between creating its folder and its lock file would otherwise leave the folder forever.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo fmt --all --check` clean; `cargo clippy --all-targets
  --locked -- -D warnings` clean; `cargo test --locked` green (mboxer-app 50 unit tests, 16 of them in
  `attachments`, `ui_query` 1, mboxer-cli 4 + 6, mboxer-core 32 + 10 + 22); with `DOCS_RS=1`, `cargo clippy
  --target aarch64-apple-darwin --all-targets --locked -- -D warnings` clean and `cargo +1.90 check --target
  aarch64-apple-darwin --all-targets --locked` clean; `python -I scripts/render-usage-guide.py --check` passes.
- 2026-10-19, lock release across processes: after the test run, the folder the test binary had claimed under
  `<temp>/mboxer/attachments/` was probed from a separate PowerShell process: the pid no longer ran, and a byte
  lock on its `.lock` file was granted, so a later start removes it. The viewer itself was not started.
- 2026-10-19, after rebasing onto task 052 (whose build locks live inside the index home, which the clean-up
  leaves alone): `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test
  --locked`, `python -I scripts/changelog.py --check` and `python -I scripts/render-usage-guide.py --check` green
  again.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none found)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
