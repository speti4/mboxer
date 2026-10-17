---
status: ready
size: M
origin: session 2026-10-17
created: 2026-10-17
milestone: M2 Viewer MVP
---

# Message reader with sanitized HTML

What: Reader pane: headers block, HTML body rendered in a sandboxed iframe (scripts stripped, remote content blocked by default with a per-message allow button), plain-text fallback, raw source view, copy and open-in-browser.
Why: HTML mail must look like mail; mboxviewer renders poorly and WebView2 does this natively.
Done when: Any message from the archive renders readably with no network requests leaving the app unless allowed.
Note from 005: the shell's CSP (`crates/mboxer-app/tauri.conf.json`) already allows `frame-src 'self' blob:`, `style-src 'unsafe-inline'` and `img-src data: blob:` for the iframe; verify that WebView2 admits `about:srcdoc` under `frame-src 'self'` and tighten the iframe with its own meta CSP. The body block to replace is `#msg-body` in `ui/index.html`; `BodyKind` in `src/types.rs` gets an `html` variant.
