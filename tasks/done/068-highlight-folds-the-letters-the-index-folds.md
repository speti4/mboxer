---
size: S
origin: done/063-free-text-follow-ups-tantivy-assertion-warnings-and-highlight.md
created: 2026-10-19
started: 2026-10-20
closed: 2026-10-20
plan: none
milestone: M6 Hardening
---

# 068 — The highlight folds the same letters the index folds

## Context

What: the highlight in `ui/query.js` strips only combining accents (`̀-ͯ` after NFD), while the index's
`AsciiFoldingFilter` also maps letters such as `ø`, `ł`, `æ`, `ß`, `œ` and `đ` to ASCII, so a hit on such a word
is found but not marked. Give the UI the same folding table the core uses (generated from the filter's mapping, or
a small shared list), with a Node test over those letters.
Why: an unhighlighted hit looks like a false positive, the same gap task 063 closed for punctuation.
Done when: a search for `strasse` marks `Straße` and `lodz` marks `Łódź` in the list and the reader.
The core side is `fold_analyzer` in `crates/mboxer-core/src/index/schema.rs` (tantivy's `AsciiFoldingFilter`);
the UI side is `foldChar` / `foldWithMap` / `foldText` in `crates/mboxer-app/ui/query.js`, and `foldWithMap` must
keep its index map right when one letter folds to two (`ß` → `ss`, `æ` → `ae`).

## Steps

1. [x] the folding table for the UI: the letters `AsciiFoldingFilter` maps beyond combining accents, as a small
   table in `query.js` (or a generated file with a core test that keeps it in step with the filter); the choice
   in the Log
2. [x] `foldChar` applies the table after the NFD strip; `foldWithMap` maps a one-to-many fold so a highlight
   range still lands on the original letters
3. [x] Node tests in `crates/mboxer-app/tests/ui/` over `Straße` / `strasse`, `Łódź` / `lodz`, `Ørsted`,
   `œuvre`, `đ`, and a core test that the same words fold the same way through `fold_analyzer`
4. [x] `node --check`, the Node tests, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
   `cargo test --locked`, the macOS-target clippy and the `rust-version` check
5. [x] `docs/reference.md` where it describes the highlight, if its wording changes

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-20 — the gap, measured. A throwaway test folded every alphanumeric character from U+0080 up through
  `fold_analyzer` and compared it with `foldChar`: the filter maps 1,135 of them, and `foldChar` missed 640, most
  of them outside the letters people write in mail (Latin Extended-B, IPA, small capitals, circled and
  parenthesized digits, full-width forms). In Latin-1, Latin Extended-A, Latin Extended Additional and the Latin
  ligatures the misses were 48: `¹²³`, `æ ð ø þ ß đ ħ ı ĳ ĸ ŀ ł ŉ ŋ œ ŧ ſ` and their capitals, `ẞ`, `ẚ ẛ ẜ ẝ ỻ ỿ`
  and their capitals, `ﬀ ﬁ ﬂ ﬃ ﬄ ﬆ`, and `İ`, which the core lower-cases to `i` plus a combining dot and then
  keeps (the filter does not touch combining marks). The other way round, NFD drops accents the filter keeps
  outside the Latin blocks (`й`, `ά`), so the highlight marks a little more than the index matched there.
- Decided 2026-10-20: a small hand table in `query.js` (`LETTER_FOLDS`, 33 lower-case keys) on top of the NFD
  strip, and a test in `crates/mboxer-app/tests/ui_query.rs` that keeps it in step: it folds every letter and
  digit of Latin-1 (from U+00A0), Latin Extended-A, Latin Extended Additional and the ligatures U+FB00–U+FB06
  through the core's `fold_analyzer` and has Node compare `foldChar` on each, so a change in tantivy's table or
  in the JS shows up as a list of characters. A table generated from the whole filter (1,135 entries) would also
  cover the rare blocks, but it is a generated file in the UI for letters mail rarely carries; the comment above
  the folding section names what is not mirrored.
- 2026-10-20 — `foldChar` now lower-cases first, looks the result up in `LETTER_FOLDS`, and only then strips the
  combining accents after NFD, the order of the core's `LowerCaser` and `AsciiFoldingFilter`; this lets `ẞ` reach
  `ß` → `ss` and `İ` reach its core fold `i̇` (a table key of two characters). The order change is neutral for the
  rest: the test passes over the four blocks.
- 2026-10-20 — the one-to-many fold. `foldWithMap` already gave every folded character of `ß` the index of `ß`,
  so a match that starts inside the letter takes it whole (`se` marks `ße`), but a match that ends inside it lost
  it (`stras`, a prefix while typing `strasse`, marked `Stra`). The new `textRange(map, start, end)` in `query.js`
  moves the end past the whole letter (`Straß`); `highlightFragments` in `main.js` takes its ranges through it,
  one line.
- 2026-10-20 — red before the change: with the old `query.js`, the cross test listed the 48 characters above and
  the new Node tests failed; green after it.

## Deviations from the plan

- Step 2 put the table after the NFD strip; it runs after the lower-casing and before the strip instead, the
  order of the core's filters (see the Log).
- `main.js` changed at its one call site (`Query.textRange`), as a range that ends inside `ß` cannot be mapped
  with `map[end]` alone.
- The keep-in-step test covers four Unicode blocks, not every letter the filter maps; the rest is named in the
  comment above the folding section of `query.js` and listed as a follow-up for the over-folding side.

## Verification

- 2026-10-20 — in the worktree, Windows 11: `node --check` on `ui/query.js`, `ui/main.js` and
  `tests/ui/query.test.js`; `node --test crates/mboxer-app/tests/ui/*.test.js`: 55 pass, among them "letters the
  index folds beyond accents fold the same way here" (`Straße`, `STRAẞE`, `Łódź`, `Ørsted`, `Œuvre`, `Đakovo`,
  `đ`, `Æsir`, `Þór`, `ﬁle`) and "a highlight lands on whole letters where one folds to several" (`strasse` marks
  `Straße`, `lodz` marks `Łódź`, `stras` marks `Straß`, `se` marks `ße`, `œuv` marks `Œuv`, `fi` marks `ﬁ`);
  `cargo test --locked`: every suite green, `ui_query::the_highlight_folds_the_letters_the_index_folds` and
  `index::schema::tests::folds_letters_beyond_combining_accents` included; `cargo fmt --all --check`;
  `cargo clippy --all-targets --locked -- -D warnings`. With `DOCS_RS=1`:
  `cargo clippy --target aarch64-apple-darwin --all-targets --locked -- -D warnings` and
  `cargo +1.90 check --target aarch64-apple-darwin --all-targets --locked` pass;
  `cargo deny check`, `uvx zizmor .github` and `python -I scripts/render-usage-guide.py --check` pass
  (`docs/reference.md` changed). No manual round in the
  viewer: the change is the pure functions in `query.js` and one call site, and the Node tests run the same
  `foldWithMap` / `termRanges` / `textRange` chain that `highlightFragments` uses for the list and the reader.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (listed in the pull
  request for the main session, see docs/maintainer-sessions.md)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see
  docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
