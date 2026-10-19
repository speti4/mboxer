---
size: S
origin: wip/057-contributing-checks-match-ci-and-private-era-leftovers.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 062 — The commit-rule wording names the scope form, the CI header gives the runner-time reason, the lint entry is complete

## Context

What: three leftovers of tasks 056 and 057. `docs/git-workflow.md` "Titles" and CONTRIBUTING's commit-rule sentence
say the hook accepts `type(scope): subject` as well as `type: subject`, and that a `Merge …` or `Revert "…"` subject
passes as git writes it. The header comment of `.github/workflows/ci.yml` still explains the tag-only macOS tests by
billing from the private period; give the runner-time reason `docs/development.md` gives (if the line count above
line 67 changes, move the `.github/zizmor.yml` ignores in the same commit). The `docs/development.md` sentence
listing the `ci.yml` jobs leaves `render-usage-guide.py --check` out of the `lint` entry.
Why: each is a sentence that contradicts what the machinery does, found right after the docs were aligned.
Done when: the three sentences match the hook, the workflow and the job, and `uvx zizmor .github` is clean.
Decided 2026-10-19: a docs and comment change only; the hook's rule (`scripts/check-commit.sh`, task 056) is the truth
the sentences follow.

## Steps

1. [x] `docs/git-workflow.md` "Titles" and CONTRIBUTING "Branches, pull requests and commits": the accepted forms
   (`type: subject`, `type(scope): subject`), the seven types, the merge and revert exception, in one sentence each
2. [x] `.github/workflows/ci.yml` header comment: the runner-time reason, same line count or `zizmor.yml` moved
3. [x] `docs/development.md`: the `lint` entry lists the usage-guide check
4. [x] `uvx zizmor .github` clean, `scripts/check-commit.sh --self-test` still passes, `python -I
   scripts/render-usage-guide.py --check` green, every relative link in the touched files resolves

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — the rule read from `scripts/check-commit.sh`: the subject matches
  `^(feat|fix|docs|ci|chore|tasks|deps)(\([^()[:space:]]+\))?: [^[:space:]]`, case-sensitive; a subject starting
  with `Merge ` or `Revert "` passes unchecked. `docs/git-workflow.md` "Titles" now says: a lower-case type, as
  `type: subject` or `type(scope): subject`, one of the seven types, and a `Merge …` or `Revert "…"` subject passes
  as git writes it. CONTRIBUTING "Branches, pull requests and commits" says the same in its own sentence; the
  Conventions bullet adds the scope form (`fix(core):`).
- 2026-10-19 — `ci.yml` header: the parenthesis "set while the repo was private and macOS minutes billed ten times
  the Linux rate" is gone; the reason is now the one `docs/development.md` gives: macOS runners are slower and
  scarcer, and the darwin clippy of the `lint` job already compiles the macOS code of every change. The header
  keeps its 13 lines, so `ci.yml:67` and `ci.yml:91` in `.github/zizmor.yml` still point at the two
  `Swatinem/rust-cache` lines and the ignores did not move.
- 2026-10-19 — `docs/development.md`: the `lint` entry of the job list names `scripts/render-usage-guide.py --check`
  next to the changelog check. The `lint` job's own comment in `ci.yml` (line 72) gets the same, in the same line.

## Deviations from the plan

- The `lint` job's comment in `ci.yml` was not named in the plan: it listed the changelog check but not the
  usage-guide check, the same gap as the `docs/development.md` entry. Fixed in place, line count unchanged.
- `changelog: housekeeping` added: the task changes the contributor docs and a workflow comment, not the app.

## Verification

- 2026-10-19 — in the worktree after the edits: `uvx zizmor .github` reports "No findings to report. Good job! (2
  ignored, 10 suppressed)"; lines 67 and 91 of `ci.yml` read `- uses: Swatinem/rust-cache@6323deb… # v2.9.2`.
- 2026-10-19 — `sh scripts/check-commit.sh --self-test`: "self-test: 35 cases passed"; the hook accepted the
  `docs:`, `ci:` and `tasks:` commits of this branch.
- 2026-10-19 — `python -I scripts/render-usage-guide.py --check`: "usage.md renders (12 KB); self-test passed";
  `python -I scripts/changelog.py --check` after the changelog run of the close: up to date.
- 2026-10-19 — a small script resolved every relative Markdown link and anchor in CONTRIBUTING.md,
  `docs/git-workflow.md` and `docs/development.md`: 13 of 13 resolve (`SECURITY.md`, `LICENSE`,
  `docs/development.md#the-macos-bundle`, `#ci-and-releases`, `#the-changelog`, `docs/git-workflow.md`,
  `maintainer-sessions.md`, `../CONTRIBUTING.md`).
- CI on the pull request: see the pull request checks.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none found
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
  — without write in the task branch; the index is regenerated when the pull request is prepared
- [x] moved to `done/` with `closed: 2026-10-19`
