---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
milestone: M5 Public release
plan: steady-baking-kurzweil.md
---

# 024 — Supply chain and CI hardening: dependabot, cargo-deny, pinned actions

## Context

What: `.github/dependabot.yml` for cargo and github-actions (weekly, with a cooldown); every action pinned by
commit SHA with a version comment, on releases past the Node 20 deprecation; `permissions: contents: read` as the
workflow default, `concurrency` and `timeout-minutes` on every job; `cargo-deny` (licences, advisories, bans,
sources) as a CI job with a `deny.toml`; a workflow linter (`zizmor`) in CI. Build-time tools only, each with
its D-02 justification line.
Why: a public repo that ships an exe needs a visible, automatic answer to "are the dependencies and the
pipeline trustworthy", and the licence check is a precondition for the third-party notices (025).
Done when: CI is green with `cargo deny check` and the workflow lint, and the first dependabot PR has arrived.
User decision 2026-10-18: MPL-2.0 is allowed globally in the licence allowlist (D-011, confirmed by the user 2026-10-18).

D-02 lines (no new crate enters the exe or the build):
- `cargo-deny` — CI job (`EmbarkStudios/cargo-deny-action`) and an optional `cargo install` on a dev machine; it
  reads `Cargo.lock`, it is never linked.
- `zizmor` — CI job (`zizmorcore/zizmor-action`) and `uvx zizmor` on a dev machine; it reads `.github/` only.

## Steps

1. [x] `deny.toml`: Windows target graph, advisories + yanked deny, licence allowlist (MIT, Apache-2.0,
   Unicode-3.0, BSD-3-Clause, Zlib, MPL-2.0), duplicates warn, wildcards deny with path crates allowed,
   crates.io only; `publish = false` on the workspace crates
2. [x] `.github/dependabot.yml`: cargo + github-actions, weekly, 7-day cooldown, grouped, `deps` prefix
3. [x] `ci.yml`: workflow-level read-only token, concurrency, timeouts, SHA pins with version comments,
   `persist-credentials: false`, `deny` and `workflow-lint` jobs, release needs all three, tag via env,
   no cache in the release job, release published with `gh release create`
4. [x] `sync-issues.yml`: pinned checkout, timeout, `issues: write` on the job, `$GITHUB_REPOSITORY`
5. [x] local `cargo deny check` and `uvx zizmor .github` green
6. [x] `docs/development.md` (layout, CI and releases), `CONTRIBUTING.md` (checks, dependency rule)
7. [x] D-011 recorded, decided 2026-10-18
8. [x] fmt, clippy, test locally
9. [ ] commit, push (after the user's go), CI run green with `test`, `deny`, `workflow-lint`; `sync-issues` green
10. [ ] first dependabot PR(s) arrive and their CI is green

## Log

- 2026-10-18 — promoted from backlog; planning starts in plan mode
- 2026-10-18 — plan: Windows-target graph only from crates.io; licences needed beyond OR-alternatives: MIT,
  Apache-2.0, Unicode-3.0, BSD-3-Clause, Zlib, MPL-2.0. Pinned: checkout v7.0.1 `3d3c42e`, rust-cache v2.9.2
  `6323deb`, cargo-deny-action v2.1.1 `3c63498`, zizmor-action v0.7.1 `85b2b9f`, dtolnay/rust-toolchain master
  `683b588` (2026-10-16; the repo has no release tags) — all on Node 24
- 2026-10-18 — `cargo install cargo-deny --locked` → 0.20.2; `cargo deny check`: advisories ok, bans ok (10
  duplicate warnings: windows-sys ×3, windows-targets, windows_x86_64_msvc, syn, png, miniz_oxide, indexmap,
  hashbrown, getrandom, bitflags), licenses ok, sources ok — no advisory ignore and no licence exception needed
- 2026-10-18 — `uvx zizmor` 1.31.1: first run `cache-poisoning` (rust-cache in the test job of a workflow that
  publishes) and `superfluous-actions` (action-gh-release); the cache finding ignored inline with the reason (the
  release job builds without a cache), the release step rewritten to `gh release create`. Pedantic persona: the
  sync job's `issues: write` moved to the job with comments; what remains is `ref-version-mismatch` on the
  dtolnay pin (no tag to name) and on the rust-cache line (the ignore comment follows the version)
- 2026-10-18 — fmt, clippy `-D warnings` clean; `cargo test` 109 passed, 0 failed

## Deviations from the plan

- The release is published with the runner's `gh release create … --verify-tag --generate-notes` instead of
  `softprops/action-gh-release` v3: zizmor flagged it as superfluous, and one third-party action fewer holds the
  `contents: write` token. 025 changes the notes source of this step anyway.
- `.github/zizmor.yml` not needed: the single accepted finding is an inline ignore with its reason.

## Verification

- 2026-10-18 local: `cargo deny check` — advisories/bans/licenses/sources ok (exit 0); `uvx zizmor .github` — "No
  findings to report" (1 ignored); `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` clean,
  `cargo test` 109 passed
- GitHub run and the dependabot PRs: pending the push

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
