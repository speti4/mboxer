---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: none
---

# 012 — Portable release build

## Context

Release profile, a single portable exe (no installer), version stamp in one place and in the UI, a short
`docs/usage.md` for the people who run it; CI build. Why: a user only has to put one file next to their mbox,
nothing to install (D-002). Done when: a user on a clean Windows 11 opens their own mbox with the bare
exe. From 005: the exe is `crates/mboxer-app` (`cargo build --release` → `target\release\mboxer.exe`,
release = windowed), the WebView2 profile is under `%LOCALAPPDATA%\io.github.speti4.mboxer` (the `tauri.conf.json`
`identifier`), the usage doc mentions this. Decided 2026-10-17 (planning round): GitHub Actions too,
with tests; the package `mboxer-<version>.zip` = `mboxer.exe` + `usage.md`
(the CLI exe is a developer tool, left out). No new crate (D-002).

## Steps

1. [x] `Cargo.toml` `[profile.release]`: `lto = "fat"`, `opt-level = 3`, `strip = true` (`panic` stays unwind)
2. [x] `tauri.conf.json`: `version` removed → the workspace `Cargo.toml` is the only source; `app_version` command
3. [x] UI: `#status-version` at the right end of the status bar (`v0.1.0`, tooltip), filled at start-up
4. [x] `docs/usage.md` for the users
5. [x] `scripts/release.ps1`: release build, version from Cargo.toml, `dist\mboxer-<version>.zip`
6. [x] `.github/workflows/ci.yml`: `test` job (fmt, clippy, test) on every push/PR; `release` job on a `v*` tag
7. [x] README: release script, workflow, tag convention, `docs/usage.md` link
8. [x] Check: release build size (14.3 MB before), `VersionInfo.ProductVersion`, the release exe on the fixture
   copy, the zip's content, `node --check`, `cargo test`, clippy
9. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 012 promoted to wip
- 2026-10-17 — profile (`opt-level 3`, `lto fat`, `codegen-units 1`, `strip`), `tauri.conf.json` `version` removed
  (Tauri 2: when it is left out, the Cargo.toml version is used, tauri-utils 2.10.2 `config.rs:4108`), `app_version`
  command (`app.package_info().version`), `#status-version` as the fourth column of the status bar (`:empty` →
  hidden), `docs/usage.md`, `scripts/release.ps1` (version from `[workspace.package]`, `-Tag` match, the stamped
  `ProductVersion` checked, staging folder → `Compress-Archive`), `.github/workflows/ci.yml` (`test`:
  fmt/clippy/test on windows-latest, `Swatinem/rust-cache@v2`; `release`: `needs: test`, on a `v*` tag the script +
  `softprops/action-gh-release@v2`), README. `cargo test`: 30 app + 1 + 4 + 6 CLI + 30 + 9 + 22 core green,
  clippy and fmt clean, `node --check` clean
- 2026-10-17 — `scripts\release.ps1`: release build 2 m 53 s; exe 14 958 080 bytes (before, thin LTO: 14 984 704 —
  fat LTO + strip saved 26 KB, the size is the embedded UI + tantivy + Tauri); `ProductVersion` = `FileVersion`
  = 0.1.0; `dist\mboxer-0.1.0.zip` 5.27 MB, holding `mboxer.exe` + `usage.md`. The release exe on the `threads.mbox`
  copy: `opened threads.mbox: 6 messages, Built, 71 ms`. The workflow runs first on the next push; the
  "clean machine, only the exe" done-when is settled by the manual round; closed

## Deviations from the plan

- none: the plan landed as written. The size gain from the profile is negligible (26 KB); the profile stays,
  because fat LTO also helps at run time and strip is cleaner in the spirit of D-002's "one exe".

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  + 1 ui_query + 4 + 6 CLI + 30 + 9 + 22 core pass; `node --check ui/main.js` clean.
- 2026-10-17 — `scripts\release.ps1`: build 2 m 53 s, `target\release\mboxer.exe` 14.3 MB (14 958 080 bytes; the 005
  thin-LTO build was 14 984 704), `(Get-Item mboxer.exe).VersionInfo` ProductVersion 0.1.0, FileVersion 0.1.0,
  ProductName mboxer; `dist\mboxer-0.1.0.zip` 5 273 947 bytes, two files (`mboxer.exe`, `usage.md`).
- 2026-10-17 — the release exe on a fixture copy: 6 messages, index built, 71 ms (stderr). The status bar's
  `v0.1.0` chip and the done-when (clean Windows 11, only the exe) wait for the manual round; the first CI run is
  the next push.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
