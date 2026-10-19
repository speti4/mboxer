---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# Attachment and reader hardening: blocked extensions, reserved names, referrer, style text

What: `BLOCKED_EXTENSIONS` in `crates/mboxer-app/src/attachments.rs` misses types that carry code or bypass the mark
of the web (`chm`, `msix`, `msixbundle`, `appinstaller`, `iso`, `img`, `vhd`, `vhdx`, `xll`, `scf`, `settingcontent-ms`,
`library-ms`, `search-ms`, `wsc`, `sct`, `cab`; on macOS `mobileconfig`, `zsh`, `bash`, `csh`, `prefpane`, `jnlp`), and
`RESERVED_NAMES` checks only the last stem, so `CON.tar.gz` passes. The sanitized frame has no `no-referrer` meta, so
a remote image loaded on *Show* sends the app origin as referrer. `scrubCss` in `ui/main.js` deletes `@import …;`
from style text, which can join the remaining pieces into a closing tag the sanitizer never saw; the frame's CSP and
sandbox still contain it, but the removal should leave `/**/` or escape `<` as `\3c`. One pull request, a test per item.
Why: these are the layers behind the sanitizer; each gap is cheap to close now and expensive after a report.
Done when: the new extensions can only be saved, a double extension is caught, the frame declares the referrer
policy, and the style-text test keeps `</style>` out of the srcdoc.
