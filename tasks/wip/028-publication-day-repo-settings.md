---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
plan: iterative-gliding-bentley.md
milestone: M5 Public release
---

# 028 — Publication day: visibility switch and the public-only repo settings

## Context

What: a checklist done in one sitting: visibility to public; a ruleset on `main` (CI required, no force push,
linear history); private vulnerability reporting; secret scanning with push protection; CodeQL (Rust, JavaScript
for `ui/`, actions); build provenance attestation in the release job; topics, homepage and a social preview image
(from a synthetic screenshot); Discussions on or off; GitHub's Community Standards page checked complete (023);
the README badges render (026); the first release after the switch (the first end-to-end run of the 025 release
job: notes from `CHANGELOG.md`, `SHA256SUMS.txt`, the licence page in the zip; D-012); after the switch, the
pre-rewrite backup folder from 030 deleted (D-010).
Why: these settings exist only for public repos or are free only there, so they cannot be prepared earlier than
the switch itself.
Done when: every item is applied or recorded in the Log with the reason it was left out.

Decided 2026-10-19:
- 028 runs ahead of 040 (the manual macOS test): standard runners are free for public repos.
- The first public release is 0.2.0; the macOS zip ships as an untested preview.
- The ruleset on `main` requires the full CI (test on windows-latest, lint, deny, workflow-lint, commit-rules), so
  the `paths-ignore` of `ci.yml` (042) goes away; macOS tests stay on tags and manual runs.
- CodeQL runs as GitHub's default setup, not as a workflow file. Discussions off. Homepage: the latest release.
- Before the switch, a content pass: no other project named or judged, no personal detail in docs or task files,
  the task files read well as public issues.

## Steps

1. [x] Content pass before the switch: other projects, personal detail, mirror-facing task text, contributor
   docs (git-workflow split), `.gitignore`, PR template, code of conduct; merged remote branches deleted
2. [x] CI and release: `paths-ignore` out, `--locked`, tag-on-main check, build provenance attestation, version
   0.2.0, CHANGELOG summary, social preview image; local checks; pull request
3. [ ] The switch (explicit go), then the public-only settings: secret scanning + push protection, private
   vulnerability reporting, Dependabot security updates, CodeQL default setup, description, homepage, topics
4. [ ] CI green on the pull request; rulesets on `main` and on `v*` tags; merge
5. [ ] 044 (macOS in the user docs) merged before the tag
6. [ ] Tag `v0.2.0`: release with both zips, `SHA256SUMS.txt`, notes, attestations verified
7. [ ] Community profile, badges, social preview uploaded, backup folder deleted (after asking)
8. [ ] Follow-ups filed; changelog stamped; close

## Log

- 2026-10-19 — promoted on `task/028-publication-day-repo-settings`
- 2026-10-19 — four reviews (docs, code, metadata/CI, task files), about 140 items: the pre-switch content goes
  into this task, the macOS user docs into 044, the rest into the backlog (045–048)
- 2026-10-19 — content pass on disjoint file sets: about 35 task and decision files, the docs, CONTRIBUTING, the
  PR template, the code of conduct, `.gitignore`; `git-workflow.md` split (`maintainer-sessions.md`); the
  leftover grep finds only the audit subject (D-009, 027, 030) and the hook's maintainer name (034). The remote
  holds only `main` (the merged branches were stale local remote-tracking refs, gone after `fetch --prune`)
- 2026-10-19 — CI and release: `paths-ignore` removed, `--locked` (CI, `release.ps1`, `bundle-macos.sh`), the
  tag-on-main step, `actions/attest-build-provenance` v4.2.2 with `subject-checksums`, the notes footer names
  `gh attestation verify`; version 0.2.0 (`cargo update --workspace` changes only the three workspace crates);
  the CHANGELOG summary; `docs/images/social-preview.png`, 1280×640, from `viewer.png`
- 2026-10-19 — local checks green: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  `cargo test --locked` (117 tests), `cargo deny check`, `uvx zizmor .github` (no findings),
  `changelog.py --check`, `tasks_check` 0/0; pull request #76
- 2026-10-19 — issue edit history is public on a public repo and held the pre-cleanup texts, so the 55 mirror
  issues (no comments, no other issues) were deleted and recreated by `scripts/sync-issues.py`: created 60, a
  second run unchanged 60. Issue numbers no longer equal task ids; the mirror finds its issues by marker
- 2026-10-19 — every task file is English and states decisions and results instead of reported answers;
  maintainer-only instructions (the local archive, the session playbook import) moved to a gitignored
  `CLAUDE.local.md`

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: 2026-10-..`
