---
size: S
origin: done/032-list-date-column-alignment-in-conversation-view.md
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
---

# 037 — Refresh the README screenshot after the date column fix

## Context

What: regenerate `docs/images/viewer.png` over the demo mbox, which still shows the ragged date column that 032
fixed; the throwaway WebView2 profile, subst drive and `--lang` setup of 026 keep the real archive out of it.
Why: the screenshot is the README's first picture and goes public with 028.
Done when: the committed screenshot shows the dates in one column and nothing but demo mail.
User decision 2026-10-19: part of the backlog wave (039 + 041 + 020 in task sessions); 037 runs in the main
session because the viewer and the manual round belong to one session at a time.

## Steps

1. [ ] debug exe built from `main` (the code of this branch); `python -I scripts/make-demo-mbox.py`
2. [ ] capture as in 026: throwaway WebView2 profile, `--lang=en-US`, the mbox on `subst M:` under a Takeout-like
   path, 1440×860, *Light* theme, splitter dragged, the Go-live conversation open with the attachment message
   selected, the title bar cropped
3. [ ] the PNG checked by eye: dates in one column, only demo mail, no local path; the real profile untouched
4. [ ] `docs/images/viewer.png` replaced

## Log

- 2026-10-19 — promoted on `task/037-refresh-the-viewer-screenshot`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
