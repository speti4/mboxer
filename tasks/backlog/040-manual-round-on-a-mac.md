---
status: waiting
waits-for: a Mac user with the macOS zip from a `bundle-macos` run (the author has no Mac)
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
milestone: M4 macOS
---

# Manual round on a Mac: the bundle starts and the 019 runtime paths work

What: on a Mac, with the zip of a `bundle-macos` run (tag or manual dispatch, 018): the `.app` starts after the
Gatekeeper first-start step of D-07; Finder "Open with" and a Dock drop list the app for an `.mbox` (the document
type in `Info.plist`) and open the file through `RunEvent::Opened` (019); an opened attachment carries the
`com.apple.quarantine` xattr; an mbox in a read-only folder indexes under `~/Library/Application Support`; the
tooltips show `⌘` in WKWebView. Record what passed and what did not in the Log.
Why: the done-when of 018 and 019 that neither CI nor the author's Windows machine can check.
Done when: every point above is seen on a Mac, and each failure is a backlog item of its own.
