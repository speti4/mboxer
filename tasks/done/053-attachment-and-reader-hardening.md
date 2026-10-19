---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
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

1. [x] `attachments.rs`: the extension lists extended (one shared list, the macOS-only entries beside it, a comment
   naming why each group is blocked), `RESERVED_NAMES` checked on the first stem of a multi-dot name; unit tests
   for `CON.tar.gz`, `setup.iso`, `Report.XLL`, a `.mobileconfig` on the macOS path
2. [x] `ui/main.js`: the srcdoc of the sanitized frame starts with `<meta name="referrer" content="no-referrer">`
   (with the existing CSP meta); `scrubCss` replaces a removed `@import`, `expression()` or `behavior` with `/**/`
   and escapes `<` in style text as `\3c `; the Node tests under `crates/mboxer-app/tests/ui/` cover
   `</@import;style><img src=…>` and a plain `@import` removal
3. [x] `docs/usage.md` and `docs/reference.md`: the list of attachment types that can only be saved follows
4. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (the Node tests run inside it)

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — tests first. `attachments.rs` got assertions for `CON.tar.gz` (and `lpt1.backup.zip`, with
  `console.tar.gz` and `report.con.txt` as the names that stay), `setup.iso`, `Report.XLL`, `x.settingcontent-ms`,
  every other new Windows type, `Wi-Fi.mobileconfig` and the new macOS types, and a check that the two lists do
  not overlap. Before the fix `cargo test -p mboxer-app --lib attachments` failed 3 of 10: `open_blocked("setup.iso")`
  false, `CON.tar.gz` left as it was, the macOS entries still inside the shared list.
- 2026-10-19 — `classifyUrl`, `scrubCss` and the frame's head metas moved from `ui/main.js` into a new pure module
  `ui/frame.js` (`window.MboxerFrame`, CommonJS under Node, as `print.js`), loaded by `index.html` before
  `main.js`, so that Node can test them; `tests/ui/frame.test.js` is run by `tests/ui_query.rs`. With the code
  moved unchanged, 4 of its 6 tests failed: the plain `@import url(x);` left nothing behind, the crafted
  `</@import;style><img src=https://example.com/x>` came out as `</style><img src=https://example.com/x>`, a `<` in
  a CSS string stayed, and the head had no referrer meta.
- 2026-10-19 — fixed. `BLOCKED_EXTENSIONS` is the shared (Windows) list, `MACOS_BLOCKED_EXTENSIONS` the macOS-only
  one, no longer a test-only copy of half the shared list; `open_blocked` checks both on both systems, and each
  group has a comment naming why it is blocked. `safe_file_name` checks `RESERVED_NAMES` on the part before the
  first dot. `scrubCss` replaces a dropped `@import` with `/**/` and escapes `<` as `\3c `; `headMetas` returns the
  charset, `<meta name="referrer" content="no-referrer">` and the CSP, and `htmlSrcdoc` prepends them with the base
  style. The HTML print document copies the frame's live head, so it carries the referrer meta too.
- 2026-10-19 — docs. `docs/reference.md` lists the new types in the attachments section and says that *Show* loads
  remote images without a referrer; `docs/usage.md` names no attachment types and does not explain the remote
  images, so it is unchanged. `docs/development.md` names `frame.js` among the modules with Node tests.

## Deviations from the plan

- `expression()` and `behavior` were never removed, they were already renamed in place (`expression-blocked(`,
  `blocked:`), which joins nothing; only the `@import` removal became `/**/`.
- The `<` escape spares the comment opener `<!--`: old HTML mail wraps its style sheet in `<!-- … -->`, an escaped
  opener would turn the first rule into an invalid selector, and `<!--` cannot end a style element.
- The new module `ui/frame.js` was not in the plan: `main.js` runs only in the webview, and the Node tests need
  the scrub and the head metas in a module of their own.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo fmt --all --check` clean; `cargo clippy --all-targets -- -D
  warnings` clean; `cargo test` green (mboxer-app 42 unit tests, `ui_query` 1 test running the five Node test
  files, mboxer-cli 4 + 6, mboxer-core 32 + 10 + 22); `node --check` on `ui/main.js` and `ui/frame.js` clean;
  `node --test crates/mboxer-app/tests/ui/frame.test.js` 6 of 6 pass (Node 24).
- 2026-10-19, browser re-parse: a scratch page in headless Edge ran the style handling of the sanitizer and the
  head of `htmlSrcdoc` over `<style></@import;style><img src=https://example.com/x></style>`, serialized the
  document and parsed the result again: 0 `img` elements with the new `scrubCss`, 1 with the old removal; the
  re-parsed head starts with the charset, the `no-referrer` meta and the CSP; `<style><!-- p{color:red} --></style>`
  came through unchanged. The viewer itself was not started.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none found)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
