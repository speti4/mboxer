---
size: S
origin: done/036-git-workflow-references-and-commit-types.md
created: 2026-10-19
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
---

# 039 — The commit hook's error messages cite D-10, not D-09

## Context

What: the two trailer error messages in `scripts/check-commit.sh` (the `Claude-Session:` trailer and the session
link) still cite D-09, which D-10 superseded; its point 3 carries the rule now. Change the two references.
Why: 036 fixed the same citation in `docs/git-workflow.md`; the hook's message is what a contributor sees first
when a commit is rejected, and it should point at the decision that is in force.
Done when: `grep D-09 scripts/check-commit.sh` is empty and the messages cite D-10 point 3; the hook still rejects
a `Claude-Session:` trailer and a session link.
User decision 2026-10-19: part of the backlog wave 039 + 041 + 020 in parallel task sessions, 037 in the main
session; 039 merges first.

## Steps

1. [ ] `scripts/check-commit.sh`: the trailer message (line 23) and the session-link message (line 27) cite
   `D-10 point 3`; the header comment (line 2) cites D-10 only
2. [ ] `grep -n D-09 scripts/check-commit.sh` is empty
3. [ ] the hook still rejects: a message with a `Claude-Session:` trailer, one with a Claude Code session link;
   a clean message passes

## Log

- 2026-10-19 — promoted on `task/039-check-commit-messages-cite-d-10`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
