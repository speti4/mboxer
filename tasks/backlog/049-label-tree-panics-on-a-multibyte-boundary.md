---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# The label tree panics on a label whose ninth byte is inside a multibyte character

What: `segments()` in `crates/mboxer-app/src/folders.rs` slices the label by byte (`label[..CATEGORY_PREFIX.len()]`) to
test for the `Category ` prefix; a label such as `Különböző` or `Beérkező levelek` has a multibyte character on that
boundary, and the slice panics inside `OpenMbox::load`, after the index is built. Every later open of the archive
fails the same way, the start-up reopen included. Replace the slice with `label.get(..n)` (or `strip_prefix` over a
case-folded view) and add a unit test with accented labels and a `Category Promotions` one.
Why: a Hungarian archive with one such user label cannot be opened at all.
Done when: the test passes and an archive with such labels opens and shows them in the tree.
