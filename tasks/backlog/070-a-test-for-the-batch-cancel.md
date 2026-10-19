---
status: idea
size: S
origin: done/061-small-hygiene-follow-ups-from-the-review.md
created: 2026-10-20
milestone: M6 Hardening
---

# A test for the Cancel between two files of a batch

What: task 061 made the cancel flag clear once per batch and checked before each file, so a Cancel between two
files stops the next one, but nothing tests it: the file loop sits inside the `open_mbox` Tauri command. Move the
loop into a function of `state.rs` that takes the paths, the cancel flag and a progress hook, and drive it from a
unit test that cancels after the first file.
Why: a behaviour fix without a test is the next regression.
Done when: the test opens two fixture files, cancels after the first, and sees the second untouched.
