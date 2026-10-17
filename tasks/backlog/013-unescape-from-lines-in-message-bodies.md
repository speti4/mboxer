---
status: idea
size: S
origin: wip/002-rust-workspace-and-mbox-stream-parser.md
created: 2026-10-17
milestone: M2 Viewer MVP
---

# Unescape `>From ` lines in message bodies for display

What: Gmail Takeout escapes body lines that start with `From ` as `>From ` (mboxrd style, 7 cases in a 64 MB sample). The reader (002) leaves them as written; the message reader (007) should strip one `>` from `^>+From ` lines in decoded text parts before display.
Why: Fidelity of the shown text; search is unaffected because the token `From` is still present. Deferred in 002 because mboxo vs mboxrd cannot be told apart from the file, so the rule is a display choice, not a parsing one (D-03).
Done when: A fixture with `>From ` and `>>From ` body lines renders as `From ` and `>From ` in the reader, and the raw bytes returned by `read_at` stay untouched.
