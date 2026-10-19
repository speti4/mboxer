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
User decision 2026-10-19: part of the backlog wave (039 + 041 + 020 in task sessions); 037 runs in the main
session because the viewer and the manual round belong to one session at a time.

## Steps

1. [x] debug exe built from `main` (the code of this branch); `python -I scripts/make-demo-mbox.py`
2. [x] capture as in 026: throwaway WebView2 profile, `--lang=en-US`, the mbox on `subst M:` under a Takeout-like
   path, 1440×860, *Light* theme, splitter dragged, the Go-live conversation open with the attachment message
   selected, the title bar cropped
3. [x] the PNG checked by eye: dates in one column, only demo mail, no local path; the real profile untouched
4. [x] `docs/images/viewer.png` replaced

## Log

- 2026-10-19 — promoted on `task/037-refresh-the-viewer-screenshot`.
- 2026-10-19 — `cargo build -p mboxer-app` a main checkoutban (`5dbff23`, ugyanaz a kód, mint ezen az ágon);
  `make-demo-mbox.py`: 50 üzenet, 42 862 bájt, SHA-256 `069f772c…` (azonos a 026-ossal).
- 2026-10-19 — a 026-os scratchpad capture scriptje újrafelhasználva, egy kiegészítéssel: a kép előtt a kurzor
  az ablakon kívülre kerül (az első futáson a kijelölt sor alatti sorban tooltip maradt). Lépések: kattintás az
  üres readerbe (aktiválás), a téma gomb *Auto* → *Light* (az eldobható profil *Auto*-val indul, a Windows
  dark), a splitter 708 → 880, a Go-live conversation kinyitva, a második (csatolmányos) üzenet kijelölve, a
  címsor levágva → 1426×822, 108 KB (előtte 105 KB).

## Deviations from the plan

- none

## Verification

- 2026-10-19 — a kép szemmel ellenőrizve: a dátumok a conversation és a sima sorokban egy oszlopban
  (2025-10-11 … 2024-11-13), csak demo levél (`example.*` címek), a status bar `M:\Takeout\Mail\…` utat mutat,
  lokálisat nem. A valódi WebView2 profil (`%LOCALAPPDATA%\io.github.speti4.mboxer\`) legfrissebb fájlja előtte
  és utána is 2026-10-18 04:20:55; `subst M: /d` után nincs subst meghajtó, viewer nem fut.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (none; the dark title
  bar the crop hides is 031, already on the backlog)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index by the main session when it prepares the pull request)
- [x] moved to `done/` with `closed: 2026-10-19`
