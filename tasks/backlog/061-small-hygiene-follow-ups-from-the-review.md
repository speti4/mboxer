---
status: idea
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Small hygiene follow-ups: toolchain pin, ignores, arguments, cancel, logs, duplicates

What: `rust-toolchain.toml` says `stable` without a version, so a new Rust release with new clippy lints turns CI red
without a code change; `.gitignore` lacks `*.pdf` (a *Save as PDF* of real mail) and editor and anonymizer sidecars;
`initial_paths` in `crates/mboxer-app/src/commands.rs` uses `std::env::args()`, which panics on a non-Unicode argument;
`cancel.store(false)` runs per file, so a Cancel between two files of a batch is lost; the `eprintln!` timing logs go;
the `>From` unescaping exists in both `reader.rs` and `view.rs`; the virtual list's spacer caps the listing near 1.2
million rows; `benchmark.ps1` needs `#Requires -Version 7` or a BOM; the `zizmor.yml` ignores are by line number; the
toolchain install step is copied five times in `ci.yml` and the cargo-about version is pinned in five places; a scheduled
`cargo deny` run would catch a new advisory; the backend's query warnings fields are always empty since task 050.
Why: none of these blocks a user today; together they are a day of friction for the next contributor.
Done when: each point is fixed or dropped with a one-line reason in the Log.
