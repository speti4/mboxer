---
status: idea
size: S
origin: done/068-highlight-folds-the-letters-the-index-folds.md
created: 2026-10-20
milestone: M6 Hardening
---

# Fold the combining dot of a lower-cased `İ` in the index

What: `fold_analyzer` lower-cases before it folds, and the lower case of `İ` is `i` plus a combining dot above
(`i̇`), which `AsciiFoldingFilter` leaves in place, so `istanbul` does not find `İstanbul`. Fold the dot away in
the analyzer (a token filter step or a normalization before the folding), which changes what the index stores and
needs a `FORMAT_VERSION` bump; the `İ` entry task 068 put into the UI's `LETTER_FOLDS` then goes.
Why: a Turkish capital in a name or a subject is a word nobody finds, and the UI carries a special case for it.
Done when: a core test finds `İstanbul` by `istanbul` through the index, the UI table has no `İ` entry, and the
release notes name the rebuild.
