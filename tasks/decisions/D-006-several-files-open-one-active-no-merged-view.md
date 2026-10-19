---
status: decided
date: 2026-10-18
cost: medium
sources: [done/016-open-multiple-mbox-files.md, D-003-message-identity-is-envelope-byte-offset.md]
---

# D-006 — Several mbox files are open side by side, one active at a time; there is no merged view

## Context

Takeout splits a large archive into several mbox files and users keep several exports next to each
other. Task 016 had to settle whether the viewer merges them into one list and label tree (one
sidecar index per file, rows merged by date, threads joined across files) or keeps them apart. A
merged view needs a file id inside every row and listing of the core, a cross-index `Listing`, and
summed label counts; message identity is the file-relative byte offset (D-003), so nothing in an
index tells one file from another.

## Decision

- The app holds a **set of open files**, each with its own open index, label tree and listing; the
  folder pane lists them one under the other, and exactly **one is active**: its label tree is
  expanded, the list, the search and the reader belong to it. A click on another file activates it.
- **No merged list, no merged label tree, no cross-file search.** A thread split across two
  Takeout parts appears in each part on its own.
- The webview addresses a message as `(file, offset, len)`: `file` is a **per-process counter**
  handed out when a file is opened, never persisted and never written into an index. D-003 and
  D-004 are unchanged: the index stays per file, beside the file.
- *Open…* **adds** to the set (the dialog allows several files) instead of replacing the open file;
  a file is removed with its own close control. The set of paths and the active one are remembered
  in the webview's storage; at start-up only files whose index is fresh are reopened, the others
  wait in the pane until clicked.

Proposed on 2026-10-18 during task 016 planning, with the stacked layout (files one below the other, only one
expanded at a time); decided on 2026-10-18 and checked in the manual round of task 016.

## Not chosen

- A merged view across files: a cross-index listing in the core, a file id in every row and command,
  summed label counts and a batch-progress overlay — size L for a benefit (one search over the
  parts) that the scope did not call for; can still come later as a new decision.
- Replacing the open file on every *Open…* plus a recent-files list: the switch costs an index
  open each time and forgets the per-file query; the stacked pane gives the recent list for free.
- A persistent file id (path hash) in the message key: nothing survives the process that would
  need it; the temp attachment folder is cleaned at every start.
