# Roadmap

A Windows 11 mbox viewer for Gmail Takeout archives (14 GB today, up to 50 GB for other users) that
replaces eneam/mboxviewer: fast, stable, searchable. Order: first a headless index core that proves
search speed on the real 14 GB file, then the Tauri UI on top of it, then the comfort features.
State lives in the task files and the generated index, never here. The `## ` headings are mirrored one-way as
GitHub milestones with identical titles, and the task files as issues under them, by `scripts/sync-issues.py`
(the task files are the source of truth); a release tag closes the milestones it ships: M1–M3 → v0.1.0 (D-08).

## M1 Index core

A Rust library plus a thin CLI that streams the mbox, builds the sidecar full-text index with
accent folding and answers searches. Done when the real 14 GB file indexes in acceptable time,
re-opens in under a second and a body search answers in under a second. No UI yet.

## M2 Viewer MVP

Tauri 2 shell (WebView2) on top of M1: three-pane layout (folders / list / reader), one search
field plus a filter panel, sanitized HTML reader, attachments list / open / save. Done when the
user stops opening mboxviewer.

## M3 Comfort

Gmail labels and threads, Gmail-style query syntax, export (.eml / PDF / print), a portable release
build for the other 2–5 users.

## M4 macOS

The same viewer on a Mac with full parity: an ad-hoc signed `.app` from CI, the runtime differences
(index home, quarantine, Finder open, keys and fonts), and a print path that works in WKWebView.
Done when a Mac user opens their archive from the release zip and every feature of M3 works,
print / Save as PDF included (assessment 2026-10-18).
