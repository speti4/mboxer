---
size: M
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M5 Public release
---

# 044 — macOS in the user docs, the security policy and the bug report form

## Context

What: the README (tagline, Download with the macOS preview zip and the Gatekeeper *Open Anyway* step, `gh
attestation verify`, Requirements, how to report a bug, Build from source on a Mac), `docs/usage.md` (an "On a
Mac" part, `shasum`, the write locations on a Mac, WebView2 fallback, plainer wording), `docs/reference.md` (the
quarantine attribute and the blocked types on a Mac, the CLI is not in the zips), `SECURITY.md` (macOS scope) and
the bug report form ("Operating system and version", the Mac index location).
Why: 0.2.0 ships a macOS zip, but every user-facing text still reads as Windows-only (pre-publication review).
Done when: a Mac user can download, verify, start and report a bug from the docs alone, before the `v0.2.0` tag.
Decided 2026-10-19: the macOS zip is described as a preview that has not yet been tested on a Mac.

## Steps

1. [ ] README: tagline, Download and start (Windows and macOS), Requirements, reporting a bug, Build from source
   on a Mac, the theme line, D-id links, the outdated "macOS build is on the roadmap" sentence
2. [ ] `docs/usage.md`: title and headings, "On a Mac", checksums and attestation, write locations, search wording,
   WebView2 fallback, reporting a problem
3. [ ] `docs/reference.md`: attachments on a Mac, the CLI outside the zips, D-id links, the shortcut wording
4. [ ] `SECURITY.md`: quarantine attribute, Mac index fallback, Gatekeeper and WKWebView out of scope
5. [ ] `.github/ISSUE_TEMPLATE/bug-report.yml`: operating system field, Mac index location option
6. [ ] every statement checked against the code (`crates/mboxer-app/src/attachments.rs`, `scripts/bundle-macos.sh`,
   the index fallback), links resolve

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
