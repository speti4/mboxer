---
size: S
origin: done/003-fulltext-sidecar-index-accent-folding.md
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: none
---

# 014 — Index fallback when the mbox folder is not writable

## Context

D-004 puts the index next to the mbox (`.mboxer/<name>/`); if that folder is not writable (DVD, read-only share,
forbidden folder), the build stops with an error today. 014: if the primary `.building` directory cannot be created,
the index is built under `%LOCALAPPDATA%\mboxer\index\<file name>-<path hash>\`, and the open looks there too.
Why: a user's archive may sit on a network share. Done when: `mboxer index` succeeds on an mbox in a
read-only folder, `mboxer search` finds the fallback index, and the test covers both locations.
Planning decision 2026-10-17: the fallback directory is keyed by the mbox's *path* (not its content fingerprint,
as the backlog line said): one place = one directory, a new export to the same place replaces the old index, no
orphan remains; freshness is still `mboxer.json` (D-004). The trigger is any I/O error while preparing the primary
`.building` (not a permission probe: on Windows the read-only folder attribute does not forbid creation, a deny
ACL, a read-only share or a *file* named `.mboxer` does — the test provokes it with the latter). D-005 proposed.

## Steps

1. [x] `fingerprint.rs`: `INDEX_HOME_ENV`, `default_index_home`, `fallback_dir`, `resolve`, `status_in`; unit tests
2. [x] `build.rs`: `BuildOptions.home`, primary → fallback, cleanup of the other location, `BuildReport.fallback`
3. [x] `search.rs`: `MboxIndex::open_in`; `mod.rs` / `lib.rs` exports
4. [x] CLI: `index` prints the fallback; `--help` mentions `MBOXER_INDEX_HOME`
5. [x] App: `OpenSummary.index_dir`, status bar tooltip
6. [x] Tests: `tests/index.rs` (blocked sidecar → fallback; primary wins; one index remains), `tests/cli.rs`
7. [x] D-005 (`/tasks decide`), README
8. [x] `cargo fmt`, clippy, `cargo test`; manual: fixture copy in a blocked folder with the CLI and the viewer
9. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 014 promoted to wip
- 2026-10-17 — core: `fallback_dir` (file name + FNV-1a 64 over the canonical absolute path, 16 hex), `resolve`
  (fresh primary > fresh fallback > stale primary > stale fallback > Missing), `status`/`open` →
  `status_in`/`open_in` with the default home; `build` switches to the fallback on an error while preparing the
  primary `.building`, and removes the unused location best-effort; `BuildReport.fallback`. A non-exhaustive
  `match` fixed (a `Fresh` primary already returns earlier). CLI `index:` line with a `(fallback: …)` suffix; the
  `index` subcommand's `after_help` names both locations and `MBOXER_INDEX_HOME` (the struct doc comment did not
  show up in the help, hence a constant). App: `OpenSummary.indexDir`, the status bar tooltip `index: <dir>`
- 2026-10-17 — tests green on the first run: 30 app + 1 + 4 + 6 CLI + 30 + 9 + 22 core; clippy, fmt clean
- 2026-10-17 — manual round: a `threads.mbox` copy under `%TEMP%\mboxer-014-manual\` blocked by a `.mboxer`
  *file* → debug CLI `index:` `%LOCALAPPDATA%\mboxer\index\threads.mbox-40f8a8f2b62873aa (fallback: …)`,
  6 messages; `search --label Work --count` 2 hits, open 8.5 ms; the debug viewer on the same copy:
  `opened threads.mbox: 6 messages, Reused, 8 ms` (reused the fallback index). Both folders deleted
- 2026-10-17 — D-005 proposed; closed

## Deviations from the plan

- none: the plan's API (`status_in`, `open_in`, `BuildOptions.home`, `BuildReport.fallback`) landed as written.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  (`summary.index_dir` = `index_dir(&mbox)`) + 1 ui_query + 4 + 6 CLI (new:
  `a_blocked_mbox_folder_indexes_into_the_home_and_search_finds_it` — builds under `MBOXER_INDEX_HOME`, `search`
  1 hit, exit 1 without a home) + 30 core unit (new: `fallback_dir_is_keyed_by_the_mbox_path`) + 9 fixtures +
  22 index (new: `a_blocked_sidecar_falls_back_to_the_index_home` — fallback build, `status_in` Fresh, `open_in`
  Ready, search finds, another home Missing, stale → in-place rebuild; `the_primary_location_wins_when_it_is_writable`
  — the primary is built, the home stays empty; when blocked, the rebuild goes to the home and the stale primary is
  deleted; in a writable folder again, the build returns there and the fallback directory disappears) pass.
- 2026-10-17 — manual round (see Log): in a blocked folder the CLI builds under `%LOCALAPPDATA%\mboxer\index\` and
  `search` finds it; the viewer reuses the same index. Done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
