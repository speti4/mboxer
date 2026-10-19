---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: steady-baking-kurzweil.md
changelog: housekeeping
---

# 024 — Supply chain and CI hardening: dependabot, cargo-deny, pinned actions

## Context

What: `.github/dependabot.yml` for cargo and github-actions (weekly, with a cooldown); every action pinned by
commit SHA with a version comment, on releases past the Node 20 deprecation; `permissions: contents: read` as the
workflow default, `concurrency` and `timeout-minutes` on every job; `cargo-deny` (licences, advisories, bans,
sources) as a CI job with a `deny.toml`; a workflow linter (`zizmor`) in CI. Build-time tools only, each with
its D-002 justification line.
Why: a public repo that ships an exe needs a visible, automatic answer to "are the dependencies and the
pipeline trustworthy", and the licence check is a precondition for the third-party notices (025).
Done when: CI is green with `cargo deny check` and the workflow lint, and the first dependabot PR has arrived.
Decided 2026-10-18: MPL-2.0 is allowed globally in the licence allowlist (D-011).

D-002 lines (no new crate enters the exe or the build):
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
9. [x] commit, push, CI run green with `test`, `deny`, `workflow-lint`; `sync-issues` green
10. [x] first dependabot PR(s) arrive and their CI is green

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
- 2026-10-18 — pushed `f20dac7` + `8cb91e5`: ci run 39140529071 green (test, deny, workflow-lint success,
  release skipped), sync-issues green with `$GITHUB_REPOSITORY`. The first two "Dependabot Updates" runs
  (cargo, github-actions) succeeded with no PR: every action is on its latest release and `Cargo.lock` is
  current ("No update needed"); the first PR comes with the next release past the 7-day cooldown
- 2026-10-18 — Dependabot alerts were off ("Dependabot alerts are disabled for this repository", 403):
  `gh api -X PUT …/vulnerability-alerts` + `…/automated-security-fixes` → enabled, not paused; 0 open alerts
- 2026-10-18 — to see the first dependabot PR the same day: rust-cache pin set back to v2.9.1 `c193711`
  (2026-03-12, past the cooldown) and a comment line in `dependabot.yml`, which triggers an immediate run
- 2026-10-18 — PR #44 "deps: Bump Swatinem/rust-cache from 2.9.1 to 2.9.2 in the actions group" arrived in
  minutes; its diff changed the SHA but kept `# v2.9.1`, because the inline zizmor ignore sat after the version
  comment → the ignore moved to `.github/zizmor.yml` (`cache-poisoning`, `ci.yml`), zizmor still clean locally.
  The first security-update run failed with `security_update_not_possible` for glib: alert #1 GHSA-wrw7-89jp-8q8g
  (medium, `VariantStrIter` unsoundness, fixed in 0.20) on glib 0.18.5, which gtk pins under Tauri's Linux
  backend; `cargo tree -i glib --target x86_64-pc-windows-msvc` prints nothing, so the exe does not contain it
- 2026-10-18 — PR #44 rebased by dependabot after `80cba01`: `c193711 # v2.9.1` → `6323deb # v2.9.2`, checks
  test/deny/workflow-lint pass (release skipped), squash-merged as `95c8ffe`. Alert #1 dismissed as
  `not_used` with the `cargo tree` reason
- 2026-10-18 — closing sweep: the dtolnay master pin, which dependabot cannot follow → backlog/033; the macOS
  targets for `deny.toml` → a line in backlog/018

## Deviations from the plan

- The release is published with the runner's `gh release create … --verify-tag --generate-notes` instead of
  `softprops/action-gh-release` v3: zizmor flagged it as superfluous, and one third-party action fewer holds the
  `contents: write` token. 025 changes the notes source of this step anyway.
- The accepted `cache-poisoning` finding moved from an inline `# zizmor: ignore` to `.github/zizmor.yml`: the
  inline comment followed the version comment, and dependabot then left `# v2.9.1` behind a v2.9.2 SHA (PR #44).

## Verification

- 2026-10-18 local: `cargo deny check` — advisories/bans/licenses/sources ok (exit 0); `uvx zizmor .github` — "No
  findings to report" (1 ignored); `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` clean,
  `cargo test` 109 passed
- 2026-10-18 GitHub: ci run 39140529071 on `8cb91e5` — test, deny, workflow-lint success, release skipped;
  sync-issues success; dependabot cargo + github-actions runs success, no update needed yet
- 2026-10-18 dependabot: PR #44 (rust-cache 2.9.1 → 2.9.2, SHA and version comment updated) green on all three
  jobs, merged `95c8ffe`; Dependabot alerts and security updates on, alert #1 (glib, Linux-only) dismissed,
  0 open alerts

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
