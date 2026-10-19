---
status: ready
size: M
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
milestone: M5 Public release
---

# macOS in the user docs, the security policy and the bug report form

What: the README (tagline, Download with the macOS preview zip and the Gatekeeper *Open Anyway* step, `gh
attestation verify`, Requirements, how to report a bug, Build from source on a Mac), `docs/usage.md` (an "On a
Mac" part, `shasum`, the write locations on a Mac, WebView2 fallback, plainer wording), `docs/reference.md` (the
quarantine attribute and the blocked types on a Mac, the CLI is not in the zips), `SECURITY.md` (macOS scope) and
the bug report form ("Operating system and version", the Mac index location).
Why: 0.2.0 ships a macOS zip, but every user-facing text still reads as Windows-only (pre-publication review).
Done when: a Mac user can download, verify, start and report a bug from the docs alone, before the `v0.2.0` tag.
