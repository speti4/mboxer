---
size: S
origin: done/052-atomic-index-swap-build-lock-and-skipped-messages.md
created: 2026-10-19
started: 2026-10-20
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

1. [ ] `cargo build --release`; no older viewer process is running; open the archive in the viewer and let the
   first open rebuild its index (format 3)
2. [ ] with the viewer open on the index: `mboxer-cli index --force <mbox>` in a terminal; expected: either a
   clear "another build holds the lock" message, or the build completes and swaps while the viewer keeps
   listing and searching; afterwards `.mboxer/` next to the mbox holds no `.old` or `.building` folder
3. [ ] the label tree shows the accented user labels of the archive with their letters intact (049)
4. [ ] the search field: `Re: számla`, a URL from a message and a time such as `10:30` each return hits, and
   the hits are highlighted in the reader (050)
5. [ ] a message with an attachment whose extension is on the blocked list offers only *Save*, no open (053)
6. [ ] every failure filed as a backlog item; the Log says what passed with the numbers seen

## Log

- 2026-10-20 — started after the merge of task 061 (#241); release build of both exes for the round

## Deviations from the plan

- none yet

## Verification

- (filled at the close: date, archive size, the result of each step)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
