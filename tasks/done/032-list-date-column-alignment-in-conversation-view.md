---
size: S
origin: done/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
milestone: M5 Public release
---

# 032 — Date column misaligned between conversation rows and plain rows in the list

## Context

What: in the conversation view, a plain row (a thread with one message) ends its subject early and puts the date
right after it, while a conversation row puts the count badge and the date at the right edge, so the dates do not
line up (visible in `docs/images/viewer.png`). Give plain rows the same badge slot, or right-align the date column
for both. Why: the list is the most-read part of the window and a ragged date column looks broken.
Done when: dates align in one column in both views and in both themes.
Done in parallel with 035 and 025; checked over synthetic data only.

## Steps

1. [x] Find the row markup and CSS of the list (`crates/mboxer-app/ui/`) for plain and conversation rows
2. [x] One date column for both row kinds, in the conversation view and the flat view
3. [x] Check in Chrome over the `ui/` folder with a stub `__TAURI__` backend and synthetic rows: both views, both
   themes, a narrow and a wide window; node tests and `cargo test` green
4. [x] Pull request, CI green

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/032-list-date-column-alignment` on
  branch `task/032-list-date-column-alignment`
- 2026-10-18 — cause: the `.row` grid has six tracks (`14px 20px 180px 1fr auto 90px`); a plain row's empty
  `.count` is `display: none`, drops out of the grid, and auto-placement pushed the date into the badge's `auto`
  track (the 90px date track stayed empty) — in the flat view this happened to every row. Fix:
  `.row .date { grid-column: -2; }`. Chosen over an "empty badge slot in the plain row" (`visibility: hidden`),
  because the grid already declares a fixed, right-aligned date track, only the placement was wrong, and the plain
  row's subject gets the badge's room.
- 2026-10-18 — the narrow window / narrow list check found a second cause: the splitter lets the list shrink to
  120px; below ~400px the fixed tracks are wider than the list, the row overflows, and the date shifts with the
  badge width and is clipped. The sender track became `minmax(0, min(180px, 40%))`; three variants measured
  (`minmax(0, 180px)`: subject 0px at 360px; `minmax(80px, 1fr)` subject: overflows again at 260px; the chosen one:
  sender 138px, subject 34px at 360px; one column down to 200px, the dates differ again only at the splitter's
  120px minimum, T 339 / P 321 / M 335). Unchanged at the default 480px.
- 2026-10-18 — PR #53, CI green; task closed on the branch.

## Deviations from the plan

- A narrow list overflowed the row grid, a second cause of ragged dates found during the narrow-window check; the
  sender track now shrinks (`minmax(0, min(180px, 40%))`). No JS changed, so no Node test was added.

## Verification

- 2026-10-18 — Chrome, the worktree's `ui/` folder served from a scratchpad Node server (127.0.0.1:8772) with a
  stub `__TAURI__` backend: 14 synthetic threads with `example.com` addresses (1, 2, 3, 5, 12, 128 messages; short
  and long subjects), one expanded conversation (member rows), the branch's first-commit `style.css` under
  `/before/`. Measured with `javascript_tool`: the right edge of every visible row's date text (Range rect),
  grouped by row kind (T thread, P plain, M member), L the list's right edge.
- Before (804px wide viewport, conversation view): at a 480px list T 682, P/M 586 (96px apart); at 360px
  T 585 > L 570 (clipped), P 537, M 551; at 260px every date outside the list. Flat view at 480px P 586 (L 690),
  next to the empty 90px track.
- After: 804px and 1884px wide viewports, conversation and flat views, 600/480/360/260px lists — one value per
  row kind in every case: e.g. 804px, conv 480 → T/P/M 682, L 690; conv 360 → 562 / L 570; conv 260 → 462 /
  L 470; flat 480 → P 682 / L 690; 1884px, conv 600 → 817 / L 825. The date always ends before the row's 8px
  padding. Light (`rgb(255, 255, 255)` background) and Dark theme: the same numbers; screenshots checked in both
  themes, both views, narrow and wide windows.
- `cargo fmt --check` ok, `cargo clippy --all-targets -- -D warnings` ok, `cargo test`: 109 Rust tests green
  (app 35, ui_query 1, cli 4 + 6, core 31, fixtures 10, index 22); the Node UI tests 30/30.
- CI run 39154494811 (PR #53, commit a8e1e64): commit-rules, deny, test, workflow-lint green.

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: 2026-10-18`
