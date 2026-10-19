---
size: S
origin: done/024-supply-chain-and-ci-hardening.md
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
milestone: M5 Public release
changelog: housekeeping
---

# 033 — Install the Rust toolchain in CI without a third-party action

## Context

What: drop `dtolnay/rust-toolchain` from `ci.yml`; the runner's preinstalled rustup installs the toolchain and the
components that `rust-toolchain.toml` names (`rustup show` or the first `cargo` call), in the test and release jobs.
Why: the repo has no release tags, so the action is pinned to a `master` commit that dependabot cannot follow (its
run reports "Latest version is 1, no update needed") and the pin would go stale silently; zizmor's pedantic
persona flags it as `ref-version-mismatch`. One third-party action fewer in the release job.
Done when: CI is green without the action and the test log shows the toolchain from `rust-toolchain.toml`.
The first task done under the parallel-work rules of docs/git-workflow.md (034), through a pull request.

## Steps

1. [x] `ci.yml`: replace `dtolnay/rust-toolchain` in the test and release jobs with a rustup step that installs
   what `rust-toolchain.toml` names; update the comments that mention the action or the toolchain
2. [x] `docs/development.md` and any other mention of the action, if there is one
3. [x] `uvx zizmor .github` clean; push the branch, open the pull request, CI green; the test log shows the
   toolchain and components from `rust-toolchain.toml`

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/033-toolchain-without-action` on
  branch `task/033-toolchain-without-action`
- 2026-10-18 — `ci.yml`: an "Install the Rust toolchain from rust-toolchain.toml" bash step in the test and the
  release job (`rustup toolchain install` without an argument, then by name, `rustup show`, in the test job also
  `rustup component list --installed`); workflow-level `env`: `CARGO_INCREMENTAL=0`, `CARGO_TERM_COLOR=always`;
  the header comment and `docs/development.md` say where the toolchain comes from. PR #50.

## Deviations from the plan

- Two rustup calls are needed, not one: the argumentless `rustup toolchain install` (rustup 1.29.1 locally too)
  only prints "using existing install" for an existing toolchain and does not update the channel. The runner
  image's stable can be older (1.99.0 in the first run, the latest was 1.99.1), and the dtolnay action always
  installed the latest, so a second `rustup toolchain install <name>` (the name from `rustup show
  active-toolchain`) brings it up to date. With a pinned version (`channel = "1.x.y"`) it is a no-op.
- The action also exported `CARGO_INCREMENTAL=0`, `CARGO_TERM_COLOR=always` and `CARGO_HOME` to the later steps.
  Without them the first run compiled incrementally and the rust-cache key (env hash) changed; the workflow now
  sets the first two. `CARGO_HOME` (the default path) was not restored; hard-coding a runner path is not worth it:
  the cache key changes once, the first `main` run after the merge compiles without a cache.

## Verification

- 2026-10-18 — `uvx zizmor .github`: "No findings" (1 ignored, 6 suppressed); `--persona pedantic`: the two
  `superfluous-actions` findings of the dtolnay steps are gone, only `anonymous-definition` infos remain.
- 2026-10-18 — the step's script locally in Git Bash (rustup 1.29.1): the argumentless install says
  "using existing install … overridden by '…\rust-toolchain.toml'", the named one "syncing channel updates …
  unchanged - rustc 1.99.1"; the installed components include `rustfmt` and `clippy`.
- 2026-10-18 — CI run 39148822817 (commit d5e2643), all four jobs green. The test job log (job 117738086624):
  no `dtolnay` line; the new step: "it's active because: overridden by 'D:\a\mboxer\mboxer\rust-toolchain.toml'",
  "stable-x86_64-pc-windows-msvc updated - rustc 1.99.1 (from rustc 1.99.0)", `rustup show` gives the active
  toolchain with the same reason, `component list --installed`: `clippy-…`, `rustfmt-…`; fmt, clippy, test green.
  The cache key differed from `main`'s (no `CARGO_HOME`, `CARGO_INCREMENTAL`, `CARGO_TERM_COLOR` in the env) →
  cache miss, ~11 minutes of clippy + test.
- 2026-10-18 — CI run 39150020746 (commit 7cb471c, the restored env): all four jobs green, the test job's env has
  `CARGO_INCREMENTAL: 0`, `CARGO_TERM_COLOR: always`; the cache key differs from `main`'s only because of
  `CARGO_HOME` (`bc559819` vs `e70cdc14`), and the run saved its own.
- Release job: runs only on a `v*` tag, did not run here. From the YAML: the same two `rustup toolchain install` +
  `rustup show` after the checkout, the workflow-level env applies there too; `scripts/release.ps1` builds with the
  toolchain `rust-toolchain.toml` makes active, the rustfmt/clippy components are installed there too (harmless).

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [ ] `tasks_check` green, README index regenerated — from the worktree the check fails only on the stale README
  index (the tool does not write outside the session's repo); the main session regenerates it at the merge
- [x] moved to `done/` with `closed: 2026-10-18`
