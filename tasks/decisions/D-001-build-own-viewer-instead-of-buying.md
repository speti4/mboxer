---
status: decided
date: 2026-10-17
cost: medium
sources: [docs/origin.md, docs/research/2026-10-17-free-mbox-viewer-alternatives.md, olvasd-el-a-draft-md-sharded-teapot.md]
---

# D-001 — Build our own mbox viewer instead of adopting an existing one

## Context

The original brief (`docs/origin.md`) asked for a free alternative first and own development only as
a second round. The survey (`docs/research/2026-10-17-free-mbox-viewer-alternatives.md`) looked for a free Windows GUI
that searches message bodies fast on a 14–50 GB Gmail Takeout mbox. Constraints: free, Windows 11,
no runtime to install, usable by non-technical people, each with their own file.

## Decision

Build our own viewer. No free tool meets the bar: every candidate either has no GUI, caps the free
tier far below 14 GB, or still scans the whole file for body search, which takes minutes at this
size. Decided on 2026-10-17.

## Not chosen

- Mbox Viewer (mboxviewerpro.com, Microsoft Store): native, labels, threading, 50 GB+ streaming, but the free tier stops at 1 GB, Pro is paid, and body search is a streaming scan, not an index.
- mboxShell (dcarrero, Rust, MIT): free and handles 50 GB, but terminal-only; body search is a scan. Kept as an implementation reference.
- zoltanf/MboxViewer (Electron, GPL-3): SQLite full-text index, but no confirmed Windows build and an Electron runtime.
- Thunderbird + ImportExportTools NG: free, but an import workflow with known stalls on very large mbox files, not a viewer.
