---
status: ready
size: S
origin: done/035-claude-md-review-before-publication.md
created: 2026-10-18
milestone: M5 Public release
---

# Git workflow doc: current decision reference, the deps type, the expected index failure

What: `docs/git-workflow.md` cites D-09 for the trailer rule, which D-10 superseded (its point 3 carries the rule);
CONTRIBUTING's commit types leave out `deps:`, which the workflow doc lists; the doc does not say that a task
session's `tasks_check` (without `write`) reports the stale index on its own close commit, which is expected.
Why: found by the 035 task session (PR #52); both files are what a contributor and every session read first.
Done when: the doc cites D-10, CONTRIBUTING lists `deps:`, and the task-session section names the expected failure.
