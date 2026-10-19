---
size: S
origin: done/063-free-text-follow-ups-tantivy-assertion-warnings-and-highlight.md
created: 2026-10-19
started: 2026-10-20
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

1. [ ] the folding table for the UI: the letters `AsciiFoldingFilter` maps beyond combining accents, as a small
   table in `query.js` (or a generated file with a core test that keeps it in step with the filter); the choice
   in the Log
2. [ ] `foldChar` applies the table after the NFD strip; `foldWithMap` maps a one-to-many fold so a highlight
   range still lands on the original letters
3. [ ] Node tests in `crates/mboxer-app/tests/ui/` over `Straße` / `strasse`, `Łódź` / `lodz`, `Ørsted`,
   `œuvre`, `đ`, and a core test that the same words fold the same way through `fold_analyzer`
4. [ ] `node --check`, the Node tests, `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
   `cargo test --locked`, the macOS-target clippy and the `rust-version` check
5. [ ] `docs/reference.md` where it describes the highlight, if its wording changes

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (filled at the close: date, the test names, the words seen marked)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
