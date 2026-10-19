---
size: S
origin: done/050-colon-in-free-text-reaches-the-query-parser.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 063 — Free-text follow-ups: the highlight split and the tantivy exclusion assertion

## Context

What: two leftovers of task 050. `highlightTerms` in `ui/query.js` highlights whole free words, while the core
analyzer splits a word on punctuation, so `re:számla` finds `Re: számla` but does not highlight it; split the same
way. `free_text.rs` wraps an excluded phrase in an `Excluded` query because tantivy 0.27's
`PhraseScorer::seek_danger` trips a debug assertion under `Exclude`; prepare the upstream report with the fixture
that shows it, so the wrapper can go once a fixed tantivy is in.
Why: an unhighlighted hit looks like a false positive, and a workaround without an upstream ticket outlives the bug.
Done when: a highlight test covers `re:számla` against `Re: számla`, and the upstream report (title, minimal
reproduction, the tantivy version) is ready in the task file for the maintainer to post.
Decided 2026-10-19: the upstream issue is posted by the maintainer, not by the task; the task delivers the text and
a standalone reproduction (a small Rust test against tantivy alone, or the fixture and the query that trips it).

## Steps

1. [ ] a failing Node test in `crates/mboxer-app/tests/ui/query.test.js`: the highlight of `re:számla`, `10:30`
   and a URL covers the same pieces the core analyzer indexes (`Re`, `számla`, `10`, `30`, …), accent-folded
2. [ ] `highlightTerms` splits a free word on the punctuation the core's tokenizer splits on (read
   `crates/mboxer-core/src/index/schema.rs` and `free_text.rs` for the exact rule) and keeps phrases whole
3. [ ] the tantivy reproduction: a minimal test that builds a two-document index and runs an excluded phrase
   query in a debug build, showing the assertion; its text, the tantivy version and the suggested title in the
   Log; a comment at the `Excluded` wrapper names the report
4. [ ] `cargo test` (Node tests included), `node --check ui/query.js`, `cargo fmt --check`, `cargo clippy
   --all-targets -- -D warnings`

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
