---
status: idea
size: S
origin: session 2026-10-17
created: 2026-10-17
milestone: M3 Comfort
---

# Gmail-style query syntax in the search field

What: Parse `from:`, `to:`, `subject:`, `after:`, `before:`, `has:attachment`, `label:`, quoted phrases, OR and negation in the single search field; the filter panel and the syntax stay in sync.
Why: Power-user speed for the main user; the panel keeps working for the others.
Done when: Each operator has a test; typing an operator reflects in the filter panel and vice versa.
Note from 006: `state.query` in `ui/main.js` is the one model behind the field, the panel and the folder pane (`queryInput()` builds the `QueryInput`), so the operator parser feeds that model and the panel renders from it. The core already treats the word being typed as a prefix (`SearchQuery.prefix_last_word`, `typing_word` in `index/search.rs`): a word carrying `:`, quotes or a leading `-` is left to the parser, and the UI highlighter skips those tokens too (`highlightTerms`); extend both when operators get their own meaning.
