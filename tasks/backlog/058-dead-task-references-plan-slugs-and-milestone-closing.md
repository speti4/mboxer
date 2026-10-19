---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Task files: dead `origin` paths, plan slugs, and who closes a milestone

What: `done/044` to `done/048` and `D-013` point at `wip/028-publication-day-repo-settings.md`, which now lives in
`done/`; rewrite them. The `sources:` and `plan:` fields of D-001 to D-011 name plan files that are not in the repo,
under slugs derived from prompts; replace each with `none` or the task ids that carried the decision. D-008,
`tasks/roadmap.md` and `scripts/sync-issues.py` say a release closes its milestones, but no workflow does it and
D-010 records that M1 to M3 were closed by hand: add the close to the release workflow or write the manual step into
the release checklist of `docs/development.md`, and record which. M5's done-when does not mention signing (029), so
the milestone has no end; amend the roadmap.
Why: the task files are mirrored into public issues, so a dangling reference or a slug of a prompt is public text.
Done when: `tasks_check` is clean, no `origin`, `sources` or `plan` names a missing file, and the milestone close has an owner.
