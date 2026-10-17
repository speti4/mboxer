---
status: idea
size: S
origin: wip/009-gmail-labels-and-threads.md
created: 2026-10-17
milestone: M3 Comfort
---

# Decode RFC 2047-encoded Gmail label names

What: `X-Gmail-Labels` is parsed raw (mail-parser's `parse_raw` for unknown headers), so an encoded-word label (`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) is indexed and shown in its encoded form. Parse it with `MessageParser::new().header_text("X-Gmail-Labels")` (unstructured parse, RFC 2047 decoded, still one `as_text()` value for `split_labels`) and bump `FORMAT_VERSION`, since the stored labels change.
Why: the author's archive has no encoded label (600 MB header sample, task 009), but a colleague's non-ASCII label names would come out garbled in the folder pane and would not match a click.
Done when: a fixture with an encoded label lists it decoded in `label_counts`, the exact and the `any_labels` filters match the decoded name, and an existing index is rebuilt once (format version).
