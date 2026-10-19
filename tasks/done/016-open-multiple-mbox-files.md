---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-18
closed: 2026-10-18
milestone: M3 Comfort
plan: task-016-glimmering-meadow.md
---

# 016 — Open multiple mbox files

## Context

Several mboxes loaded at once, one under the other in the folder panel (Thunderbird "account" pattern), the
structure of one expanded at a time (All mail + label tree); the list, the search and the reader refer to the
active file. Why: Takeout splits large archives into several files, and the users keep several exports side
by side; switching is one click, without a restart and re-indexing. Done when: the user opens a
second mbox without a restart; the panel shows every loaded file, the active one expanded;
a click on the other one switches the list, search, label tree; a file can be closed; after a restart the files
with a fresh index are reloaded. Decided 2026-10-18: no merged view (reading (a) of the backlog
dropped), the files appear one under the other, one active at a time. Planning decisions (approved
plan): backend `OpenFiles` set + active, UI key `(file id, offset, len)`, the file id an in-process
counter (D-006, proposed); Open… multi-select and adds; the search text is kept per file; `mboxer.files`
+ `mboxer.activePath` in localStorage, at start-up only the files with a fresh index are reloaded, the ones with a
stale index wait as a faded root; no new crate (D-002).

## Steps

1. [x] `src/state.rs`: `OpenFiles` (insert/dedupe, activate, close, infos), `OpenMbox.id` + `canonical`; unit tests
2. [x] `src/types.rs`: `FileInfo`, `OpenSummary.id/files`, `OpenEvent::Opening{position,count}` / `Failed{path}` / `Done`, `IndexStatusInfo`, `MessageView.file`
3. [x] `src/commands.rs` + `lib.rs`: `open_mbox(paths, activate)` batch + multi-pick, `activate_file`, `close_file`, `index_status`; a `file` param on every file command
4. [x] `src/attachments.rs`: temp folder `{file}-{offset}-{index}`
5. [x] `ui/files.js` (new) + `tests/ui/files.test.js`, run by `tests/ui_query.rs`
6. [x] `ui/index.html`: `files.js`, `#reopen-list`, Open button title
7. [x] `ui/main.js`: `state.files/remembered/fileQueries`, batch open events, `activateFile`, root rows in `renderFolders`, `×` close, `file` in every invoke, reload in `start()`
8. [x] `ui/style.css`: `.folder.root`, `.root.unloaded`, `.close-file`
9. [x] `docs/usage.md` "Several files" paragraph, one line in `README.md`
10. [x] `/tasks decide` → D-006 (proposed)
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --test`; Chrome stub check; `tasks_check`; commit
12. [x] Manual round: the 14 GB archive + a second mbox in the real Tauri window

## Log

- 2026-10-18 — plan approved; 016 promoted to wip; D-006 (proposed) recorded
- 2026-10-18 — implemented: backend `OpenFiles` set (per-process `file` id), batch `open_mbox` (`Opening{position,count}`,
  `Failed{path}`, `Done`), `activate_file` / `close_file` / `index_status`; UI: root rows in the folder panel, one
  expanded at a time, search text remembered per file, `files.js` (`mboxer.files` + `lastPath` migration), reload at
  start-up only for a fresh index, stale → faded root. `cargo test`: 35 app (4 new `OpenFiles` tests) + 1 UI-node +
  core/CLI green; `node --test` 30 pass (16 query + 6 theme + 8 files); `cargo fmt --check`, `clippy --all-targets -D
  warnings` clean
- 2026-10-18 — Chrome stub check (scratchpad `serve-ui-files.js` + `stub-files.js`, 127.0.0.1:8768; A fresh, B stale,
  C missing, the "dialog" returns B+D): at start A loaded+active, B a faded root, C dropped from storage; a click on B
  builds it and makes it active; a search in B goes with `file: 2`, `read_message` carries the file id too; switching
  to A empties the search field, back to B "foo" returns; ArrowUp from All mail skips the active root and switches to
  A; × on A: B stays active, storage updated; Open… (B again + D): B keeps its query, D a non-active root; × on the
  active B: D takes over; after the last × the empty view, search disabled, storage empty. The check found two bugs,
  both fixed: reopening the active file cleared the search text; keyboard navigation got stuck on the active root.
  Screenshots dark + light: fine
- 2026-10-18 — commit `cba85f3` (feat, task 016)
- 2026-10-18 — with one real mbox at hand, two synthetic samples (`target/manual-round/sample-threads.mbox`,
  `sample-attachments.mbox`, from the core fixtures) stood in as the second and third file; the debug exe started
  with the 14 GB archive. Manual round (multi-select Open…, switching and searching per file, an attachment from
  both files, closing a non-active and the active row, reload after a restart, a stale index as a faded root):
  passed. No follow-up; D-006 `decided`; closed

## Deviations from the plan

- Instead of `state.known` the UI draws the `state.remembered` list (`{paths, active}`, the value of files.js): the
  panel order is the remembered order, and the loaded files' `FileInfo` is matched to it by path — one source of truth
- The order of the given paths is kept (the one to activate first); only the dialog selection is sorted by name
- After a switch a `set_query` always runs (with the restored query and the current Conversations mode) instead of
  trusting the backend's old listing; not after a fresh open

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass (35 app,
  including 4 new `OpenFiles` tests; the UI-node runner 3 files, 30 pass). Chrome stub check as in the Log, with 2
  bugs fixed
- 2026-10-18 — manual round in the real Tauri window, the 14 GB archive + two sample mboxes: several files open at
  once, one under the other in the panel, one active; switching, search, attachments, closing, reload after a
  restart — passed. Done-when met

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
