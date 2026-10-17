---
status: ready
size: M
origin: session 2026-10-17
created: 2026-10-17
milestone: M1 Index core
---

# Rust workspace and streaming mbox parser

What: Create the Cargo workspace (core library + CLI; the Tauri app joins later) and a streaming mbox reader that splits messages on `From ` lines with a bounded buffer, records each message's byte offset and length, and parses headers and MIME parts (mail-parser or equivalent).
Why: Everything else (index, UI) consumes this; a 50 GB file must never be loaded into memory. mboxShell (MIT) is the reference for the streaming approach.
Done when: `cargo run -- scan <mbox>` walks the 14 GB file with flat memory, prints message count and the top headers of a sample, and unit tests cover a Gmail Takeout sample with nested MIME, quoted-printable and base64 parts.
