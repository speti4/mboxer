---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
plan: local-repoban-profi-a-clever-pie.md
---

# 022 — One-way sync of tasks/ to GitHub issues and milestones

## Context

Task 021 put the roadmap milestones on GitHub, but a milestone without issues is an empty progress bar.
What: `scripts/sync-issues.py` mirrors every task file (`backlog/ wip/ done/ dropped/`) and every decision
(`D-NN`) into a GitHub issue — title with the id, English prose from the Context section, metadata line, link
to the file, milestone from the frontmatter, labels for stage / status / size, open or closed by the folder —
and keeps the milestones in step with `tasks/roadmap.md`; `.github/workflows/sync-issues.yml` runs it on every
push to `main` that touches `tasks/`. One-way and idempotent: GitHub is never edited by hand, a no-op run
changes nothing. Why: the milestone progress bars become real, and the backlog is visible to collaborators
without cloning, while the task files stay the only source of truth. User decisions 2026-10-18: variant
"one-way sync from CI" over a one-shot backfill or GitHub-as-backlog; decisions mirrored with a `decision`
label; issue bodies in English only (the 16 Hungarian `## Context` sections of `done/` files are translated
once, Log / Verification untouched); the script lives in this repo, a project-pal template later.
Done when: 29 issues exist with the right state, milestone and labels, a second run reports `unchanged 29`,
the workflow is green, and closing this task closes its own issue through CI.

## Steps

1. [x] `scripts/sync-issues.py` (stdlib + `gh api`): parser, roadmap → milestones, labels, diff, `--dry-run` / `--check`
2. [x] Dry run: 22 tasks + 8 decisions parsed (this file included), 30 creates planned, 0 errors
3. [x] Translate the `## Context` of the 16 Hungarian `done/` files (one-time exception to "done is never rewritten")
4. [x] Backfill from the dev machine; second run `unchanged 30`; milestone counters 0/3, 0/5, 0/8, 3/0
5. [x] `.github/workflows/sync-issues.yml`; docs (CLAUDE.md convention, README, tasks/README preamble, roadmap, D-08)
6. [ ] `tasks_check`, commit, push; the workflow run is green with `unchanged 30`
7. [ ] Close: move to done, push, the workflow closes this issue

## Log

- 2026-10-18 — plan approved; 022 opened straight in wip/
- 2026-10-18 — `scripts/sync-issues.py` written (stdlib + `gh api` through subprocess, ~330 lines): the flat frontmatter
  subset of the standard, `_lead()` = the `## Context` section or, when a file has none (013 is a done file in backlog
  shape), the prose between the H1 and the first `## `; decisions mirror their whole body; marker
  `<!-- sync-issues: task 021 -->` is the lookup key; only differing fields are PATCHed; `--dry-run` / `--check`.
  First dry run found two things: 013 had an empty body (fixed with `_lead`), and `python -I` ignores
  `PYTHONIOENCODING`, so the script reconfigures stdout to UTF-8 itself (the titles carry em dashes).
- 2026-10-18 — the 16 Hungarian `## Context` sections translated by a subagent, Edit tool only, LF kept, every diff hunk
  inside the Context range (checked mechanically against the hunk headers); Log / Verification untouched.
- 2026-10-18 — backfill from the dev machine: `applied: created 30, updated 0, unchanged 0`, the 4 milestone descriptions
  rewritten from the roadmap paragraphs; second run `created 0, updated 0, unchanged 30`; `--check` exits 0.
  Issue numbers happen to equal the task ids (#1–#22) and D-01–D-08 are #23–#30 (the repo had no issues or PRs).
- 2026-10-18 — follow-up not filed here: lifting the script into project-pal as a `/tasks init` template is a project-pal
  task, to be opened in that repo when this has run for a while (user decision 2026-10-18: script lives here first).

## Deviations from the plan

- none yet

## Verification

- pending

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
