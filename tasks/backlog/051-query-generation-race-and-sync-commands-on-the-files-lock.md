---
status: ready
size: M
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# `set_query` takes its generation before the files lock, and the sync commands block on that lock

What: in `crates/mboxer-app/src/commands.rs` `set_query` increments the generation counter and only then takes the
files mutex; when two queries wait on a third, the later one can win the lock first, so the backend ends on an older
listing than the UI waits for, every `list_rows` page is discarded and the list stays at `…`. Take the generation
under the lock, or return early from a query older than the current generation. In the same change, `list_rows`,
`toggle_thread`, `position_of`, `activate_file` and `close_file` are sync commands blocking the webview on the same
mutex while a relevance query holds it, so scrolling during a slow query freezes the window; split the listing from
the search state or make the readers async, and make the comment above `set_query` true.
Why: fast typing over a large archive is the load the viewer is built for.
Done when: a test over a slow query and two queued ones ends on the newest listing, and the window scrolls during a
relevance query over a large archive.
