---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M4 macOS
---

# 018 — macOS build, `.app` bundle and CI

## Context

What: a `scripts/bundle-macos.sh` that builds `aarch64-apple-darwin` + `x86_64-apple-darwin`, joins them with
`lipo`, assembles the `.app` (`Info.plist` with the Cargo.toml version and the `io.github.speti4.mboxer`
identifier, `.mbox` document type, `icon.icns` via `iconutil` from PNGs that `make-icon.py` also emits),
ad-hoc signs it (`codesign -s -`) and packs `dist/mboxer-<version>-macos.zip` (`.app` + `usage.md`) with
`ditto`; `ci.yml` gets `macos-latest` in the `test` matrix and a second release artefact, `deny.toml` the two
macOS targets in `[graph] targets` (D-011). No Tauri CLI (D-02).
Why: the first of three macOS tasks (plan 2026-10-18); it proves the workspace compiles and tests pass on macOS
before 019 and 020. First step on Windows: `cargo check --target aarch64-apple-darwin -p mboxer-app`.
Done when: the `macos-latest` CI job is green and the release zip holds an `.app` that starts on a Mac
(manual round by a Mac user; the author has no Mac, the Gatekeeper step is D-07).
Run as a task session under docs/git-workflow.md, in parallel with 036, 038 and 019 (plan 2026-10-19); the
author has no Mac, so the script is verified by the CI job and the "starts on a Mac" check is a follow-up.
Docs: this task writes `docs/development.md` (build and bundle); 019 writes `docs/usage.md`.

## Steps

1. [ ] `rustup target add aarch64-apple-darwin x86_64-apple-darwin`; `cargo check --target aarch64-apple-darwin
   -p mboxer-app` passes on Windows (fix what the check finds, inside this task's scope)
2. [ ] `scripts/make-icon.py` also emits the PNG set an `iconset` needs (16…512 at 1x and 2x)
3. [ ] `scripts/bundle-macos.sh`: the two target builds, `lipo`, `.app` layout with `Info.plist` (version from
   `Cargo.toml`, `io.github.speti4.mboxer`, the `.mbox` document type), `iconutil`, `codesign -s -`, `ditto` zip
   of `.app` + `docs/usage.md` into `dist/mboxer-<version>-macos.zip`
4. [ ] `.github/workflows/ci.yml`: `macos-latest` in the `test` matrix; the release job builds and uploads the
   macOS zip next to the Windows one (checksums cover both, 025)
5. [ ] `deny.toml`: the two macOS targets under `[graph] targets`; `cargo deny check` and `uvx zizmor .github` pass
6. [ ] `docs/development.md`: the macOS build and bundle steps
7. [ ] Pull request, CI green on both runners

## Log

- 2026-10-19 — promoted in the main session, branch `task/018-macos-bundle-and-ci`

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
