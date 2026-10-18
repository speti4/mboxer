---
status: proposed
date: 2026-10-18
cost: cheap
sources: [done/021-github-milestones-and-first-release.md, done/022-sync-tasks-to-github-issues.md, local-repoban-profi-a-clever-pie.md]
---

# D-08 — GitHub issues and milestones mirror `tasks/` one-way through a script; nothing is edited on GitHub by hand; a release tag closes the milestones it ships

## Context

The repo's task standard (`tasks/`, the `/tasks` skill, the generated README index) is the source of truth for
what is open, in flight and done, and every task carries `milestone:` naming a `## ` heading of
`tasks/roadmap.md`. The user asked on 2026-10-18 whether the roadmap milestones should also appear on GitHub
(`speti4/mboxer/milestones`). A GitHub milestone measures progress over its issues, so milestones alone (task
021) were an empty progress bar, and a hand-maintained set of issues next to the task files would be a second
source of truth that drifts within days. The repo is private; the issues are for the author and collaborators.

## Decision

- The roadmap headings exist as GitHub milestones with **identical titles**, so the frontmatter value and the
  GitHub title map 1:1; the description is the roadmap paragraph.
- Every task file and every decision record is **one GitHub issue**, created and updated **only by
  `scripts/sync-issues.py`** (task 022), which CI runs on every push to `main` that touches `tasks/`: the id in
  the title, the Context prose (English) in the body, labels for stage / status / size, `decision` for a
  decision record, the milestone from the frontmatter, open or closed by the folder (`done/` completed,
  `dropped/` and superseded decisions not planned). The mirror is **one-way and idempotent**: GitHub is never
  edited by hand; comments on an issue are welcome but do not flow back.
- A **release tag closes the milestones it ships**: `v0.1.0` closed M1–M3; M4 stays open until its release.
  The script never changes a milestone's state.

Proposed by the session on 2026-10-18 from the user's choices "milestone-ok + v0.1.0 release" and then
"one-way sync from CI, decisions too, English bodies"; awaiting the user's confirmation.

## Not chosen

- GitHub milestones only, without issues: an empty progress bar next to a roadmap that already says the same.
- Hand-made issues per task: double bookkeeping against the task standard; drifts.
- A local-only sync (a `/tasks sync` subcommand): drifts as soon as one run is forgotten; CI makes it automatic.
- GitHub Issues as the backlog with task files only for history: loses the frontmatter, the offline and
  agent-driven `/tasks` workflow, and splits the thinking across two places.
