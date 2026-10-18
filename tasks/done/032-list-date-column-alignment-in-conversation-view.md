---
size: S
origin: wip/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
---

# 032 — Date column misaligned between conversation rows and plain rows in the list

## Context

What: in the conversation view, a plain row (a thread with one message) ends its subject early and puts the date
right after it, while a conversation row puts the count badge and the date at the right edge, so the dates do not
line up (visible in `docs/images/viewer.png`). Give plain rows the same badge slot, or right-align the date column
for both. Why: the list is the most-read part of the window and a ragged date column looks broken.
Done when: dates align in one column in both views and in both themes.
Run as a task session under docs/git-workflow.md, in parallel with 035 and 025; checked over synthetic data only.

## Steps

1. [x] Find the row markup and CSS of the list (`crates/mboxer-app/ui/`) for plain and conversation rows
2. [x] One date column for both row kinds, in the conversation view and the flat view
3. [x] Check in Chrome over the `ui/` folder with a stub `__TAURI__` backend and synthetic rows: both views, both
   themes, a narrow and a wide window; node tests and `cargo test` green
4. [x] Pull request, CI green

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/032-list-date-column-alignment` on
  branch `task/032-list-date-column-alignment`
- 2026-10-18 — ok: a `.row` grid hat sávos (`14px 20px 180px 1fr auto 90px`); a plain sor üres `.count`-ja
  `display: none`, kiesik a gridből, és az auto-placement a dátumot a badge `auto` sávjába tolta (a 90px-es
  dátumsáv üres maradt) — a flat nézetben minden sorral ez történt. Javítás: `.row .date { grid-column: -2; }`.
  Az „üres badge-hely a plain sorban" (`visibility: hidden`) változat helyett ez, mert a grid már deklarál egy
  fix, jobbra igazított dátumsávot, csak az elhelyezés volt rossz, és a plain sor tárgya megkapja a badge helyét.
- 2026-10-18 — a keskeny ablak / keskeny lista próbája egy második okot hozott: a splitter 120px-ig engedi a
  listát, ~400px alatt a fix sávok szélesebbek a listánál, a sor kilóg, a dátum badge-szélességtől függően tolódik
  és levágódik. A feladó sávja `minmax(0, min(180px, 40%))` lett; három változatot mértem (`minmax(0, 180px)`:
  360px-nél a tárgy 0px; `minmax(80px, 1fr)` tárgy: 260px-nél újra kilóg; a választott: 360px-nél feladó 138px,
  tárgy 34px; 200px-ig egy oszlop, csak a splitter 120px-es minimumánál térnek el újra a dátumok, T 339 / P 321
  / M 335). Alapértelmezett 480px-nél változatlan.
- 2026-10-18 — PR #53, CI zöld; task zárva a branchben.

## Deviations from the plan

- A narrow list overflowed the row grid, a second cause of ragged dates found during the narrow-window check; the
  sender track now shrinks (`minmax(0, min(180px, 40%))`). No JS changed, so no Node test was added.

## Verification

- 2026-10-18 — Chrome, a worktree `ui/` mappája egy scratchpad Node szerverről (127.0.0.1:8772) stub
  `__TAURI__` backenddel: 14 szintetikus szál `example.com` címekkel (1, 2, 3, 5, 12, 128 üzenetes; rövid és
  hosszú tárgyak), egy kinyitott beszélgetés (member sorok), `/before/` alatt a branch első commitjának
  `style.css`-e. Mérés `javascript_tool`-lal: minden látható sor dátumszövegének jobb széle (Range rect),
  soronként csoportosítva (T thread, P plain, M member), L a lista jobb széle.
- Előtte (804px széles viewport, conversation nézet): 480px listánál T 682, P/M 586 (96px eltérés); 360px-nél
  T 585 > L 570 (levágva), P 537, M 551; 260px-nél minden dátum a listán kívül. Flat nézet 480px-nél P 586
  (L 690), az üres 90px-es sáv mellett.
- Utána: 804px és 1884px széles viewport, conversation és flat nézet, 600/480/360/260px lista — minden esetben
  egyetlen érték soronként: pl. 804px, conv 480 → T/P/M 682, L 690; conv 360 → 562 / L 570; conv 260 → 462 /
  L 470; flat 480 → P 682 / L 690; 1884px, conv 600 → 817 / L 825. A dátum mindig a sor 8px-es paddingje
  előtt végződik. Light (`rgb(255, 255, 255)` háttér) és Dark téma: ugyanazok a számok; képernyőképek mindkét
  témában, mindkét nézetben, keskeny és széles ablakban megnézve.
- `cargo fmt --check` ok, `cargo clippy --all-targets -- -D warnings` ok, `cargo test`: 109 Rust teszt zöld
  (app 35, ui_query 1, cli 4 + 6, core 31, fixtures 10, index 22); a Node UI tesztek 30/30.
- CI run 39154494811 (PR #53, commit a8e1e64): commit-rules, deny, test, workflow-lint zöld.

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: 2026-10-18`
