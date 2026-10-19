---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M5 Public release
changelog: housekeeping
---

# 046 — CI extras once the minutes are free: MSRV, changelog check, fork guard

## Context

What: a `cargo +1.90 check --locked` job for the declared `rust-version`; `scripts/changelog.py --check` in the
lint job; `if: github.repository == 'speti4/mboxer'` on the issue sync; the zizmor cache-poisoning ignore scoped
to the two cache lines; `* text=auto eol=lf` in `.gitattributes`; `release.ps1` falls back to `py -3`;
`peak-memory.ps1` quotes its arguments like `benchmark.ps1`.
Why: the pre-publication review of the tooling found these; on a public repo the extra jobs cost nothing.
Done when: each item is in place or dropped with a reason in the Log, CI green.

## Steps

1. [x] `msrv` job: the toolchain of `rust-version` (1.90), `cargo check --locked --all-targets`; not added to the
   required checks here (the ruleset is changed by hand after the merge, if at all)
2. [x] `changelog.py --check` in the lint job
3. [x] fork guard on the issue sync job
4. [x] zizmor cache-poisoning ignore scoped to the two rust-cache lines, if zizmor supports a line scope; otherwise
   dropped with the reason
5. [x] `.gitattributes` default text rule, with a check that `git add --renormalize .` changes nothing
6. [x] `release.ps1` falls back to `py -3`; `peak-memory.ps1` quotes like `benchmark.ps1`
7. [x] `uvx zizmor .github` clean; `docs/development.md` describes the new jobs

## Log

- 2026-10-19 — promoted
- 2026-10-19 — the changelog check cannot run on a tag: with a local probe tag `v0.2.0` on HEAD (deleted right
  after, never pushed), `changelog.py --check` exits 1, because the tag moves Unreleased under the version and
  only a run after the tag writes that stamp. The step has `if: github.ref_type != 'tag'`, so the release run
  stays green; the first pull request after a release fails the check until it commits the stamp
- 2026-10-19 — `msrv` runs on ubuntu for `aarch64-apple-darwin` with `DOCS_RS=1`, as the lint job does: the
  Windows target pulls in tauri-build's resource compiler, which a Linux runner lacks, and the language features
  do not differ between the targets. The version is read from `Cargo.toml` (`rust-version`), not written twice.
  No Rust cache in the job, so the zizmor cache-poisoning ignore stays at two lines
- 2026-10-19 — zizmor 1.31.1 accepts `file:line` entries: `ci.yml:67` and `ci.yml:91`. Moving one entry by a
  line brings the finding back (1 high), so the scope is real; a comment says the entries move with the lines
- 2026-10-19 — `.gitattributes`: `* text=auto eol=lf` first; `git add --renormalize .` stages only
  `.gitattributes` itself; `git ls-files --eol` shows the icons as `-text` and only the two CRLF fixtures as
  `i/crlf` (their own `-text` rule)
- 2026-10-19 — ci.yml header and env comment no longer mention the former toolchain action; `release.ps1`'s
  `Find-Tool` comment explains the PATH case without session wording

## Deviations from the plan

- Step 1 checks `--target aarch64-apple-darwin` instead of the host target; the Windows-only code is checked by
  the 1.90 toolchain locally (below), not in CI

## Verification

- 2026-10-19 — local, toolchain 1.90.0 installed with rustup: `DOCS_RS=1 cargo +1.90 check --locked
  --all-targets --target aarch64-apple-darwin` finished; `cargo +1.90 check --locked --all-targets` (Windows host)
  finished
- 2026-10-19 — `uvx zizmor .github`: no findings (2 ignored, 10 suppressed); `release.ps1` and
  `peak-memory.ps1` parse with 0 errors; `changelog.py --check` up to date without a tag, exit 1 with the probe
  tag (the reason for the tag condition)
- CI on the pull request: see the pull request checks

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the one open
  question (msrv as a required check) is in the pull request's Follow-ups for the maintainer
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — without write in the task branch; the index is regenerated
  when the pull request is prepared
- [x] moved to `done/` with `closed: 2026-10-19`
