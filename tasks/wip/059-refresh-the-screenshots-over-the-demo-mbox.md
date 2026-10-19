---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 059 — Refresh the README screenshot and the social preview over the demo mbox

## Context

What: `docs/images/viewer.png` and `docs/images/social-preview.png` show a `v0.1.0` status bar, the old
`Message-Id` and "index reused" strings. Retake both with the recipe of task 037 (throwaway WebView2 profile, demo
mbox on a neutral drive letter, `--lang=en-US`), on the current build, and update the alt text if the picture
changes.
Why: the front page and the social card are the first thing a visitor sees, and they show a version that no longer
exists.
Done when: both images show the current version and strings over made-up mail only, and the README renders them.
Decided 2026-10-19: taken in the main session over the maintainer's screen; the social preview is composed from the
new viewer picture the way 028 did it (1280×640, title banner) and uploaded in the repository settings by hand.

## Steps

1. [ ] `cargo build -p mboxer-app` on the branch, `python -I scripts/make-demo-mbox.py`
2. [ ] the capture over the demo mbox with the 037 recipe: light theme, the Go-live conversation expanded, the
   message with the attachment selected, the title bar cropped; the picture checked by eye before it is copied
3. [ ] `docs/images/social-preview.png` composed from the new picture, 1280×640, the same banner as before
4. [ ] README alt text still true; `subst` drive removed, no viewer running, the real WebView2 profile untouched

## Log

- 2026-10-19 — promoted on `task/059-refresh-the-screenshots` in the main session.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: 2026-10-19`
