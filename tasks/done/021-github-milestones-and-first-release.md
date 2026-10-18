---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: local-repoban-profi-a-clever-pie.md
---

# 021 — GitHub milestones from the roadmap and the first release v0.1.0

## Context

The local task standard is done (roadmap with four milestones, every task with a `milestone:` field), but the GitHub
repo has no milestone, issue, tag or release, while `docs/usage.md` sends the users to the "GitHub releases page".
What: mirroring the four roadmap milestones as GitHub milestones (same titles, M1–M3 closed, M4 open), a `v0.1.0` tag,
which the existing CI release job packages and publishes, hand-written release notes, one line each in the docs, D-08.
Why: for the users the release zip is what counts, a milestone alone would be empty. User decision 2026-10-18: the
"milestones + v0.1.0 release" variant; the tasks/ → issues mirror script is not needed (it would be a project-pal
feature, only for a GitHub audience). Done when: 4 milestones in the right state, the `v0.1.0` release with one
`mboxer-0.1.0.zip` asset, the zip containing `mboxer.exe` (ProductVersion 0.1.0) + `usage.md`.

## Steps

1. [x] GitHub milestones (`gh api`): `M1 Index core`, `M2 Viewer MVP`, `M3 Comfort` closed, `M4 macOS` open, leírás a roadmapből
2. [x] Docs: `README.md` (milestone-tükör + Releases), `tasks/roadmap.md` (egy mondat), `docs/usage.md` (Releases URL)
3. [x] D-08 proposed: a milestone-tükör szabálya
4. [x] Commit, `v0.1.0` annotated tag, push; a CI `release` job zöld, a release megjelenik
5. [x] Release notes `gh release edit v0.1.0 --notes-file`
6. [x] Ellenőrzés: milestone lista, zip letöltése és tartalma, ProductVersion
7. [x] `tasks_check`, close, commit

## Log

- 2026-10-18 — plan approved; 021 opened straight in wip/
- 2026-10-18 — milestones #1–#4 created through `gh api repos/speti4/mboxer/milestones` (title = roadmap heading,
  description = the roadmap paragraph + `Tasks: tasks/roadmap.md`), #1–#3 `closed`, #4 `open`, no due date
- 2026-10-18 — docs: README "Status and backlog" line, roadmap intro sentence, usage.md Releases URL; D-08 proposed;
  commit `8d3fced`; annotated tag `v0.1.0` on it, pushed with `main`
- 2026-10-18 — CI run 39124034625 (`v0.1.0`): `test` + `release` green (only a Node 20 deprecation notice from
  `actions/checkout@v4` and `softprops/action-gh-release@v2`, no action needed); release `v0.1.0` published with
  `mboxer-0.1.0.zip` (5 341 403 byte); title `mboxer 0.1.0` + hand-written notes set with `gh release edit`
  (the generated notes were a bare "Full Changelog" line, the repo has no PRs)
- 2026-10-18 — no follow-up: the issue-mirror script is out of scope by decision (D-08 names when to revisit),
  the 0.2.0 bump belongs to the first M4 task

## Deviations from the plan

- none: the plan landed as written.

## Verification

- 2026-10-18 — `gh api repos/speti4/mboxer/milestones?state=all`: `1 M1 Index core closed`, `2 M2 Viewer MVP closed`,
  `3 M3 Comfort closed`, `4 M4 macOS open`, titles byte-equal to the `## ` headings of `tasks/roadmap.md`.
- 2026-10-18 — `gh run watch 39124034625 --exit-status` → exit 0; `gh release view v0.1.0`: not a draft, one asset
  `mboxer-0.1.0.zip`. Downloaded into the scratchpad and unpacked: `mboxer.exe` 15 118 336 byte + `usage.md`
  6 299 byte (the usage.md in the zip already carries the Releases URL); `FileVersionInfo`: ProductVersion 0.1.0,
  FileVersion 0.1.0, ProductName mboxer.
- 2026-10-18 — `tasks_check`: 0 errors, 0 warnings, README index regenerated.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
