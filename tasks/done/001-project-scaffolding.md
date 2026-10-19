---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
plan: olvasd-el-a-draft-md-sharded-teapot.md
changelog: housekeeping
---

# 001 — Project scaffolding: repo structure and task standard

## Context

Greenfield repo; `draft.md` held the problem statement (a 14 GB Gmail Takeout mbox that needs a fast body
search). Planning on 2026-10-17 settled the frame: build instead of buy (D-001), Rust + Tauri 2 + sidecar
full-text index (D-002). This task lays down the repo structure and the task standard; no app code.

## Steps

1. [x] Project registered in the maintainer's planning tool
2. [x] `docs/origin.md` (ex draft.md), `docs/research/2026-10-17-free-mbox-viewer-alternatives.md`
3. [x] `tasks/` standard: roadmap, D-001, D-002, backlog 002–012, generated README
4. [x] `README.md`, `CLAUDE.md`, `.gitignore`
5. [x] `tasks_check` green, first commit on `main`

## Log

- 2026-10-17 — plan approved; folders created, `draft.md` moved to `docs/origin.md`
- 2026-10-17 — a Bash heredoc script for the repo files failed to parse (and would have halved the
  backslashes in the Windows plan paths); all files written with the Write tool instead
- 2026-10-17 — project registered in the maintainer's planning tool; its validation: 455 files
  strict-valid, 0 warnings
- 2026-10-17 — `tasks_check` (write): 11 backlog · 1 wip · 2 decisions; 0 errors, 0 warnings;
  README index written

## Deviations from the plan

- Files written via Write instead of Bash heredocs (tooling, not content).

## Verification

- 2026-10-17 — planning-tool validation: 455 strict-valid, 0 failing, 0 warnings.
- 2026-10-17 — `tasks_check`: 0 errors, 0 warnings, README up to date,
  hub checked; 11 backlog items (7 ready, 4 idea) across the three roadmap milestones, 2 decisions.
- 2026-10-17 — `ls tasks/` = `README.md roadmap.md backlog wip done dropped decisions`;
  `draft.md` gone from the root, present as `docs/origin.md`.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none arose beyond backlog 002–012
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
