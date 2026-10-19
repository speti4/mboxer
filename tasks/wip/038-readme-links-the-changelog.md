---
size: S
origin: done/025-release-notes-changelog-and-checksums.md
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M5 Public release
---

# 038 — README links the changelog next to the releases

## Context

What: the README's download step and its docs list point to the releases but not to `CHANGELOG.md`, which 025 added
(D-012); add the link, and a word on checking the zip against `SHA256SUMS.txt` where the download step is.
Why: a visitor reading the front page should find what changed and how to verify the download without the docs.
Done when: the README links `CHANGELOG.md` and names the checksum file at the download step.
Run as a task session under docs/git-workflow.md, in parallel with 036, 018 and 019 (plan 2026-10-19).

## Steps

1. [ ] `README.md`: the download step links `CHANGELOG.md` and says how to check the zip against `SHA256SUMS.txt`
2. [ ] `README.md`: the docs list gains the changelog entry
3. [ ] Pull request, CI green

## Log

- 2026-10-19 — promoted in the main session, branch `task/038-readme-changelog-link`

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
