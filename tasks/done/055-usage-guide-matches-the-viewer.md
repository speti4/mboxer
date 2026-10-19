---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
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

1. [x] a table in the Log: every shortcut, button, menu item and status-line text of `ui/index.html`, `ui/main.js`,
   `ui/query.js`, `ui/frame.js`, `ui/print.js`, against the sentence of `docs/usage.md` and `docs/reference.md` that
   covers it (or "missing")
2. [x] the fixes: Esc and the sort, the second `label:`, "window state", the failure screen's wording, `Ctrl+K`,
   *Save all…*, *Copy*, the Source view, the free-text syntax of task 050, the skipped-messages line of 052, the
   attachment types of 053; `aria-label` of the theme switch names the options
3. [x] `python -I scripts/render-usage-guide.py --check` (the renderer stops on Markdown it does not support), every
   relative link resolves, `cargo test` if a UI string changed (the Node tests), `node --check` on touched JS

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — the UI read end to end (`index.html`, `main.js`, `query.js`, `theme.js`, `files.js`, `frame.js`,
  `print.js`, `style.css`, `attachments.rs`, `commands.rs`, `tauri.conf.json`, the core's `error.rs`). The table:
  what the UI has, what the docs said before this task ("U" = `usage.md`, "R" = `reference.md`), and the fix.

| UI item (where in the code) | Before | Fix |
|---|---|---|
| *Open…* button, `Ctrl+O`, start screen *Open mbox files…* | U, R | U names the button *Open…* |
| start screen *Reopen <name>* buttons (`renderReopenList`) | missing | U, R |
| progress overlay: "Opening…", "Checking the index…", "Building the search index…", file N of M, MB/s, *Cancel* | U, R (progress, cancel) | R names the texts |
| failure screen "Could not open the file", *Retry*, *Open another file…* (`showFailure`) | U said "*Open failed* in red" | U, R: the screen, and "Open failed: …" in the status bar when another file is open |
| failure reasons: "no mbox envelope … at byte offset N", I/O error, "another build holds …; wait for it to finish" | U (partly) | U, R |
| file rows in the pane: click to switch, click on the active row = *All mail*, `×` closes or forgets, grey italic not-loaded rows | U, R (partly) | R complete |
| per-file query kept on a switch (`fileQueries`, in memory only) | U ("waits for you") | U, R: while mboxer runs, not saved |
| remembered files at start (`restoreFiles`), "N remembered files are gone" | U, R | R names the status text and the order |
| "window state" in the web view's data | U named it; nothing saves the window (no window-state plugin, `tauri.conf.json` opens 1400 × 900 centred) | U, R list what local storage holds: the files, theme, pane widths, *HTML*/*Text*, *Conversations*, filter panel open |
| splitters between the panes (`mboxer.paneWidths`) | missing | R |
| folder pane: click, *All mail*, chevrons | U, R | U adds *All mail* |
| folder pane keys: `ArrowUp`/`ArrowDown`/`Home`/`End`, `ArrowRight` unfold or to the list, `ArrowLeft` fold, `Enter` | R (fold only) | U, R |
| list keys: arrows, `PageUp`/`PageDown`, `Home`/`End`, `Enter`, `ArrowRight` (open thread or to reader), `ArrowLeft` (close thread or to folders) | U, R (partly) | U, R complete |
| conversation chevron: a click on the chevron opens, a click on the row only selects | R said "or a click" | R fixed |
| reader keys: `Esc`, `ArrowLeft` back to the list (also inside the body frame) | U, R (`Esc` only) | U, R |
| search field `Ctrl+F` / `Ctrl+K` (text selected) | U `Ctrl+F` only, R both | U adds `Ctrl+K` |
| `Esc` in the search field: empties the text, label included, keeps the sort (`main.js` keydown) | U, R said it clears the whole query | U, R: the sort stays; `Esc` in an empty field moves to the list |
| *Clear* in the panel: text and sort back to *Newest first* (`resetQueryModel`) | R grouped it with `Esc` | U, R |
| `Esc` in the filter panel, `Enter` in *From* move to the list | missing | R |
| `Ctrl+Shift+F`, *Filters* button and its badge | U, R (no badge) | R lists the panel's fields and what the badge counts |
| *Conversations* checkbox, *Sort* (*Newest first*, *Best match*) | U (no), R | U, R |
| `label:` first vs further (`query.js` `parse`) | U said every `label:` includes sub-labels | U, R: the first is the node with its sub-labels, further ones exact |
| free text of task 050: AND, `"phrase"`, `OR`, `-word`, `NOT`, trailing `*`, a colon is plain text | U (AND, phrase, OR, -word), R complete | U complete |
| query warnings "Query: after: expects a day as YYYY-MM-DD", "Query: has: expects `attachment`" | R (no text) | U, R name the text |
| highlighting in the list, the subject, the body (not over 2 million characters) | U, R | U, R |
| *HTML* / *Text* / *Source* (`setBodyMode`), *HTML* disabled without an HTML part | R | U new section *Reading a message*, R adds the details |
| *Copy* and "Copied" (`copyBody`) | R | U, R |
| remote bar "N remote images or styles blocked", *Show* | R, U (print only) | U |
| links open in the browser (`open_url`) | R | U |
| *MIME parts* list | missing | R |
| attachment chip, *Save…*, *Save all…* (shown from two attachments), "N files saved to <folder>" | U (partly), R | U, R |
| blocked attachment types (`BLOCKED_EXTENSIONS`, `MACOS_BLOCKED_EXTENSIONS`), chip tooltip "executable attachments can only be saved" | R (sampled list) | U short list, R both lists in full |
| *Save .eml…* `Ctrl+S`, "Saved <path>" | U, R | R: mboxrd quoting undone, status text |
| *Print…* `Ctrl+P`, print preview, *Save as PDF*, Mac print window (`print.js`) | U, R | R: the print document's parts; `Ctrl+P` without a message does nothing |
| theme button *Auto* → *Light* → *Dark*, tooltip (`theme.js`) | U, R | U, R: a click moves to the next mode |
| status bar: path, counts, "Index reused (built …)", "Index built in N s, K unparsed messages, J too large to index", tooltip with the index folder, version | U (skipped line only) | U, R (new *Status bar* section) |
| status flashes: "Saved …", "Copied", "Open cancelled", "Open failed: …", errors | missing | R |
| index locations (D-004, D-005), `.lock` file | U, R | unchanged, they match |

- 2026-10-19 — decided by the rule above: `Esc` keeps the sort order, as the code does. The sort is a choice
  beside the text, and *Best match* has no effect without words, so keeping it across `Esc` loses nothing;
  *Clear* stays the way to reset everything. No `main.js` change.
- 2026-10-19 — UI change: `aria-label="Theme"` removed from the theme button in `ui/index.html`. A fixed label
  replaced the visible *Auto* / *Light* / *Dark* as the accessible name; without it the name is the visible mode
  and the tooltip ("Theme: follows the system. Click for light.") is its description. No test covers the markup.
- 2026-10-19 — the attachment-folder row of the guide's "Where mboxer writes" table and the temp-folder
  sentences of the reference's *Attachments* section are left as they are: task 054 rewrites them.

## Deviations from the plan

- The theme button's `aria-label` is removed rather than rewritten: the visible text already names the current
  mode, and a fixed label listing all three modes would hide which one is on.
- The guide gained a *Reading a message* section (the views, *Copy*, remote images, attachments): those
  features had no place in the existing sections.

## Verification

- 2026-10-19 — `python -I scripts/render-usage-guide.py --check`: "usage.md renders (17 KB); self-test passed".
- 2026-10-19 — a link check over both docs (every relative Markdown link, the file and the `#anchor`): 5 relative
  links, 0 broken; the four bare URLs of the guide are unchanged.
- 2026-10-19 — no JS file changed, so no `node --check`; no string covered by a Node or Rust test changed (the
  tests under `crates/mboxer-app/tests/ui/` do not read `index.html`), so no `cargo test` run for this task.
- 2026-10-19 — each row of the table above checked against the code it names; the window size against
  `tauri.conf.json` and the plugin list in `lib.rs` (dialog and opener only).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none found)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
