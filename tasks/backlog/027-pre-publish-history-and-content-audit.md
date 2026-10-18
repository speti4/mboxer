---
status: ready
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Pre-publish audit of the history and the content

What: `gitleaks` over the full git history; a PII scan over `docs/`, `tasks/` and the commit messages; an
inventory of what is private but not secret: local absolute paths (the `plan:` fields, the archive path in
CLAUDE.md), the benchmark and origin texts, the Hungarian Log sections, the attribution trailers on every commit.
The outcome is a decision record (D-09): what gets published (tasks/ as is, scrubbed, or left out) and whether
the history is kept as is or rewritten before the visibility switch.
Why: flipping a repo to public publishes its whole history at once and cannot be taken back; the project's own
rule is that no real message content ever enters the repo, and that has to be proven, not assumed.
Done when: the audit report is in this task's Log and D-09 is decided.
