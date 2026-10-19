---
status: ready
size: S
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# CONTRIBUTING's pre-commit list matches CI, and the private-era wording goes

What: "Checks before a commit" in CONTRIBUTING.md and the pull request template leave out what CI runs (`--locked`,
the darwin `cargo check`, the MSRV build, `scripts/changelog.py --check`, `scripts/render-usage-guide.py --check`, the
commit rules) and say CI runs "on every push" where it runs on pushes to `main` and on pull requests.
`docs/development.md` still says the Unreleased changelog list may lag behind `tasks/done/` (the check requires an
exact match), counts "four jobs" where there are five, and, with `ci.yml`, explains the tag-only macOS tests by
billing, which no longer applies to a public repo. The "one exe" wording ignores the macOS `.app`; `docs/origin.md`
says 13 GB where every other page says 14 GB; `CODE_OF_CONDUCT.md` names discussions, which the repo does not have.
Why: the docs are the contract with contributors; each mismatch costs one red check or one wrong assumption.
Done when: a contributor who runs the documented list gets the same result as CI, and no page refers to the private period.
