---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M5 Public release
---

# 048 — The usage guide in the zips in a format that opens with a double-click

## Context

What: `release.ps1` and `bundle-macos.sh` put `usage.md` into the zips; Windows has no default app for `.md` and
the tables read poorly as plain text. Render it to `usage.html` next to `THIRD-PARTY-LICENSES.html` (or ship
`.txt`), without a new runtime dependency.
Why: the guide is for people who only run the program.
Done when: both zips hold a guide that opens in the default browser or text editor and reads well.
Decided 2026-10-19: merged after the `v0.2.0` release, so the first public release run is not changed.

## Steps

1. [ ] the renderer: how `usage.md` becomes `usage.html` at release time with tools the release jobs already
   have (Python standard library, the cargo-about step, or a small converter in `scripts/`); the choice and the
   reason in the Log
2. [ ] `scripts/release.ps1` and `scripts/bundle-macos.sh` pack the rendered guide; the macOS licence-only path
   and the CI artifact paths still work
3. [ ] the docs that name the zip contents (README, `docs/usage.md`, `docs/development.md`, the release notes
   footer in `scripts/changelog.py`) follow
4. [ ] a local `release.ps1` run (or its packing part) shows the guide in the zip and it renders in a browser;
   the macOS script checked with `bash -n` and a dry path where possible

## Log

- 2026-10-19 — promoted

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
