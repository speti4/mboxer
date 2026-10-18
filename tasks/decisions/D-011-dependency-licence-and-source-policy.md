---
status: proposed
date: 2026-10-18
cost: cheap
sources: [wip/024-supply-chain-and-ci-hardening.md, backlog/025-release-notes-changelog-and-checksums.md, steady-baking-kurzweil.md]
---

# D-011 — Crates come from crates.io only, under a fixed licence allowlist that includes MPL-2.0, and an advisory fails CI

## Context

The exe compiles in about 300 crates (Tauri, windows-rs, tantivy, mail-parser and their trees). Before the repo
goes public there has to be a written, machine-checked answer to which licences and sources the shipped graph
may carry, and which security advisories stop a build; 025's third-party notices are generated from the same
graph. Task 024 measured the Windows-target graph on 2026-10-18: crates.io only; beyond OR-alternatives it needs
MIT, Apache-2.0, Unicode-3.0, BSD-3-Clause (brotli), Zlib (foldhash, zlib-rs) and MPL-2.0 (cssparser, selectors,
dtoa-short, option-ext under Tauri and dirs).

## Decision

`deny.toml` at the root is the policy and `cargo deny check` in CI enforces it on every push and pull request:
licences are allowed from the list MIT, Apache-2.0, Unicode-3.0, BSD-3-Clause, Zlib and MPL-2.0, and a new
licence is added to the list by a change to this decision. MPL-2.0 is allowed globally because its copyleft is
file-level and the crates are used unmodified; its notice and the source location go into the licence file of 025.
Sources are crates.io only (no other registry, no git dependencies). A RustSec advisory or a yanked crate fails
the check; an ignore names the advisory id and the reason in `deny.toml`. Duplicate versions are reported, not
failed. The user chose the global MPL-2.0 allowance on 2026-10-18; the decision stays proposed until the user
confirms the record.

## Not chosen

- MPL-2.0 only as per-crate exceptions: every new MPL crate under Tauri would break CI for no legal difference.
- No MPL-2.0: replacing the crates means leaving Tauri's HTML tooling, which D-02 rules out.
- Failing on duplicate versions: Tauri and windows-rs pin several versions themselves, so the check would stay red.
- Advisories only on a weekly schedule: a pull request that pulls a vulnerable crate has to fail at review time.
