---
status: ready
size: S
origin: wip/057-contributing-checks-match-ci-and-private-era-leftovers.md
created: 2026-10-19
milestone: M6 Hardening
---

# The commit-rule wording names the scope form, the CI header gives the runner-time reason, the lint entry is complete

What: three leftovers of tasks 056 and 057. `docs/git-workflow.md` "Titles" and CONTRIBUTING's commit-rule sentence
say the hook accepts `type(scope): subject` as well as `type: subject`, and that a `Merge …` or `Revert "…"` subject
passes as git writes it. The header comment of `.github/workflows/ci.yml` still explains the tag-only macOS tests by
billing from the private period; give the runner-time reason `docs/development.md` gives (if the line count above
line 67 changes, move the `.github/zizmor.yml` ignores in the same commit). The `docs/development.md` sentence
listing the `ci.yml` jobs leaves `render-usage-guide.py --check` out of the `lint` entry.
Why: each is a sentence that contradicts what the machinery does, found right after the docs were aligned.
Done when: the three sentences match the hook, the workflow and the job, and `uvx zizmor .github` is clean.
