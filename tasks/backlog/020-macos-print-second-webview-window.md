---
status: ready
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M4 macOS
---

# Print on macOS through a second webview window

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
