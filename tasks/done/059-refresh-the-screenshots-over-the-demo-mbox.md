---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 059 — Refresh the README screenshot and the social preview over the demo mbox

## Context

What: `docs/images/viewer.png` and `docs/images/social-preview.png` showed a `v0.1.0` status bar, the old
`Message-Id` and "index reused" strings. Both retaken with the recipe of task 037 (throwaway WebView2 profile, demo
mbox on a neutral drive letter, `--lang=en-US`), on the current build.
Why: the front page and the social card are the first thing a visitor sees, and they showed a version that no
longer exists.
Done when: both images show the current version and strings over made-up mail only, and the README renders them.
Decided 2026-10-19: taken in the main session; the social preview is composed from the new viewer picture the way
028 did it (1280×640, title banner) and uploaded in the repository settings by hand.

## Steps

1. [x] `cargo build --release -p mboxer-app` on the branch, `python -I scripts/make-demo-mbox.py`
2. [x] the capture over the demo mbox with the 037 recipe: light theme, the Go-live conversation expanded, the
   message with the attachment selected, the title bar cropped; the picture checked by eye before it is copied
3. [x] `docs/images/social-preview.png` composed from the new picture, 1280×640, the same banner as before
4. [x] README alt text still true; `subst` drive removed, no viewer running, the real WebView2 profile untouched

## Log

- 2026-10-19 — promoted on `task/059-refresh-the-screenshots` in the main session.
- 2026-10-19 — the demo mbox (50 messages, 42 862 bytes) copied under a `subst M:` drive as
  `M:\Takeout\Mail\All mail Including Spam and Trash.mbox`, so the status bar shows no local path.
- 2026-10-19 — the first run used the debug exe, whose console window landed in front of the viewer on the
  capture; the release exe (`windows` subsystem, no console) was built and used instead. The first run also
  rebuilt the sidecar index (the format changed since the last picture), so the second run shows "Index reused".
- 2026-10-19 — the capture script of task 037 reused from a scratch copy: window 1440×860, a click into the empty
  reader (activation), the theme button *Auto* → *Light*, the list/reader splitter 708 → 880, the Go-live
  conversation expanded through its chevron, the second message (with the attachment) selected, the cursor parked
  outside the window, the title bar cropped → 1426×822, 108 KB.
- 2026-10-19 — the social preview composed with the 028 script (Pillow 12.2): the banner text unchanged, the new
  picture scaled under it → 1280×640, 225 KB. The upload in the repository settings is the maintainer's manual
  step after the merge.

## Deviations from the plan

- The release exe instead of the debug one, because the debug build opens a console window that can cover the
  viewer on the capture.

## Verification

- 2026-10-19 — the picture checked by eye: the status bar reads `v0.2.0` and "Index reused (built …)", the reader
  shows `Message-ID`, only demo mail (`example.*` addresses), the `M:\Takeout\Mail\…` path and no local one; the
  layout matches the previous picture (same rows, same selection), so the README alt text stays true. The social
  preview shows the same picture under the banner. After the run: `subst M: /d` done, no `mboxer` process, the
  newest file of the real WebView2 profile is older than the run (22:05 against a 23:2x capture), the throwaway
  profile and the demo copy removed.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-19`
