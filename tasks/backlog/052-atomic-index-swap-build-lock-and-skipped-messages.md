---
status: ready
size: M
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Swap the index atomically, lock the build, and skip a bad message instead of failing the build

What: `build()` in `crates/mboxer-core/src/index/build.rs` removes the old index folder and then renames
`.building` into place; on Windows a reader that still holds the old index makes the removal fail half-way, the
old index is partly gone and the new one stays in `.building`, so a full build of a 14 GB file is lost. Rename the
old folder aside first and remove it after the swap (retrying on Windows), hold a lock file in `.mboxer/` so two
builds of one mbox cannot delete each other's `.building`, and turn one `MessageTooLarge` or read error inside the
stream into a counted skip (`parse_failures` already exists) instead of aborting the build. Tests: a swap with the
old index open, two concurrent builds, a fixture with one oversized message.
Why: a rebuild after a Takeout refresh is the one moment the viewer touches the index, and it should never leave
the archive without one.
Done when: the three tests pass and `mboxer-cli index --force` over the large archive survives an open viewer.
