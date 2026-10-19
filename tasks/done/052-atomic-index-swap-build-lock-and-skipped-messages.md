---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 052 — Swap the index atomically, lock the build, and skip a bad message instead of failing the build

## Context

What: `build()` in `crates/mboxer-core/src/index/build.rs` removes the old index folder and then renames
`.building` into place; on Windows a reader that still holds the old index makes the removal fail half-way, the
old index is partly gone and the new one stays in `.building`, so a full build of a 14 GB file is lost.
`prepare_building_dir` deletes any existing `.building`, so two builds of one mbox destroy each other, and one
`MessageTooLarge` or read error inside the stream (`let raw = item?`) aborts the whole build.
Why: a rebuild after a Takeout refresh is the one moment the viewer touches the index, and it should never leave
the archive without one.
Done when: a swap with the old index open, two concurrent builds, and a fixture with one oversized message each
have a test that passes, and `mboxer-cli index --force` over a large archive survives an open viewer.
Decided 2026-10-19: filed by the post-release review; the index format and `mboxer.json` stay as they are, a lock
file in the index home is acceptable.

## Steps

1. [x] tests first in `crates/mboxer-core/tests/index.rs`: a rebuild while a `Searcher` / `Listing` holds the old
   index open (Windows file locks), two builds of the same mbox started at once, a fixture with one message over
   a small `max_message_bytes`; each red before the change
2. [x] the swap: rename the old folder aside, rename `.building` into place, remove the old folder afterwards
   with a retry loop on Windows (`fs::remove_dir_all` fails on an open file), and clean the aside folder on the
   next build if it survived a crash
3. [x] the lock: a lock file in the index home (`fs4` is already in the dependency graph through tantivy, or
   `File::create_new` with the pid inside), held for the build, a clear `Error` when another build holds it
4. [x] the skip: an oversized or unreadable message is counted (`parse_failures` or a new `skipped` counter in
   `IndexMeta` / `BuildReport`), its envelope offset logged, and the build goes on; the CLI prints the count
5. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` (the
   index section) says what happens to a skipped message and to a second build

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — tests first, in a new `crates/mboxer-core/tests/index_build.rs` with a new fixture
  `one-oversized.mbox` (three messages, the middle one about 2.2 KB; the test sets `max_message_bytes` to 1024).
  Against the old code: the oversized test failed with `MessageTooLarge { offset: 252, limit: 1024 }`; the
  concurrent test failed because the second build ran beside the first and returned a `BuildReport` (it had
  deleted and recreated the first build's `.building`); the open-index test (a `MboxIndex` and a `Listing` held
  during the rebuild) passed on Windows 11: `fs::remove_dir_all` deletes open, memory-mapped tantivy files
  (POSIX delete semantics), so the old removal fails only on a file held without delete sharing. A Windows-only
  test for that case (`a_held_old_index_stays_whole_when_the_swap_cannot_run`, a handle on `mboxer.json` opened
  with `FILE_SHARE_READ` only) ran against the old source and failed: `.building` stayed beside a partly deleted
  index (`["simple-lf.mbox", "simple-lf.mbox.building"]`).
- 2026-10-19 — the swap. Renaming the old folder aside broke the existing
  `a_blocked_sidecar_falls_back_to_the_index_home` test (a rebuild while the index is open): `PermissionDenied`
  (os error 5). Windows refuses to rename a folder while any file inside is open, with delete sharing or
  without (checked with a .NET `FileStream` too). Decided: the old index's files move one by one into
  `<name>.old` (`mboxer.json` first, so a half-moved folder never reads as an index), the emptied folder goes,
  `.building` takes its name, and `.old` is deleted last; renames and removals retry with pauses of 50, 100, 200,
  400 and 800 ms. A file that cannot move sends the moved ones back (`mboxer.json` last) and the build fails with
  the old index whole; a `.old` (or `.old.N`) that a viewer still holds, and a stale `.building`, are cleared by
  the next build. `mboxer.json` is still written last into `.building`. The freshness check (`status_of` over
  `index_dir` and `fallback_dir` only) never looks at the `.building` or `.old` siblings; the module doc of
  `fingerprint.rs` now says so.
- 2026-10-19 — the lock. Decided: `File::try_lock` from std (stable since Rust 1.89, within the MSRV 1.90) on
  `<index home>/<name>-<hash>.lock`, the fallback folder's name plus `.lock`, so the viewer and `mboxer-cli`
  share it whichever folder the index goes to. No new crate: `cargo tree -i fs4` shows fs4 0.13.1 through tantivy,
  but std covers it. An OS lock rather than `File::create_new` with a pid: the system releases it when the
  process dies, so a stale lock cannot exist and `--force` does not need to break one. The file stays on disk:
  removing it while another build opens it could let two builds lock two different files. A second build gets
  the new `Error::BuildLocked { lock }`: "another build holds `<path>`; wait for it to finish". The lock needs a
  writable index home even when the index goes beside the mbox.
- 2026-10-19 — the skip. `MboxReader` no longer ends the stream on `MessageTooLarge`: it reads on in bounded
  lines to the next envelope that starts a line and yields the error as one item. An oversized first line, a
  file that does not start with an envelope and an I/O error still end the stream: a failing disk must not leave
  a silently partial index. The build counts each skipped message (`BuildReport::skipped`, the first ten offsets
  in `BuildReport::skipped_offsets`, `IndexMeta::skipped` with a serde default, so `FORMAT_VERSION` stays 2 and an
  older `mboxer.json` still reads); a build that indexed nothing but skipped something fails with the first
  `MessageTooLarge`, as a non-mbox file did before. The offsets reach the user through the report: there is no
  log path in the core. `mboxer-cli index` prints `skipped:` and one line per kept offset; the viewer's
  `IndexState::Built` carries `skipped` and its status line adds "N too large to index" when it is not 0.
- 2026-10-19 — checks in the worktree: `cargo fmt --all --check` clean, `cargo clippy --all-targets -- -D
  warnings` clean, `cargo test` green (mboxer-core 37 unit, 10 fixtures, 22 index, 6 index_build; mboxer-cli 4 +
  6; mboxer-app 42 + 1), `node --check` on `ui/main.js` clean. `Cargo.toml` and `Cargo.lock` unchanged, so no
  `cargo deny check`.
- 2026-10-19 — `docs/usage.md`: the lock file in the table of what mboxer writes, and two entries under "When
  something is off" (another build holds the file; messages too large to index);
  `python -I scripts/render-usage-guide.py --check` passes.

## Deviations from the plan

- The tests are in a new `tests/index_build.rs`, not `tests/index.rs`: task 050 edits that file at the same time.
- The old index's files are moved aside one by one instead of renaming the folder: Windows refuses the folder
  rename while a viewer holds the index (see the Log).
- The lock is an OS lock through std `File::try_lock`, neither `fs4` nor a pid file.
- Only an oversized message is skipped; an I/O error inside the stream still fails the build (see the Log). A
  message that does not parse was already indexed by its offset and counted under `parse_failures`.
- The viewer shows the skipped count too, not only the CLI.
- The done-when's manual round (`mboxer-cli index --force` over the real archive beside an open viewer) was not
  run in this task: the tests that hold the old index open stand in for it.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo test -p mboxer-core --test index_build` — 6 passed:
  `a_rebuild_replaces_an_index_that_is_still_open` (a `MboxIndex` and a `Listing` keep answering while the
  rebuild swaps the folder; no `.building` left; the next build leaves only the index folder),
  `a_second_build_of_the_same_mbox_is_refused_while_the_first_runs` (the second build returns `BuildLocked`
  while the first waits in its progress callback; the first finishes, the index is fresh, a third build runs),
  `an_oversized_message_is_skipped_and_the_build_goes_on` (2 messages indexed, `skipped` 1 at offset 252 in the
  report and in `mboxer.json`, the neighbours searchable, the filler text not),
  `a_file_of_nothing_but_oversized_text_is_still_an_error`, `folders_a_crashed_build_left_behind_are_cleared`
  and `a_held_old_index_stays_whole_when_the_swap_cannot_run` (the build fails after the retries, the old index
  keeps its files and still answers).
- Same day: the whole `cargo test` green, `cargo fmt --all --check` and `cargo clippy --all-targets -- -D
  warnings` clean; new unit tests for the reader's resync after an oversized message (including text that looks
  like an envelope in the middle of a long line), an old `mboxer.json` without `skipped`, the aside names and
  the lock path.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
