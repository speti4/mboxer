---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# The usage guide says what the viewer does

What: `docs/usage.md` (and `docs/reference.md` where it repeats it) differs from the UI in several places: *Esc*
keeps the *Best match* sort while the text says it clears the whole query; only the first `label:` includes
sub-labels and later ones are exact matches; "window state" is named but nothing saves it; the failure screen is
"Could not open the file", not "Open failed in red"; `Ctrl+K`, *Save all…*, *Copy* and the Source view are not
documented. Fix the text or the behaviour, whichever is right, and read both files against `ui/main.js` and
`ui/query.js` once more end to end. `aria-label="Theme"` on the theme switch hides the visible Auto / Light / Dark
from screen readers; name the options.
Why: the guide ships in every zip as `usage.html`, so a wrong sentence reaches every user.
Done when: every shortcut and command of the UI has a line in the guide and no sentence contradicts the code.
