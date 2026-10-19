---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
---

# 055 — The usage guide says what the viewer does

## Context

What: `docs/usage.md` (and `docs/reference.md` where it repeats it) differs from the UI in several places: *Esc*
keeps the *Best match* sort while the text says it clears the whole query; only the first `label:` includes
sub-labels and later ones are exact matches; "window state" is named but nothing saves it; the failure screen is
"Could not open the file", not "Open failed in red"; `Ctrl+K`, *Save all…*, *Copy* and the Source view are not
documented. Fix the text or the behaviour, whichever is right, and read both files against `ui/main.js` and
`ui/query.js` once more end to end. `aria-label="Theme"` on the theme switch hides the visible Auto / Light / Dark
from screen readers; name the options.
Why: the guide ships in every zip as `usage.html`, so a wrong sentence reaches every user.
Done when: every shortcut and command of the UI has a line in the guide and no sentence contradicts the code.
Decided 2026-10-19: filed by the post-release review; where the text and the code disagree, the code wins unless the
documented behaviour is clearly the better one, and then the change is a one-line fix in the UI, noted in the Log.

## Steps

1. [ ] a table in the Log: every shortcut, button, menu item and status-line text of `ui/index.html`, `ui/main.js`,
   `ui/query.js`, `ui/frame.js`, `ui/print.js`, against the sentence of `docs/usage.md` and `docs/reference.md` that
   covers it (or "missing")
2. [ ] the fixes: Esc and the sort, the second `label:`, "window state", the failure screen's wording, `Ctrl+K`,
   *Save all…*, *Copy*, the Source view, the free-text syntax of task 050, the skipped-messages line of 052, the
   attachment types of 053; `aria-label` of the theme switch names the options
3. [ ] `python -I scripts/render-usage-guide.py --check` (the renderer stops on Markdown it does not support), every
   relative link resolves, `cargo test` if a UI string changed (the Node tests), `node --check` on touched JS

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
