---
status: ready
size: S
origin: done/051-query-generation-race-and-sync-commands-on-the-files-lock.md
created: 2026-10-19
milestone: M6 Hardening
---

# Stop an overtaken search early instead of letting it run to its end

What: since task 051 the newest query always wins, but a query overtaken in the middle of its search still runs to
its end before the newest one gets its turn on the search lock. A tantivy collector that checks the file's newest
generation every N documents (or a cancellation flag the search loop reads) would end the stale search within
milliseconds and hand the lock over.
Why: on a large archive a relevance query takes seconds; typing three characters in that window queues three
searches and the list waits for all of them.
Done when: a test over a slow query and a newer one shows the stale search ending early, and the queued-query wait
in the 051 test drops accordingly.
