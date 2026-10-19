---
status: ready
size: S
origin: done/036-git-workflow-references-and-commit-types.md
created: 2026-10-19
milestone: M5 Public release
---

# The commit hook's error messages cite D-10, not D-09

What: the two trailer error messages in `scripts/check-commit.sh` (the `Claude-Session:` trailer and the session
link) still cite D-09, which D-10 superseded; its point 3 carries the rule now. Change the two references.
Why: 036 fixed the same citation in `docs/git-workflow.md`; the hook's message is what a contributor sees first
when a commit is rejected, and it should point at the decision that is in force.
Done when: `grep D-09 scripts/check-commit.sh` is empty and the messages cite D-10 point 3; the hook still rejects
a `Claude-Session:` trailer and a session link.
