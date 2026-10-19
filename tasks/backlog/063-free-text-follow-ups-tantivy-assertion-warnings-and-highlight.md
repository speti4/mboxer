---
status: idea
size: S
origin: wip/050-colon-in-free-text-reaches-the-query-parser.md
created: 2026-10-19
milestone: M6 Hardening
---

# Free-text follow-ups: the tantivy exclusion assertion, the empty warnings, the highlight split

What: three leftovers of task 050. `free_text.rs` wraps an excluded phrase in an `Excluded` query because tantivy
0.27's `PhraseScorer::seek_danger` trips a debug assertion under `Exclude`; report it upstream with the fixture that
shows it and drop the wrapper once a fixed tantivy is in. The backend no longer produces query warnings, so
`SearchResult::warnings`, `Listing::warnings` and the app's `warnings` field are always empty: remove them or give
them a use (a message for a query that has no searchable word). `highlightTerms` in `ui/query.js` highlights whole
free words, while the core analyzer splits a word on punctuation, so `re:számla` finds `Re: számla` but does not
highlight it; split the same way.
Why: a workaround without an upstream ticket outlives the bug, dead fields mislead, an unhighlighted hit looks false.
Done when: the upstream issue is linked in the Log, the fields are gone or used, and a highlight test covers `re:számla`.
