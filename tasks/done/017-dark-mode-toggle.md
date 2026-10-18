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
and Source views follow the theme. No new crate (D-02).

## Steps

1. [x] `ui/style.css`: `color-scheme`, új tokenek (`--on-accent`, `--warn*`, `--overlay`, `--reader-canvas`), a
   hardcode-olt színek tokenre, dark készlet `@media (prefers-color-scheme: dark) :root:not([data-theme="light"])`
   és `:root[data-theme="dark"]` alatt
2. [x] `ui/theme.js` (új): `MboxerTheme` — `read / write / apply / next / label / title`, betöltéskor `apply(read())`
3. [x] `ui/index.html`: `theme.js` a `<head>`-ben, `#theme-btn` a toolbarban
4. [x] `ui/main.js`: `setTheme`, gomb, `start()`, Tauri `setTheme` best effort; `IFRAME_STYLE` világos vászon
5. [x] `capabilities/default.json`: `core:window:allow-set-theme`
6. [x] `tests/ui/theme.test.js` + `tests/ui_query.rs` mindkét Node tesztet futtatja
7. [x] `docs/usage.md` bekezdés a témáról
8. [x] `node --check`, `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; Chrome stub-check;
   `tasks_check`; commit
9. [x] User kézi kör a 14 GB archívumon dark módú Windows alatt (indulás, címsor, váltás, újraindítás)

## Log

- 2026-10-18 — plan approved; 017 promoted to wip
- 2026-10-18 — implementálva (8 fájl, +162/−24): tokenek + dark készlet, `theme.js`, gomb, `setTheme`, capability,
  Node teszt, usage doc. `node --test`: 22 pass (16 query + 6 theme); `cargo fmt --check`, `cargo clippy
  --all-targets -- -D warnings` tiszta; `cargo test` zöld (30 app + 1 ui node + 4 + 6 CLI + 30 + 9 + 22 core)
- 2026-10-18 — Chrome stub-check (scratchpad `serve-ui-theme.js`, stub `__TAURI__.core` + `window.getCurrentWindow().setTheme`
  recorder, 127.0.0.1:8767): 25/25 ellenőrzés OK — auto: nincs `data-theme`, `setTheme(null)`; light: `--bg #ffffff`,
  `color-scheme light`, mentve, gomb "Light" + tooltip; dark: `--bg #1e1f22`, `color-scheme dark`, toolbar
  `#26282c`, kijelölt sor `#2d4a6e`, `setTheme("dark")`; reload után `data-theme="dark"` már a `theme.js`-ből,
  `setTheme` induláskor `["dark"]`; HTML nézet iframe body `#fff` / `#1f2328` / `color-scheme light` dark módban;
  Text nézet `--fg`; mark a sorban `#a37c00` + világos fg, a bodyban `#7a5c00`. Képernyőkép: remote bar, warn státusz,
  overlay olvasható. A Chrome maga is dark módú volt, így az auto ág (`prefers-color-scheme`) is lefutott
- 2026-10-18 — commit `53a49b6` (feat, task 017)
- 2026-10-18 — a user kézi köre az igazi Tauri ablakban, a 14 GB archívumon: "sikeres a teszt". A tervben feltételesen
  említett follow-up (WebView2 fehér villanás a betöltés előtt) nem merült fel, nem kerül backlogra; closed

## Deviations from the plan

- none

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass
  (új: `tests/ui/theme.test.js` 6 teszt a `node --test` alatt, az `ui_query.rs` mindkét Node fájlt futtatja).
  Chrome stub-check 25/25 (részletek a Logban).
- 2026-10-18 — User kézi kör dark módú Windows alatt, a 14 GB archívummal: sötét indulás, a gomb élőben vált, a
  beállítás megmarad újraindítás után — done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
