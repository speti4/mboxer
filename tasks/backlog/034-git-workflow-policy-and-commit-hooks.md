---
status: ready
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Git workflow policy for contributors and parallel sessions, with the commit rules enforced

What: `docs/git-workflow.md` (contributors: branch → PR → squash merge, commit format, dependabot PRs; maintainer and
agent sessions: one worktree per task, new ids only in the main session, the task index regenerated on main after a
merge, one manual round at a time, push only on the user's go), imported into CLAUDE.md with `@` and linked from
CONTRIBUTING; a versioned `commit-msg` git hook (`.githooks/`, `core.hooksPath`) that rejects a `Claude-Session:`
trailer and a non-noreply author, plus the same check as a CI job for pushes and PRs that bypass the local hook.
Why: parallel sessions need the rules in context at start (user decision 2026-10-18: a policy file, not a decision);
the trailer rule (D-09) is lost to the system reminder unless something checks it. Branch ruleset on main: 028.
Done when: a commit with the trailer fails locally and in CI, and a second session in a worktree follows the file.
