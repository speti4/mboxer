---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# The commit hook checks the type prefix, and the required checks hold after a release and a PR edit

What: `scripts/check-commit.sh` only rejects the session trailer, the session link and a wrong author address, while
`docs/git-workflow.md` and CONTRIBUTING say the hook checks the `feat:` / `fix:` type prefix; add the prefix check
(the types the docs list, `deps:` for dependabot) to the hook and the `commit-rules` job. `commit-rules.yml` runs on
`pull_request` without `types: [opened, synchronize, reopened, edited]`, so an edited title or body is never
re-checked. The `lint` job's `changelog.py --check` fails on every pull request between a tag push and the stamp
commit, dependabot's included; tolerate a tag without a stamp or move the check to the release workflow. The `msrv`
job's comment says it is not required, but the `main` ruleset requires it; fix the comment and `docs/development.md`.
Why: the docs describe an enforcement the machinery does not perform, so a red required check cannot be explained from them.
Done when: a commit without a type prefix fails the hook and the job, an edited pull request text re-runs
`commit-rules`, and a pull request opened right after a tag push is green.
