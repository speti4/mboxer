---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M5 Public release
changelog: housekeeping
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
3. [x] The switch (explicit go), then the public-only settings: secret scanning + push protection, private
   vulnerability reporting, Dependabot security updates, CodeQL default setup, description, homepage, topics
4. [x] CI green on the pull request; rulesets on `main` and on `v*` tags; merge
5. [x] 044 (macOS in the user docs) merged before the tag
6. [x] Tag `v0.2.0`: release with both zips, `SHA256SUMS.txt`, notes, attestations verified
7. [x] Community profile, badges, social preview uploaded, backup folder deleted (after asking)
8. [x] Follow-ups filed; changelog stamped; close

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
  `CLAUDE.local.md`. A second delete-and-recreate of the 60 issues kept them free of edit history
- 2026-10-19 — the switch: `gh repo edit --visibility public`; then secret scanning and push protection,
  private vulnerability reporting, Dependabot security updates, CodeQL default setup (Rust, JavaScript/TypeScript,
  Python, Actions; first analysis 0 alerts), the description, the homepage (the latest release), 11 topics,
  Discussions off — each read back through the API
- 2026-10-19 — rulesets: `main` (pull request with squash only, no approval required, required checks
  `test (windows-latest)`, `lint`, `deny`, `workflow-lint`, `commit-rules`, linear history, no force push or
  deletion, no bypass) and `release tags` (`refs/tags/v*`: no update, deletion or force push). #76 merged under
  the ruleset
- 2026-10-19 — a Dependabot security job failed on GHSA-wrw7-89jp-8q8g (glib < 0.20): glib 0.18 comes only
  through Tauri's Linux graph (gtk 0.18), absent from the Windows and macOS builds and unfixable without gtk-rs
  0.20; the alert was dismissed as `not_used` with that reason
- 2026-10-19 — the pre-release round as parallel task sessions: 044 (#198), 047 (#197), 045 (#200) and 046
  (#199) merged in that order, each rebased on the previous merge with the index and the changelog regenerated;
  047's housekeeping group applied to 046 and to this task. After #199, `msrv` became a required check
- 2026-10-19 — `v0.2.0` tagged on `main` (18f575d, its CI green); the release run passed all ten jobs (the macOS
  tests and bundle included) and published `mboxer 0.2.0` with both zips and `SHA256SUMS.txt`
- 2026-10-19 — research for 029: SignPath Foundation (free for open source) is the only option that fits;
  recorded as D-013, and 029 now waits for the application to be accepted. 048 (the usage guide as HTML in the
  zips) is ready as #201, held back until after this release
- 2026-10-19 — social preview uploaded in the repository settings; the pre-rewrite backup folder from 030 deleted
  (nine files: the history bundle and the old GitHub exports)

## Deviations from the plan

- The content pass grew from a step into a round of its own: four reviews, three parallel edit passes, the
  English translation of every task file, and the mirror issues recreated twice so no edit history kept the old
  texts.
- 044 to 047 were done before the tag instead of after the release, as parallel task sessions while the switch
  settled; 048 stayed after the release.
- `msrv` (from 046) joined the required checks, which the plan did not foresee.

## Verification

- 2026-10-19 — `gh repo view`: `PUBLIC`, the description and homepage as decided, 11 topics, Discussions off;
  `security_and_analysis`: secret scanning and push protection enabled; private vulnerability reporting
  `enabled: true`; CodeQL default setup `configured`, first run green, 0 open alerts
- 2026-10-19 — `gh api repos/speti4/mboxer/rulesets`: `main` (required checks `test (windows-latest)`, `lint`,
  `deny`, `workflow-lint`, `commit-rules`, `msrv`) and `release tags`, both active
- 2026-10-19 — community profile: health 100 %, code of conduct, contributing, licence, PR template, README
  found (the issue forms are YAML, which the API reports as no issue template)
- 2026-10-19 — release run of `v0.2.0`: test (windows-latest and macos-latest), lint, msrv, deny, workflow-lint,
  licences-macos, bundle-macos and release all green; the tag-on-main step passed
- 2026-10-19 — the downloaded release: `sha256sum -c SHA256SUMS.txt` OK for both zips; `gh attestation verify`
  exit 0 for both, built by `.github/workflows/ci.yml` at `refs/tags/v0.2.0`; the Windows zip holds
  `mboxer.exe`, `THIRD-PARTY-LICENSES.html` and `usage.md`, the macOS zip `mboxer.app`, the licence page and
  `usage.md`
- 2026-10-19 — README badges: `release: v0.2.0`, `license: MIT`, `ci - passing`

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (045–048 filed and
  done or in review; 029 updated with D-013; no other follow-up)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-19`
