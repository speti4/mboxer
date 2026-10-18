---
size: S
origin: wip/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-18
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

1. [ ] Find the row markup and CSS of the list (`crates/mboxer-app/ui/`) for plain and conversation rows
2. [ ] One date column for both row kinds, in the conversation view and the flat view
3. [ ] Check in Chrome over the `ui/` folder with a stub `__TAURI__` backend and synthetic rows: both views, both
   themes, a narrow and a wide window; node tests and `cargo test` green
4. [ ] Pull request, CI green

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/032-list-date-column-alignment` on
  branch `task/032-list-date-column-alignment`

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the main session, before the merge)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
