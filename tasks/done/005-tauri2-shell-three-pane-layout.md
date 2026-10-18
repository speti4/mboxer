---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M2 Viewer MVP
plan: folytassuk-a-backlog-k-vetkez-quirky-summit.md
---

# 005 — Tauri 2 shell with the three-pane layout

## Context

The first item of M2 and the frame that 006 (search + filter panel), 007 (sanitized HTML reader) and 008
(attachments) build on: the Tauri 2 / WebView2 app in the workspace, a file-open dialog, index build with
progress or reuse of a fresh index (D-04), three-pane layout (folders / message list / reader), a
virtualized list sorted by date, keyboard navigation. Done when: the 14 GB mbox opens, the index is
built or reused, and the list scrolls smoothly end to end even in a debug build.

User decisions 2026-10-17 (planning round): the GUI crate produces `mboxer.exe`, the CLI binary becomes
`mboxer-cli.exe` (D-02: one shipped exe); the folders panel is *All mail* + Gmail labels with counts
from the index (a click filters; threads/hierarchy belong to 009); the reader panel is a header block +
plain text body (html→text for HTML-only mail) — 007 replaces the body block with a sandboxed `srcdoc`
iframe.

Build-time crates (D-02 justification, runtime dependencies stay at zero):
- `tauri 2` — WebView2 window, IPC, async runtime, asset embedding; the D-02 shell, one portable exe.
- `tauri-build 2` (build-dep) — Windows icon/version resource, app manifest (Common Controls v6 for the
  dialog), capability schema.
- `tauri-plugin-dialog 2` — native Windows file-open dialog from the Rust side (`blocking_pick_file`).
- `serde` (already in) — the IPC types; `tempfile` (dev, already in) — fixture copy for the app's tests.
- No npm, bundler or tauri CLI: vanilla HTML/CSS/JS under `ui/`, `cargo run -p mboxer-app` starts it.
  MSRV: because of Tauri 2.12 the workspace `rust-version` goes 1.88 → 1.90.

## Steps

1. [x] Workspace: `Cargo.toml` member + deps + `rust-version`, `.gitignore`; CLI bináris → `mboxer-cli` (Cargo.toml, `tests/cli.rs`, scripts, README)
2. [x] Core `index/listing.rs`: `AllMatches` collector, `Listing` (`rows`, `position_of`), `MboxIndex::list`; `hit_from_address` kiemelése
3. [x] Core: `MboxIndex::label_counts` a term-szótárból; `html_words` → `message::html_to_plain_text`
4. [x] Core tesztek `tests/index.rs` (listing, rows, position_of, label_counts, empty) — zöld a tauri előtt
5. [x] App crate váz: `Cargo.toml`, `build.rs`, `tauri.conf.json`, `capabilities/default.json`, `icons/icon.ico` (`scripts/make-icon.py`), `main.rs`/`lib.rs`, placeholder UI; első `cargo run -p mboxer-app`
6. [x] App modulok: `types.rs`, `state.rs` (`open_or_build`), `view.rs`, `folders.rs` + unit tesztek
7. [x] App `commands.rs`: `open_mbox` (dialógus, Channel, cancel), `cancel_open`, `set_query`, `list_rows`, `read_message`, `position_of`
8. [x] Frontend `ui/`: grid + splitterek, virtualizált lista (28 px sor, 200-as blokkok, generation), billentyűk, reader, állapotok, státuszsor
9. [x] Docs: README (build/run, layout), CLAUDE.md Scope
10. [x] Kézi ellenőrzés a 14 GB-on (build progress + cancel, reused open, görgetés, címke-szűrés, hibás fájl) → Verification
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` zöld; follow-upok (007/012 megjegyzés); `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 005 promoted to wip
- 2026-10-17 — core first: `Listing` with a custom `AllMatches` collector (every match, key = date or score,
  ties in document order), `label_counts` from the `labels` term dictionary (`doc_freq` per segment),
  `html_words` moved to `message::html_to_plain_text`; the new listing tests passed on the first run. One
  expectation was wrong, not the code: `Opened` counts 2 in `gmail-crlf.mbox` because the first message
  carries it on a folded header continuation line
- 2026-10-17 — app crate skeleton (`tauri.conf.json` at the crate root with `frontendDist: "ui"`, icon from
  `scripts/make-icon.py`); the first `cargo build -p mboxer-app` compiled the whole Tauri stack without an
  `rc.exe` or icon surprise. Workspace `rust-version` 1.88 → 1.90 (Tauri 2.12 MSRV)
- 2026-10-17 — `types` / `state` / `view` / `folders` / `commands` and the static UI written; `cargo test`
  green on the second try (`OpenMbox` needed `Debug` for `unwrap_err`): 11 app unit + 4 + 5 CLI + 20 + 7 + 11
  core. `cargo clippy --all-targets -- -D warnings` clean on the first run
- 2026-10-17 — debug exe launched headless-checked on the real archive (stderr captured): the 004 index is
  reused, 86 085 messages listed in 61 ms; `README.md` as input → "no mbox envelope … at byte offset 0";
  a missing path → "I/O error … (os error 2)". The visual part (scrolling, keyboard, label filter, cancel
  during a rebuild) is the user's manual run, see Verification
- 2026-10-17 — follow-ups: no new task; two notes appended to existing backlog items instead — 007 (CSP already
  shaped for the srcdoc iframe, `about:srcdoc` under `frame-src 'self'` to verify in WebView2) and 012 (the
  exe is `crates/mboxer-app`, WebView2 profile folder, size options)
- 2026-10-17 — user's manual run passed; closed

## Deviations from the plan

- The open outcome is also logged to stderr (`opened … N messages, Reused/Built, ms`), which the plan did
  not list: it made the headless check on the real archive possible and costs nothing in the windowed
  release build.
- `BodyKind` gained a `raw` variant (an unparsable message shows its bytes) so no message is ever blank.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  11 app unit (`types` serde shapes + `QueryInput` → `SearchQuery`, `folders` order, `view` plain / html-only /
  legacy charsets / raw fallback, `state` build-then-reuse / query change / cancel leaves no index / text
  file rejected) + 4 + 5 CLI (renamed exe) + 20 + 7 + 11 core (3 new listing/label tests) pass.
- 2026-10-17 — `target\debug\mboxer.exe "<14 GB mbox>"` with stderr captured: index reused, 86 085 messages
  listed, 61 ms from the open command to `Ready`; `README.md` → `Failed` with the core's envelope message;
  a missing path → `Failed` with the I/O error.
- 2026-10-17 — `cargo build --release`: `target\release\mboxer.exe` 14.3 MB (the viewer, windowed) and
  `target\release\mboxer-cli.exe` 6.7 MB; `crates/mboxer-app/gen/` stays untracked.
- 2026-10-17 — user's manual run of `target\debug\mboxer.exe "<14 GB mbox>"`: scrollbar drag end to end and
  PageDown held without blank rows, label click narrows the list and the status bar, Up/Down followed by the
  reader with Enter/Esc focus moves, splitter widths kept, Ctrl+O then Cancel leaves the state untouched, a
  non-mbox file shows the red "Open failed" status. Reported as "minden teszt rendben". Done-when met: the 14 GB
  mbox opens on the reused index and the list scrolls through all 86 085 messages in a debug build.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — notes went into 007 and 012, no new item
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
