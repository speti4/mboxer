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
with a fresh index are reloaded. User decision 2026-10-18: no merged view (reading (a) of the backlog
dropped), the files appear one under the other, one active at a time. Planning decisions (approved
plan): backend `OpenFiles` set + active, UI key `(file id, offset, len)`, the file id an in-process
counter (D-006, proposed); Open… multi-select and adds; the search text is kept per file; `mboxer.files`
+ `mboxer.activePath` in localStorage, at start-up only the files with a fresh index are reloaded, the ones with a
stale index wait as a faded root; no new crate (D-002).

## Steps

1. [x] `src/state.rs`: `OpenFiles` (insert/dedupe, activate, close, infos), `OpenMbox.id` + `canonical`; unit tesztek
2. [x] `src/types.rs`: `FileInfo`, `OpenSummary.id/files`, `OpenEvent::Opening{position,count}` / `Failed{path}` / `Done`, `IndexStatusInfo`, `MessageView.file`
3. [x] `src/commands.rs` + `lib.rs`: `open_mbox(paths, activate)` batch + multi-pick, `activate_file`, `close_file`, `index_status`; `file` param minden fájl-commandon
4. [x] `src/attachments.rs`: temp mappa `{file}-{offset}-{index}`
5. [x] `ui/files.js` (új) + `tests/ui/files.test.js`, `tests/ui_query.rs` futtatja
6. [x] `ui/index.html`: `files.js`, `#reopen-list`, Open gomb title
7. [x] `ui/main.js`: `state.files/remembered/fileQueries`, batch open események, `activateFile`, root sorok a `renderFolders`-ben, `×` bezárás, `file` minden invoke-ban, `start()` visszatöltés
8. [x] `ui/style.css`: `.folder.root`, `.root.unloaded`, `.close-file`
9. [x] `docs/usage.md` "Several files" bekezdés, `README.md` egy sor
10. [x] `/tasks decide` → D-006 (proposed)
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --test`; Chrome stub-check; `tasks_check`; commit
12. [x] User kézi kör: a 14 GB archívum + egy második mbox az igazi Tauri ablakban

## Log

- 2026-10-18 — plan approved; 016 promoted to wip; D-006 (proposed) felvéve
- 2026-10-18 — implementálva: backend `OpenFiles` set (per-process `file` id), batch `open_mbox` (`Opening{position,count}`,
  `Failed{path}`, `Done`), `activate_file` / `close_file` / `index_status`; UI: root sorok a folder panelen, egyszerre egy
  kinyitva, keresőszöveg fájlonként megjegyezve, `files.js` (`mboxer.files` + `lastPath` migráció), indulási visszatöltés
  csak friss indexre, stale → halvány root. `cargo test`: 35 app (4 új `OpenFiles` teszt) + 1 UI-node + core/CLI zöld;
  `node --test` 30 pass (16 query + 6 theme + 8 files); `cargo fmt --check`, `clippy --all-targets -D warnings` tiszta
- 2026-10-18 — Chrome stub-check (scratchpad `serve-ui-files.js` + `stub-files.js`, 127.0.0.1:8768; A friss, B stale,
  C hiányzó, a "dialógus" B+D-t ad): induláskor A betöltve+aktív, B halvány root, C kiesett a tárolóból; B-re kattintva
  épül és aktív lesz; keresés B-ben `file: 2`-vel megy, `read_message` is a fájl-id-t viszi; A-ra váltva üres kereső,
  vissza B-re a "foo" visszajön; ArrowUp az All mail-ről az aktív rootot átugorva A-ra vált; × A-n: B marad aktív, a
  tároló frissül; Open… (B újra + D): B megtartja a lekérdezését, D nem aktív root; × az aktív B-n: D veszi át; az
  utolsó × után empty nézet, kereső letiltva, tároló üres. Két hibát a check talált és javítottam: az aktív fájl
  újranyitása törölte a keresőszöveget; a billentyűs navigáció az aktív rooton ragadt. Képernyőkép dark + light: rendben
- 2026-10-18 — commit `cba85f3` (feat, task 016)
- 2026-10-18 — a usernek egy mbox-a van, ezért két szintetikus minta (`target/manual-round/sample-threads.mbox`,
  `sample-attachments.mbox`, a core fixture-ökből) állt a második és harmadik fájl szerepébe; a debug exe a 14 GB
  archívummal indult. A user kézi köre (multi-select Open…, váltás és keresés fájlonként, melléklet mindkét fájlból,
  bezárás nem aktív és aktív soron, újraindítás utáni visszatöltés, stale index halvány rootként): "minden ok". Nincs
  follow-up; D-006 `decided`; closed

## Deviations from the plan

- `state.known` helyett a UI a `state.remembered` (`{paths, active}`, a files.js értéke) listát rajzolja: a panel sorrendje
  a megjegyzett sorrend, a betöltött fájlok `FileInfo`-ját path szerint párosítja hozzá — egy helyen van az igazság
- Megadott path-ok sorrendje megmarad (az aktiválandó elöl), csak a dialógus kijelölése rendeződik név szerint
- A váltás után mindig fut egy `set_query` (a visszaállított lekérdezéssel és a mai Conversations móddal), nem bízunk a
  backend régi listingjében; friss megnyitás után nem

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass (35 app,
  köztük 4 új `OpenFiles` teszt; UI-node runner 3 fájl, 30 pass). Chrome stub-check a Log szerint, 2 javított hibával
- 2026-10-18 — User kézi kör az igazi Tauri ablakban, a 14 GB archívum + két minta mbox: több fájl egyszerre nyitva,
  egymás alatt a panelen, egy aktív; váltás, keresés, mellékletek, bezárás, újraindítás utáni visszatöltés — "minden ok".
  Done-when met

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
