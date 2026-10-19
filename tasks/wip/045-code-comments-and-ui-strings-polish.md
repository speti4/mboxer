---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M5 Public release
---

# 045 — Polish the code comments, UI strings and crate metadata

## Context

What: the CLI help names `mboxer-cli`, not `mboxer`; the unused `dialog:default` capability goes; Rust error
messages shown in the UI start with a capital, as do "Index reused / built"; "Message-ID"; stale comments (state.rs
time window, print.rs platforms, attachments.rs path limits, "the exe's version", "pre-016"); one spelling (-ize);
the dependency notes and crate descriptions in `Cargo.toml` cover macOS; `tauri.conf.json` copyright and publisher;
optionally a fixture persona other than the maintainer's first name.
Why: the pre-publication review of the source found these; none blocks the release.
Done when: each item is changed or dropped with a reason in the Log; tests green.

## Steps

1. [ ] CLI: help and error texts and the test doc comment name `mboxer-cli`
2. [ ] capability: `dialog:default` removed if the UI never calls the dialog plugin, description fixed
3. [ ] UI strings: capitalised error and status texts, "Message-ID"
4. [ ] comments: the stale ones fixed, one spelling (-ize, matching the UI strings)
5. [ ] `Cargo.toml` notes and crate descriptions cover macOS; `tauri.conf.json` copyright and publisher if
   tauri-build stamps them into the exe
6. [ ] fixture persona: renamed with the tests, or dropped with a reason
7. [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`, the
   Node UI tests

## Log

- 2026-10-19 — promoted

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
