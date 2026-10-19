---
status: decided
date: 2026-10-18
cost: one-way
supersedes: D-009-publish-scrubbed-rewrite-history-before-switch.md
sources: [D-009-publish-scrubbed-rewrite-history-before-switch.md, done/027-pre-publish-history-and-content-audit.md, wip/030-scrub-and-rewrite-history-per-d-09.md, folytassuk-a-030-task-replicated-fairy.md]
---

# D-010 — The repo goes public scrubbed, with the history rewritten once and pushed into a recreated GitHub repo

## Context

D-009 settled what the public repo carries (task 027's audit): `tasks/` and `docs/` scrubbed in place, the history
rewritten once with `git filter-repo`, `Claude-Session:` trailers gone, the GitHub noreply address on every
commit. Its point 2 then force-pushed the rewritten `main` and tag into the existing GitHub repo, keeping the
release, the issues and the milestones. Planning task 030 showed that a force push does not finish the job on
GitHub: unreachable old commits stay viewable by SHA until GitHub Support runs a garbage collection, and 15
Actions runs plus the commit SHAs cited in task Logs link straight to them, so the third-party name would stay
one click away once the repo is public. Everything GitHub holds for this repo is derivable from the repo itself:
the 39 issues (no comments), their labels and the milestones come from `scripts/sync-issues.py` (D-008), the
release from the tag's CI job; only the hand-written release notes, the closed state of M1–M3 and the repo
description are not, and they are small enough to restore by hand.

## Decision

D-009 stands, with point 2 replaced:

1. **`tasks/` and `docs/` are published, scrubbed in place** (task 030), exactly as D-009 point 1 lists.
2. **The history is rewritten once, and the GitHub repo is recreated from it**: `git filter-repo` replaces the
   employer name, the archive path, `C:\Users\<name>` and the 011 status line in every past blob (the annotated
   `v0.1.0` tag is rewritten with it); a bundle of the old history and an export of the GitHub side are kept
   outside the repo until 028 closes; then `speti4/mboxer` is deleted and created again (private, same name and
   description), `main` is pushed (the sync regenerates labels, milestones and issues, so issue #N equals task N
   for 001–030 and the decisions follow), M1–M3 are closed by hand, the `v0.1.0` tag is pushed (the CI job
   rebuilds and publishes the zip) and the saved release notes are restored. Old Actions runs, cached views and
   unreachable objects disappear with the old repo. Commit SHAs cited in task files are remapped to the new
   history in one commit.
3. **Commit trailers**: as D-009 point 3 — no `Claude-Session:`, `Co-Authored-By: Claude …` stays.
4. **Author identity**: as D-009 point 4 — the GitHub noreply address on every commit, past (`--mailmap`, tagger
   included) and future (`git config user.email`).
5. 030 lands before 028 flips the visibility.

The user chose the recreated repo on 2026-10-18 while planning task 030 ("nem lenne egyszerűbb egyet
backupolni, majd törölni a remotet és full újrahúzni a remote oldalt?") and approved the plan the same day.

## Not chosen

- Force push into the existing repo (D-009 point 2): the old commits stay reachable by SHA through the Actions
  runs and the cited SHAs until a GitHub Support GC that is neither scheduled nor confirmable from outside.
- Force push plus deleting the old runs plus a Support ticket: more steps and a wait on a third party for the
  result the recreated repo gives at once; the only cost of recreating is the issue numbering of D-001–D-008 and
  tasks 023–029, which nobody outside the repo references yet.
- A fresh repo from a squashed snapshot: loses the history the rewrite keeps, for nothing extra.
