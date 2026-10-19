---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 056 — The commit hook checks the type prefix, and the required checks hold after a release and a PR edit

## Context

What: `scripts/check-commit.sh` only rejects the session trailer, the session link and a wrong author address, while
`docs/git-workflow.md` and CONTRIBUTING say the hook checks the `feat:` / `fix:` type prefix. `commit-rules.yml` runs
on `pull_request` without `types: [opened, synchronize, reopened, edited]`, so an edited title or body is never
re-checked. The `lint` job's `changelog.py --check` fails on every pull request between a tag push and the stamp
commit, dependabot's included. The `msrv` job's comment says it is not required, but the `main` ruleset requires it.
Why: the docs describe an enforcement the machinery does not perform, so a red required check cannot be explained
from them.
Done when: a commit without a type prefix fails the hook and the job, an edited pull request text re-runs
`commit-rules`, and a pull request opened right after a tag push is green.
Decided 2026-10-19: filed by the post-release review; the allowed types are the ones `docs/git-workflow.md` lists
(`feat fix docs ci chore tasks deps`), and a merge commit or a revert subject is accepted as git writes it.

## Steps

1. [x] `scripts/check-commit.sh`: the subject must start with one of the listed types and a colon (`type: ` or
   `type(scope): `), case-sensitive, with the error message citing `docs/git-workflow.md`; a merge or revert
   subject passes; the script's self-test (or a small test list run in CI) covers both sides
2. [x] `.github/workflows/commit-rules.yml`: `pull_request` with `types: [opened, synchronize, reopened, edited]`,
   the pull request title checked with the same rule as a subject
3. [x] the changelog check in `ci.yml`: tolerate a tag that has no stamp commit yet (compare against the last
   stamped version, or skip with a notice when `HEAD` is a pull request onto a `main` whose latest tag is
   unstamped), so a dependabot pull request right after a release stays green; the chosen rule in the Log
4. [x] the `msrv` job comment and the required-checks list in `docs/development.md` and `git-workflow.md` say
   which six checks the ruleset requires
5. [x] `uvx zizmor .github` clean; `bash -n` on the hook; a local commit with a bad subject rejected, a good one
   accepted (evidence in the Log)

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — `scripts/check-commit.sh`: the subject is the first line that is not blank once the comment lines
  and the `git commit -v` diff are gone; it matches `^(feat|fix|docs|ci|chore|tasks|deps)(\([^()[:space:]]+\))?:
  [^[:space:]]`, case-sensitive, so `Feat:`, `feature:`, `feat:text`, `feat : text`, `feat():` and an empty
  description fail. `Merge …` and `Revert "…"` pass as git writes them; `fixup!` subjects fail (the branch is
  squash-merged, so nothing needs them). The error names the types and `docs/git-workflow.md`; the trailer, link
  and author checks and their D-010 wording are unchanged. `--self-test` feeds 35 messages (good and bad
  subjects, blank and comment lines before the subject, the trailer, the link, the author address) through the
  script itself; the `commit-rules` job runs it before the commit check
- 2026-10-19 — `commit-rules.yml`: `pull_request` with `types: [opened, synchronize, reopened, edited]`. The pull
  request text the job already checked starts with the title, so the subject rule covers the title with no
  second code path
- 2026-10-19 — the changelog window. Rule chosen: `changelog.py --check` drops the newest release tags that
  CHANGELOG.md has no `## <version>` heading for, prints a notice (`::notice::` on Actions) and checks their
  tasks under Unreleased, which is what the committed file still says. Only the newest run of unstamped tags is
  dropped, so an older version whose section went missing still fails; a pull request in the window that closes
  a task without its line still fails; a run without `--check` stamps the tag as before, so the release flow of
  task 047 (tag, then the stamp commit with the next change) is unchanged. Comparing against the last stamped
  version keeps the check meaningful in the window, where a skip would let a missing line through
- 2026-10-19 — `ci.yml`: the `msrv` comment says the ruleset requires it (the `main` ruleset lists
  `test (windows-latest)`, `lint`, `deny`, `workflow-lint`, `commit-rules`, `msrv`); the changelog step comment
  names the window rule. Both comments are below line 91, so the `ci.yml:67` / `ci.yml:91` ignores still point at
  the rust-cache lines. `docs/development.md` already named the six checks; it gains one sentence on the window
  in the release steps. `docs/git-workflow.md` names the six required checks and the re-run on an edit

## Deviations from the plan

- `docs/development.md` needed no change to its required-checks sentence (it already listed the six); the
  six are named in `docs/git-workflow.md` instead

## Verification

- 2026-10-19 — `sh scripts/check-commit.sh --self-test`: `self-test: 35 cases passed`; `bash -n` on
  `scripts/check-commit.sh` and `.githooks/commit-msg`: no errors
- 2026-10-19 — the hook in the worktree: `git commit --allow-empty -m "open a second mbox"` and `-m "Feat: open
  a second mbox"` rejected with the subject message (exit 1); `chore: throwaway commit to try the hook`
  accepted, then removed with `git reset --soft HEAD~1`
- 2026-10-19 — the window, in a throwaway clone with a local tag `v0.3.0` on HEAD: the old `--check` exits 1,
  the new one prints the notice and exits 0; with a task moved to `done/` and no changelog run it exits 1; after
  `changelog.py` stamps `## 0.3.0`, `--check` exits 0 with no notice
- 2026-10-19 — `uvx zizmor .github`: no findings (2 ignored, 10 suppressed); `python -I scripts/changelog.py
  --check`: up to date
- CI on the pull request: see the pull request checks

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the follow-ups
  are in the pull request's Follow-ups for the main session to file
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
  — without write in the task branch; the index is regenerated when the pull request is prepared
- [x] moved to `done/` with `closed: 2026-10-19`
