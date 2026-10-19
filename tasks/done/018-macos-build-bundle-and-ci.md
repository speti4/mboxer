---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M4 macOS
---

# 018 — macOS build, `.app` bundle and CI

## Context

What: a `scripts/bundle-macos.sh` that builds `aarch64-apple-darwin` + `x86_64-apple-darwin`, joins them with
`lipo`, assembles the `.app` (`Info.plist` with the Cargo.toml version and the `io.github.speti4.mboxer`
identifier, `.mbox` document type, `icon.icns` via `iconutil` from PNGs that `make-icon.py` also emits),
ad-hoc signs it (`codesign -s -`) and packs `dist/mboxer-<version>-macos.zip` (`.app` + `usage.md`) with
`ditto`; `ci.yml` gets `macos-latest` in the `test` matrix and a second release artefact, `deny.toml` the two
macOS targets in `[graph] targets` (D-011). No Tauri CLI (D-002).
Why: the first of three macOS tasks (plan 2026-10-18); it proves the workspace compiles and tests pass on macOS
before 019 and 020. First step on Windows: `cargo check --target aarch64-apple-darwin -p mboxer-app`.
Done when: the `macos-latest` CI job is green and the release zip holds an `.app` that starts on a Mac
(manual round on a Mac, 040; the Gatekeeper step is D-007).
Done in parallel with 036, 038 and 019. The script is verified by the CI job, and the "starts on a Mac"
check is a follow-up (040).
Docs: this task writes `docs/development.md` (build and bundle); 019 writes `docs/usage.md`.

## Steps

1. [x] `rustup target add aarch64-apple-darwin x86_64-apple-darwin`; `cargo check --target aarch64-apple-darwin
   -p mboxer-app` passes on Windows (fix what the check finds, inside this task's scope)
2. [x] `scripts/make-icon.py` also emits the PNG set an `iconset` needs (16…512 at 1x and 2x)
3. [x] `scripts/bundle-macos.sh`: the two target builds, `lipo`, `.app` layout with `Info.plist` (version from
   `Cargo.toml`, `io.github.speti4.mboxer`, the `.mbox` document type), `iconutil`, `codesign -s -`, `ditto` zip
   of `.app` + `docs/usage.md` into `dist/mboxer-<version>-macos.zip`
4. [x] `.github/workflows/ci.yml`: `macos-latest` in the `test` matrix; the release job builds and uploads the
   macOS zip next to the Windows one (checksums cover both, 025)
5. [x] `deny.toml`: the two macOS targets under `[graph] targets`; `cargo deny check` and `uvx zizmor .github` pass
6. [x] `docs/development.md`: the macOS build and bundle steps
7. [x] Pull request, CI green on both runners

## Log

- 2026-10-19 — promoted in the main session, branch `task/018-macos-bundle-and-ci`
- 2026-10-19 — a darwin `cargo check` on Windows stopped on two things, neither in our code: the
  `objc2-exception-helper` build script asks for a C compiler (needed only for linking; the `DOCS_RS` variable
  makes it skip itself, documented in development.md), and tauri-codegen expects a PNG icon on every non-Windows
  target (`icons/icon.png`, written by `make-icon.py`, listed in `tauri.conf.json`). The sources had no
  `cfg(windows)` gap: the mark of the web in `attachments.rs` was already gated.
- 2026-10-19 — the bundle script, the iconset PNGs (committed, like `icon.ico`, so the macOS runner needs no
  Pillow), CI (test matrix, a `bundle-macos` job on a tag or a manual run, the release job joins the two zips and
  the checksums), `deny.toml`, the release notes footer, development.md. PR #61.

## Deviations from the plan

- The `bundle-macos` job runs not only on a tag but also on `workflow_dispatch`: without a Mac this is the only way
  the script really runs (lipo, iconutil, codesign, ditto), and it is also how a build for a manual test is made
  (the zip is an artifact for a day). The `commit-rules` job is skipped on a manual run (no commit range).
- The release notes footer (`scripts/changelog.py --notes`) also names the macOS zip and the first-start step:
  `SHA256SUMS.txt` covers two zips, and the text described only one.

## Verification

- 2026-10-19 — on Windows: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`, then `DOCS_RS=1 cargo
  check --target aarch64-apple-darwin -p mboxer-app -p mboxer-cli` and the same for `x86_64-apple-darwin`: both
  `Finished`. `python -I scripts/make-icon.py`: the ten `icon.iconset/*.png` and `icon.png` appeared, `icon.ico`
  stayed byte-identical (`git status` showed no change). `bash -n scripts/bundle-macos.sh` and
  `uvx --from shellcheck-py shellcheck -s bash scripts/bundle-macos.sh`: clean. `cargo fmt --check`, `cargo
  clippy --all-targets -- -D warnings`, `cargo test` (every test ok), `cargo deny check` (three targets:
  advisories, bans, licenses, sources ok, no new licence needed), `uvx zizmor .github` (no findings), `python -I
  scripts/changelog.py --check` (up to date).
- 2026-10-19 — CI, PR #61 (run 39241983968): `test (windows-latest)` 3m04s, `test (macos-latest)` 3m53s, `deny`,
  `workflow-lint`, `commit-rules` all pass. A manual run on the branch (`gh workflow run ci.yml`, run
  39241991149): `bundle-macos` success in 11.5 minutes (two release builds 6m10s + 4m40s, `lipo -archs` →
  `x86_64 arm64`, 23 MB binary, `iconutil`, `codesign --verify --deep --strict`, `ditto`; the zip 9 MB), the
  `macos-zip` artifact uploaded (mboxer-0.1.0-macos.zip + SHA256SUMS.txt, kept for a day). Not checkable from
  here: that the `.app` starts on a Mac, and the Gatekeeper step (D-007) — a follow-up in the PR.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (follow-ups listed
  in the PR description and filed before the merge)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is regenerated when the PR is prepared)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
