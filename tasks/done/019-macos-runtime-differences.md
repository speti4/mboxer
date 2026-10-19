---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-gentle-goose.md
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
Run as a task session under docs/git-workflow.md, in parallel with 036, 038 and 018 (plan 2026-10-19); the
`macos-latest` runner arrives with 018, so this branch rebases onto `main` after 018 merges. The author has no
Mac: the on-Mac checks of the done-when are a follow-up. Docs: this task writes `docs/usage.md`; 018 writes
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
- 2026-10-19 — Index home: `default_index_home` a platform per-user adatmappáját használja (`user_data_dir`,
  tiszta függvény, unit teszt mindhárom OS-re): `%LOCALAPPDATA%` Windowson, `~/Library/Application Support`
  macOS-en, `$XDG_DATA_HOME` / `~/.local/share` máshol (Linux eddig egyből a temp alá ment; az XDG út egy sor,
  ezért bekerült), utána marad a temp. D-005 egy `Amended 2026-10-19:` sorral bővült, a státusz `decided`. A CLI
  `index` after-help szövege `cfg`-vel a platform saját írásmódját mutatja (`index_help()`).
- 2026-10-19 — Quarantine: `mark_of_the_web` macOS ágon a rendszer `xattr -w com.apple.quarantine
  "0081;<hex idő>;mboxer;"` parancsát futtatja, a kimenet és a hiba eldobva (mint a Windows ADS ág); nincs új
  crate. A tiltott kiterjesztések listája a 14 macOS típussal bővült (`MACOS_BLOCKED_EXTENSIONS` tesztkonstans:
  mind blokkolt kis- és nagybetűvel, és mind benne van a fő listában). A MotW teszt macOS-en `xattr -p`-vel
  ellenőrzi az attribútumot (a mac runneren fut majd).
- 2026-10-19 — Finder open: `tauri::Builder::build(...).run(callback)`; a `RunEvent::Opened { urls }` ág
  `cfg(target_os = "macos")`, a többi platformon a callback üres. `commands::open_from_system` a `file://`
  URL-eket útvonallá alakítja; amíg a webview nem kérte le az induló útvonalakat, `AppState::system_opens`-ben
  várnak (az esemény a webview betöltése előtt is jöhet), utána `open-files` Tauri eseményként mennek ki. Az
  `initial_path` parancs `initial_paths` lett (`Vec<String>`: argv[1] + a várakozó rendszer-megnyitások); a
  `core:default` jogosultság lefedi az esemény-figyelést, a capability fájl nem változott.
- 2026-10-19 — UI: `files.js` `create(os)` gyárral épül, az alapértelmezett példány `detectOs(navigator)`-ral
  (`userAgentData.platform`, különben `navigator.platform`; szinkron, nincs IPC, Node alatt is megy). Windowson
  a kulcs kisbetűs és `/`→`\`, máshol csak a záró `/` esik le (a kis-nagybetű marad: egy bejegyzés két fájlra
  rosszabb hiba, mint két bejegyzés egyre). `main.js queryKey` = `Files.pathKey`. `nameShortcutsForMac()` a
  `title` / `placeholder` / üres-képernyő gomb `Ctrl+` szövegét `⌘`-re (`Ctrl+Shift+` → `⇧⌘`) írja át Macen.
  Monospace stack három helyen (`style.css` ×2, a nyomtatási CSS a `main.js`-ben). Theme: "follows the system".
  `files.test.js` a két OS-paraméterrel fut (40 node-teszt a korábbi 30 helyett), `docs/usage.md` és
  `docs/reference.md` a mac útvonalakkal és a `⌘` megjegyzéssel.
- 2026-10-19 — A `cargo check --target aarch64-apple-darwin -p mboxer-app` két akadálya Windowson:
  `objc2-exception-helper` C-fájlt fordít (`cc` nincs a gépen) → a crate saját `DOCS_RS=1` kapcsolója kihagyja
  (a check nem linkel); `tauri::generate_context!()` nem-Windows célra `icons/icon.png`-t kér → a 018 PR-jének
  (#61) bájtra azonos `icon.png`-je és ugyanaz a `tauri.conf.json` sor került be, hogy a rebase konfliktus
  nélkül menjen.

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

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — task session:
  the follow-ups are in the pull request description, the main session files them
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — task session: `tasks_check` without `write`, the index is
  regenerated by the main session before the merge
- [x] moved to `done/` with `closed: YYYY-MM-DD`
