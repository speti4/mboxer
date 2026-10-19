---
size: S
origin: wip/003-fulltext-sidecar-index-accent-folding.md
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: 12-14-15-taskokat-fejlessz-k-le-keen-whale.md
---

# 014 — Index fallback when the mbox folder is not writable

## Context

D-004 puts the index next to the mbox (`.mboxer/<név>/`); if that folder is not writable (DVD, read-only share,
forbidden folder), the build stops with an error today. 014: if the primary `.building` directory cannot be created,
the index is built under `%LOCALAPPDATA%\mboxer\index\<fájlnév>-<útvonal-hash>\`, and the open looks there too.
Why: a user's archive may sit on a network share. Done when: `mboxer index` succeeds on an mbox in a
read-only folder, `mboxer search` finds the fallback index, and the test covers both locations.
Planning decision 2026-10-17: the fallback directory is keyed by the mbox's *path* (not its content fingerprint,
as the backlog line said): one place = one directory, a new export to the same place replaces the old index, no
orphan remains; freshness is still `mboxer.json` (D-004). The trigger is any I/O error while preparing the primary
`.building` (not a permission probe: on Windows the read-only folder attribute does not forbid creation, a deny
ACL, a read-only share or a *file* named `.mboxer` does — the test provokes it with the latter). D-005 proposed.

## Steps

1. [x] `fingerprint.rs`: `INDEX_HOME_ENV`, `default_index_home`, `fallback_dir`, `resolve`, `status_in`; unit tesztek
2. [x] `build.rs`: `BuildOptions.home`, elsődleges → fallback, a másik hely takarítása, `BuildReport.fallback`
3. [x] `search.rs`: `MboxIndex::open_in`; `mod.rs` / `lib.rs` exportok
4. [x] CLI: `index` kiírja a fallbacket; `--help` említi `MBOXER_INDEX_HOME`-ot
5. [x] App: `OpenSummary.index_dir`, státuszsor tooltip
6. [x] Tesztek: `tests/index.rs` (blokkolt sidecar → fallback; elsődleges nyer; egy index marad), `tests/cli.rs`
7. [x] D-005 (`/tasks decide`), README
8. [x] `cargo fmt`, clippy, `cargo test`; kézi: fixture-másolat blokkolt mappában CLI-vel és a viewerrel
9. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 014 promoted to wip
- 2026-10-17 — core: `fallback_dir` (fájlnév + FNV-1a 64 a kanonikus abszolút útvonalon, 16 hex), `resolve`
  (friss elsődleges > friss fallback > stale elsődleges > stale fallback > Missing), `status`/`open` →
  `status_in`/`open_in` alapértelmezett home-mal; `build` az elsődleges `.building` előkészítésének hibájára
  vált a fallbackre, a nem használt helyet best-effort törli; `BuildReport.fallback`. Egy nem kimerítő `match`
  javítva (a `Fresh` elsődleges már korábban visszatér). CLI `index:` sor `(fallback: …)` utótaggal, az `index`
  subcommand `after_help`-je a két helyet és `MBOXER_INDEX_HOME`-ot írja (a struct doc-komment nem jelent meg a
  help-ben, ezért konstans). App: `OpenSummary.indexDir`, a státuszsor tooltipje `index: <dir>`
- 2026-10-17 — tesztek elsőre zöldek: 30 app + 1 + 4 + 6 CLI + 30 + 9 + 22 core; clippy, fmt tiszta
- 2026-10-17 — kézi kör: `threads.mbox` másolat `%TEMP%\mboxer-014-manual\` alatt egy `.mboxer` *fájllal*
  blokkolva → debug CLI `index:` `%LOCALAPPDATA%\mboxer\index\threads.mbox-40f8a8f2b62873aa (fallback: …)`,
  6 üzenet; `search --label Work --count` 2 találat, open 8,5 ms; a debug viewer ugyanazon a másolaton:
  `opened threads.mbox: 6 messages, Reused, 8 ms` (a fallback indexet újrahasználta). Mindkét mappa törölve
- 2026-10-17 — D-005 proposed; closed

## Deviations from the plan

- none: the plan's API (`status_in`, `open_in`, `BuildOptions.home`, `BuildReport.fallback`) landed as written.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  (`summary.index_dir` = `index_dir(&mbox)`) + 1 ui_query + 4 + 6 CLI (új:
  `a_blocked_mbox_folder_indexes_into_the_home_and_search_finds_it` — `MBOXER_INDEX_HOME` alá épít, a `search`
  1 találat, home nélkül exit 1) + 30 core unit (új: `fallback_dir_is_keyed_by_the_mbox_path`) + 9 fixtures +
  22 index (új: `a_blocked_sidecar_falls_back_to_the_index_home` — fallback build, `status_in` Fresh, `open_in`
  Ready, keresés talál, más home Missing, stale → in-place rebuild; `the_primary_location_wins_when_it_is_writable`
  — elsődleges épül, a home üres; blokkolva a rebuild a home-ba megy és a stale elsődleges törlődik; újra írható
  mappában az építés visszatér és a fallback könyvtár eltűnik) pass.
- 2026-10-17 — kézi kör (lásd Log): blokkolt mappában a CLI a `%LOCALAPPDATA%\mboxer\index\` alá épít és a `search`
  megtalálja; a viewer ugyanazt az indexet újrahasználja. Done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
