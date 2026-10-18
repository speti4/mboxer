---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
plan: none
milestone: M5 Public release
---

# 035 — CLAUDE.md fit for a public repo: no personal settings, no contradictions, a commands block

## Context

What: drop the "User communication: Hungarian" line (it belongs in the author's global file; a contributor's session
would answer in Hungarian); one rule for the dependency justification (task file vs `Cargo.toml` comment, today both,
CONTRIBUTING says the comment); point to CONTRIBUTING's checks instead of a stale list (cargo deny and zizmor since
024); cut what the repo already says (edition, the doubled English / kebab-case line); a short Commands block out of
the Scope paragraph; the Tasks section as one pointer line.
Why: review 2026-10-18 against the current memory / best-practices docs (keep it lean, no contradictions, no
frequently changing facts); the file is public from 028 on.
Done when: CLAUDE.md has no personal or contradicting line and a fresh session finds build, test and checks in it.
Run as a task session under docs/git-workflow.md, in parallel with 025 and 032.

## Steps

1. [ ] CLAUDE.md: the cuts and the Commands block listed in Context; the `@docs/git-workflow.md` import stays
2. [ ] The dependency justification rule: the `Cargo.toml` comment (as CONTRIBUTING says), CLAUDE.md points there
3. [ ] Every rule CLAUDE.md drops is still stated somewhere the repo keeps (CONTRIBUTING, docs, decisions)
4. [ ] Pull request, CI green

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/035-claude-md-review` on branch
  `task/035-claude-md-review`

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the main session, before the merge)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
