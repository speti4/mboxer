---
status: idea
size: M
origin: session 2026-10-17
created: 2026-10-17
milestone: M3 Comfort
---

# Open multiple mbox files

What: The app handles more than one `.mbox` file. Two readings to settle when the task is promoted: (a) several files open at once in one merged list and label tree (one sidecar index per file, results merged by date), (b) at least a quick switch between files (recent-files list, each with its own index). Today one path comes from the command line and one index is open.
Why: Takeout splits large archives into several files, and the other users may keep several exports side by side.
Done when: The user opens a second mbox without restarting the app; list, search and labels reflect the chosen file (or all open files, per the chosen reading).
Note: Message identity is the file-relative envelope byte offset (D-03), so a multi-file view needs a file id in the message key.
