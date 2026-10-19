---
size: S
origin: done/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
reason: rare and cosmetic; the next theme switch always draws the right title bar, so a fix is not worth its cost (user decision 2026-10-19)
reopen-when: a user reports it, or tao changes how set_theme repaints the Windows title bar
---

# 031 — Native title bar stays dark when the Light override runs under dark-mode Windows

## Context

What: on a dark-mode Windows, switching the toolbar theme to *Light* left the native title bar dark while its text
turned dark, so the caption was hard to read (seen in the 026 screenshot runs, debug build, throwaway WebView2
profile). Check whether `setTheme("light")` reaches the window and whether a repaint or focus change fixes it.
Why: 017 promised the title bar follows the override. Done when: the caption matches the override in all three
modes.
User decision 2026-10-19: every open task is done before the repo goes public (028); 031 runs now.

## Log

- 2026-10-19 — promoted on `task/031-title-bar-stays-dark-under-light-override`.
- 2026-10-19 — reprodukálva (task session, debug build, eldobható WebView2 profil, demo mbox, dark Windows): a
  címsor pixel színe 22 *Light* mintából kétszer `#202020` maradt (egy normál váltás és egy 30 ms-mal az ablak
  aktiválása utáni kattintás), a többi 20-ban `#F3F3F3`; *Dark* és *Auto* mindig helyes; indulás eltárolt
  *Light*-tal 2/2 helyes. A hibás eset után a következő *Light* váltás mindig világos címsort adott. Az ok
  valószínűleg időzítés: a DWM attribútum átáll (`0`), de a címsor nem rajzolódik újra.
- 2026-10-19 — a képernyő-automatizálás (kurzor, kattintások) a debug ablakon kívülre is kattintott, a munka
  leállítva, a felvételek törölve.
- 2026-10-19 — dropped: ritka, kozmetikai, egy újabb váltás megoldja (user decision).

## Deviations from the plan

- dropped after the reproduction, no fix

## Verification

- 2026-10-19 — reproduction only, see the Log; no code changed.
