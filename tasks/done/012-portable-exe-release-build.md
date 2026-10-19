---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: 12-14-15-taskokat-fejlessz-k-le-keen-whale.md
---

# 012 — Portable release build

## Context

Release profile, a single portable exe (no installer), version stamp in one place and in the UI, a short
`docs/usage.md` for the people who run it; CI build. Why: a user only has to put one file next to their mbox,
nothing to install (D-002). Done when: a user on a clean Windows 11 opens their own mbox with the bare
exe. From 005: the exe is `crates/mboxer-app` (`cargo build --release` → `target\release\mboxer.exe`,
release = windowed), the WebView2 profile is under `%LOCALAPPDATA%\io.github.speti4.mboxer` (the `tauri.conf.json`
`identifier`), the usage doc mentions this. User decision 2026-10-17 (planning round): GitHub Actions too,
with tests ("a gh action with tests"); the package `mboxer-<version>.zip` = `mboxer.exe` + `usage.md`
(the CLI exe is a developer tool, left out). No new crate (D-002).

## Steps

1. [x] `Cargo.toml` `[profile.release]`: `lto = "fat"`, `opt-level = 3`, `strip = true` (`panic` marad unwind)
2. [x] `tauri.conf.json`: `version` törlése → a workspace `Cargo.toml` az egyetlen forrás; `app_version` command
3. [x] UI: `#status-version` a státuszsor jobb szélén (`v0.1.0`, tooltip), induláskor töltve
4. [x] `docs/usage.md` a felhasználóknak
5. [x] `scripts/release.ps1`: release build, verzió a Cargo.toml-ból, `dist\mboxer-<verzió>.zip`
6. [x] `.github/workflows/ci.yml`: `test` job (fmt, clippy, test) minden pushra/PR-ra; `release` job `v*` tagre
7. [x] README: release script, workflow, tag-konvenció, `docs/usage.md` link
8. [x] Ellenőrzés: release build mérete (előtte 14,3 MB), `VersionInfo.ProductVersion`, release exe a fixture-másolaton,
   zip tartalma, `node --check`, `cargo test`, clippy
9. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 012 promoted to wip
- 2026-10-17 — profil (`opt-level 3`, `lto fat`, `codegen-units 1`, `strip`), `tauri.conf.json` `version` törölve
  (Tauri 2: elhagyva a Cargo.toml verziója, tauri-utils 2.10.2 `config.rs:4108`), `app_version` command
  (`app.package_info().version`), `#status-version` negyedik oszlop a státuszsorban (`:empty` → rejtett),
  `docs/usage.md`, `scripts/release.ps1` (verzió a `[workspace.package]`-ből, `-Tag` egyezés, a stampelt
  `ProductVersion` ellenőrzése, staging mappa → `Compress-Archive`), `.github/workflows/ci.yml` (`test`:
  fmt/clippy/test windows-latest-en, `Swatinem/rust-cache@v2`; `release`: `needs: test`, `v*` tagre a script +
  `softprops/action-gh-release@v2`), README. `cargo test`: 30 app + 1 + 4 + 6 CLI + 30 + 9 + 22 core zöld,
  clippy és fmt tiszta, `node --check` tiszta
- 2026-10-17 — `scripts\release.ps1`: release build 2 m 53 s; exe 14 958 080 byte (előtte, thin LTO: 14 984 704 —
  a fat LTO + strip 26 KB-ot hozott, a méret az embedded UI + tantivy + Tauri); `ProductVersion` = `FileVersion`
  = 0.1.0; `dist\mboxer-0.1.0.zip` 5,27 MB, benne `mboxer.exe` + `usage.md`. A release exe a `threads.mbox`
  másolatán: `opened threads.mbox: 6 messages, Built, 71 ms`. A workflow a következő push-nál fut először; a
  "tiszta gépen, csak az exe" done-when a user körében dől el; closed

## Deviations from the plan

- none: the plan landed as written. A méretnyereség a profilból elhanyagolható (26 KB), a profil marad, mert a
  fat LTO-nak futásidő-haszna is van és a strip a D-002 "egy exe" szellemében tisztább.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  + 1 ui_query + 4 + 6 CLI + 30 + 9 + 22 core pass; `node --check ui/main.js` clean.
- 2026-10-17 — `scripts\release.ps1`: build 2 m 53 s, `target\release\mboxer.exe` 14,3 MB (14 958 080 byte; a 005
  thin-LTO buildje 14 984 704), `(Get-Item mboxer.exe).VersionInfo` ProductVersion 0.1.0, FileVersion 0.1.0,
  ProductName mboxer; `dist\mboxer-0.1.0.zip` 5 273 947 byte, két fájl (`mboxer.exe`, `usage.md`).
- 2026-10-17 — a release exe egy fixture-másolaton: 6 üzenet, index épült, 71 ms (stderr). A státuszsor `v0.1.0`
  chipje és a done-when (tiszta Windows 11, csak az exe) a user kézi körére vár; a CI első futása a
  következő push.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
