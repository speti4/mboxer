---
size: S
origin: done/025-release-notes-changelog-and-checksums.md
created: 2026-10-18
started: 2026-10-19
plan: n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M5 Public release
closed: 2026-10-19
changelog: housekeeping
---

# 038 — README links the changelog next to the releases

## Context

What: the README's download step and its docs list point to the releases but not to `CHANGELOG.md`, which 025 added
(D-012); add the link, and a word on checking the zip against `SHA256SUMS.txt` where the download step is.
Why: a visitor reading the front page should find what changed and how to verify the download without the docs.
Done when: the README links `CHANGELOG.md` and names the checksum file at the download step.
Done in parallel with 036, 018 and 019.

## Steps

1. [x] `README.md`: the download step links `CHANGELOG.md` and says how to check the zip against `SHA256SUMS.txt`
2. [x] `README.md`: the docs list gains the changelog entry
3. [x] Pull request, CI green

## Log

- 2026-10-19 — promoted in the main session, branch `task/038-readme-changelog-link`
- 2026-10-19 — task session in the `.claude/worktrees/038-readme-changelog-link` worktree: the README's download
  step points to `CHANGELOG.md` and gives the `Get-FileHash` / `SHA256SUMS.txt` comparison in one sentence (the same
  one-liner `docs/usage.md` has, without a code block, since the step list uses none); the docs list's new line is
  the changelog and the releases page; closed on the branch

## Deviations from the plan

- none

## Verification

- 2026-10-19 — every relative link of `README.md` (14, the new `CHANGELOG.md` included) points to an existing file
  or folder (a shell loop with `[ -e ]` from the worktree root); the only lines over 112 characters are the three
  existing badge / image lines; the `Get-FileHash` sentence gives the same command as line 20 of `docs/usage.md`
- 2026-10-19 — `tasks_check` on the worktree: only the README index "stale" report (the main session regenerates it
  before the merge)

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
