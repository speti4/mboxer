---
size: S
origin: done/002-rust-workspace-and-mbox-stream-parser.md
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M2 Viewer MVP
plan: folytassuk-a-007-taskkal-stateful-castle.md
---

# 013 — Unescape `>From ` lines in message bodies for display

What: Gmail Takeout escapes body lines that start with `From ` as `>From ` (mboxrd style, 7 cases in a 64 MB sample). The reader (002) leaves them as written; the message reader (007) should strip one `>` from `^>+From ` lines in decoded text parts before display.
Why: Fidelity of the shown text; search is unaffected because the token `From` is still present. Deferred in 002 because mboxo vs mboxrd cannot be told apart from the file, so the rule is a display choice, not a parsing one (D-003).
Done when: A fixture with `>From ` and `>>From ` body lines renders as `From ` and `>From ` in the reader, and the raw bytes returned by `read_at` stay untouched.

## Log

- 2026-10-17 — bundled into task 007 (decided in its planning round): `unescape_from_lines` in
  `crates/mboxer-app/src/view.rs` runs on the plain-text body only; the Source view and `read_at`
  show the bytes as written. Fixture `html-remote.mbox` carries `>From ` and `>>From ` lines.

## Verification

- 2026-10-17 — `cargo test -p mboxer-app`: `mboxrd_quoting_comes_off_for_display_only` (`>From`, `>>From`,
  `From `, `>Frome`, `> From`, CRLF, no trailing newline) and `remote_fixture_carries_its_html_and_unquoted_text`
  (the fixture's two lines come out as `From a szerverről` and `>From kétszer`) pass; see 007 for the manual round.
