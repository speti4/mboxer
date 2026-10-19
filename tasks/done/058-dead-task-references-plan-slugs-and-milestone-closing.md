---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
changelog: housekeeping
---

# 058 — Task files: dead `origin` paths, plan slugs, and who closes a milestone

## Context

What: `done/044` to `done/048` and `D-013` point at `wip/028-publication-day-repo-settings.md`, which now lives in
`done/`; rewrite them. The `sources:` and `plan:` fields of D-001 to D-011 name plan files that are not in the repo,
under slugs derived from prompts; replace each with `none` or the task ids that carried the decision. D-008,
`tasks/roadmap.md` and `scripts/sync-issues.py` say a release closes its milestones, but no workflow does it and
D-010 records that M1 to M3 were closed by hand: add the close to the release job or write the manual step into the
release checklist of `docs/development.md`, and record which. M5's done-when does not mention signing (029), so the
milestone has no end; amend the roadmap.
Why: the task files are mirrored into public issues, so a dangling reference or a slug of a prompt is public text.
Done when: `tasks_check` is clean, no `origin`, `sources` or `plan` names a missing file, and the milestone close has
an owner.
Decided 2026-10-19: filed by the post-release review; this task may edit the named `done/` and `decisions/` files
and `tasks/roadmap.md` (an exception to the task-session rule, limited to references and the M5 paragraph), never
their meaning or their Log.

## Steps

1. [x] `origin:` of `done/044`–`048` and the `sources:` entry of `D-013` point at `done/028-publication-day-repo-settings.md`
2. [x] D-001 to D-011: every `sources:` or `plan:` entry that names a file absent from the repo is replaced by the
   task ids that carried the decision (or dropped); a list of the old names in the Log, no slug kept
3. [x] the milestone close: decide between a step in the release job (close every milestone whose tasks are all in
   `done/` when the tag is pushed, through `gh api`) and a line in the release checklist of `docs/development.md`;
   the reason in the Log; D-008, the roadmap preamble and the `sync-issues.py` docstring say the same thing
4. [x] `tasks/roadmap.md`: M5's done-when names the Windows signing (029) or states that it is M6's, so the
   milestone can close
5. [x] `tasks_check` without `write` clean apart from the stale index; `python -I scripts/sync-issues.py --dry-run`
   shows only the expected updates; `uvx zizmor .github` clean if a workflow changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — `origin:` of `done/044` to `done/048` and the `sources:` entry of `D-013` now name
  `done/028-publication-day-repo-settings.md`.
- 2026-10-19 — D-001 to D-011 carry no `plan:` field; their `sources:` lists held 11 entries naming plan files
  outside the repo (10 prompt-derived plan names removed from D-001…D-011, one of them shared by D-001 and D-002).
  Each name was matched to the task that carried the decision through that task's own `plan:` field: in D-001 and
  D-002 the entry became `done/001-project-scaffolding.md`; in D-003 to D-011 the carrying task (002, 003, 014,
  016, 018, 021 and 022, 027, 030, 024) was already listed, so the entry was dropped. Every list keeps the flat
  `[a, b]` form.
- 2026-10-19 — the milestone close is a manual step of the release checklist in `docs/development.md` (step 3),
  not a step of the release job: the job's `permissions:` block grants `contents`, `id-token` and `attestations`
  only, and closing a milestone needs `issues: write`, a scope the token that publishes the release does not need
  otherwise. The step closes every milestone whose task files are all in `done/` or `dropped/` and whose done-when
  the release meets, through `gh api -X PATCH …/milestones/<number> -f state=closed`. D-008 (one clarifying
  sentence after "The script never changes a milestone's state"), the roadmap preamble, the issue-mirror paragraph
  of `docs/development.md` and the `sync-issues.py` docstring now say the maintainer closes the milestones a release
  ships, by hand. No workflow changed.
- 2026-10-19 — M5's done-when now names an Authenticode-signed exe in a release (029, waiting for SignPath
  Foundation, D-013), rather than moving 029 to M6: D-008 already records that M5 closes once 029 is done, and M6
  is the post-release review round, which 029 is not part of. M5 closes when 029 ships.
- 2026-10-19 — a resolver over the frontmatter of every file under `tasks/` (each `.md` entry of `origin:`,
  `sources:`, `supersedes:` and a decision's `plan:`, looked up under `tasks/`, `tasks/decisions/` and the repo
  root) found one more dead entry outside this task's exception: `backlog/062`'s `origin:` names `wip/057-…`, now in
  `done/`. Left for the main session (a follow-up in the pull request).

## Deviations from the plan

- D-008 gained one sentence beyond its `sources:` line: Step 3 asks it to say who closes a milestone, and the
  sentence states the mechanism without changing the decision (a release still closes the milestones it ships).
- `backlog/062`'s dead `origin:` is not fixed here: the file is outside the exception of this task.
- The `plan:` fields of the task files (42 of them name a plan file kept outside the repo, the standard's bare plan
  name, this file included) are not touched: the task covers D-001 to D-011 only; a follow-up in the pull request.

## Verification

- 2026-10-19 — in the worktree, after the reference commit: the resolver checked 89 `.md` entries and reports one
  dead entry, `backlog/062`'s `origin:` (on `main` before the change: 18); `grep -rn "wip/028" tasks/` finds only
  this file. `python -I scripts/sync-issues.py --dry-run`: created 0, updated 18, unchanged 57 — the
  bodies of 044 to 048, D-001 to D-011 and D-013 (the changed origins and sources, D-008's sentence), the M5
  milestone description (the roadmap paragraph), and this task's issue (body and the wip label); no milestone state
  change. `python -I scripts/changelog.py --check`: "CHANGELOG.md is up to date". `python -I -m py_compile
  scripts/sync-issues.py` passes. `tasks_check` (without `write`): 0 errors, 0 warnings before the move to `done/`;
  after it, the one finding is the stale README index, which the main session regenerates. No workflow changed,
  so zizmor was not needed.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (the follow-ups are in
  the pull request description; the main session files them)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
