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
Why: Tauri/wry on macOS silently ignores `iframe.contentWindow.print()` (tauri-apps/tauri #13451, upstream, no
fix planned); the whole 011 print path depends on it. Full parity is the user's choice (2026-10-18).
Done when: on a Mac *Print…* (`⌘P`) opens the native panel with the message laid out as the reader shows it,
and *Save as PDF* writes the same picture; the Windows print preview is unchanged.
User decision 2026-10-19: part of the backlog wave 039 + 041 + 020 in parallel task sessions. The author has no
Mac: the macOS check is a cross-compile here, the run on a Mac goes into 040's manual round.

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
6. [ ] Windows manual check: *Print…* still opens the WebView2 print preview with the same picture — the
   user's, after the pull request (the main session holds the one manual round); unticked in its `## Checks`
7. [x] 040's checklist gains the macOS print point (Print… panel, *Save as PDF*) — through the pull request's
   `## Follow-ups`, since a task session does not edit another task file

## Log

- 2026-10-19 — promoted on `task/020-macos-print-second-webview-window`.
- 2026-10-19 — API-forrás (Cargo.lock: tauri 2.12.4, tauri-runtime-wry 2.12.2, wry 0.57.0, a registry
  forrásából): `WebviewWindowBuilder` + `WebviewUrl::CustomProtocol`, `on_page_load` (`PageLoadEvent::Finished`),
  `on_navigation`, `on_new_window`, `Builder::register_uri_scheme_protocol` (a `UriSchemeContext` megadja a kérő
  webview címkéjét), `WebviewWindow::print()`. A wry macOS `print()`: `printOperationWithPrintInfo` +
  `runOperationModalForWindow(webview.window(), delegate: None)` — a panel sheetként a webview saját ablakán
  nyílik, és nincs visszajelzés a végéről. A tauri `webview` plugin macOS-en a felső `window.print()`-et
  `plugin:webview|print`-re (ugyanerre) cseréli; a #13451 szerint ez működik, csak az iframe `print()` néma.
- 2026-10-19 — Választás: **második ablak**, nem a fallback. A felső `window.print()` a háromtáblás appot
  nyomtatná: `@media print`-tel csak a reader maradna, de a HTML-törzs egy tartalomra méretezett iframe, amit a
  WebKit oldalhatáron nem tör megbízhatóan; a külön ablak egy önálló, a Windows-szal azonos print dokumentumot
  nyomtat. Az ablak **látható** (820×1000, középen, címe `Print: <tárgy>`), nem rejtett: a panel sheetként az ő
  ablakára kerül, egy rejtett ablakon a sheet láthatatlan maradhat, és mivel a wry nem ad befejezés-jelzést, az
  ablakot nem lehet a nyomtatás után biztonsággal bezárni (egy nyitott sheet alól bezárva a nyomtatás
  megszakad). Így az ablak egyben előnézet, mint a WebView2-é; a user zárja (`⌘W`), és a fő ablakkal együtt
  záródik (különben az app nem lépne ki).
- 2026-10-19 — Rust: `src/print.rs`. `PrintJobs` (az `AppState`-ben): job id → HTML + `allow_remote` +
  `printed`; minden nyomtatás saját `print-<id>` ablakot kap (egy nyitott sheet alatt nem cserélődik a tartalom).
  Az `mboxer-print:` séma (csak macOS-en regisztrálva) a kérő webview címkéje alapján csak a saját ablakának adja
  a dokumentumot, minden más webview (a fő is) 404-et kap; a válasz fejléce `Content-Security-Policy: default-src
  'none'; img-src data:[ https: http:]; style-src 'unsafe-inline'` (a remote képek csak akkor, ha a user a
  readerben engedte), plusz a dokumentum saját, a reader-frame-ből másolt CSP meta-ja. `on_page_load` Finished →
  `take_print` (egyszer, egy reload nem nyomtat újra) → `print()` egy külön szálról, hogy a navigációs callback
  után, eseményként fusson a fő szálon (fő szálról a `run_on_main_thread` inline futna). `on_navigation` csak a
  sémán belül enged, `on_new_window` → `Deny`. `on_window_event` Destroyed: a print ablak eldobja a jobját, a
  `main` bezárása a print ablakokat is. `print_message` (async, hogy az ablak ne szinkron commandban épüljön):
  `check_html` (nem üres, ≤ 128 MB), macOS-en `open_window`, máshol hiba.
- 2026-10-19 — UI: új `ui/print.js` modul (`window.MboxerPrint`, Node alatt CommonJS): `usesPrintWindow(os)`,
  `dataUrl(type, bytes)` (base64 32 KB-os darabokban, a média típus paraméterei nélkül), `inlineBlobImages(doc,
  lookup)` (minden `blob:` kép `data:` URL lesz, blobonként egyszer olvasva; ismeretlen blob → `src` törölve). A
  `fillCidImages` a blob URL mellé a Blob-ot is megjegyzi (`state.cidBlobs`, üzenetváltáskor törlődik). A
  `printDocument` a dokumentumot adja vissza, a `serializeDocument` ugyanazt a `<!doctype html>…` stringet
  állítja elő, mint eddig, így a Windows ág (`el.printFrame.srcdoc = …`) bájtra azonos. Macen
  `printInWindow`: inlining, majd `print_message` `{ html, subject, allowRemote }`; hiba → status sor.
- 2026-10-19 — Capability: nem került új bejegyzés a `capabilities/` alá (lásd Deviations).

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
  round (pull request follow-up). The Windows print preview unchanged: the user's manual check after the pull
  request (the Windows path builds the same string into the same frame).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — task session:
  the follow-ups are in the pull request description, the main session files them
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — task session: `tasks_check` without `write`, the index is
  regenerated by the main session before the merge
- [x] moved to `done/` with `closed: 2026-10-19`
