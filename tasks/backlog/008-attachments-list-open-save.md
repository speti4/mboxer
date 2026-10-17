---
status: ready
size: M
origin: session 2026-10-17
created: 2026-10-17
milestone: M2 Viewer MVP
---

# Attachments: list, open, save

What: Attachment strip in the reader: name, size, type; open with the default app (temp file), save as, save all. Decoding is done on demand from the message offset, never at index time.
Why: Must-have for the MVP (user decision 2026-10-17): the archive is often consulted for a file.
Done when: Attachments from the archive open and save correctly for base64 and quoted-printable parts, including inline images referenced by cid.
Note from 007: the HTML sanitizer (`sanitizeHtml` in `ui/main.js`) strips `cid:` image sources and leaves the content id in `data-mboxer-cid` on the `<img>`; this task fills those images (a `data:` URL from the decoded part, or a command that serves the part) when it builds the srcdoc. `tauri-plugin-opener` is already registered: `open_path` opens the temp file with the default app, from Rust.
