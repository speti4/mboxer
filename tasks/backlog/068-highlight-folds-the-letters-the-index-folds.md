---
status: idea
size: S
origin: done/063-free-text-follow-ups-tantivy-assertion-warnings-and-highlight.md
created: 2026-10-19
milestone: M6 Hardening
---

# The highlight folds the same letters the index folds

What: the highlight in `ui/query.js` strips only combining accents (`̀-ͯ` after NFD), while the index's
`AsciiFoldingFilter` also maps letters such as `ø`, `ł`, `æ`, `ß`, `œ` and `đ` to ASCII, so a hit on such a word
is found but not marked. Give the UI the same folding table the core uses (generated from the filter's mapping, or
a small shared list), with a Node test over those letters.
Why: an unhighlighted hit looks like a false positive, the same gap task 063 closed for punctuation.
Done when: a search for `strasse` marks `Straße` and `lodz` marks `Łódź` in the list and the reader.
