---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
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

1. [ ] `scripts/check-commit.sh`: the subject must start with one of the listed types and a colon (`type: ` or
   `type(scope): `), case-sensitive, with the error message citing `docs/git-workflow.md`; a merge or revert
   subject passes; the script's self-test (or a small test list run in CI) covers both sides
2. [ ] `.github/workflows/commit-rules.yml`: `pull_request` with `types: [opened, synchronize, reopened, edited]`,
   the pull request title checked with the same rule as a subject
3. [ ] the changelog check in `ci.yml`: tolerate a tag that has no stamp commit yet (compare against the last
   stamped version, or skip with a notice when `HEAD` is a pull request onto a `main` whose latest tag is
   unstamped), so a dependabot pull request right after a release stays green; the chosen rule in the Log
4. [ ] the `msrv` job comment and the required-checks list in `docs/development.md` and `git-workflow.md` say
   which six checks the ruleset requires
5. [ ] `uvx zizmor .github` clean; `bash -n` on the hook; a local commit with a bad subject rejected, a good one
   accepted (evidence in the Log)

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
