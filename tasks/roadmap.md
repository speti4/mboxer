# Roadmap

An mbox viewer for Windows 11 and macOS for Gmail Takeout archives (tested on a 14 GB export, up to 50 GB as the
target): fast, stable, searchable. Order: first a headless index core that proves
search speed on the real 14 GB file, then the Tauri UI on top of it, then the comfort features.
State lives in the task files and the generated index, never here. The `## ` headings are mirrored one-way as
GitHub milestones with identical titles, and the task files as issues under them, by `scripts/sync-issues.py`
(the task files are the source of truth); the maintainer closes the milestones a release ships once it is
published, by hand (the release steps in `docs/development.md`): M1–M3 → v0.1.0; M5 starts with v0.2.0, the first
public release (D-008).

## M1 Index core

A Rust library plus a thin CLI that streams the mbox, builds the sidecar full-text index with
accent folding and answers searches. Done when the real 14 GB file indexes in acceptable time,
re-opens in under a second and a body search answers in under a second. No UI yet.

## M2 Viewer MVP

Tauri 2 shell (WebView2) on top of M1: three-pane layout (folders / list / reader), one search
field plus a filter panel, sanitized HTML reader, attachments list / open / save. Done when the
viewer covers the everyday lookup workflow.

## M3 Comfort

Gmail labels and threads, Gmail-style query syntax, export (.eml / PDF / print), a portable release
build anyone can run without installing anything.

## M4 macOS

The same viewer on a Mac with full parity: an ad-hoc signed `.app` from CI, the runtime differences
(index home, quarantine, Finder open, keys and fonts), and a print path that works in WKWebView.
Done when a Mac user opens their archive from the release zip and every feature of M3 works,
print / Save as PDF included.

## M5 Public release

The repo goes public as a project others can trust and contribute to: community health files, a
hardened supply chain, reproducible release notes and checksums, a front-page README, an audited
history, then the visibility switch and the settings GitHub only offers free on public repos. Done
when the repo is public, the audit is clean, a release made after the switch carries checksums, third-party
licences and generated notes, and a release ships an Authenticode-signed exe (029, which waits for SignPath
Foundation to accept the project, D-013). Ships before M4 is closed (028); M4 waits for the manual test on a
Mac (040).

## M6 Hardening

The findings of the first review after the public release: the bugs a large archive or an accented label name
brings out (a panic in the label tree, a colon in free text, a list stuck under fast typing, a non-atomic index
swap), the layers behind the sanitizer, and the docs, hook and CI text that promise more than the machinery does.
Done when every item of the review round is closed or dropped and a release carries the fixes.
