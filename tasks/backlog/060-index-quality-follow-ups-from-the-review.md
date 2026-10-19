---
status: idea
size: M
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Index quality follow-ups: token length, completions, nested parts, limits, multi-thread order

What: smaller findings in `crates/mboxer-core`: `RemoveLongFilter` in `schema.rs` measures bytes before accent folding,
so a 40-character ASCII or a 20-character accented word is dropped, and `MAX_TOKEN_CHARS` is misnamed; the prefix
completion in `search.rs` takes the first 200 terms in dictionary order, which a short prefix over a large archive
spends on hex and base64 junk; `build.rs` skips an HTML-only nested `message/rfc822` when the outer mail has a plain
part; `limit` and `offset` reach tantivy uncapped (a huge `--limit` aborts the process); `read_at_from` ignores the
build's `max_message_bytes`; the tie order comment in `listing.rs` holds for one indexing thread only and every test
builds with one; `from_addr` and `to_addr` are written and never read; the buffer-reuse comments in `reader.rs` are
stale; a corrupt `mboxer.json` makes `index --force` fail; labels are split on every comma, to be checked against an export.
Why: each one is a quiet wrong answer or a wasted megabyte on the large archive.
Done when: each point has a test or a measured non-issue noted in the Log, and the comments match the code.
