---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
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
macOS targets in `[graph] targets` (D-011). No Tauri CLI (D-002).
Why: the first of three macOS tasks (plan 2026-10-18); it proves the workspace compiles and tests pass on macOS
before 019 and 020. First step on Windows: `cargo check --target aarch64-apple-darwin -p mboxer-app`.
Done when: the `macos-latest` CI job is green and the release zip holds an `.app` that starts on a Mac
(manual round by a Mac user; the author has no Mac, the Gatekeeper step is D-007).
Run as a task session under docs/git-workflow.md, in parallel with 036, 038 and 019 (plan 2026-10-19); the
author has no Mac, so the script is verified by the CI job and the "starts on a Mac" check is a follow-up.
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
- 2026-10-19 — a darwin `cargo check` Windowson két dolgon akadt el, egyik sem a mi kódunk: az
  `objc2-exception-helper` build scriptje C fordítót kér (csak a linkeléshez kell; a `DOCS_RS` változóval
  kihagyja magát, ezt dokumentálja a development.md), és a tauri-codegen minden nem-Windows targeten PNG ikont
  vár (`icons/icon.png`, a `make-icon.py` írja, a `tauri.conf.json` listázza). A forrásokban nem volt
  `cfg(windows)` hézag: az `attachments.rs` mark-of-the-web-je már gate-elt.
- 2026-10-19 — a bundle script, az iconset PNG-k (commitolva, mint az `icon.ico`: a macOS runnernek így nem kell
  Pillow), a CI (test matrix, `bundle-macos` job tagre vagy kézi futtatásra, a release job összefűzi a két zipet
  és a checksumokat), `deny.toml`, a release notes lábléce, development.md. PR #61.

## Deviations from the plan

- A `bundle-macos` job nem csak tagre fut, hanem `workflow_dispatch`-re is: a szerző Mac nélkül csak így tudja
  a scriptet valóban lefuttatni (lipo, iconutil, codesign, ditto), és a tesztelő is így kap buildet (a zip egy
  napig artifact). A `commit-rules` job kézi futtatáskor kimarad (nincs commit range).
- A release notes lábléce (`scripts/changelog.py --notes`) is megnevezi a macOS zipet és az első indítás lépését:
  a `SHA256SUMS.txt` két zipet fed, a szöveg eddig egyet írt le.

## Verification

- 2026-10-19 — Windowson: `rustup target add aarch64-apple-darwin x86_64-apple-darwin`, majd `DOCS_RS=1 cargo
  check --target aarch64-apple-darwin -p mboxer-app -p mboxer-cli` és ugyanez `x86_64-apple-darwin`-ra: mindkettő
  `Finished`. `python -I scripts/make-icon.py`: a tíz `icon.iconset/*.png` és az `icon.png` megjelent, az
  `icon.ico` bájtra azonos maradt (a `git status` nem jelezte). `bash -n scripts/bundle-macos.sh` és
  `uvx --from shellcheck-py shellcheck -s bash scripts/bundle-macos.sh`: tiszta. `cargo fmt --check`, `cargo
  clippy --all-targets -- -D warnings`, `cargo test` (minden teszt ok), `cargo deny check` (három target:
  advisories, bans, licenses, sources ok, új licence nem kellett), `uvx zizmor .github` (no findings), `python -I
  scripts/changelog.py --check` (up to date).
- 2026-10-19 — CI, PR #61 (run 39241983968): `test (windows-latest)` 3m04s, `test (macos-latest)` 3m53s, `deny`,
  `workflow-lint`, `commit-rules` mind pass. Kézi futtatás a branchen (`gh workflow run ci.yml`, run
  39241991149): `bundle-macos` success 11.5 perc alatt (két release build 6m10s + 4m40s, `lipo -archs` →
  `x86_64 arm64`, 23 MB bináris, `iconutil`, `codesign --verify --deep --strict`, `ditto`; a zip 9 MB), a
  `macos-zip` artifact feltöltve (mboxer-0.1.0-macos.zip + SHA256SUMS.txt, egy napig él). Ami nem ellenőrizhető
  innen: hogy az `.app` elindul egy Macen és a Gatekeeper-lépés (D-007) — follow-up a PR-ban.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (task session: a
  follow-upok a PR leírásában, a fő session veszi fel őket)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (task session: az indexet a fő session generálja)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
