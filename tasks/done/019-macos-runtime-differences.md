---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M4 macOS
---

# 019 — macOS runtime differences: index home, quarantine, Finder open, UI keys and fonts

## Context

What: `default_index_home` falls back to `$HOME/Library/Application Support/mboxer/index` on macOS (D-005 amended,
CLI help and doc strings follow); `mark_of_the_web` gets a macOS arm writing the `com.apple.quarantine` xattr
(best effort, the `xattr` command, no new crate); `BLOCKED_EXTENSIONS` adds the macOS executable types (`command
tool pkg mpkg dmg app sh scpt applescript workflow terminal webloc inetloc fileloc`); `RunEvent::Opened` (Finder
"Open with", Dock drop) feeds `open_mbox`; the UI's path keys (`files.js`, `main.js queryKey`) become OS-aware,
tooltips show `⌘` on a Mac, the monospace stack becomes `ui-monospace, Menlo, Consolas, monospace`, "follows
Windows" becomes "follows the system"; `files.test.js` takes the OS as a parameter.
Why: the Windows assumptions the 2026-10-18 sweep found outside printing (020) and packaging (018).
Done when: `cargo test` and the node tests pass on both CI runners; on a Mac an opened attachment carries the
quarantine xattr, a read-only mbox folder indexes under `~/Library`, and Finder "Open with" opens the file.
Done in parallel with 036, 038 and 018; the `macos-latest` runner arrives with 018, so this work lands after
018. The on-Mac checks of the done-when are a follow-up (040). Docs: this task writes `docs/usage.md`; 018 writes
`docs/development.md`.

## Steps

1. [x] `mboxer-core`: `default_index_home` with the macOS fallback; D-005 amended in place (an `Amended` line, the
   decision stays `decided`); CLI help and doc strings follow
2. [x] `mboxer-core`: `mark_of_the_web` macOS arm (`xattr -w com.apple.quarantine`, best effort);
   `BLOCKED_EXTENSIONS` gains the macOS executable types, with a test
3. [x] `mboxer-app`: `RunEvent::Opened` feeds `open_mbox` (Finder "Open with", Dock drop)
4. [x] `ui/files.js`, `ui/main.js`: OS-aware path keys and `⌘` tooltips; the monospace stack; "follows the
   system"; `tests/ui/files.test.js` parameterised by OS; `docs/usage.md` follows
5. [x] `cargo test`, the node tests and `cargo check --target aarch64-apple-darwin` pass on Windows
6. [ ] Pull request, CI green (both runners once 018 is merged and the branch is rebased)

## Log

- 2026-10-19 — promoted in the main session, branch `task/019-macos-runtime-differences`
- 2026-10-19 — Index home: `default_index_home` uses the platform's per-user data folder (`user_data_dir`, a pure
  function, unit tested for all three OSes): `%LOCALAPPDATA%` on Windows, `~/Library/Application Support` on
  macOS, `$XDG_DATA_HOME` / `~/.local/share` elsewhere (Linux used to go straight to the temp folder; the XDG path
  is one line, so it went in), then the temp folder as before. D-005 gained an `Amended 2026-10-19:` line, the
  status stays `decided`. The CLI `index` after-help text shows the platform's own spelling through `cfg`
  (`index_help()`).
- 2026-10-19 — Quarantine: on macOS `mark_of_the_web` runs the system's `xattr -w com.apple.quarantine
  "0081;<hex time>;mboxer;"`, output and errors discarded (like the Windows ADS arm); no new crate. The blocked
  extensions list gained the 14 macOS types (test constant `MACOS_BLOCKED_EXTENSIONS`: all blocked in lower and
  upper case, and all present in the main list). On macOS the MotW test checks the attribute with `xattr -p`
  (it runs on the mac runner).
- 2026-10-19 — Finder open: `tauri::Builder::build(...).run(callback)`; the `RunEvent::Opened { urls }` arm is
  `cfg(target_os = "macos")`, on the other platforms the callback is empty. `commands::open_from_system` turns the
  `file://` URLs into paths; until the webview has asked for the start-up paths they wait in
  `AppState::system_opens` (the event can come before the webview loads), afterwards they go out as an
  `open-files` Tauri event. The `initial_path` command became `initial_paths` (`Vec<String>`: argv[1] + the
  waiting system opens); the `core:default` permission covers listening to the event, the capability file did not
  change.
- 2026-10-19 — UI: `files.js` is built by a `create(os)` factory, the default instance with `detectOs(navigator)`
  (`userAgentData.platform`, else `navigator.platform`; synchronous, no IPC, works under Node too). On Windows the
  key is lower case and `/`→`\`, elsewhere only the trailing `/` goes (case stays: one entry for two files is a
  worse bug than two entries for one). `main.js queryKey` = `Files.pathKey`. `nameShortcutsForMac()` rewrites the
  `Ctrl+` text of `title` / `placeholder` / the empty-screen button to `⌘` (`Ctrl+Shift+` → `⇧⌘`) on a Mac.
  The monospace stack in three places (`style.css` ×2, the print CSS in `main.js`). Theme: "follows the system".
  `files.test.js` runs with both OS parameters (40 node tests instead of 30), `docs/usage.md` and
  `docs/reference.md` with the mac paths and the `⌘` note.
- 2026-10-19 — The two obstacles of `cargo check --target aarch64-apple-darwin -p mboxer-app` on Windows:
  `objc2-exception-helper` compiles a C file (no `cc` on the machine) → the crate's own `DOCS_RS=1` switch skips
  it (a check does not link); `tauri::generate_context!()` asks for `icons/icon.png` on a non-Windows target → the
  byte-identical `icon.png` of 018's PR (#61) and the same `tauri.conf.json` line went in, so the rebase goes
  without a conflict.

## Deviations from the plan

- `docs/reference.md` is also edited (one index sentence, the `⌘` note, "follows the system"): CONTRIBUTING asks
  for it with every user-facing change; the task text named only `docs/usage.md`.
- `crates/mboxer-app/icons/icon.png` and the `tauri.conf.json` icon line are added here as well as in 018 (#61),
  byte for byte the same, because the macOS cross-check cannot pass without them.
- Linux index home: `$XDG_DATA_HOME/mboxer/index` (default `~/.local/share`) instead of the temp folder; one
  arm of the same match, recorded in the D-005 amendment.

## Verification

- 2026-10-19, Windows 11, the worktree: `cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings`
  clean; `cargo test` green (111 Rust tests across the crates, the node runner included); `node --check` on
  `ui/files.js`, `ui/main.js`, `ui/theme.js`, `tests/ui/files.test.js`, `tests/ui/theme.test.js` clean;
  `node --test` over the three UI test files: 40 pass, 0 fail (`files.test.js` runs every case for `windows`
  and `mac`, plus `detectOs` and the cross-OS spelling test); `DOCS_RS=1 cargo check --target
  aarch64-apple-darwin -p mboxer-app -p mboxer-cli` and `... -p mboxer-app --tests` finish clean (the
  `RunEvent::Opened` arm, `open_from_system`, the `xattr` arm and the macOS test branch all type-check).
- Not verified here (no Mac): the quarantine attribute on an opened attachment, the `~/Library` index of a
  read-only mbox folder, Finder "Open with" / Dock drop through `RunEvent::Opened`; the macOS CI runner comes
  with 018. These are the follow-up line in the pull request.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the follow-ups are
  in the pull request description and are filed before the merge
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — `tasks_check` without `write` here; the index is regenerated
  before the merge
- [x] moved to `done/` with `closed: YYYY-MM-DD`
