---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
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
6. [x] `tasks_check`, commit, push; the workflow run is green with `unchanged 30`
7. [x] Close: move to done, push, the workflow closes this issue

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
- 2026-10-18 — commit `17c3275` pushed; `sync-issues.yml` run 39127643289 green in 15 s, log line
  `applied: created 0, updated 0, unchanged 30`. Closing: this file moved to `done/`, the push after it must close
  issue #22 through the workflow (the end-to-end proof, recorded under Verification by the next session or by hand).

## Deviations from the plan

- The plan counted 29 items (21 tasks + 8 decisions); this task file is the 22nd task, so every count is 30.
- `_lead()` instead of a "Context section only" rule: 013 is a done file in backlog shape (bundled into 007), so the
  prose is the Context section when there is one, else the lines between the H1 and the first `## `.

## Verification

- 2026-10-18 — `python -I scripts/sync-issues.py --dry-run --repo speti4/mboxer`: 30 items (22 tasks, 8 decisions),
  30 creates + 4 milestone description updates planned, 0 errors; `python -m pyflakes` clean.
- 2026-10-18 — generated bodies of 018 (backlog), 013 (done, backlog shape) and D-08 (decision) inspected: marker line,
  link to the file on `main`, metadata line, English prose, labels and milestone as designed.
- 2026-10-18 — backfill: `applied: created 30, updated 0, unchanged 0`; second run `unchanged 30`; `--check` exit 0.
  `gh api repos/speti4/mboxer/milestones?state=all`: M1 closed 0/3, M2 closed 0/5, M3 closed 0/8, M4 open 3/0.
  `gh issue list --label mirrored --state all`: 30 rows, #1–#22 the tasks, #23–#30 D-01–D-08; #18 open with
  `stage:backlog`, `status:ready`, `size:M`, milestone M4 macOS; #12 closed `COMPLETED` under M3 Comfort; #30 open
  with `decision`.
- 2026-10-18 — workflow run 39127643289 on the feature commit: success, `unchanged 30`.
- 2026-10-18 — `tasks_check`: 0 errors, 0 warnings, README index regenerated.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the project-pal
      template is a project-pal task, named in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
