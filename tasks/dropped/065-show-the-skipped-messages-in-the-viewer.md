---
size: S
origin: done/052-atomic-index-swap-build-lock-and-skipped-messages.md
created: 2026-10-19
closed: 2026-10-19
reason: Gmail caps a message near 50 MB, so a Takeout export never reaches the 256 MB skip limit; the skip of task 052 is a safeguard that no real archive triggers, and a list of skipped messages would stay empty
reopen-when: a real archive reports a skipped message in the status line
milestone: M6 Hardening
---

# Show the skipped messages in the viewer, not only their count

What: since task 052 a message over the size limit is skipped by the build and counted; the viewer's status line
says "N too large to index" but lists no row for such a message and gives no way to find it. Show the skipped
offsets (and the size of each) in the index details or a status-bar tooltip, and let a skipped message be opened
raw from its offset where the reader allows it.
Why: a user who knows one mail is missing should be able to tell that it was skipped and why.
Done when: the skipped messages are visible in the viewer with their offset and size, and the usage guide says so.
