---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-18
closed: 2026-10-18
milestone: M3 Comfort
plan: backlogb-l-a-task-011-jaunty-castle.md
---

# 011 — Export a message: .eml, print / PDF

## Context

Handing the selected message on: saving as .eml (the raw bytes from the offset, D-03), print and
PDF through the WebView2 (Chromium) print path. Why: handing a found message on is the second most
common step after the search. Done when: the saved .eml opens in Outlook / Thunderbird;
the print preview (and the PDF saved from it) shows the same as the reader.

Planning decisions 2026-10-18 (approved plan; user decision: one `Print…` button, the PDF via the print
preview's "Save as PDF" target, no separate native PDF save, no new crate — D-02): printing goes from a hidden,
script-free `sandbox="allow-same-origin allow-modals"` iframe (`contentWindow.print()`;
WebView2's native print APIs would all print the top-level document, which is the three-pane app); the print
document is built from the reader's finished DOM (headers + the body of the current view, the cid images'
blob URLs resolve same-origin, the search `<mark>`s unwrapped); the .eml is the bytes after the envelope
with the mboxrd `>From ` quoting undone at byte level, with Mark-of-the-Web, with a name derived from the subject;
`Ctrl+P` print, `Ctrl+S` save .eml.

## Steps

1. [x] core `reader.rs`: `eml_bytes`, unit teszt; `lib.rs` export; fixture teszt `tests/fixtures.rs`
2. [x] app `attachments.rs` `eml_file_name` + teszt; `commands.rs` `save_message_eml`; `lib.rs` handler
3. [x] `index.html` + `style.css`: `Save .eml…`, `Print…` a reader-barban, rejtett `#print-frame`
4. [x] `main.js`: `PRINT_STYLE`, `printDocument`, `printMessage`, `saveEml`, `Ctrl+P` / `Ctrl+S`
5. [x] Chrome stub-check a print dokumentumra és a két billentyűre
6. [x] `docs/usage.md` bekezdés; `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
   `node --check`; `tasks_check`; commit
7. [x] User kézi kör a 14 GB archívumon (.eml Outlook/Thunderbird, Print preview → Save as PDF, cid képes
   HTML levél, Text nézet, hosszú levél lapozása)

## Log

- 2026-10-18 — plan approved; 011 promoted to wip
- 2026-10-18 — core: `eml_bytes` (`reader.rs`, one `>` off every `>+From ` line, bytes otherwise untouched),
  unit test + fixture test on `gmail-crlf.mbox` (headers byte for byte, CRLF kept, `From a k=C3=B6nyvel` unquoted,
  the plain `From Anna wrote:` line untouched, length = raw − 1, the second message identical). App: `eml_file_name`
  (subject → `safe_file_name`, else `message-<offset>.eml`), `save_message_eml` command on the `save_attachment`
  pattern (save dialog in the last folder, `.eml` filter, Mark-of-the-Web, `remember_dir`). UI: `Save .eml…` and
  `Print…` at the right end of the reader bar, hidden `#print-frame` (`sandbox="allow-same-origin allow-modals"`,
  0×0, `visibility: hidden`), `PRINT_STYLE`, `printHeader` / `printDocument` / `printMessage` / `saveEml`,
  `Ctrl+P` / `Ctrl+S` in `globalShortcut` (Ctrl+P taken over even without a message so the webview never prints
  the whole window). `cargo test` green (31 app + 1 ui node + 4 + 6 CLI + 31 core unit + 10 fixtures + 22 index),
  clippy and fmt clean, `node --check` ok
- 2026-10-18 — Chrome stub-check (scratchpad `serve-ui-print.js`, 127.0.0.1:8768, stub `invoke` with one message:
  HTML body + cid PNG + plain text + two attachments; `print()` of the frame replaced by a counter through a capturing
  `load` listener): HTML view → the print frame's document has `title` = subject, h1, dt rows From/To/Date/Labels/
  Attachments (`logo.png (68 B), számla.pdf (12 kB)`), the reader frame's CSP meta copied, 0 `mark`, the body
  paragraph, the cid image from `blob:` with `naturalWidth` 1 and `complete`, three `<style>`s, the header first in
  `body`; Text view → `pre.mboxer-print-body` with the plain text, no note, CSP `default-src 'none'; style-src
  'unsafe-inline'`; Source view → `pre.source` with the source; Ctrl+S on the document → one `save_message_eml`
  call `{offset: 0, len: 2000}` and the status line names the saved `.eml` file; Ctrl+P dispatched inside the body
  frame → printed, `defaultPrevented`. Then the real `print()` on the frame: the page's JS blocked (the extension's
  script injection timed out) — the Chromium print preview opened and stayed open, i.e. the call was not ignored
  (an ignored sandboxed `print()` returns at once with a console warning; none logged); the extension cannot
  dismiss a browser dialog, so the tab was closed. The visual check of the preview is the user's manual round
- 2026-10-18 — `docs/usage.md`: "Handing a message on" section, `Ctrl+S` / `Ctrl+P` in the keyboard line
- 2026-10-18 — commit `1777d13` (feat, task 011); `target\debug\mboxer.exe` started on the 14 GB archive for the
  user's manual round
- 2026-10-18 — a user kézi köre a Tauri ablakban, a 14 GB archívumon: "minden működik". Follow-up: none filed;
  a tervben feltételesen említett 1×1 px-es print-frame nem kellett, a 0×0 rejtett frame nyomtat; a natív
  "Save as PDF…" (`PrintToPdf` második ablakból) a user döntése szerint nem kell, nem kerül backlogra. Closed

## Deviations from the plan

- `state.printing` is reset when the frame's `load` handler returns from `print()` (not on `afterprint`), and the
  print document stays in the frame until the message changes: whether or not `print()` blocks until the preview
  closes, the preview keeps its content and a second Ctrl+P while it is open is ignored by Chromium itself.
- No `contentWindow.focus()` before `print()`: Chromium does not need it and it would move the focus off the reader.

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass
  (new: `eml_bytes_undo_the_mboxrd_quoting_only`, `eml_export_keeps_the_bytes_and_unquotes_from_lines`,
  `eml_file_names_come_from_the_subject`), `node --check ui/main.js` clean; Chrome stub-check as in the Log.
- 2026-10-18 — user's manual round in the Tauri window on the 14 GB archive: the .eml opens in the mail client,
  the print preview shows the message as the reader does and Save as PDF writes it, reported as "minden működik".
  Done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none filed
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
