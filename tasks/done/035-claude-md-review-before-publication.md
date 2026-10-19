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

What: drop the "User communication: Hungarian" line (a personal preference, not a repo rule; a contributor's AI assistant
would answer in Hungarian); one rule for the dependency justification (task file vs `Cargo.toml` comment, today both,
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
- 2026-10-18 — CLAUDE.md átírva (Layout, Commands, Data, Conventions, Tasks), PR #52, CI zöld; lezárva a branchen
- 2026-10-18 — main session a merge előtt: a záró commit csak a `git mv`-t vitte, a fájl szerkesztései (Steps,
  Log, Verification, `closed`) commitolatlanul maradtak a worktree-ben — most commitolva; a két follow-up
  (D-010 hivatkozás, `deps:` a CONTRIBUTING-ban, plusz a várt index-hiba) → `backlog/036`; index újragenerálva.

## Deviations from the plan

- none yet

## Verification

- 2026-10-18 — CLAUDE.md előtte: Scope (crate-ok + build/run/test + docs-leltár), Data, Conventions (7 sor),
  Tasks (1 hosszú sor); utána: Layout, Commands, Data (változatlan szabályok), Conventions (2 sor), Tasks (1 sor).
  Hová kerültek az elhagyott szabályok (mind megvolt már, új helyre nem kellett költöztetni):
  - "User communication: Hungarian" — személyes beállítás, nem repó-szabály; a repóba nem kell
  - Rust edition 2024 — `Cargo.toml` (`edition = "2024"`) és CONTRIBUTING.md Conventions
  - fmt + clippy lista — CONTRIBUTING.md "Checks before a commit" (cargo deny, zizmor is); CLAUDE.md oda mutat
  - English / kebab-case — CONTRIBUTING.md Conventions
  - függőség-indoklás — egy szabály: `Cargo.toml` komment (CONTRIBUTING.md "Dependencies: one exe", PR template);
    a "task fájlban" változat törölve, CLAUDE.md a CONTRIBUTING szakaszra mutat
  - docs/ leltár — docs/development.md Layout
  - "closing files every follow-up with /tasks add", "/tasks check before a task commit" — ellentmondott a
    docs/git-workflow.md-nek (id-t és follow-upot a main session ad); a standard a `tasks/README.md`-ben, a
    tasks_check tool leírásában
  - megtartva: `@docs/git-workflow.md` import, Data outside the repo, D-008 (H1 + Context angol), decision
    `proposed` amíg a user meg nem erősíti (máshol nem áll kimondva, ezért CLAUDE.md-ben maradt)
- 2026-10-18 — PR #52: CI run 39153999869 zöld (test, deny, workflow-lint, commit-rules); `scripts/check-commit.sh`
  a commit üzenetre és a PR body-ra exit 0; `tasks_check` (write nélkül) 0 hiba

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: 2026-10-18`
