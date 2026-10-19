---
status: idea
size: S
origin: done/052-atomic-index-swap-build-lock-and-skipped-messages.md
created: 2026-10-19
milestone: M6 Hardening
---

# Show the skipped messages in the viewer, not only their count

What: since task 052 a message over the size limit is skipped by the build and counted; the viewer's status line
says "N too large to index" but lists no row for such a message and gives no way to find it. Show the skipped
offsets (and the size of each) in the index details or a status-bar tooltip, and let a skipped message be opened
raw from its offset where the reader allows it.
Why: a user who knows one mail is missing should be able to tell that it was skipped and why.
Done when: the skipped messages are visible in the viewer with their offset and size, and the usage guide says so.
