---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: peaceful-jumping-adleman.md
milestone: M4 macOS
---

# 042 — Cut the CI minutes: macOS tests on tags and manual runs

## Context

What: the `test (macos-latest)` leg runs only on a `v*` tag and on a manual run; a Linux job compiles the macOS
target with clippy on every code change instead; docs- and tasks-only changes skip ci.yml, while the commit rules
move into their own workflow that runs on every push and pull request.
Why: GitHub bills macOS minutes 10× and Windows 2×; since #61 every run costs 30–40 macOS-billed minutes, twice
per change (pull request and main push), and the account goes over its 3000-minute Pro quota (~2000/month from
mboxer alone). Asked by the claude-setup session (its task 002, step 3).
Done when: a main push and a pull request run carry no `macos-*` job, a manual run does, and the monthly estimate
is under ~700 billed minutes.
User decision 2026-10-19: the real macOS tests run on tag + workflow_dispatch only, not on pull requests; a
pull request that touches macOS code gets a manual run on its branch before the merge.

## Steps

1. [x] `ci.yml`: `paths-ignore` (`docs/**`, `tasks/**`, `**/*.md`) on push and pull_request
2. [x] `ci.yml`: the `test` matrix holds `macos-latest` only on a tag or a manual run; `cargo fmt --check` leaves it
3. [x] `ci.yml`: new ubuntu `lint` job — `cargo fmt --check`, clippy for `aarch64-apple-darwin` with `DOCS_RS=1`;
   `bundle-macos` and `release` need it
4. [x] `commit-rules` job moves to `.github/workflows/commit-rules.yml` (no paths filter)
5. [x] docs: `docs/development.md` (CI and releases, the macOS bundle), `CONTRIBUTING.md`, `scripts/check-commit.sh`
6. [x] local checks: `uvx zizmor .github`, darwin clippy, fmt, clippy, test, `cargo deny check`
7. [x] pull request run without `macos-*`; one manual run on the branch with `test (macos-latest)` and `bundle-macos`

## Log

- 2026-10-19 — before: the last 12 ci runs, `test (macos-latest)` 0–4 min (×10 billed, rounded up per job),
  `test (windows-latest)` 2–4 min (×2); 38 pull request + 23 push + 1 manual ci run in 30 days; 28 of 59 main
  commits since 2026-09-19 touch only `docs/`, `tasks/` or Markdown. `main` has no branch protection, so a
  skipped workflow blocks no merge.
- 2026-10-19 — `ci.yml`: `paths-ignore` a push és pull_request triggeren; a `test` matrix `os` listája
  `fromJSON(...)` kifejezés (macOS csak tag / `workflow_dispatch`); új `lint` job (ubuntu: fmt + clippy
  `aarch64-apple-darwin`, `DOCS_RS=1`), a `bundle-macos` és a `release` `needs` listáján is; a `commit-rules` job
  átkerült a `commit-rules.yml`-be (path filter nélkül, saját concurrency group). A concurrency marad: PR-on
  cancel, main és tag fut végig. `.github/zizmor.yml` indoklása a `lint` job cache-ére is kiterjed.
- 2026-10-19 — docs: `docs/development.md` (CI and releases, The macOS bundle), `CONTRIBUTING.md`,
  `scripts/check-commit.sh` fejléckomment.
- 2026-10-19 — local: `cargo fmt --check` ok; `DOCS_RS=1 cargo clippy --target aarch64-apple-darwin --all-targets
  -- -D warnings` ok (37 s); `cargo clippy --all-targets -- -D warnings` ok; `cargo test` ok; `cargo deny check`
  advisories/bans/licenses/sources ok; `uvx zizmor .github` no findings (2 ignored: a cache-poisoning a test és
  a lint jobon, `.github/zizmor.yml`); `actionlint` 1.7.12 a két workflow-ra: nincs hiba.

## Deviations from the plan

- none yet

## Verification

- 2026-10-19 — PR #66, `ci` run 39251322792 (pull_request): `test (windows-latest)` 175 s, `lint` (ubuntu) 163 s,
  `deny` 55 s, `workflow-lint` 7 s, mind success; macOS job nem futott (a `bundle-macos` skipped — a skipped job
  is viseli a `macos-latest` labelt, de nem fut és nem számláz). `commit-rules` run 39251322878 külön
  workflow-ként: success, 5 s.
- 2026-10-19 — kézi futtatás a branchen, `ci` run 39251320450 (workflow_dispatch): `test (macos-latest)` 68 s,
  `bundle-macos` 675 s, `test (windows-latest)` 176 s, `lint` 175 s, mind success — a tag / dispatch ág
  visszahozza a macOS lábat.
- Új darabszám PR-futásonként: ubuntu ~5 számlázott perc + Windows 3 × 2 = 6, macOS 0 (előtte +30–40).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-19`
