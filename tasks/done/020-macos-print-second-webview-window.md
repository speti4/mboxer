---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M4 macOS
---

# 020 — Print on macOS through a second webview window

## Context

What: on macOS `printMessage` hands the print document (cid images as `data:` URLs, not blob URLs, which do not
cross windows) to a `print_message` command; Rust opens a hidden `print` webview window with that HTML, calls
`Webview::print()` after the page has loaded (the native print panel, PDF through its *Save as PDF*), and closes
the window afterwards; a capability entry covers the `print` window. Windows keeps the task 011 iframe path.
Fallback if the second window does not print: a top-level `window.print()` with `@media print` CSS that leaves
only the reader body visible.
Why: Tauri/wry on macOS silently ignores `iframe.contentWindow.print()` (tauri-apps/tauri #13451, upstream, no
fix planned); the whole 011 print path depends on it. Full parity is the user's choice (2026-10-18).
Done when: on a Mac *Print…* (`⌘P`) opens the native panel with the message laid out as the reader shows it,
and *Save as PDF* writes the same picture; the Windows print preview is unchanged.
User decision 2026-10-19: part of the backlog wave 039 + 041 + 020 in parallel task sessions. The author has no
Mac: the macOS check is a cross-compile here, the run on a Mac goes into 040's manual round.

## Steps

1. [ ] read the 011 print path (`printMessage` in `crates/mboxer-app/ui/main.js`, the print document builder,
   the cid image handling) and the 019 platform switches (how the UI knows it runs on macOS)
2. [ ] macOS branch in `printMessage`: cid images inlined as `data:` URLs, the document handed to a new
   `print_message` command; the Windows iframe path untouched
3. [ ] Rust `print_message` (cfg macOS, a no-op or error elsewhere): hidden `print` webview window with the
   HTML, `Webview::print()` after the page load, the window closed afterwards; capability entry for `print`
   in `crates/mboxer-app/capabilities/`; the print window's CSP keeps remote content blocked
4. [ ] tests where the logic is testable without a window (the data-URL inlining, the command's input checks)
5. [ ] `DOCS_RS=1 cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings`, the CONTRIBUTING.md
   checks, `node --check` on `ui/main.js`
6. [ ] Windows manual check: *Print…* still opens the WebView2 print preview with the same picture
7. [ ] 040's checklist gains the macOS print point (Print… panel, *Save as PDF*) — through the pull request's
   `## Follow-ups`, since a task session does not edit another task file

## Log

- 2026-10-19 — promoted on `task/020-macos-print-second-webview-window`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
