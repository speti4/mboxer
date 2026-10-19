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

1. [x] CONTRIBUTING.md "Checks before a commit" and `.github/pull_request_template.md`: the list matches the jobs
   of `ci.yml` (read the workflow, do not guess), with the extra commands a `Cargo.toml` / `.github/` change needs,
   and the sentence on when CI runs is true
2. [x] `docs/development.md`: the changelog paragraph matches `changelog.py --check`, the job count, the required
   checks (the six the ruleset names), the macOS-tests reason without billing (keep the tag-only rule, give the
   real reason: runner time), and `ci.yml`'s header comment the same (the required checks and `ci.yml` stay with
   task 056, see Deviations)
3. [x] the "one exe" wording in CONTRIBUTING, CLAUDE.md and the template says "one exe or app"; `docs/origin.md`
   14 GB; `CODE_OF_CONDUCT.md` names the channel that exists (issues, the security policy)
4. [x] a grep of the repo for `private`, `before publication`, `billing`, `minutes`, `0.1.0` and every hit judged
   (evidence in the Log); `python -I scripts/render-usage-guide.py --check` and `changelog.py --check` still green

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — the checks read from `ci.yml`: `test` (Windows: clippy and test, both `--locked`), `lint` (Linux:
  `cargo fmt --check`, clippy for `aarch64-apple-darwin` with `DOCS_RS=1`, `changelog.py --check` except on a tag,
  `render-usage-guide.py --check`), `msrv` (`cargo +<rust-version> check` for `aarch64-apple-darwin`, `--locked`,
  `DOCS_RS=1`), `deny` (`cargo deny check` plus the `about.toml` / `deny.toml` licence comparison),
  `workflow-lint` (zizmor); `commit-rules.yml` runs `scripts/check-commit.sh`. Both workflows trigger on pushes to
  `main` and on pull requests; `ci.yml` also on `v*` tags and manual runs, where the `test` matrix adds macOS. The
  darwin check in CI is a clippy, not a `cargo check`, and the documented list follows CI.
- 2026-10-19 — CONTRIBUTING "Checks before a commit": every change runs fmt, clippy and test with `--locked`, the
  macOS-target clippy, the 1.90 check (the `rust-version`) and the changelog check, with `DOCS_RS` set for the two
  darwin commands and removed after them; a change to `Cargo.toml`, `Cargo.lock`, `.github/` or the docs adds
  `cargo deny check`, `uvx zizmor .github` and the usage-guide check. The CI sentence says pushes to `main` and pull
  requests, the macOS tests on a tag or a manual run. The pull request template's checklist names the same groups.
  The `rust-version` comment in `Cargo.toml` points at the command, so a bump updates both.
- 2026-10-19 — `docs/development.md`: the changelog paragraph says the Unreleased list matches `tasks/done/` exactly,
  the closing change runs `python -I scripts/changelog.py`, a rebase reruns it; the tag paragraph counts five jobs;
  the tag-only macOS tests are explained by runner time (slower, scarcer runners, twice per change, the darwin
  clippy covers the macOS code in between); the layout line lists the MSRV, changelog and usage-guide checks.
- 2026-10-19 — "one exe": CONTRIBUTING's intro and "Dependencies: one exe" body say one portable exe (Windows) or
  app (macOS); the section title stays, because CLAUDE.md, the pull request template and task 035 cite it by name.
  CLAUDE.md and the template say exe or app. `docs/origin.md`: 14 GB. `CODE_OF_CONDUCT.md`: issues, pull requests
  and reports under the security policy (the repo has discussions turned off). `deny.toml`'s header says CI runs on
  pushes to `main`, pull requests and release tags.
- 2026-10-19 — the grep (`git grep -i -E "private|before publication|billing|billed|minutes|0\.1\.0|preview"` over
  everything except `tasks/`, `crates/` and `Cargo.lock`; a second grep over `crates/` for the period phrases found
  nothing), each hit judged:
  - fixed: `docs/development.md` "set while the repo was private"; CONTRIBUTING and `deny.toml` "on every push".
  - left for task 056: `.github/workflows/ci.yml` header "set while the repo was private and macOS minutes billed
    ten times the Linux rate" (the workflow is 056's in this round; a follow-up in the pull request).
  - kept, mail privacy or a private channel, not the private period: the issue templates, `config.yml`, CONTRIBUTING
    (mail stays private, a private report), `CODE_OF_CONDUCT.md` (private channel, private security advisory),
    `SECURITY.md`, `docs/git-workflow.md` (a private Claude Code session).
  - kept, not the period: every `timeout-minutes`; `about.toml` `private = { ignore = true }` (cargo-about skips
    unpublished crates); the benchmark's "minutes later" and its "mboxer 0.1.0" stamp (a dated record); the usage
    guide's and README's "two minutes" index build; `scripts/make-demo-mbox.py` `billing@` (a made-up address).
  - kept, history: `CHANGELOG.md` (0.1.0 section, the task titles of 021, 042, 046), `scripts/changelog.py`'s 0.1.0
    example.
  - kept, still true: "preview" for the macOS build in the README, `docs/usage.md` and `CHANGELOG.md` (not tested
    on a Mac yet), and the print preview in `docs/development.md`, `docs/reference.md` and `docs/usage.md`.
- 2026-10-19 — links: a small script resolved every relative Markdown link and anchor in CONTRIBUTING.md,
  CLAUDE.md, CODE_OF_CONDUCT.md, `docs/development.md`, `docs/origin.md` and the pull request template: 13 links,
  all resolve (`SECURITY.md`, `docs/development.md#the-macos-bundle`, `#ci-and-releases`, `#the-changelog`,
  `docs/git-workflow.md`, `../CONTRIBUTING.md`, `LICENSE`).

## Deviations from the plan

- The required checks in `docs/development.md` and the header comment of `ci.yml` are left alone: task 056 changes
  the workflows and the sentences on the required checks and the msrv job in the same round. The billing sentence
  of the `ci.yml` header goes to the pull request as a follow-up.
- `changelog: housekeeping` added: the task changes the contributor docs, not the app.
- The darwin check in the list is clippy, not `cargo check`, because the `lint` job runs clippy for that target.
- `deny.toml` and the `rust-version` comment in `Cargo.toml` were not named in the plan: the first said "every
  push", the second ties the hard-coded 1.90 in CONTRIBUTING to the place it is written.

## Verification

- 2026-10-19 — in the worktree: `python -I scripts/render-usage-guide.py --check` ("usage.md renders (12 KB);
  self-test passed") and `python -I scripts/changelog.py --check` ("CHANGELOG.md is up to date") exit 0 after the
  docs commit; the link script reports 13 of 13 relative links and anchors resolved; a GitHub API read shows
  discussions turned off for `speti4/mboxer`. Every command of the new list is copied from `ci.yml` (no cargo run
  in this docs-only task; CI runs the same commands on the pull request).

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (the follow-ups are in
  the pull request description; the main session files them)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
