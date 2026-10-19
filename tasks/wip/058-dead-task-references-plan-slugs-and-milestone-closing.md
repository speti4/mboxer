---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
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

1. [ ] `origin:` of `done/044`–`048` and the `sources:` entry of `D-013` point at `done/028-publication-day-repo-settings.md`
2. [ ] D-001 to D-011: every `sources:` or `plan:` entry that names a file absent from the repo is replaced by the
   task ids that carried the decision (or dropped); a list of the old names in the Log, no slug kept
3. [ ] the milestone close: decide between a step in the release job (close every milestone whose tasks are all in
   `done/` when the tag is pushed, through `gh api`) and a line in the release checklist of `docs/development.md`;
   the reason in the Log; D-008, the roadmap preamble and the `sync-issues.py` docstring say the same thing
4. [ ] `tasks/roadmap.md`: M5's done-when names the Windows signing (029) or states that it is M6's, so the
   milestone can close
5. [ ] `tasks_check` without `write` clean apart from the stale index; `python -I scripts/sync-issues.py --dry-run`
   shows only the expected updates; `uvx zizmor .github` clean if a workflow changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
