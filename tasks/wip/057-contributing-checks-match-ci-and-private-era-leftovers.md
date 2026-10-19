---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
---

# 057 — CONTRIBUTING's pre-commit list matches CI, and the private-era wording goes

## Context

What: "Checks before a commit" in CONTRIBUTING.md and the pull request template leave out what CI runs (`--locked`,
the darwin `cargo check`, the MSRV build, `scripts/changelog.py --check`, `scripts/render-usage-guide.py --check`, the
commit rules) and say CI runs "on every push" where it runs on pushes to `main` and on pull requests.
`docs/development.md` still says the Unreleased changelog list may lag behind `tasks/done/` (the check requires an
exact match), counts "four jobs" where there are five, and, with `ci.yml`, explains the tag-only macOS tests by
billing, which no longer applies to a public repo. The "one exe" wording ignores the macOS `.app`; `docs/origin.md`
says 13 GB where every other page says 14 GB; `CODE_OF_CONDUCT.md` names discussions, which the repo does not have.
Why: the docs are the contract with contributors; each mismatch costs one red check or one wrong assumption.
Done when: a contributor who runs the documented list gets the same result as CI, and no page refers to the private
period.
Decided 2026-10-19: filed by the post-release review; a docs-only change, the CI jobs themselves are task 056's.

## Steps

1. [ ] CONTRIBUTING.md "Checks before a commit" and `.github/pull_request_template.md`: the list matches the jobs
   of `ci.yml` (read the workflow, do not guess), with the extra commands a `Cargo.toml` / `.github/` change needs,
   and the sentence on when CI runs is true
2. [ ] `docs/development.md`: the changelog paragraph matches `changelog.py --check`, the job count, the required
   checks (the six the ruleset names), the macOS-tests reason without billing (keep the tag-only rule, give the
   real reason: runner time), and `ci.yml`'s header comment the same
3. [ ] the "one exe" wording in CONTRIBUTING, CLAUDE.md and the template says "one exe or app"; `docs/origin.md`
   14 GB; `CODE_OF_CONDUCT.md` names the channel that exists (issues, the security policy)
4. [ ] a grep of the repo for `private`, `before publication`, `billing`, `minutes`, `0.1.0` and every hit judged
   (evidence in the Log); `python -I scripts/render-usage-guide.py --check` and `changelog.py --check` still green

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
