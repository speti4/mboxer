---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 060 — Index quality follow-ups: token length, completions, nested parts, limits, multi-thread order

## Context

What: smaller findings in `crates/mboxer-core`: `RemoveLongFilter` in `schema.rs` measures bytes before accent
folding, so a 40-character ASCII or a 20-character accented word is dropped, and `MAX_TOKEN_CHARS` is misnamed; the
prefix completion in `search.rs` takes the first 200 terms in dictionary order, which a short prefix over a large
archive spends on hex and base64 junk; `build.rs` skips an HTML-only nested `message/rfc822` when the outer mail has
a plain part; `limit` and `offset` reach tantivy uncapped (a huge `--limit` aborts the process); `read_at_from`
ignores the build's `max_message_bytes`; the tie order comment in `listing.rs` holds for one indexing thread only
and every test builds with one; `from_addr` and `to_addr` are written and never read; the buffer-reuse comments in
`reader.rs` are stale; a corrupt `mboxer.json` makes `index --force` fail; labels are split on every comma.
Why: each one is a quiet wrong answer or a wasted megabyte on the large archive.
Done when: each point has a test or a measured non-issue noted in the Log, and the comments match the code.
Decided 2026-10-19: the index format may change (`FORMAT_VERSION` bump) only if dropping `from_addr`/`to_addr`
needs it; the label-comma question is settled by a fixture shaped like a Takeout header, not by the real archive.

## Steps

1. [ ] `schema.rs`: the long-token filter counts characters after folding (or a custom filter), the constant and
   its comment say so; a test with a 40-character ASCII and a 25-character accented word
2. [ ] `search.rs` completions: the prefix expansion prefers short or frequent terms (document frequency from the
   term dictionary) over dictionary order, or raises the cap with a cost note; a test with junk tokens ahead of
   the real word
3. [ ] `build.rs`: an HTML-only nested `message/rfc822` is indexed when the outer mail has a plain part; a fixture
4. [ ] `limit` and `offset` capped at the message count in `search.rs` and the CLI; a test with `--limit` far above
5. [ ] `read_at_from` takes the build's `max_message_bytes` (from `mboxer.json`) instead of the default
6. [ ] `listing.rs`: the tie-order comment matches a multi-thread build, and at least one listing test builds with
   `threads: 4`
7. [ ] `from_addr` / `to_addr`: removed from the schema or given a reader; the comments in `reader.rs` about buffer
   reuse match the `mem::take` handoff
8. [ ] a corrupt `mboxer.json` reads as stale, so `index --force` rebuilds; a test
9. [ ] labels: a fixture with a quoted label containing a comma (`X-Gmail-Labels: "Work, Home",Inbox`) decides
   whether the split is wrong; the finding and the fix or the non-issue in the Log
10. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` where
    a behaviour changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
