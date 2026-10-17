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
