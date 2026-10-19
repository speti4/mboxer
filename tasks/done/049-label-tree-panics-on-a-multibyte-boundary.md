---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 049 — The label tree panics on a label whose ninth byte is inside a multibyte character

## Context

What: `segments()` in `crates/mboxer-app/src/folders.rs` slices the label by byte (`label[..CATEGORY_PREFIX.len()]`)
to test for the `Category ` prefix; a label such as `Különböző` or `Beérkező levelek` has a multibyte character on
that boundary, and the slice panics inside `OpenMbox::load`, after the index is built. Every later open of the
archive fails the same way, the start-up reopen included.
Why: a Hungarian archive with one such user label cannot be opened at all.
Done when: a unit test with accented labels and a `Category Promotions` one passes, and an archive with such labels
opens and shows them in the tree.
Decided 2026-10-19: filed by the post-release review; the fix is the prefix test alone, no change to the tree shape.

## Steps

1. [x] a failing unit test beside `segments()` with `Különböző`, `Beérkező levelek`, `Category Promotions`,
   `category updates` and a label shorter than the prefix
2. [x] the prefix test through `label.get(..n)` (or `strip_prefix` over a case-folded view), never a byte slice;
   a second look at every other byte slice of a label or a header value in `mboxer-app`
3. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; a fixture mbox with such a
   label opened in the viewer, the label shown in the tree (opened through `OpenMbox::load` in a test, see
   Deviations)

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — red: `a_multibyte_char_on_the_prefix_boundary_is_no_category` in `folders.rs` (`Különböző`,
  `Beérkező levelek`, `Beérkező levelek/Fontos`, `Category Promotions`, `category updates`, `category ének`, the
  4-byte `Céh`, the exactly-9-byte `Category `, the 10-byte `Kategória`, and a tree of them) panicked with "end
  byte index 9 is not a char boundary; it is inside 'ö' (bytes 8..10 of string)" at `folders.rs:97`.
- 2026-10-19 — fix: `strip_category_prefix()` takes the head with `label.get(..n)`, compares it with
  `eq_ignore_ascii_case` and returns the rest with `label.get(n..)`; `get` returns `None` off a char boundary,
  so such a label is no category and keeps its slash hierarchy. The old `label.len() > n` guard is gone: an
  exactly-9-byte `Category ` gives an empty rest, which falls through to the one-segment path as before.
- 2026-10-19 — end to end: `accented_labels_on_the_category_prefix_boundary_open` in `state.rs` writes a
  three-message synthetic mbox (raw UTF-8 `Különböző`, `Beérkező levelek`, `Category Promotions`, and a
  Q-encoded `Különböző`) to a temp folder and opens it with `OpenMbox::load`: three rows, the tree
  `Inbox, Categories, Beérkező levelek, Különböző`, `Különböző` with count 2, `Promotions` under `Categories`.
  With `folders.rs` stashed back to the old code, the same test panicked at `folders.rs:97`.
- 2026-10-19 — the other range slices in `mboxer-app/src`: `split_extension()` in `attachments.rs` cuts at
  `rfind('.')` (an ASCII dot, always a boundary), `unescape_from_lines()` in `view.rs` cuts after a run of ASCII
  `>`; both are safe. The range slices in `mboxer-core` and `mboxer-cli` are over `[u8]`, and the two `str`
  cuts there (`build.rs` truncation, `typing_word()` in `search.rs`) walk char boundaries already. Nothing else
  to change.
- 2026-10-19 — close: `CHANGELOG.md` regenerated with `python -I scripts/changelog.py` (the CI `lint` job's
  `--check` fails on a pull request that closes a task without its changelog line): a new *M6 Hardening*
  heading under *Unreleased* with the 049 line.

## Deviations from the plan

- Step 3's viewer look is replaced by the `state.rs` test, which runs the code path that panicked
  (`OpenMbox::load` → `label_tree`) over a synthetic mbox and checks the tree it returns; seeing the labels in
  the window is left to the maintainer's next manual round.

## Verification

- 2026-10-19 — in the worktree: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D warnings`
  clean; `cargo test`: 119 tests passed (44 in `mboxer-app`, 2 of them new). Both new tests failed with the
  byte-index panic before the fix and pass after it.
- 2026-10-19 — not checked here: the label in the viewer's tree over a real archive (the window was not
  started in this session).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
