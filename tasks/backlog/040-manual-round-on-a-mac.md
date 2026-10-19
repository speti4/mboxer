---
status: waiting
waits-for: macOS manual test pending (the zip of a `bundle-macos` run on main)
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
milestone: M4 macOS
---

# Manual round on a Mac: the bundle starts, the 019 runtime paths and the 020 print path work on a Mac

What: on a Mac, with the zip of a `bundle-macos` run (tag or manual dispatch, 018): the `.app` starts after the
Gatekeeper first-start step of D-007; Finder "Open with" and a Dock drop list the app for an `.mbox` (the document
type in `Info.plist`) and open the file through `RunEvent::Opened` (019); an opened attachment carries the
`com.apple.quarantine` xattr; an mbox in a read-only folder indexes under `~/Library/Application Support`; the
tooltips show `⌘` in WKWebView; `⌘P` opens a print window with the native panel over it, the message laid out as
in the reader (cid images included), *PDF › Save as PDF* writes the same picture, a second `⌘P` opens a second
window, and closing the main window closes the print windows and quits (020). Record what passed and what did
not in the Log.
Why: the done-when of 018, 019 and 020 that CI and a Windows build cannot check.
Done when: every point above is seen on a Mac, and each failure is a backlog item of its own.
