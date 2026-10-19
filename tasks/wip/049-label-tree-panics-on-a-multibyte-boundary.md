---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
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

1. [ ] a failing unit test beside `segments()` with `Különböző`, `Beérkező levelek`, `Category Promotions`,
   `category updates` and a label shorter than the prefix
2. [ ] the prefix test through `label.get(..n)` (or `strip_prefix` over a case-folded view), never a byte slice;
   a second look at every other byte slice of a label or a header value in `mboxer-app`
3. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; a fixture mbox with such a
   label opened in the viewer, the label shown in the tree

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
