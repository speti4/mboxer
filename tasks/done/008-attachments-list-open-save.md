---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M2 Viewer MVP
plan: folytassuk-a-008-taskkal-transient-dolphin.md
---

# 008 — Attachments: list, open, save

## Context

Today the reader shows only the MIME tree listing, the attachments cannot be reached; the 007
sanitizer stripped the `cid:` images and left the content id in `data-mboxer-cid` on the `<img>`.
008 adds the attachment strip under the headers (name, size, type), a click on the chip opens
with the default application (temp file), Save… and Save all…, and fills in the cid images in the
sandboxed iframe. Decoding is on demand from the message offset (D-003), the index does not change.
Why: the archive is often opened for one file (user decision 2026-10-17, MVP must-have).
Done when: base64 and quoted-printable attachments open and can be saved byte for byte, the
cid-referenced inline images show in the HTML view.

Planning decisions 2026-10-17 (made in the plan, the user approved the plan): the strip under the
headers; named inline images in the strip too with an `inline` marker, nameless ones only serve the
cid; executable extensions are Save only (Open disabled); Mark-of-the-Web on the temp and the
saved files; temp `%TEMP%\mboxer\<offset>-<part>\`, emptied at start-up; text attachments
transfer-decoded from the raw bytes (byte-exact csv); no new crate.

## Steps

1. [x] core `message.rs`: `walk_parts`, `PartInfo.index/content_id/disposition`, `Attachment` +
   `attachments`, `is_inline`, `part_at`, `part_bytes`; `lib.rs` export; fixture
   `attachments.mbox`; tesztek `tests/fixtures.rs`
2. [x] app `attachments.rs`: `read_attachment`, `safe_file_name`, `open_blocked`, `unique_path`,
   `write_with_motw`, temp dir + `clean_temp`; unit tesztek
3. [x] `types.rs` `AttachmentView`, `PartView.index`; `view.rs` `attachments` + teszt;
   `commands.rs` `read_part`, `open_attachment`, `save_attachment`, `save_all_attachments`;
   `state.rs` `last_save_dir`; `lib.rs` handler + `clean_temp`
4. [x] `index.html` + `style.css`: melléklet-sáv, chipek, Save all
5. [x] `main.js`: `renderAttachments`, open / save / save all, `fillCidImages` blob URL-lel,
   revoke message-váltáskor, height sync
6. [x] Node stub-DOM check a pure helperekre; Chrome check: blob kép a srcdoc frame-ben
7. [x] README reader bekezdés; Log, Deviations, Verification
8. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; kézi kör a
   fixture-ön és a 14 GB-on (user); `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 008 promoted to wip
- 2026-10-17 — core: `walk_parts` (document order, nested rfc822 expanded; the position is the part's
  address), `part_tree` on top of it with `index` / `content_id` / `disposition`, `attachments()`
  (the outer message's `mail-parser` attachments, a nested message named `<subject>.eml`),
  `is_inline` shared with `summarize`, `part_at`, `part_bytes` (binary: `contents()`; text and
  unencoded rfc822: the raw range of the owner transfer-decoded, the boundary's line break
  trimmed). Found: `contents()` of an unencoded `message/rfc822` part is the whole outer message in
  mail-parser 0.11 (its `raw_message` is the outer buffer), so the eml and the sizes come from the
  raw range. Fixture `attachments.mbox` (inline PNG with cid, QP windows-1250 csv, RFC 2231 PDF,
  rfc822, `report.exe`, `..\..\sub/evil.txt`; a second message with a nameless inline gif and an
  upper-case `CID:` reference). A quoted filename's `\` is an RFC 2822 quoted-pair, the fixture
  doubles them. 23 + 8 core tests green
- 2026-10-17 — app: `attachments.rs` (`read_attachment` refuses body parts, `read_listed_attachments`,
  `safe_file_name`, `extension_of`, `open_blocked`, `unique_path`, `write_with_motw`, `write_temp`,
  `clean_temp`), commands `read_part` (raw `ipc::Response`), `open_attachment`, `save_attachment`,
  `save_all_attachments` with a shared `blocking` helper and `open_path`, `last_save_dir` in the
  state, `AttachmentView` with `canOpen` from the Rust blocklist (one source of truth, no JS copy),
  `SaveAllReport`; `clean_temp` on a thread at start. 24 app tests green, clippy clean
- 2026-10-17 — UI: strip under the headers (`#attachments`, chips = Open, `Save…` beside each,
  `Save all…` from 2 up), `fillCidImages` in the frame's load handler (blob URLs cached per message
  in `state.cidUrls`, revoked in `resetBodyState`), `formatBytes` got a bytes branch, `flashStatus`
  a duration. Node check 25 cases green; Chrome: both cid images load from blob under the frame
  CSP (`naturalWidth` 1), the unknown cid keeps its placeholder, no resource-timing entry
- 2026-10-17 — `target\debug\mboxer.exe` on a scratchpad copy of `attachments.mbox`: index built in
  75 ms, 2 messages, window left open for the user's manual round
- 2026-10-17 — implementation committed as `508ddef`
- 2026-10-17 — user's manual round on the fixture and the 14 GB archive: "minden ok". Follow-ups:
  none filed; the `img`-only cid filling stays an accepted limit under Deviations. Closed

## Deviations from the plan

- The executable blocklist lives in Rust only: `AttachmentView.canOpen` tells the webview which
  chip to disable; the plan had the list in `main.js` too.
- Nested `message/rfc822`: the plan assumed `contents()` gives the nested bytes; mail-parser 0.11
  returns the outer buffer for an unencoded one, so `part_bytes` slices the raw range (and uses the
  decoded buffer for an encoded one). `part_tree` / `attachments` sizes are `part_bytes().len()`.
- `fillCidImages` fills `img` elements only (not `source`); other cid references keep the
  placeholder — accepted limit.
- `formatBytes` shows `N B` under 1 kB (the progress overlay too; harmless).

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  24 app (new: `safe_file_name` incl. path components, forbidden characters, trailing dots, reserved
  names, the length cap; blocklist; `unique_path`; MotW stream read back; fixture attachments and the
  "not an attachment" refusals; temp folder; `AttachmentView` / `SaveAllReport` serde, `canOpen`
  false for `Report.EXE`; `attachments_travel_with_the_view`) + 4 + 5 CLI + 23 core unit (boundary
  line break) + 8 fixtures (`attachments_are_listed_and_decoded_byte_for_byte`: six names in order,
  PNG signature + IEND and size, csv bytes `N\xe9v;\xd6sszeg\r\n…` exactly, `%PDF` prefix, eml
  starts with `From: Carol` and holds no `report.exe`, exe `MZ`, 7bit text exact, nameless inline
  gif in message 2) + 12 index pass.
- 2026-10-17 — `node --check ui/main.js` clean; stub check of `formatBytes`, `normalizeCid` (brackets,
  percent-decoding, a bad escape kept), `findCidAttachment` (exact, bracketed, case-insensitive
  fallback, exact beats fallback, unknown, empty), `listedAttachments`, names and icons: 25 cases,
  0 failures.
- 2026-10-17 — Chrome (sanitizer block of `main.js` + `findCidAttachment`, page from 127.0.0.1): three
  `data-mboxer-cid` images after sanitizing; after blob URLs: `logo` and `upper` (`CID:Photo@…`)
  loaded with `naturalWidth` 1 and `complete`, `missing` still carries `data-mboxer-cid`; the frame
  CSP is `default-src 'none'; img-src data: blob:; style-src 'unsafe-inline'`; the frame's resource
  timing is empty (blob is no fetch); no console errors.
- 2026-10-17 — the viewer on the fixture copy: `opened attachments.mbox: 2 messages, Built { build_secs:
  0, parse_failures: 0 }, 75 ms`, no panic. User's manual round (PDF open, csv save compared with
  `fc /b`, `report.exe` chip disabled, Save all with ` (2)`, inline image in HTML, Zone.Identifier
  on a saved file, `%TEMP%\mboxer` empty after a restart).
- 2026-10-17 — user's manual round on the fixture copy and on the 14 GB archive: attachments open with
  the default application, Save and Save all write the files, inline images show in the HTML view.
  Reported as "minden ok". Done-when met: base64 and quoted-printable attachments open and save
  correctly, cid-referenced inline images render.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none filed, one accepted limit under Deviations
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
