---
status: ready
size: L
origin: session 2026-10-17
created: 2026-10-17
milestone: M1 Index core
---

# Sidecar full-text index with accent folding

What: Build a persistent index next to the mbox (tantivy or equivalent): fields for from, to, cc, subject, date, labels, has-attachment, attachment names, body text; accent- and case-insensitive tokenizer; message offsets stored so the reader can seek. Reuse the index when the mbox is unchanged (size + mtime + hash sample), rebuild otherwise.
Why: The whole point of the project: body search in under a second on 14–50 GB (D-02).
Done when: Index build runs end to end on the 14 GB file; reopening is under 1 s; a body search for `szamla` returns the same hits as `számla` in under 1 s.
