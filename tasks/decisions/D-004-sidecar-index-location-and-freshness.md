---
status: decided
date: 2026-10-17
cost: medium
sources: [done/003-fulltext-sidecar-index-accent-folding.md, D-002-rust-tauri2-sidecar-fulltext-index.md, folytassuk-a-003-taskkal-parsed-panda.md]
---

# D-004 — The sidecar index lives in `.mboxer/<mbox name>/` beside the mbox and is rebuilt whole when stale

## Context

D-002 fixed that search runs on a one-time sidecar index built next to the mbox. Task 003 had to
settle where exactly the index sits, how a later open decides whether it may reuse it, and what
happens when it may not. A 14 GB Takeout export is never edited in place; a new export is a new
file. Users may keep several mbox files in one folder, and the index must never be mistaken for
the user's data.

## Decision

- Engine: tantivy (`mmap` + `lz4-compression` features only; no C code in the build).
- Location: `<folder of the mbox>/.mboxer/<mbox file name>/` — one directory per mbox, hidden by
  convention, deletable without loss (it is a cache). The build writes into a `<name>.building`
  sibling and renames it into place only after the commit and the metadata file succeeded.
- Freshness: `mboxer.json` in the index directory records a format version and a fingerprint of
  the mbox (size, mtime, FNV-1a 64 over the first and last MiB plus 16 × 64 KiB sampled blocks).
  On open, any mismatch — fingerprint, format version, unreadable metadata — makes the index
  stale, and a stale index is rebuilt from scratch. There is no incremental update.
- Message identity inside the index is the D-003 pair (`offset`, `len`); hits are served from
  stored fields, the mbox is only seeked when a message is opened.
- Tokenization: words, lower-cased, ASCII-folded (`AsciiFoldingFilter`, covers ő/ű) on both the
  index and the query side; phrase positions are kept on every text field.

Proposed on 2026-10-17 during task 003 planning; decided on 2026-10-17 after the 14 GB run.

## Not chosen

- Index under `%LOCALAPPDATA%\mboxer\<hash>`: survives a read-only mbox location, but is lost
  when the mbox moves and invisible to the user; kept as a fallback rather than the default (became D-005).
- A full-file hash as the fingerprint: 107 s on the 14 GB file at every open.
- Incremental indexing of appended messages: Takeout never appends; a new export is a new file.
- A hand-written key=value metadata file instead of JSON: serde is already in the build through tantivy.
- zstd doc-store compression: pulls in a C build for a marginal size gain on small stored fields.
