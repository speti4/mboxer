---
status: idea
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
---

# Polish the code comments, UI strings and crate metadata

What: the CLI help names `mboxer-cli`, not `mboxer`; the unused `dialog:default` capability goes; Rust error
messages shown in the UI start with a capital, as do "Index reused / built"; "Message-ID"; stale comments (state.rs
time window, print.rs platforms, attachments.rs path limits, "the exe's version", "pre-016"); one spelling (-ize);
the dependency notes and crate descriptions in `Cargo.toml` cover macOS; `tauri.conf.json` copyright and publisher;
optionally a fixture persona other than the maintainer's first name.
Why: the pre-publication review of the source found these; none blocks the release.
Done when: each item is changed or dropped with a reason in the Log; tests green.
