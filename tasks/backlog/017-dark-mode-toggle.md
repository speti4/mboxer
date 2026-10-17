---
status: idea
size: S
origin: session 2026-10-17
created: 2026-10-17
milestone: M3 Comfort
---

# Dark mode toggle with OS default

What: A dark theme for the shell. The default follows the OS (`prefers-color-scheme`); a toolbar toggle overrides it (auto / light / dark) and the choice is persisted. `ui/style.css` already defines every colour as a token on `:root` (`--bg`, `--fg`, `--accent`, ...), so the work is a second token set under the media query plus a `data-theme` override, and a check that the sanitized reader iframe, the mark and the selection colours stay readable.
Why: The viewer is used for long sessions and OS-wide dark mode is common on the users' machines.
Done when: With the OS in dark mode the app starts dark; the toggle flips the theme live; the setting survives a restart.
