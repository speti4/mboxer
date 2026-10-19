---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M4 macOS
---

# 019 — macOS runtime differences: index home, quarantine, Finder open, UI keys and fonts

## Context

What: `default_index_home` falls back to `$HOME/Library/Application Support/mboxer/index` on macOS (D-05 amended,
CLI help and doc strings follow); `mark_of_the_web` gets a macOS arm writing the `com.apple.quarantine` xattr
(best effort, the `xattr` command, no new crate); `BLOCKED_EXTENSIONS` adds the macOS executable types (`command
tool pkg mpkg dmg app sh scpt applescript workflow terminal webloc inetloc fileloc`); `RunEvent::Opened` (Finder
"Open with", Dock drop) feeds `open_mbox`; the UI's path keys (`files.js`, `main.js queryKey`) become OS-aware,
tooltips show `⌘` on a Mac, the monospace stack becomes `ui-monospace, Menlo, Consolas, monospace`, "follows
Windows" becomes "follows the system"; `files.test.js` takes the OS as a parameter.
Why: the Windows assumptions the 2026-10-18 sweep found outside printing (020) and packaging (018).
Done when: `cargo test` and the node tests pass on both CI runners; on a Mac an opened attachment carries the
quarantine xattr, a read-only mbox folder indexes under `~/Library`, and Finder "Open with" opens the file.
Run as a task session under docs/git-workflow.md, in parallel with 036, 038 and 018 (plan 2026-10-19); the
`macos-latest` runner arrives with 018, so this branch rebases onto `main` after 018 merges. The author has no
Mac: the on-Mac checks of the done-when are a follow-up. Docs: this task writes `docs/usage.md`; 018 writes
`docs/development.md`.

## Steps

1. [ ] `mboxer-core`: `default_index_home` with the macOS fallback; D-05 amended in place (an `Amended` line, the
   decision stays `decided`); CLI help and doc strings follow
2. [ ] `mboxer-core`: `mark_of_the_web` macOS arm (`xattr -w com.apple.quarantine`, best effort);
   `BLOCKED_EXTENSIONS` gains the macOS executable types, with a test
3. [ ] `mboxer-app`: `RunEvent::Opened` feeds `open_mbox` (Finder "Open with", Dock drop)
4. [ ] `ui/files.js`, `ui/main.js`: OS-aware path keys and `⌘` tooltips; the monospace stack; "follows the
   system"; `tests/ui/files.test.js` parameterised by OS; `docs/usage.md` follows
5. [ ] `cargo test`, the node tests and `cargo check --target aarch64-apple-darwin` pass on Windows
6. [ ] Pull request, CI green (both runners once 018 is merged and the branch is rebased)

## Log

- 2026-10-19 — promoted in the main session, branch `task/019-macos-runtime-differences`

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
