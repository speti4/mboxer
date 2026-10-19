---
status: decided
date: 2026-10-17
cost: medium
sources: [docs/origin.md, olvasd-el-a-draft-md-sharded-teapot.md]
---

# D-002 — Rust + Tauri 2 (WebView2) with a one-time sidecar full-text index

## Context

The app must open 14–50 GB mbox files on Windows 11, search bodies in well under a second, render
HTML mail, and ship as one portable exe with nothing for the user to install. "Zero dependency"
was clarified on 2026-10-17 to mean the runtime: build-time crates are fine when justified.

## Decision

- Language: Rust. UI: Tauri 2 on WebView2 (part of Windows 11, so no runtime install); HTML mail
  renders in a sandboxed iframe with remote content blocked.
- Search: the first open builds a sidecar full-text index next to the mbox (tantivy or equivalent,
  rebuildable; the mbox is never modified). Later opens reuse it. Metadata and body are both indexed.
- Tokenization is accent- and case-insensitive (`szamla` matches `számla`), on both index and query side.
- Search UX: one search field plus a clickable filter panel (date range, sender, label, attachment);
  Gmail-style query syntax is a later milestone.
- Each user has their own mbox; the index lives beside it; no sharing or server component.

Decided on 2026-10-17.

## Not chosen

- C# / .NET (WPF, WinUI 3): fine on Windows, but a larger self-contained exe and a second ecosystem to maintain.
- Go (Wails / Fyne): weaker desktop UI story.
- egui: pure Rust, but HTML mail would show as plain text only.
- Native Win32 (windows-rs): most code for the least gain; HTML would still need WebView2.
- Header-only index with on-demand body scan: fast start, but body search stays slow on 50 GB.
- No index, streaming scan of the whole file: every search takes minutes on 14 GB.
