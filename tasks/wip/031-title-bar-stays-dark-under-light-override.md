---
size: S
origin: wip/026-readme-front-page.md
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
---

# 031 — Native title bar stays dark when the Light override runs under dark-mode Windows

## Context

What: on a dark-mode Windows, switching the toolbar theme to *Light* left the native title bar dark while its text
turned dark, so the caption was hard to read (seen in the 026 screenshot runs, debug build, throwaway WebView2
profile). Check whether `setTheme("light")` reaches the window and whether a repaint or focus change fixes it.
Why: 017 promised the title bar follows the override. Done when: the caption matches the override in all three
modes.
User decision 2026-10-19: every open task is done before the repo goes public (028); 031 runs now, while the
viewer is free.

## Steps

1. [ ] reproduce on dark-mode Windows: start with a throwaway WebView2 profile, cycle *Auto → Light → Dark → Auto*,
   capture the window with its title bar after each step (and after a focus change)
2. [ ] find where it breaks: the UI call (`setTheme` in `crates/mboxer-app/ui/main.js`), the
   `core:window:allow-set-theme` capability, Tauri / tao's Windows `set_theme` (immersive dark mode attribute,
   whether the caption repaints without a non-client redraw)
3. [ ] fix at the right layer (a redraw after the theme change, or the theme set from Rust), Windows only if the
   cause is Windows only; startup with a stored *Light* / *Dark* also matches
4. [ ] the CONTRIBUTING.md checks; `node --check` if `ui/` changes
5. [ ] captures of all three modes, title bar included, after the fix

## Log

- 2026-10-19 — promoted on `task/031-title-bar-stays-dark-under-light-override`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
