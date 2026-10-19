---
size: S
origin: done/036-git-workflow-references-and-commit-types.md
created: 2026-10-19
started: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
closed: 2026-10-19
changelog: housekeeping
---

# 039 — The commit hook's error messages cite D-010, not D-009

## Context

What: the two trailer error messages in `scripts/check-commit.sh` (the `Claude-Session:` trailer and the session
link) still cite D-009, which D-010 superseded; its point 3 carries the rule now. Change the two references.
Why: 036 fixed the same citation in `docs/git-workflow.md`; the hook's message is what a contributor sees first
when a commit is rejected, and it should point at the decision that is in force.
Done when: `grep D-009 scripts/check-commit.sh` is empty and the messages cite D-010 point 3; the hook still rejects
a `Claude-Session:` trailer and a session link.
Decided 2026-10-19: done in parallel with 041, 020 and 037; 039 merges first.

## Steps

1. [x] `scripts/check-commit.sh`: the trailer message (line 23) and the session-link message (line 27) cite
   `D-010 point 3`; the header comment (line 2) cites D-010 only
2. [x] `grep -n D-009 scripts/check-commit.sh` is empty
3. [x] the hook still rejects: a message with a `Claude-Session:` trailer, one with a Claude Code session link;
   a clean message passes

## Log

- 2026-10-19 — promoted on `task/039-check-commit-messages-cite-d-10`.
- 2026-10-19 — `scripts/check-commit.sh`: the two messages cite `D-010 point 3`, the header comment D-010 only.
  `docs/` and `CONTRIBUTING.md` do not quote the messages verbatim, so nothing to do there; the D-009 line of
  `CHANGELOG.md` is the historical title of task 030 and stays.

## Deviations from the plan

- none

## Verification

- 2026-10-19 — `grep -n D-009 scripts/check-commit.sh`: empty. The script on scratch message files (maintainer
  name + noreply address): `Claude-Session:` trailer → exit 1 with the "(D-010 point 3)" message;
  `claude.ai/code/session_…` link → exit 1, "(D-010 point 3)"; clean message → exit 0; non-noreply address →
  exit 1 (D-010), unchanged. Through the `.githooks/commit-msg` hook too: clean → 0, trailer → 1. The hook passed
  the fix's commit.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (index: by the main session when it prepares the PR)
- [x] moved to `done/` with `closed: 2026-10-19`
