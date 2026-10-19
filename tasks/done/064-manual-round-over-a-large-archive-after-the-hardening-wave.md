---
size: S
origin: done/052-atomic-index-swap-build-lock-and-skipped-messages.md
created: 2026-10-19
started: 2026-10-20
closed: 2026-10-20
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 064 — Manual round over a large archive after the hardening wave: rebuild under an open viewer, accented labels, free text

## Context

What: on a large real archive, with the viewer open on its index: `mboxer-cli index --force` over the same mbox
(052: the second build reports the lock or the swap completes and the viewer keeps working, no `.old` or `.building`
left behind); an archive with accented user labels opens and shows them in the tree (049); `Re: számla`, a URL and
a time in the search field return hits and highlight them (050); an attachment with a newly blocked extension can
only be saved (053). Record what passed and what did not in the Log; each failure becomes a backlog item.
Why: tasks 049, 050, 052 and 053 were verified with tests and fixtures only; the done-when of 052 names the real
archive.
Done when: every point above is seen on a large archive, and each failure has a backlog item of its own.
The round runs on `main` after task 061 (the last code change of the wave), with release builds of both exes.
Since task 060 the index format is 3, so the first open rebuilds the archive's sidecar once before the round starts.

## Steps

1. [x] `cargo build --release`; no older viewer process is running; open the archive in the viewer and let the
   first open rebuild its index (format 3)
2. [x] with the viewer open on the index: `mboxer-cli index --force <mbox>` in a terminal; expected: either a
   clear "another build holds the lock" message, or the build completes and swaps while the viewer keeps
   listing and searching; afterwards `.mboxer/` next to the mbox holds no `.old` or `.building` folder
3. [x] the label tree shows the accented user labels of the archive with their letters intact (049)
4. [x] the search field: `Re: számla`, a URL from a message and a time such as `10:30` each return hits, and
   the hits are highlighted in the reader (050)
5. [x] a message with an attachment whose extension task 053 added to the blocked list (`chm`, `msix`, `iso`,
   `img`, `vhd`, `xll`, `cab`, `wsc`, `sct`, …) offers only *Save*, no open (053)
6. [x] every failure filed as a backlog item; the Log says what passed with the numbers seen

## Log

- 2026-10-20 — started after the merge of task 061 (#241); release build of both exes for the round
- 2026-10-20 — step 1 passed. The archive (14.18 GB on a hard disk, 86 085 messages) opened in the release viewer
  and rebuilt its format-3 index on the first open: 129 s, 108 MB/s once the disk cache was warm, 0 unparsed
  messages; the label tree and the list came up right after the build
- 2026-10-20 — step 2 passed. `mboxer-cli index --force` over the same mbox while the viewer held the index: the
  build ran to the end (no lock, since a viewer only reads), 32.6 s with the file in the OS cache (about
  530 MB/s, 4 threads), 86 085 messages, 0 parse failures, 0 skipped, 134 MB index; the viewer listed, opened
  messages and answered searches throughout, and kept working on the swapped index afterwards; `.mboxer\` holds
  the one index folder, no `.old` and no `.building`
- 2026-10-20 — step 3 passed on a stand-in. The large archive has no accented user label, so the fixture
  `labels-encoded.mbox` (a copy under `target/manual-round/`) was opened beside it in the same viewer: the tree
  shows `Számlák` (2, a raw UTF-8 and a Q-encoded header) and `Munka` → `Számlák` (1, B-encoded) with their
  letters intact, and `label:Számlák` lists 2 of 3. Task 049 had seen the label only through a test until now
- 2026-10-20 — step 4 passed on the large archive. `Re: számla`: 1747 of 86 085 hits, `Re` and `számla` marked
  in the subjects (a prefix such as `számlája` too); a domain (`<name>.hu`): 21 869 hits, the pieces marked in
  addresses, links and the bare name in a signature, as task 063 designed it; `10:30`: 751 hits, the time marked
  in a subject and in a quoted header line of the reader. Seen, by design: a two-letter word such as `re` is also
  marked inside longer words of the list (`Andrea`), the substring rule that `ui/query.js` documents
- 2026-10-20 — step 5 passed on a stand-in. The archive has no attachment with one of the extensions task 053
  added, so a copy of the `attachments.mbox` fixture with its `report.exe` renamed to `report.chm` was opened in
  the same viewer: the `report.chm` row offers *Save…* only, and its tooltip says that executable attachments can
  only be saved; the other attachments of the message keep their buttons
- 2026-10-20 — step 6: no failure, so no backlog item from this round. The stand-in copies live under
  `target/manual-round/` (ignored by git) and are not part of the repo

## Deviations from the plan

- Steps 3 and 5 ran on fixture copies opened beside the large archive, because the archive has neither an
  accented user label nor an attachment with a newly blocked extension; the viewer and its index are the same,
  so each point was still seen in the real viewer rather than in a test

## Verification

- 2026-10-20 — the round over a 14.18 GB archive of 86 085 messages on a hard disk, release builds of `main` at
  task 061: first open rebuilt the format-3 index in 129 s with 0 unparsed messages; a forced CLI rebuild under
  the open viewer completed in 32.6 s, swapped cleanly (no `.old`, no `.building`) and the viewer never stalled;
  accented labels intact in the tree (fixture); `Re: számla` 1747 hits, a domain 21 869 hits, `10:30` 751 hits,
  all highlighted; a `.chm` attachment can only be saved (fixture). No code changed in this task.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-20`
