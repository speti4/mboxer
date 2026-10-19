---
size: S
origin: done/032-list-date-column-alignment-in-conversation-view.md
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
---

# 037 — Refresh the README screenshot after the date column fix

## Context

What: regenerate `docs/images/viewer.png` over the demo mbox, which still shows the ragged date column that 032
fixed; the throwaway WebView2 profile, subst drive and `--lang` setup of 026 keep the real archive out of it.
Why: the screenshot is the README's first picture and goes public with 028.
Done when: the committed screenshot shows the dates in one column and nothing but demo mail.
Decided 2026-10-19: done alongside 039, 041 and 020; 037 drives the viewer, which one piece of work
uses at a time.

## Steps

1. [x] debug exe built from `main` (the code of this branch); `python -I scripts/make-demo-mbox.py`
2. [x] capture as in 026: throwaway WebView2 profile, `--lang=en-US`, the mbox on `subst M:` under a Takeout-like
   path, 1440×860, *Light* theme, splitter dragged, the Go-live conversation open with the attachment message
   selected, the title bar cropped
3. [x] the PNG checked by eye: dates in one column, only demo mail, no local path; the real profile untouched
4. [x] `docs/images/viewer.png` replaced

## Log

- 2026-10-19 — promoted on `task/037-refresh-the-viewer-screenshot`.
- 2026-10-19 — `cargo build -p mboxer-app` in the main checkout (`5dbff23`, the same code as this branch);
  `make-demo-mbox.py`: 50 messages, 42 862 bytes, SHA-256 `069f772c…` (identical to 026's).
- 2026-10-19 — the 026 scratchpad capture script reused, with one addition: before the shot the cursor moves
  outside the window (in the first run a tooltip stayed on the row below the selected one). Steps: a click into the
  empty reader (activation), the theme button *Auto* → *Light* (the throwaway profile starts with *Auto*, Windows
  is dark), the splitter 708 → 880, the Go-live conversation expanded, the second message (with the attachment)
  selected, the title bar cropped → 1426×822, 108 KB (105 KB before).

## Deviations from the plan

- none

## Verification

- 2026-10-19 — the picture checked by eye: the dates of conversation and plain rows in one column
  (2025-10-11 … 2024-11-13), only demo mail (`example.*` addresses), the status bar shows the `M:\Takeout\Mail\…`
  path, no local one. The newest file of the real WebView2 profile (`%LOCALAPPDATA%\io.github.speti4.mboxer\`) is
  2026-10-18 04:20:55 before and after; after `subst M: /d` there is no subst drive and no viewer running.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none; the dark title
  bar the crop hides is 031, already on the backlog)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index by the main session when it prepares the pull request)
- [x] moved to `done/` with `closed: 2026-10-19`
