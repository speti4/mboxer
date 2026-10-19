---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
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
Why: Tauri/wry on macOS silently ignores `iframe.contentWindow.print()` (tauri-apps/tauri#13451, upstream, no
fix planned); the whole 011 print path depends on it. Decided 2026-10-18: full parity.
Done when: on a Mac *Print…* (`⌘P`) opens the native panel with the message laid out as the reader shows it,
and *Save as PDF* writes the same picture; the Windows print preview is unchanged.
Done in parallel with 039 and 041. The macOS check is a cross-compile here; the run on a Mac goes into 040's
manual round.

## Steps

1. [x] read the 011 print path (`printMessage` in `crates/mboxer-app/ui/main.js`, the print document builder,
   the cid image handling) and the 019 platform switches (how the UI knows it runs on macOS)
2. [x] macOS branch in `printMessage`: cid images inlined as `data:` URLs, the document handed to a new
   `print_message` command; the Windows iframe path untouched
3. [x] Rust `print_message` (cfg macOS, a no-op or error elsewhere): hidden `print` webview window with the
   HTML, `Webview::print()` after the page load, the window closed afterwards; capability entry for `print`
   in `crates/mboxer-app/capabilities/`; the print window's CSP keeps remote content blocked
4. [x] tests where the logic is testable without a window (the data-URL inlining, the command's input checks)
5. [x] `DOCS_RS=1 cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings`, the CONTRIBUTING.md
   checks, `node --check` on `ui/main.js`
6. [ ] Windows manual check: *Print…* still opens the WebView2 print preview with the same picture — a manual
   check after the pull request; unticked in its `## Checks`
7. [x] 040's checklist gains the macOS print point (Print… panel, *Save as PDF*) — through the pull request's
   `## Follow-ups`, since a task session does not edit another task file

## Log

- 2026-10-19 — promoted on `task/020-macos-print-second-webview-window`.
- 2026-10-19 — API source (Cargo.lock: tauri 2.12.4, tauri-runtime-wry 2.12.2, wry 0.57.0, from the registry
  sources): `WebviewWindowBuilder` + `WebviewUrl::CustomProtocol`, `on_page_load` (`PageLoadEvent::Finished`),
  `on_navigation`, `on_new_window`, `Builder::register_uri_scheme_protocol` (the `UriSchemeContext` gives the
  requesting webview's label), `WebviewWindow::print()`. wry's macOS `print()`: `printOperationWithPrintInfo` +
  `runOperationModalForWindow(webview.window(), delegate: None)` — the panel opens as a sheet on the webview's own
  window, with no report of its end. On macOS the tauri `webview` plugin replaces the top-level `window.print()`
  with `plugin:webview|print` (the same call); per #13451 this works, only the iframe `print()` is silent.
- 2026-10-19 — Choice: **a second window**, not the fallback. A top-level `window.print()` would print the
  three-pane app: with `@media print` only the reader would remain, but the HTML body is an iframe sized to its
  content, which WebKit does not reliably break at page boundaries; the separate window prints a standalone print
  document identical to the Windows one. The window is **visible** (820×1000, centred, titled `Print: <subject>`),
  not hidden: the panel goes onto its window as a sheet, a sheet on a hidden window could stay invisible, and as
  wry gives no completion signal the window cannot safely be closed after printing (closing it under an open
  sheet aborts the print). So the window doubles as the preview, like WebView2's; it is closed with `⌘W` and
  closes together with the main window (otherwise the app would not quit).
- 2026-10-19 — Rust: `src/print.rs`. `PrintJobs` (in `AppState`): job id → HTML + `allow_remote` + `printed`;
  every print gets its own `print-<id>` window (the content is not swapped under an open sheet). The
  `mboxer-print:` scheme (registered on macOS only) gives the document only to its own window, by the requesting
  webview's label; every other webview (the main one too) gets a 404; the response header is
  `Content-Security-Policy: default-src 'none'; img-src data:[ https: http:]; style-src 'unsafe-inline'` (remote
  images only if they were allowed in the reader), plus the document's own CSP meta copied from the reader frame.
  `on_page_load` Finished → `take_print` (once; a reload does not print again) → `print()` from a separate thread,
  so it runs on the main thread as an event after the navigation callback (from the main thread
  `run_on_main_thread` would run inline). `on_navigation` allows only the scheme, `on_new_window` → `Deny`.
  `on_window_event` Destroyed: a print window drops its job, closing `main` closes the print windows too.
  `print_message` (async, so the window is not built in a synchronous command): `check_html` (not empty,
  ≤ 128 MB), `open_window` on macOS, an error elsewhere.
- 2026-10-19 — UI: new `ui/print.js` module (`window.MboxerPrint`, CommonJS under Node): `usesPrintWindow(os)`,
  `dataUrl(type, bytes)` (base64 in 32 KB chunks, without the media type's parameters), `inlineBlobImages(doc,
  lookup)` (every `blob:` image becomes a `data:` URL, each blob read once; an unknown blob → `src` removed).
  `fillCidImages` also remembers the Blob next to the blob URL (`state.cidBlobs`, cleared on a message switch).
  `printDocument` returns the document, `serializeDocument` produces the same `<!doctype html>…` string as before,
  so the Windows arm (`el.printFrame.srcdoc = …`) is byte-identical. On a Mac `printInWindow`: inlining, then
  `print_message` `{ html, subject, allowRemote }`; an error → the status line.
- 2026-10-19 — Capability: no new entry under `capabilities/` (see Deviations).

## Deviations from the plan

- The print window is visible and stays open until the user closes it (or the main window closes), instead of
  a hidden window closed after printing: wry opens the print panel as a sheet on the webview's own window and
  reports no end (`runOperationModalForWindow` with no delegate), so a hidden window could hide the panel and the
  app cannot tell when closing is safe. The window doubles as the preview, as WebView2's does on Windows.
- One window per print (`print-<id>`), not a single `print` window: a second *Print…* while a panel is open on
  the first window must not swap the document under it.
- No capability file for the print window: it runs no script (the page CSP has no `script-src`, the sanitizer
  drops scripts), so it needs no permission, and a capability could only widen what it may call. The
  `mboxer-print:` scheme is a registered custom protocol, which Tauri counts as a local origin; the app has no
  command manifest, so this changes nothing for the main window either.
- `docs/reference.md` and `docs/development.md` are edited as well as `docs/usage.md` (CONTRIBUTING asks for the
  reference with every user-facing change; development.md gains the two print paths).

## Verification

- 2026-10-19, Windows 11, the worktree: `cargo fmt --check` clean; `cargo clippy --all-targets -- -D warnings`
  clean; `DOCS_RS=1 cargo clippy --target aarch64-apple-darwin --all-targets -- -D warnings` clean (the macOS
  `open_window`, `serve`, the scheme registration and the `print_message` macOS arm type-check); `cargo test`
  green (mboxer-app 42 unit tests, 6 of them new in `print::tests`: labels, the page CSP with and without remote
  images, titles, the size check, a window gets only its own document with the CSP header, a job prints once and
  goes with its window; the node runner with `print.test.js` added); `node --check` on `ui/main.js` and
  `ui/print.js` clean; `node --test tests/ui/print.test.js`: 4 pass (OS switch, data URLs incl. 100 kB through
  the chunking, blob images inlined / unknown ones dropped / data and remote ones kept, one read per blob).
- Not verified here (no Mac): the print window, the native panel and *Save as PDF* on a Mac — the 040 manual
  round (pull request follow-up). The Windows print preview unchanged: a manual check after the pull request (the
  Windows path builds the same string into the same frame).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the follow-ups are
  in the pull request description and are filed before the merge
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — `tasks_check` without `write` here; the index is regenerated
  before the merge
- [x] moved to `done/` with `closed: 2026-10-19`
