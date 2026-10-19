---
status: idea
size: S
origin: done/068-highlight-folds-the-letters-the-index-folds.md
created: 2026-10-20
milestone: M6 Hardening
---

# The highlight folds only what the index folds outside the Latin blocks

What: task 068 aligned the highlight's letter table with the index's `AsciiFoldingFilter` for the Latin blocks,
but the highlight still strips combining accents everywhere after NFD, while the index keeps them outside those
blocks (`й` stays `й`, `ά` stays `ά`). A search for `и` therefore marks `й` in the list and the reader although the
index did not match it. Strip accents only where the filter folds them, with a Node test over a Cyrillic and a
Greek word.
Why: a mark the index did not match reads as a hit it did not find; small, but the same gap 068 closed the other
way round.
Done when: `и` leaves `й` unmarked and `α` leaves `ά` unmarked, while the Latin cases of 068 still pass.
