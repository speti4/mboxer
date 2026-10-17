---
size: L
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M1 Index core
plan: folytassuk-a-003-taskkal-parsed-panda.md
---

# 003 — Sidecar full-text index with accent folding

## Context

Az M1 szíve: a 002-es streaming reader fölé egy perzisztens tantivy index kerül az mbox mellé
(`.mboxer/<mbox fájlnév>/`), ami a metaadatot (from, to, cc, subject, date, labels, has-attachment,
attachment names) és a body szöveget indexeli ékezet- és kisbetű-független tokenizálással, és a D-03
szerinti `offset/len` azonosítót tárolja a seek-hez. Újranyitáskor az index újrahasznált, ha az mbox nem
változott (méret + mtime + mintavételes hash + formátumverzió), különben újraépül. Done when: az építés
végigfut a 14 GB-os fájlon; újranyitás < 1 s; body keresés `szamla`-ra ugyanazt adja, mint `számla`-ra,
< 1 s alatt.

User döntések 2026-10-17: minimális CLI (`mboxer index`, `mboxer search`) ebben a taskban, a szűrő
flag-ek és a benchmark doc a 004-ben marad; D-03 `decided`-re áll a 003 commitjával.

Build-time crate-ek (D-02 indoklás, runtime dependency marad nulla):
- `tantivy 0.27` (`mmap`, `lz4-compression`; zstd nélkül, az C-t fordítana) — invertált index, tokenizer-lánc
  beépített `AsciiFoldingFilter`-rel (Ő/ő/Ű/ű benne), fast field, mmap újranyitás; saját index hónapok.
  MSRV 1.88 → workspace `rust-version` 1.85 → 1.88.
- `serde 1` + `serde_json 1` — a sidecar `mboxer.json`-ja (fingerprint, formátumverzió); a tantivy miatt már
  a build része.
- `tempfile 3` (dev) — tesztindex ideiglenes könyvtárban; a tantivy `mmap` feature-je már behúzza.

## Steps

1. [x] Függőségek: `Cargo.toml` (tantivy, serde, serde_json, tempfile dev; `rust-version = "1.88"`)
2. [x] `mboxer-core/src/index/fingerprint.rs`: `.mboxer/<név>/` helye, `mboxer.json`, FNV-1a mintavételes hash, `IndexStatus`
3. [x] `index/schema.rs`: `"fold"` tokenizer (Simple → RemoveLong(40) → LowerCaser → AsciiFolding), mezők (offset, len, date, thread_id, message_id, from/to/cc, from_addr/to_addr, subject, body, labels, has_attachment, attachment_names)
4. [x] `index/build.rs`: egy menetes építés `.building/` könyvtárba, progress + cancel, atomi rename, parse-hibás üzenet is bekerül, body cap 4 MiB, html-only → `html_to_text`
5. [x] `index/search.rs`: `MboxIndex::open` (Ready / Missing / Stale), `SearchQuery` → QueryParser + programozott szűrők, `(Count, TopDocs)` DateDesc / Relevance, `Hit` a tárolt mezőkből
6. [x] `error.rs`: `Index`, `IndexMeta`, `Json`, `Cancelled`
7. [x] CLI: `mboxer index <mbox> [--force --threads --quiet]`, `mboxer search <mbox> <query> [--limit --relevance]` open/search idővel
8. [x] Tesztek: unit (FNV, tokenizer, blokkválasztás) + `tests/index.rs` (ékezet/kisbetű, phrase, szűrők, lapozás, html-only fixture, fingerprint Fresh/Stale/Missing, cancel, parse-hibás üzenet, `read_at` roundtrip)
9. [x] Futtatás a 14 GB-on `scripts/peak-memory.ps1` alatt: build idő, peak RAM (< 1 GB), indexméret; `search szamla` = `search számla`, < 1 s; open < 1 s → Verification
10. [x] D-03 → decided; D-04 proposed (sidecar hely + frissességi szabály)
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` zöld; `tasks_check`; commit

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
- 2026-10-17 — D-03 set to decided (user, planning round); D-04 proposed (sidecar location + freshness rule)
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
  - the top hit's `offset/len` pair is a valid D-03 identity (the integration test proves `read_at` on every hit)
- 2026-10-17 — `tasks_check` (write): 0 errors, 0 warnings; README index regenerated.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — 014 filed
  (read-only location fallback); the "parallel parse" idea is dropped: the build is within 15 % of the raw scan
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
