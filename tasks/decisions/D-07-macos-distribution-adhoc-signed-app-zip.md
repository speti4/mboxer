---
status: decided
date: 2026-10-18
cost: cheap
sources: [backlog/018-macos-build-bundle-and-ci.md, D-02-rust-tauri2-sidecar-fulltext-index.md, m-rj-k-fel-hogy-mekkora-scalable-boole.md]
---

# D-07 — macOS builds ship as an ad-hoc signed universal `.app` in a zip, without notarization

## Context

The macOS assessment of 2026-10-18 (plan file in `sources`) needed a distribution rule before the build
task (018): a bare Mach-O is not what a Mac user double-clicks, Apple Silicon refuses any unsigned
binary from the internet, and Gatekeeper blocks an app that is not notarized by a Developer ID.
Notarization needs the Apple Developer Program (99 USD a year), a Developer ID certificate and a Mac or
CI secrets to sign with. The audience is the same 2–5 users as on Windows, where the unsigned exe
already goes through a SmartScreen click (task 012). The author has no Mac at the moment.

## Decision

- The macOS artefact is one **`.app` bundle** (universal binary: `aarch64` + `x86_64` joined with
  `lipo`), assembled by a shell script from a plain `cargo build --release` — no Tauri CLI, no
  `.dmg`, no installer (D-02's one-artefact rule carried over).
- The bundle is **ad-hoc signed** (`codesign -s -`), so it loads on Apple Silicon, and **not
  notarized**: the user allows it once under *System Settings → Privacy & Security → Open Anyway*
  after the first blocked start (macOS 15 and later no longer offer the right-click *Open* bypass).
  `docs/usage.md` gets that paragraph, as it has the SmartScreen one.
- The zip is `mboxer-<version>-macos.zip` (`.app` + `usage.md`), published by the same tag-driven
  release job as the Windows zip.
- Developer ID signing and notarization stay open as a later decision, when the user count or the
  Gatekeeper friction justifies the yearly fee; the Tauri documentation's CI variables cover it.

Proposed by the session on 2026-10-18 from the user's choice "aláírás nélkül, kézi engedélyezés";
confirmed by the user on 2026-10-18 ("D-07-t megerősítem").

## Not chosen

- Developer ID + notarization now: 99 USD a year and an Apple device for the certificate, for a
  handful of users who can click once.
- A `.dmg`: a second packaging tool for no gain over a zip when there is nothing to install.
- Tauri CLI (`cargo tauri build`) for the bundle: a 5–10 minute `cargo install` in CI and a second
  build path next to the Windows one; the hand-written bundle is ~80 lines and visible.
- Apple Silicon only: cheaper CI minutes, but an Intel Mac among the users would be a surprise;
  the second target costs one more `cargo build`.
