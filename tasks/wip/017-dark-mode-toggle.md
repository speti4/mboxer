---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-18
milestone: M3 Comfort
plan: n-zz-k-t-a-backlogot-virtual-mango.md
---

# 017 — Dark mode toggle with OS default

## Context

Sötét téma a shellnek. Az alapértelmezés az OS-t követi (`prefers-color-scheme`), a toolbar gombja
felülírja (auto / light / dark) és a választás megmarad (`mboxer.theme` a localStorage-ban). A `ui/style.css`
már minden színt tokenként ad a `:root`-on, így a munka egy második tokenkészlet a media query alatt plusz a
`data-theme` felülírás. Why: a viewer hosszú munkamenetekre való és a userek gépein gyakori az OS-szintű dark
mode. Done when: dark módú Windows alatt az app sötéten indul; a gomb élőben vált; a beállítás túléli az
újraindítást. Tervezési döntések 2026-10-18 (jóváhagyott terv): a felülírás egy `<head>`-ben töltött
`ui/theme.js`-ből kerül a `<html data-theme>`-re még az első paint előtt (a CSP tiltja az inline scriptet);
a natív címsor a Tauri `setTheme`-jét kapja (`core:window:allow-set-theme`); a HTML olvasó iframe mindkét
témában világos vásznon marad (a HTML levelek világos lapra készülnek, Thunderbird/Outlook is így tesz), a Text
és Source nézet követi a témát. Nincs új crate (D-02).

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
9. [ ] User kézi kör a 14 GB archívumon dark módú Windows alatt (indulás, címsor, váltás, újraindítás)

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

## Deviations from the plan

- none

## Verification

- 2026-10-18 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` pass
  (új: `tests/ui/theme.test.js` 6 teszt a `node --test` alatt, az `ui_query.rs` mindkét Node fájlt futtatja).
  Chrome stub-check 25/25 (részletek a Logban). Nyitva: a user kézi köre dark módú Windows alatt (9. lépés) — a natív
  címsor `setTheme`-je csak az igazi Tauri ablakban ellenőrizhető.

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
