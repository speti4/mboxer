---
size: L
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M1 Index core
plan: none
---

# 003 — Sidecar full-text index with accent folding

## Context

The heart of M1: a persistent tantivy index goes on top of the 002 streaming reader, next to the mbox
(`.mboxer/<mbox file name>/`), indexing the metadata (from, to, cc, subject, date, labels, has-attachment,
attachment names) and the body text with accent- and case-insensitive tokenization, and storing the
`offset/len` identity of D-003 for the seek. On reopen the index is reused if the mbox has not changed
(size + mtime + sampled hash + format version), otherwise it is rebuilt. Done when: the build runs
through the 14 GB file; reopen < 1 s; a body search for `szamla` returns the same as for `számla`,
in < 1 s.

Decided 2026-10-17: a minimal CLI (`mboxer index`, `mboxer search`) in this task, the filter
flags and the benchmark doc stay in 004; D-003 goes to `decided` with the 003 commit.

Build-time crates (D-002 justification, runtime dependencies stay at zero):
- `tantivy 0.27` (`mmap`, `lz4-compression`; without zstd, which would compile C) — inverted index, tokenizer
  chain with the built-in `AsciiFoldingFilter` (Ő/ő/Ű/ű included), fast fields, mmap reopen; an own index
  would take months. MSRV 1.88 → workspace `rust-version` 1.85 → 1.88.
- `serde 1` + `serde_json 1` — the sidecar's `mboxer.json` (fingerprint, format version); already part of the
  build because of tantivy.
- `tempfile 3` (dev) — test index in a temporary directory; tantivy's `mmap` feature already pulls it in.

## Steps

1. [x] Dependencies: `Cargo.toml` (tantivy, serde, serde_json, tempfile dev; `rust-version = "1.88"`)
2. [x] `mboxer-core/src/index/fingerprint.rs`: the `.mboxer/<name>/` location, `mboxer.json`, FNV-1a sampled hash, `IndexStatus`
3. [x] `index/schema.rs`: `"fold"` tokenizer (Simple → RemoveLong(40) → LowerCaser → AsciiFolding), fields (offset, len, date, thread_id, message_id, from/to/cc, from_addr/to_addr, subject, body, labels, has_attachment, attachment_names)
4. [x] `index/build.rs`: single-pass build into a `.building/` directory, progress + cancel, atomic rename, messages that fail to parse are indexed too, body cap 4 MiB, html-only → `html_to_text`
5. [x] `index/search.rs`: `MboxIndex::open` (Ready / Missing / Stale), `SearchQuery` → QueryParser + programmatic filters, `(Count, TopDocs)` DateDesc / Relevance, `Hit` from the stored fields
6. [x] `error.rs`: `Index`, `IndexMeta`, `Json`, `Cancelled`
7. [x] CLI: `mboxer index <mbox> [--force --threads --quiet]`, `mboxer search <mbox> <query> [--limit --relevance]` with open/search timing
8. [x] Tests: unit (FNV, tokenizer, block selection) + `tests/index.rs` (accents/case, phrase, filters, paging, html-only fixture, fingerprint Fresh/Stale/Missing, cancel, message that fails to parse, `read_at` roundtrip)
9. [x] Run on the 14 GB file under `scripts/peak-memory.ps1`: build time, peak RAM (< 1 GB), index size; `search szamla` = `search számla`, < 1 s; open < 1 s → Verification
10. [x] D-003 → decided; D-004 proposed (sidecar location + freshness rule)
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 003 promoted to wip
- 2026-10-17 — Plan agent review against the cached tantivy 0.27 source changed four plan details before coding:
  tantivy writes its own `meta.json` → ours is `mboxer.json`; `date` is FAST|STORED only (the RangeQuery runs on
  the columnar); `from_addr`/`to_addr` raw fields added for the 004 exact-sender filter; body text capped at
  4 MiB and the writer budget set to 4 × 128 MiB
- 2026-10-17 — `cargo build` clean on the second try (`TopDocs::with_limit(1)` alone is not a collector in 0.27,
  `order_by_score()` added); 20 unit + 7 + 9 integration tests
- 2026-10-17 — html-only fixture failed: mail-parser's `html_to_text` drops the whitespace after a closing tag
  (`<b>Árajánlat</b> a` → `Árajánlata`), so the word was unsearchable → `html_words` inserts a space before every
  tag before calling it (unit test covers it)
- 2026-10-17 — D-003 set to decided; D-004 proposed (sidecar location + freshness rule)
- 2026-10-17 — 14 GB run and search timings under Verification; follow-up 014 filed; closed

## Deviations from the plan

- `Opened::Ready` boxes the `MboxIndex` (clippy `large_enum_variant`); callers deref once.
- The build options expose `memory_per_thread_bytes` instead of one total budget, so a `--threads` change keeps
  the per-thread arena at tantivy's sweet spot.
- `StaleReason::Unreadable` added: a corrupt `mboxer.json` is "rebuild", not an error.
- `Mailbox::display()` moved into the core (the CLI had the same formatting) because the index stores it.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  20 unit + 7 (reader fixtures) + 9 (`tests/index.rs`) integration tests pass (tantivy 0.27.0, cargo 1.99.1).
- 2026-10-17 — real archive, `target\release\mboxer.exe index <14 GB mbox> --quiet` under `scripts\peak-memory.ps1`:
  - messages: 86 085 (= the 002 scan); parse failures: 0
  - build: 122 s (the 002 scan alone was 107 s → the index costs 15 s on top of the disk time; no parse worker needed)
  - peak working set: 313.9 MB (target < 1 GB)
  - index size: 129 MB in `.mboxer\<name>\` (0.9 % of the mbox), `mboxer.json` records format 1 + fingerprint
  - a second `mboxer index` returns "index is up to date" without building
- 2026-10-17 — `mboxer search` on the same index (warm file cache; numbers only, no content recorded):
  - `szamla`, `számla`, `SZÁMLA`, `szamla OR számla`: 1137 hits each — accent and case folding confirmed on
    both sides; search 0.5–0.8 ms, open 7–10 ms (targets: < 1 s each)
  - phrase `"számla melléklet"`: 7 hits; `meeting`: 2326; `ajánlat`: 422; `--relevance` 0.4 ms
  - the top hit's `offset/len` pair is a valid D-003 identity (the integration test proves `read_at` on every hit)
- 2026-10-17 — `tasks_check` (write): 0 errors, 0 warnings; README index regenerated.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — 014 filed
  (read-only location fallback); the "parallel parse" idea is dropped: the build is within 15 % of the raw scan
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
