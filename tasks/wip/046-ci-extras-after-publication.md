---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
plan: none
milestone: M5 Public release
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

1. [ ] `msrv` job: the toolchain of `rust-version` (1.90), `cargo check --locked --all-targets`; not added to the
   required checks here (the ruleset is changed by hand after the merge, if at all)
2. [ ] `changelog.py --check` in the lint job
3. [ ] fork guard on the issue sync job
4. [ ] zizmor cache-poisoning ignore scoped to the two rust-cache lines, if zizmor supports a line scope; otherwise
   dropped with the reason
5. [ ] `.gitattributes` default text rule, with a check that `git add --renormalize .` changes nothing
6. [ ] `release.ps1` falls back to `py -3`; `peak-memory.ps1` quotes like `benchmark.ps1`
7. [ ] `uvx zizmor .github` clean; `docs/development.md` describes the new jobs

## Log

- 2026-10-19 — promoted

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
