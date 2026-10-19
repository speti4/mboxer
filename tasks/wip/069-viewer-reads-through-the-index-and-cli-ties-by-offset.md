---
size: S
origin: done/060-index-quality-follow-ups-from-the-review.md
created: 2026-10-19
started: 2026-10-20
plan: none
milestone: M6 Hardening
---

# 069 — The viewer reads a message through the index's limit, and the CLI's paging breaks ties by offset

## Context

What: two leftovers of task 060. The viewer still reads messages with `read_at` (`commands.rs`, `view.rs`,
`attachments.rs`); `MboxIndex::read_message` would honour the `max_message_bytes` the index was built with, should
the viewer ever build with a non-default limit. `MboxIndex::search`, which the CLI's paging uses, still breaks date
ties by document address, so `mboxer-cli search` lists same-date messages of a multi-thread build out of file
order; the listing behind the viewer already orders by offset.
Why: one read path and one tie rule, so the CLI and the viewer never disagree about the same archive.
Done when: the viewer's three call sites use `read_message`, and a CLI test over a four-thread build lists
same-date messages in file order.
The viewer's call sites are `read_message` and the attachment commands in `crates/mboxer-app/src/commands.rs`,
`read_message_view` in `view.rs` and `read_attachment` in `attachments.rs`; the CLI's paging is
`MboxIndex::search` in `crates/mboxer-core/src/index/search.rs`, the viewer's listing order is in `listing.rs`.

## Steps

1. [ ] the viewer's read path: `read_message_view`, the raw read behind *Source* / *Save .eml* and
   `read_attachment` take the message through `MboxIndex::read_message` (or a reader built from the index's
   limit), so the index's `max_message_bytes` holds; `read_at` stays for the places with no index
2. [ ] `MboxIndex::search` breaks a date tie by file offset, the same rule as the listing; the tie order
   documented where the listing's is
3. [ ] tests: a core or CLI test over a four-thread build of a fixture with same-date messages lists them in
   file order through `mboxer-cli search`; an app test that a message above the limit reads as the index
   says it does
4. [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   macOS-target clippy and the `rust-version` check
5. [ ] `docs/reference.md` where it describes the CLI's order or the message limit, if the wording changes

## Log

- 2026-10-20 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (filled at the close: date, the test names, the order seen)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
