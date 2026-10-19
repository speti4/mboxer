---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
---

# 053 — Attachment and reader hardening: blocked extensions, reserved names, referrer, style text

## Context

What: `BLOCKED_EXTENSIONS` in `crates/mboxer-app/src/attachments.rs` misses types that carry code or bypass the mark
of the web (`chm`, `msix`, `msixbundle`, `appinstaller`, `iso`, `img`, `vhd`, `vhdx`, `xll`, `scf`, `settingcontent-ms`,
`library-ms`, `search-ms`, `wsc`, `sct`, `cab`; on macOS `mobileconfig`, `zsh`, `bash`, `csh`, `prefpane`, `jnlp`), and
`RESERVED_NAMES` checks only the last stem, so `CON.tar.gz` passes. The sanitized frame has no `no-referrer` meta, so
a remote image loaded on *Show* sends the app origin as referrer. `scrubCss` in `ui/main.js` deletes `@import …;`
from style text, which can join the remaining pieces into a closing tag the sanitizer never saw; the frame's CSP and
sandbox still contain it, but the removal should leave `/**/` or escape `<` as `\3c`.
Why: these are the layers behind the sanitizer; each gap is cheap to close now and expensive after a report.
Done when: the new extensions can only be saved, a double extension is caught, the frame declares the referrer
policy, and the style-text test keeps `</style>` out of the srcdoc.
Decided 2026-10-19: filed by the post-release review; one pull request, a test per item, no change to the
sanitizer's allowlist itself.

## Steps

1. [ ] `attachments.rs`: the extension lists extended (one shared list, the macOS-only entries beside it, a comment
   naming why each group is blocked), `RESERVED_NAMES` checked on the first stem of a multi-dot name; unit tests
   for `CON.tar.gz`, `setup.iso`, `Report.XLL`, a `.mobileconfig` on the macOS path
2. [ ] `ui/main.js`: the srcdoc of the sanitized frame starts with `<meta name="referrer" content="no-referrer">`
   (with the existing CSP meta); `scrubCss` replaces a removed `@import`, `expression()` or `behavior` with `/**/`
   and escapes `<` in style text as `\3c `; the Node tests under `crates/mboxer-app/tests/ui/` cover
   `</@import;style><img src=…>` and a plain `@import` removal
3. [ ] `docs/usage.md` and `docs/reference.md`: the list of attachment types that can only be saved follows
4. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (the Node tests run inside it)

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
