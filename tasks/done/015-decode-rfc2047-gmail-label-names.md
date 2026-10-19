---
size: S
origin: done/009-gmail-labels-and-threads.md
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: none
---

# 015 — Decode RFC 2047-encoded Gmail label names

## Context

mail-parser returns the `X-Gmail-Labels` header raw as an unknown header (`parse_raw`), so an
encoded-word label (`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) lands in the index and the folders pane in encoded form,
and a click does not find it. Solution: setting `header_text("X-Gmail-Labels")` on the parser (unstructured parse,
RFC 2047 decoded, still one `as_text()` value for `split_labels`) and `FORMAT_VERSION` 1 → 2, because
the stored labels change. Why: the test archive has no encoded label (600 MB header sample, 009), but
another user's non-ASCII label names would show up garbled. Done when: a fixture with an encoded label lists the
label decoded in `label_counts`, the exact and the `any_labels` filter find the decoded name, and the existing index
is rebuilt once (format version). Decided 2026-10-17 (planning round): order 015 → 014 → 012, one
commit per task; no new crate (D-002).

## Steps

1. [x] `message.rs`: one shared `LazyLock<MessageParser>` with `header_text("X-Gmail-Labels")`
2. [x] `fingerprint.rs`: `FORMAT_VERSION` 2
3. [x] Fixture `labels-encoded.mbox` (Q-encoded, B-encoded with a comma and a `/`, raw UTF-8)
4. [x] Tests: `tests/index.rs` (`label_counts`, exact, `any_labels`, stored labels), `tests/fixtures.rs` (`summarize`)
5. [x] README "Labels" paragraph
6. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; 14 GB: one rebuild, then "up to date"
7. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 015 promoted to wip
- 2026-10-17 — first attempt with `MessageParser::default().header_text("X-Gmail-Labels")`: 4 fixture tests failed
  (Subject stayed encoded, Message-ID with `<>`, no attachment). Cause: mail-parser 0.11 uses its built-in
  header table only with an empty `header_map` (`parsers/header.rs:35`); anything configured → only the
  configured headers. So `PARSER` lists the whole built-in table (`with_mime_headers` +
  `with_date_headers` + `with_address_headers` + `with_message_ids` + Subject/Comments text, List-* address,
  Return-Path id, Keywords/Content-Language comma, Received, MIME-Version raw) and adds the `X-Gmail-Labels`
  text parse to it; every other unknown header stays raw. One `LazyLock` static, not a parser built per message
- 2026-10-17 — fixture `labels-encoded.mbox` (3 messages: Q-encoded, B-encoded `Munka/Számlák,Inbox` in one encoded
  word, raw UTF-8); `cargo test`: 30 app + 1 + 4 + 5 CLI + 29 + 9 + 20 core green; clippy, fmt clean
- 2026-10-17 — a `target\debug\mboxer.exe` left running (010 manual round) locked the binary; closed
- 2026-10-17 — 14 GB: release CLI `index` → format 1 → 2 rebuild 126.9 s, 86,085 messages, 0 parse failures,
  index 134.8 MB; second run "index is up to date"; closed

## Deviations from the plan

- The parser is not `default() + header_text` but the built-in table listed by hand + `header_text` (see Log):
  mail-parser's behaviour allows no other way; the fixture tests (Subject, From, Message-ID, Date,
  attachments, legacy charsets) show that the table is complete.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  + 1 ui_query + 4 + 5 CLI + 29 core unit + 9 fixtures (new: `encoded_label_names_are_decoded_in_the_summary`)
  + 20 index (new: `encoded_label_names_are_decoded_before_they_are_stored` — `label_counts` =
  `Inbox 3, Munka/Számlák 1, Számlák 2`; exact `Számlák` 2 hits, no `=?` in the stored labels;
  `any_labels [Munka, Munka/Számlák]` 1 hit; the encoded form as a label 0 hits) pass.
- 2026-10-17 — the 14 GB archive: `mboxer-cli index` rebuilt once (format 1 → 2, 126.9 s, 86,085 messages,
  0 parse failures, 134,833,535 bytes), the second `index` "up to date"; `search --label Inbox --count` 65,646,
  `search számla --count` 1137 (= 003), open 8.7 ms. Done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
