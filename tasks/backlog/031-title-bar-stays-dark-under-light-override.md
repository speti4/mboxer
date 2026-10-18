---
status: idea
size: S
origin: wip/026-readme-front-page.md
created: 2026-10-18
---

# Native title bar stays dark when the Light override runs under dark-mode Windows

What: on a dark-mode Windows, switching the toolbar theme to *Light* left the native title bar dark while its text
turned dark, so the caption was hard to read (seen in the 026 screenshot runs, debug build, throwaway WebView2
profile). Check whether `setTheme("light")` reaches the window and whether a repaint or focus change fixes it.
Why: 017 promised the title bar follows the override. Done when: the caption matches the override in all three modes.
