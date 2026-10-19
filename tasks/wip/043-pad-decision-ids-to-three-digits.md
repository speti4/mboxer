---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
---

# 043 — Pad the decision ids to three digits before the repo goes public

## Context

What: `/tasks renumber` renames the ten two-digit decision files to three digits and rewrites the references inside
`tasks/`; the ~30 files outside it are fixed by hand (crate comments, docs, CLAUDE.md, README, CONTRIBUTING,
`scripts/check-commit.sh`, workflows). `scripts/sync-issues.py` keys the decision issues by their id
(`<!-- sync-issues: decision D-NN -->`), so it must match the old two-digit key to the new one instead of
opening ten new issues; `CHANGELOG.md` stays in line with `scripts/changelog.py --check`.
Why: mixed widths sort and read wrong; the public repo starts consistent (user decision 2026-10-19).
Done when: `tasks_check` has no narrow-id warning, a grep for two-digit decision ids finds only history that
must stay, and a sync run edits the ten decision issues in place.
Split (2026-10-19): the main session runs the renumber inside `tasks/` on this branch (the tool writes only in
the main checkout); the task session does everything outside `tasks/`.

## Steps

1. [ ] `tasks_renumber` with `apply` (main session): ten decision files renamed, the references in `tasks/`
   rewritten, the index regenerated
2. [ ] the references outside `tasks/` (the tool's `names` list and every bare two-digit decision id): code
   comments, docs, CLAUDE.md, README, CONTRIBUTING, scripts, workflows, the pull request template
3. [ ] `scripts/sync-issues.py`: an existing issue whose marker carries the two-digit key is matched to the
   three-digit one and edited in place (title, body, marker); a dry run shows ten edits and no creates
4. [ ] `CHANGELOG.md` agrees with `python -I scripts/changelog.py --check`
5. [ ] the CONTRIBUTING.md checks; `scripts/check-commit.sh` still rejects a trailer and a session link
6. [ ] after the merge: the sync run on `main` edits the ten decision issues, opens none

## Log

- 2026-10-19 — promoted on `task/043-pad-decision-ids-to-three-digits`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
