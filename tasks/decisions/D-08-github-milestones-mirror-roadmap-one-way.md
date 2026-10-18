---
status: proposed
date: 2026-10-18
cost: cheap
sources: [wip/021-github-milestones-and-first-release.md, local-repoban-profi-a-clever-pie.md]
---

# D-08 — GitHub milestones mirror `tasks/roadmap.md` one-way; no hand-made issues; a release tag closes the milestones it ships

## Context

The repo's task standard (`tasks/`, the `/tasks` skill, the generated README index) is the source of truth for
what is open, in flight and done, and every task carries `milestone:` naming a `## ` heading of
`tasks/roadmap.md`. The user asked on 2026-10-18 whether the roadmap milestones should also appear on GitHub
(`speti4/mboxer/milestones`). A GitHub milestone measures progress over its issues; the repo has no issues, and
a hand-maintained set of issues next to the task files would be a second source of truth that drifts within
days. What the 2–5 users actually need from GitHub is the release zip (`docs/usage.md` points there).

## Decision

- The four roadmap headings exist as GitHub milestones with **identical titles**, so the frontmatter value and
  the GitHub title map 1:1. The description is the roadmap paragraph; nothing else lives there.
- The mirror is **one-way**: `tasks/roadmap.md` and the task files are edited, GitHub is updated from them.
  **No issues are created by hand**; the milestone pages carry no issue counts until (if ever) a script mirrors
  the task files, which would be a project-pal feature, not an mboxer one.
- A **release tag closes the milestones it ships**: `v0.1.0` closes M1–M3; M4 stays open until its release.
  A milestone is closed on GitHub when its last task moves to `done/` and the release is tagged.

Proposed by the session on 2026-10-18 from the user's choice "milestone-ok + v0.1.0 release"; awaiting the
user's confirmation.

## Not chosen

- GitHub milestones only, without a release: an empty progress bar next to a roadmap that already says the same.
- Hand-made issues per task: double bookkeeping against the task standard; drifts.
- A tasks/ → issues mirror script now: real progress bars, but a script to maintain for an audience that does
  not exist yet; revisit when colleagues or external contributors work from GitHub.
- Nothing on GitHub: the users still need a Releases page to download from.
