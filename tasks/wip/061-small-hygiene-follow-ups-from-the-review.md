---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 061 — Small hygiene follow-ups: toolchain pin, ignores, arguments, cancel, logs, duplicates

## Context

What: `rust-toolchain.toml` says `stable` without a version, so a new Rust release with new clippy lints turns CI
red without a code change; `.gitignore` lacks `*.pdf` (a *Save as PDF* of real mail) and editor and anonymizer
sidecars; `initial_paths` in `crates/mboxer-app/src/commands.rs` uses `std::env::args()`, which panics on a
non-Unicode argument; `cancel.store(false)` runs per file, so a Cancel between two files of a batch is lost; the
`eprintln!` timing logs go; the `>From` unescaping exists in both `reader.rs` and `view.rs`; the virtual list's
spacer caps the listing near 1.2 million rows; `benchmark.ps1` needs `#Requires -Version 7` or a BOM; the
`zizmor.yml` ignores are by line number; the toolchain install step is copied five times in `ci.yml` and the
cargo-about version is pinned in five places; a scheduled `cargo deny` run would catch a new advisory; the
backend's query warnings fields are always empty since task 050.
Why: none of these blocks a user today; together they are a day of friction for the next contributor.
Done when: each point is fixed or dropped with a one-line reason in the Log.
Decided 2026-10-19: the toolchain is pinned to the current stable version with a comment that dependabot does not
bump it, so the bump is a conscious `chore:`; the warnings fields are removed, not given a use.

## Steps

1. [ ] `rust-toolchain.toml`: the current stable version pinned (`rustc --version`), the comment says why;
   `docs/development.md` names the bump as a `chore:`
2. [ ] `.gitignore`: `*.pdf`, the common editor folders, `.anonilm-map.json`
3. [ ] `commands.rs`: `initial_paths` through `args_os` with a lossy conversion; `cancel.store(false)` once per
   batch, so a Cancel between two files stops the next; the `eprintln!` timing logs removed
4. [ ] the `>From` unescaping in one place (`mboxer-core`), `view.rs` calls it
5. [ ] the virtual list: the spacer height capped at the browser limit with a measured note, or the row count
   above 1.2 million paged; the choice in the Log (a test only where the UI module can be driven under Node)
6. [ ] `benchmark.ps1`: `#Requires -Version 7`; `zizmor.yml` ignores by rule and job instead of line number if
   zizmor supports it, else a comment naming the lines; the toolchain install step as one reusable step or
   composite action in `ci.yml`; the cargo-about version in one place the others read
7. [ ] a scheduled `cargo deny check advisories` job (weekly, `schedule:` + `workflow_dispatch`), not a required
   check
8. [ ] the empty warnings fields (`SearchResult::warnings`, `Listing::warnings`, the app's `warnings` and the UI's
   status-bar text for them) removed
9. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check`,
   `uvx zizmor .github`; CONTRIBUTING and `docs/development.md` follow where a command or a job changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
