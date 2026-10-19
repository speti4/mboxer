---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
changelog: housekeeping
---

# 043 — Pad the decision ids to three digits before the repo goes public

## Context

What: `/tasks renumber` renames the ten two-digit decision files to three digits and rewrites the references inside
`tasks/`; the ~30 files outside it are fixed by hand (crate comments, docs, CLAUDE.md, README, CONTRIBUTING,
`scripts/check-commit.sh`, workflows). `scripts/sync-issues.py` keys the decision issues by their id
(`<!-- sync-issues: decision D-NN -->`), so it must match the old two-digit key to the new one instead of
opening ten new issues; `CHANGELOG.md` stays in line with `scripts/changelog.py --check`.
Why: mixed widths sort and read wrong; the public repo starts consistent (decided 2026-10-19).
Done when: `tasks_check` has no narrow-id warning, a grep for two-digit decision ids finds only history that
must stay, and a sync run edits the ten decision issues in place.
Split (2026-10-19): the renumber tool rewrites `tasks/`; everything outside `tasks/` is fixed by hand.

## Steps

1. [x] `tasks_renumber` with `apply` (main session): ten decision files renamed, the references in `tasks/`
   rewritten, the index regenerated
2. [x] the references outside `tasks/` (the tool's `names` list and every bare two-digit decision id): code
   comments, docs, CLAUDE.md, README, CONTRIBUTING, scripts, workflows, the pull request template
3. [x] `scripts/sync-issues.py`: an existing issue whose marker carries the two-digit key is matched to the
   three-digit one and edited in place (title, body, marker); a dry run shows ten edits and no creates
4. [x] `CHANGELOG.md` agrees with `python -I scripts/changelog.py --check`
5. [x] the CONTRIBUTING.md checks; `scripts/check-commit.sh` still rejects a trailer and a session link
6. [ ] after the merge: the sync run on `main` edits the ten decision issues, opens none

## Log

- 2026-10-19 — promoted on `task/043-pad-decision-ids-to-three-digits`.
- 2026-10-19 — main session: `tasks_renumber` apply (10f2e49), ten decision files `D-0NN-…`, the references in
  `tasks/` rewritten, index regenerated.
- 2026-10-19 — task session, outside `tasks/`: the word-bounded forms `D-01` … `D-10` → `D-001` … `D-010` with a
  scratch script over the files git tracks (except `tasks/` and `CHANGELOG.md`): 53 replacements in 30 files —
  Rust/JS comments (15 files, comments only), `Cargo.toml` (5), docs (`reference`, `development`, `git-workflow`,
  `research`: 10), README, CONTRIBUTING, CLAUDE.md, `scripts/` (`check-commit.sh`, `bundle-macos.sh`,
  `sync-issues.py`), workflows (2), PR template; also the research file's old `tasks/decisions/D-01-….md` link.
  `scripts/__pycache__/` falls under the `__pycache__/` line of `.gitignore`, not a source.
- 2026-10-19 — `scripts/sync-issues.py`: `match_issues` first looks for the marker's exact key, then for the issue
  whose marker carries the same number at another width (`same_id`: the number without leading zeros, general, not
  a list; it holds for tasks too). It edits the existing issue in place: the body is replaced together with the new
  marker, the title too; the log line says `was D-0N`. No new dry-run mode was needed; the existing `--dry-run`
  ran live (GET only).
- 2026-10-19 — `CHANGELOG.md`: `--check` was already stale before the task (the tasks closed between 018 and 042
  were missing; the changelog is updated only before a release, CI does not check it), so the run refreshed the
  whole Unreleased list, and 030's line became `per D-009`. The 0.1.0 section cites no decision and is unchanged;
  no released history changed. The file names of 030 and 039 (`…-per-d-09.md`, `…-cite-d-10.md`) are task file
  names the renumber does not rename, so the changelog links keep the old slugs.
- 2026-10-19 — left in `tasks/` (not the task session's job, listed under the PR's Follow-ups): `tasks/roadmap.md:8`
  `(D-08)`; quotes and history: a verbatim quote in line 34 of `D-007`, and Log lines of 036 and 039 with
  suffixed forms of the old ids that the tool did not rewrite.

## Deviations from the plan

- none

## Verification

- 2026-10-19 — `grep -rnE "D-(0[0-9]|10)\b"` without `tasks/`, `target/`, `.git/`, `.claude/`: empty
  (case-insensitive, only the two `CHANGELOG.md` links to the 030/039 file names).
- 2026-10-19 — `python -I scripts/sync-issues.py --dry-run` on the real repo (read-only): the ten decision issues
  (#31–#40) `update (body, title, was D-0N)`, D-011/D-012 body only; `created 0, updated 41, unchanged 14`, no
  orphan. The other updates are task issue bodies (the renumber rewrote the task files' references).
- 2026-10-19 — `python -I scripts/changelog.py --check`: up to date.
- 2026-10-19 — `scripts/check-commit.sh` with scratch messages: clean → 0; `Claude-Session:` trailer → 1, "(D-010
  point 3)"; session link → 1, "(D-010 point 3)"; non-noreply address → 1 (D-010); through the
  `.githooks/commit-msg` hook the trailer → 1.
- 2026-10-19 — `uvx zizmor .github`: no findings; `cargo deny check`: ok;
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`: all green
  (the code change is comments only).

## Closing checklist

- [x] every follow-up is in the pull request's `## Follow-ups` (the main session files them; task session rule)
- [x] Verification written, dated
- [x] `tasks_check` green apart from the stale index; the README index is regenerated by the main session
- [x] moved to `done/` with `closed: 2026-10-19`
