---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-18
closed: 2026-10-18
milestone: M3 Comfort
plan: n-zz-k-t-a-backlogot-virtual-mango.md
---

# 017 — Dark mode toggle with OS default

## Context

A dark theme for the shell. The default follows the OS (`prefers-color-scheme`), the toolbar button
overrides it (auto / light / dark) and the choice is kept (`mboxer.theme` in localStorage). `ui/style.css`
already gives every color as a token on `:root`, so the work is a second token set under the media query plus the
`data-theme` override. Why: the viewer is meant for long sessions and OS-level dark mode is common on the users'
machines. Done when: under a dark-mode Windows the app starts dark; the button switches live; the setting survives
a restart. Planning decisions 2026-10-18 (approved plan): the override goes from a `ui/theme.js` loaded in the
`<head>` onto `<html data-theme>` before the first paint (the CSP forbids inline scripts);
the native title bar gets Tauri's `setTheme` (`core:window:allow-set-theme`); the HTML reader iframe stays on a
light canvas in both themes (HTML mails are made for a light page, Thunderbird/Outlook do the same), the Text
and Source views follow the theme. No new crate (D-002).

## Steps

1. [x] `ui/style.css`: `color-scheme`, new tokens (`--on-accent`, `--warn*`, `--overlay`, `--reader-canvas`), the
   hard-coded colours moved to tokens, a dark set under `@media (prefers-color-scheme: dark) :root:not([data-theme="light"])`
   and `:root[data-theme="dark"]`
2. [x] `ui/theme.js` (new): `MboxerTheme` — `read / write / apply / next / label / title`, `apply(read())` on load
3. [x] `ui/index.html`: `theme.js` in the `<head>`, `#theme-btn` in the toolbar
4. [x] `ui/main.js`: `setTheme`, button, `start()`, Tauri `setTheme` best effort; `IFRAME_STYLE` light canvas
5. [x] `capabilities/default.json`: `core:window:allow-set-theme`
6. [x] `tests/ui/theme.test.js` + `tests/ui_query.rs` runs both Node tests
7. [x] `docs/usage.md` paragraph on the theme
8. [x] `node --check`, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; Chrome stub check;
   `tasks_check`; commit
9. [x] Manual round on the 14 GB archive under a dark-mode Windows (start, title bar, switching, restart)

## Log

- 2026-10-18 — plan approved; 017 promoted to wip
- 2026-10-18 — implemented (8 files, +162/−24): tokens + dark set, `theme.js`, button, `setTheme`, capability,
  Node test, usage doc. `node --test`: 22 pass (16 query + 6 theme); `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings` clean; `cargo test` green (30 app + 1 ui node + 4 + 6 CLI + 30 + 9 + 22 core)
- 2026-10-18 — Chrome stub check (scratchpad `serve-ui-theme.js`, stub `__TAURI__.core` + `window.getCurrentWindow().setTheme`
  recorder, 127.0.0.1:8767): 25/25 checks OK — auto: no `data-theme`, `setTheme(null)`; light: `--bg #ffffff`,
  `color-scheme light`, saved, button "Light" + tooltip; dark: `--bg #1e1f22`, `color-scheme dark`, toolbar
  `#26282c`, selected row `#2d4a6e`, `setTheme("dark")`; after a reload `data-theme="dark"` already comes from
  `theme.js`, `setTheme` at start-up `["dark"]`; HTML view iframe body `#fff` / `#1f2328` / `color-scheme light` in dark
  mode; Text view `--fg`; mark in the row `#a37c00` + light fg, in the body `#7a5c00`. Screenshot: remote bar, warn
  status, overlay readable. Chrome itself ran in dark mode, so the auto branch (`prefers-color-scheme`) ran too
- 2026-10-18 — commit `563fcc2` (feat, task 017)
- 2026-10-18 — manual round in the real Tauri window, on the 14 GB archive: passed. The follow-up the plan named
  conditionally (a WebView2 white flash before loading) did not come up, so it is not filed; closed

## Deviations from the plan

- none

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass
  (new: `tests/ui/theme.test.js` 6 tests under `node --test`, `ui_query.rs` runs both Node files).
  Chrome stub check 25/25 (details in the Log).
- 2026-10-18 — manual round under a dark-mode Windows, with the 14 GB archive: dark start, the button switches
  live, the setting survives a restart — done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
