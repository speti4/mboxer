---
size: S
origin: done/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
reason: rare and cosmetic; the next theme switch always draws the right title bar, so a fix is not worth its cost (decided 2026-10-19)
reopen-when: a user reports it, or tao changes how set_theme repaints the Windows title bar
---

# 031 — Native title bar stays dark when the Light override runs under dark-mode Windows

## Context

What: on a dark-mode Windows, switching the toolbar theme to *Light* left the native title bar dark while its text
turned dark, so the caption was hard to read (seen in the 026 screenshot runs, debug build, throwaway WebView2
profile). Check whether `setTheme("light")` reaches the window and whether a repaint or focus change fixes it.
Why: 017 promised the title bar follows the override. Done when: the caption matches the override in all three
modes.
Decided 2026-10-19: every open task is done before the repo goes public (028); 031 runs now.

## Log

- 2026-10-19 — promoted on `task/031-title-bar-stays-dark-under-light-override`.
- 2026-10-19 — reproduced (task session, debug build, throwaway WebView2 profile, demo mbox, dark Windows): the
  title bar's pixel colour stayed `#202020` in 2 of 22 *Light* samples (one normal switch and one click 30 ms after
  activating the window), `#F3F3F3` in the other 20; *Dark* and *Auto* always correct; a start with a stored
  *Light* 2/2 correct. After a failing case the next *Light* switch always gave a light title bar. The cause is
  probably timing: the DWM attribute switches (`0`), but the title bar is not repainted.
- 2026-10-19 — the screen automation (cursor, clicks) also clicked outside the debug window; the work was stopped
  and the recordings deleted.
- 2026-10-19 — dropped: rare, cosmetic, another switch fixes it (decided 2026-10-19).

## Deviations from the plan

- dropped after the reproduction, no fix

## Verification

- 2026-10-19 — reproduction only, see the Log; no code changed.
