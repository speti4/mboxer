---
size: S
origin: done/035-claude-md-review-before-publication.md
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M5 Public release
---

# 036 — Git workflow doc: current decision reference, the deps type, the expected index failure

## Context

What: `docs/git-workflow.md` cites D-09 for the trailer rule, which D-10 superseded (its point 3 carries the rule);
CONTRIBUTING's commit types leave out `deps:`, which the workflow doc lists; the doc does not say that a task
session's `tasks_check` (without `write`) reports the stale index on its own close commit, which is expected; and
a task session must see `git status` clean before it reports (035's close commit left the file's edits behind);
the main session prepares a pull request only after the previous merge is pulled, because `tasks_next_id` sees
only the checked-out tree and an id filed on an unmerged branch would be handed out twice (032 waited for #52).
Why: found by the 035 and 032 task sessions (PRs #52, #53) and their merges; both files are what a contributor and
every session read first.
Done when: the doc cites D-10, CONTRIBUTING lists `deps:`, the task-session section names the expected failure
and the clean-status check before the report, and the main-session section says to prepare after the last merge.
Run as a task session under docs/git-workflow.md, in parallel with 038, 018 and 019 (plan 2026-10-19).

## Steps

1. [ ] `docs/git-workflow.md`: the trailer rule cites D-10 (point 3), not D-09
2. [ ] `CONTRIBUTING.md`: the commit type list gains `deps:` (dependabot), matching the workflow doc
3. [ ] Task-session section: the stale-index report of `tasks_check` without `write` is expected on the close
   commit; `git status` must be clean before the session reports
4. [ ] Main-session section: a pull request is prepared only after the previous merge is pulled (`tasks_next_id`
   reads the checked-out tree)
5. [ ] Pull request, CI green

## Log

- 2026-10-19 — promoted in the main session, branch `task/036-git-workflow-references`

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
