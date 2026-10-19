---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
milestone: M5 Public release
---

# 035 — CLAUDE.md fit for a public repo: no personal settings, no contradictions, a commands block

## Context

What: drop the line that set the assistant's chat language (a personal preference, not a repo rule; a contributor's AI
assistant would follow it); one rule for the dependency justification (task file vs `Cargo.toml` comment, today both,
CONTRIBUTING says the comment); point to CONTRIBUTING's checks instead of a stale list (cargo deny and zizmor since
024); cut what the repo already says (edition, the doubled English / kebab-case line); a short Commands block out of
the Scope paragraph; the Tasks section as one pointer line.
Why: review 2026-10-18 against Claude Code's CLAUDE.md guidance (keep it lean, no contradictions, no
frequently changing facts); the file is public from 028 on.
Done when: CLAUDE.md has no personal or contradicting line and a new AI assistant session finds build, test and checks in it.
Done in parallel with 025 and 032.

## Steps

1. [x] CLAUDE.md: the cuts and the Commands block listed in Context; the `@docs/git-workflow.md` import stays
2. [x] The dependency justification rule: the `Cargo.toml` comment (as CONTRIBUTING says), CLAUDE.md points there
3. [x] Every rule CLAUDE.md drops is still stated somewhere the repo keeps (CONTRIBUTING, docs, decisions)
4. [x] Pull request, CI green

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/035-claude-md-review` on branch
  `task/035-claude-md-review`
- 2026-10-18 — CLAUDE.md rewritten (Layout, Commands, Data, Conventions, Tasks), PR #52, CI green; closed on the
  branch
- 2026-10-18 — main session before the merge: the closing commit carried only the `git mv`, the file's edits
  (Steps, Log, Verification, `closed`) stayed uncommitted in the worktree — committed now; the two follow-ups
  (the D-010 reference, `deps:` in CONTRIBUTING, plus the expected index error) → `backlog/036`; index regenerated.

## Deviations from the plan

- none yet

## Verification

- 2026-10-18 — CLAUDE.md before: Scope (crates + build/run/test + docs inventory), Data, Conventions (7 lines),
  Tasks (1 long line); after: Layout, Commands, Data (rules unchanged), Conventions (2 lines), Tasks (1 line).
  Where the dropped rules live (all were already stated elsewhere, nothing had to move):
  - the chat-language line — a personal setting, not a repo rule; it does not belong in the repo
  - Rust edition 2024 — `Cargo.toml` (`edition = "2024"`) and CONTRIBUTING.md Conventions
  - the fmt + clippy list — CONTRIBUTING.md "Checks before a commit" (cargo deny, zizmor too); CLAUDE.md points
    there
  - English / kebab-case — CONTRIBUTING.md Conventions
  - dependency justification — one rule: the `Cargo.toml` comment (CONTRIBUTING.md "Dependencies: one exe", PR
    template); the "in the task file" variant removed, CLAUDE.md points to the CONTRIBUTING section
  - the docs/ inventory — docs/development.md Layout
  - "closing files every follow-up with /tasks add", "/tasks check before a task commit" — contradicted
    docs/git-workflow.md (ids and follow-ups come from the main session); the standard is in `tasks/README.md` and
    the tasks_check tool's description
  - kept: the `@docs/git-workflow.md` import, Data outside the repo, D-008 (H1 + Context in English), a decision
    stays `proposed` until it is confirmed (not stated anywhere else, so it stays in CLAUDE.md)
- 2026-10-18 — PR #52: CI run 39153999869 green (test, deny, workflow-lint, commit-rules); `scripts/check-commit.sh`
  exit 0 on the commit message and the PR body; `tasks_check` (without write) 0 errors

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: 2026-10-18`
