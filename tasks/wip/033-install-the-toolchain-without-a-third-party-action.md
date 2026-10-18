---
size: S
origin: wip/024-supply-chain-and-ci-hardening.md
created: 2026-10-18
started: 2026-10-18
plan: none
milestone: M5 Public release
---

# 033 — Install the Rust toolchain in CI without a third-party action

## Context

What: drop `dtolnay/rust-toolchain` from `ci.yml`; the runner's preinstalled rustup installs the toolchain and the
components that `rust-toolchain.toml` names (`rustup show` or the first `cargo` call), in the test and release jobs.
Why: the repo has no release tags, so the action is pinned to a `master` commit that dependabot cannot follow (its
run reports "Latest version is 1, no update needed") and the pin would go stale silently; zizmor's pedantic
persona flags it as `ref-version-mismatch`. One third-party action fewer in the release job.
Done when: CI is green without the action and the test log shows the toolchain from `rust-toolchain.toml`.
Run as the first task session under docs/git-workflow.md (task 034): its own worktree, a pull request, no new id.

## Steps

1. [ ] `ci.yml`: replace `dtolnay/rust-toolchain` in the test and release jobs with a rustup step that installs
   what `rust-toolchain.toml` names; update the comments that mention the action or the toolchain
2. [ ] `docs/development.md` and any other mention of the action, if there is one
3. [ ] `uvx zizmor .github` clean; push the branch, open the pull request, CI green; the test log shows the
   toolchain and components from `rust-toolchain.toml`

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/033-toolchain-without-action` on
  branch `task/033-toolchain-without-action`

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
