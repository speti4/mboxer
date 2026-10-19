---
status: idea
size: S
origin: done/060-index-quality-follow-ups-from-the-review.md
created: 2026-10-19
milestone: M6 Hardening
---

# The viewer reads a message through the index's limit, and the CLI's paging breaks ties by offset

What: two leftovers of task 060. The viewer still reads messages with `read_at` (`commands.rs`, `view.rs`,
`attachments.rs`); `MboxIndex::read_message` would honour the `max_message_bytes` the index was built with, should
the viewer ever build with a non-default limit. `MboxIndex::search`, which the CLI's paging uses, still breaks date
ties by document address, so `mboxer-cli search` lists same-date messages of a multi-thread build out of file
order; the listing behind the viewer already orders by offset.
Why: one read path and one tie rule, so the CLI and the viewer never disagree about the same archive.
Done when: the viewer's three call sites use `read_message`, and a CLI test over a four-thread build lists
same-date messages in file order.
