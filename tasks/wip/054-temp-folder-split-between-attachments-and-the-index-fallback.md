---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
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

1. [ ] tests first in `attachments.rs`: `clean_temp` leaves `<temp>/mboxer/index` alone; a folder of a live process
   is kept, one of a dead pid is removed; a second copy of the same attachment from another "process" folder does
   not collide; reopening an attachment whose copy is held open returns the existing copy when its bytes match
2. [ ] the copies go to `<temp>/mboxer/attachments/<pid>/{file}-{offset}-{index}/`; `clean_temp` removes only
   `attachments/<pid>` folders whose process is gone (a pid probe per platform, or a lock file held by the process,
   decide and write the reason in the Log) and never touches `index/`
3. [ ] reuse: when the target file exists and its length and bytes match, open it instead of rewriting; a
   mismatch gets a fresh folder name
4. [ ] `docs/usage.md` ("Where mboxer writes") and `docs/reference.md` name the new folder; `cargo fmt --check`,
   `cargo clippy --all-targets -- -D warnings`, `cargo test`

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
