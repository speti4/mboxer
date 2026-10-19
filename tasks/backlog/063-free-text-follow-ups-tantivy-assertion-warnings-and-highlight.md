---
status: idea
size: S
origin: done/050-colon-in-free-text-reaches-the-query-parser.md
created: 2026-10-19
milestone: M6 Hardening
---

# Free-text follow-ups: the highlight split and the tantivy exclusion assertion

What: two leftovers of task 050. `highlightTerms` in `ui/query.js` highlights whole free words, while the core
analyzer splits a word on punctuation, so `re:számla` finds `Re: számla` but does not highlight it; split the same
way. `free_text.rs` wraps an excluded phrase in an `Excluded` query because tantivy 0.27's `PhraseScorer::seek_danger`
trips a debug assertion under `Exclude`; report it upstream with the fixture that shows it and drop the wrapper once
a fixed tantivy is in.
Why: an unhighlighted hit looks like a false positive, and a workaround without an upstream ticket outlives the bug.
Done when: a highlight test covers `re:számla` against `Re: számla`, and the upstream issue is linked in the Log.
