---
size: S
origin: wip/057-contributing-checks-match-ci-and-private-era-leftovers.md
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
milestone: M6 Hardening
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

1. [ ] `docs/git-workflow.md` "Titles" and CONTRIBUTING "Branches, pull requests and commits": the accepted forms
   (`type: subject`, `type(scope): subject`), the seven types, the merge and revert exception, in one sentence each
2. [ ] `.github/workflows/ci.yml` header comment: the runner-time reason, same line count or `zizmor.yml` moved
3. [ ] `docs/development.md`: the `lint` entry lists the usage-guide check
4. [ ] `uvx zizmor .github` clean, `scripts/check-commit.sh --self-test` still passes, `python -I
   scripts/render-usage-guide.py --check` green, every relative link in the touched files resolves

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
