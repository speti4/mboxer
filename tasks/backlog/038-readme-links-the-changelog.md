---
status: ready
size: S
origin: done/025-release-notes-changelog-and-checksums.md
created: 2026-10-18
milestone: M5 Public release
---

# README links the changelog next to the releases

What: the README's download step and its docs list point to the releases but not to `CHANGELOG.md`, which 025 added
(D-012); add the link, and a word on checking the zip against `SHA256SUMS.txt` where the download step is.
Why: a visitor reading the front page should find what changed and how to verify the download without the docs.
Done when: the README links `CHANGELOG.md` and names the checksum file at the download step.
