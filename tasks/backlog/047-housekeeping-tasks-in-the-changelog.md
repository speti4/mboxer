---
status: idea
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
---

# Keep project housekeeping apart from user-facing changes in the changelog

What: `scripts/changelog.py` lists tasks such as the history rewrite, the CLAUDE.md review or the id padding next
to features; give them a group of their own ("Project housekeeping") or a frontmatter flag the generator honours.
Why: release notes are read by users, who look for what changed in the app.
Done when: the next release's notes show the user-facing changes first and the housekeeping apart or not at all.
