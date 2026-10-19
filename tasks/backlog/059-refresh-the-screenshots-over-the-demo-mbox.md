---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Refresh the README screenshot and the social preview over the demo mbox

What: `docs/images/viewer.png` and `docs/images/social-preview.png` show a `v0.1.0` status bar, the old
`Message-Id` and "index reused" strings, and an archive path that is not the demo mbox of `docs/development.md`.
Retake both with the recipe of task 037 (throwaway profile, demo mbox on a neutral drive letter, `--lang`), on the
current build, and update the alt text if the picture changes.
Why: the front page and the social card are the first thing a visitor sees, and they show a version that no longer
exists.
Done when: both images show the current version and strings over made-up mail only, and the README renders them.
