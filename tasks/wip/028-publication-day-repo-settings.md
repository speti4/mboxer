---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
plan: none
milestone: M5 Public release
---

# 028 — Publication day: visibility switch and the public-only repo settings

## Context

What: a checklist done in one sitting: visibility to public; a ruleset on `main` (CI required, no force push,
linear history); private vulnerability reporting; secret scanning with push protection; a CodeQL workflow (Rust,
JavaScript for `ui/`, actions); build provenance attestation in the release job; topics, homepage and a social
preview image (from a synthetic screenshot); Discussions on or off; GitHub's Community Standards page checked
complete (023); the README badges render (026); the first release after the switch (the first end-to-end run of
the 025 release job: notes from `CHANGELOG.md`, `SHA256SUMS.txt`, the licence page in the zip; D-012); after the
switch, the pre-rewrite backup folder from 030 deleted (D-010).
Why: these settings exist only for public repos or are free only there, so they cannot be prepared earlier than
the switch itself.
Done when: every item is applied or recorded in the Log with the reason it was left out.

User decisions 2026-10-19:
- 028 runs ahead of the 040 Mac round: the account is GitHub Pro and the month's Actions minutes ran out; public
  repos run standard runners free.
- The first public release is 0.2.0 (the workspace version in `Cargo.toml`; the bug-report template's placeholder
  follows); the tag `v0.2.0` after the switch runs the release job.
- The ruleset on `main` requires the full CI (test on windows-latest, lint, deny, workflow-lint, commit-rules),
  no force push, linear history; the `paths-ignore` of `ci.yml` (042) goes away. macOS tests stay on tags and
  manual runs.
- Discussions off.
- Repo description: "Viewer for large Gmail Takeout mbox archives on Windows and macOS: Rust core, sidecar
  full-text index, Tauri 2 UI".
- The social preview image is made from `docs/images/viewer.png` and uploaded by hand (no API).
- HEAD is clean of home-folder paths since #75; older commits keep theirs (no new history rewrite).
- The visibility switch is irreversible: the user gives an explicit go right before it; the backup folder is
  deleted only after asking.

## Steps

1. [ ] the plan (plan mode), then the steps here

## Log

- 2026-10-19 — promoted on `task/028-publication-day-repo-settings`; planned in this (main) session

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: 2026-10-..`
