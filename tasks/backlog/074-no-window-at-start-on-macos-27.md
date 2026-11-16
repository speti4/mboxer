---
status: waiting
waits-for: the manual round on macOS 27 with a build that holds task 073, and the output of a start from Terminal
size: S
origin: done/073-macos-bundle-start-up-smoke-test.md
created: 2026-11-16
milestone: M4 macOS
hub-action: personal/projects/mboxer/actions/2026-11-16-macos-27-no-window-at-start.md
---

# No window at start on macOS 27.0.1: the app runs with its Dock icon only

What: the manual round on a Mac (040) found that release 0.3.0 starts on macOS 27.0.1 and shows its Dock icon,
but no window. On macOS 26.7.2 the same bundle opens its window (the smoke test of 073), so the fault is
specific to macOS 27 or to that Mac; 073 fixed a window placed partly off a small screen, which may or may not
be this fault, and the later changelogs of tao, wry and Tauri name no fix for it.
Next: a build with 073 on that Mac; if the window is still missing, the output of a start from Terminal and a
`sample` of the process (`scripts/smoke-macos.sh` collects both).
Why: the first point of the manual round, the app has to start.
Done when: the app shows its window on macOS 27.0.1, or the cause is known and filed where it belongs.
