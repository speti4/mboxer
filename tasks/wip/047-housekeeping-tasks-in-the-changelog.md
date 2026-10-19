---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M5 Public release
---

# 047 — Keep project housekeeping apart from user-facing changes in the changelog

## Context

What: `scripts/changelog.py` lists tasks such as the history rewrite, the CLAUDE.md review or the id padding next
to features; give them a group of their own ("Project housekeeping") or a frontmatter flag the generator honours.
Why: release notes are read by users, who look for what changed in the app; 0.2.0 is the first public release.
Done when: the next release's notes show the user-facing changes first and the housekeeping apart or not at all,
before the `v0.2.0` tag.

## Steps

1. [ ] the mechanism: a frontmatter field the generator reads (it must pass `tasks_check` and the issue mirror),
   or a grouping rule; the choice and its reason in the Log
2. [ ] `scripts/changelog.py`: the housekeeping group after the milestone groups (or left out of the notes)
3. [ ] the housekeeping tasks marked (done files only touch the frontmatter)
4. [ ] `CHANGELOG.md` regenerated, `--check` green, `--notes 0.2.0` read through
5. [ ] the docs that describe the changelog (`docs/development.md`, the script's docstring) updated

## Log

- 2026-10-19 — promoted

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
