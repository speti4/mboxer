---
status: ready
size: S
origin: done/052-atomic-index-swap-build-lock-and-skipped-messages.md
created: 2026-10-19
milestone: M6 Hardening
---

# Manual round over a large archive after the hardening wave: rebuild under an open viewer, accented labels, free text

What: on a large real archive, with the viewer open on its index: `mboxer-cli index --force` over the same mbox
(052: the second build reports the lock or the swap completes and the viewer keeps working, no `.old` or `.building`
left behind); an archive with accented user labels opens and shows them in the tree (049); `Re: számla`, a URL and
a time in the search field return hits and highlight them (050); an attachment with a newly blocked extension can
only be saved (053). Record what passed and what did not in the Log; each failure becomes a backlog item.
Why: tasks 049, 050, 052 and 053 were verified with tests and fixtures only; the done-when of 052 names the real
archive.
Done when: every point above is seen on a large archive, and each failure has a backlog item of its own.
