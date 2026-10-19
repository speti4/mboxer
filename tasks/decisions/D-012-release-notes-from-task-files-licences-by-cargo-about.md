---
status: decided
date: 2026-10-18
cost: cheap
sources: [done/025-release-notes-changelog-and-checksums.md, D-011-dependency-licence-and-source-policy.md]
---

# D-012 — Release notes and the changelog come from the done task files, the licence page from cargo-about

## Context

The v0.1.0 notes were written by hand because GitHub's generated notes were a bare "Full Changelog" line, and the
zip carried no notices for the MIT / Apache / MPL crates compiled into the exe. Task 025 built the release side:
the task files already hold the history (H1, milestone, the folder), and `deny.toml` already holds the licence
allowlist (D-011). The open questions were which source the notes take, how a task is assigned to a release when
several close on the tag's day, and what renders the licence page.

## Decision

`scripts/changelog.py` writes `CHANGELOG.md` from `tasks/done/` and prints one release's notes for the release job
(`gh release create --notes-file`): a task belongs to the first `vX.Y.Z` tag whose tree already holds it under
`done/` (the `closed` date cannot tell tasks closed on the tag's day apart), grouped by milestone, under a
hand-written summary per section that every run keeps. The release zip carries `THIRD-PARTY-LICENSES.html`
rendered by `cargo-about` (a pinned build-time tool, not in the exe), whose `about.toml` accepts exactly the
licences `deny.toml` allows, checked in CI; a `SHA256SUMS.txt` is published next to the zip. Decided on
2026-10-18.

## Not chosen

- GitHub's generated notes: PR titles only, and the task files stay the plan and the history (D-008).
- Assigning tasks by the `closed` date: several tasks closed on the v0.1.0 day on both sides of the tag.
- A mode of `scripts/sync-issues.py`: the release job would need `gh` and an issues token for an offline file job.
- A hand-kept `CHANGELOG.md`: it drifts from the task files.
- A hand-collected licence file: it repeats cargo-about's licence-file search and goes stale with every update.
