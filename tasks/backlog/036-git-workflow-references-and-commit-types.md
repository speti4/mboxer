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
session's `tasks_check` (without `write`) reports the stale index on its own close commit, which is expected; and
a task session must see `git status` clean before it reports (035's close commit left the file's edits behind);
the main session prepares a pull request only after the previous merge is pulled, because `tasks_next_id` sees
only the checked-out tree and an id filed on an unmerged branch would be handed out twice (032 waited for #52).
Why: found by the 035 and 032 task sessions (PRs #52, #53) and their merges; both files are what a contributor and
every session read first.
Done when: the doc cites D-10, CONTRIBUTING lists `deps:`, the task-session section names the expected failure
and the clean-status check before the report, and the main-session section says to prepare after the last merge.
