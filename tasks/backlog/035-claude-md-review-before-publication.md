---
status: ready
size: S
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# CLAUDE.md fit for a public repo: no personal settings, no contradictions, a commands block

What: drop the "User communication: Hungarian" line (it belongs in the author's global file; a contributor's session
would answer in Hungarian); one rule for the dependency justification (task file vs `Cargo.toml` comment, today both,
CONTRIBUTING says the comment); point to CONTRIBUTING's checks instead of a stale list (cargo deny and zizmor since
024); cut what the repo already says (edition, the doubled English / kebab-case line); a short Commands block out of
the Scope paragraph; the Tasks section as one pointer line.
Why: review 2026-10-18 against the current memory / best-practices docs (keep it lean, no contradictions, no
frequently changing facts); the file is public from 028 on.
Done when: CLAUDE.md has no personal or contradicting line and a fresh session finds build, test and checks in it.
