---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: a-task-021-hez-kapcsol-d-an-mellow-hearth.md
changelog: housekeeping
---

# 027 — Pre-publish audit of the history and the content

## Context

What: `gitleaks` over the full git history; a PII scan over `docs/`, `tasks/` and the commit messages; an
inventory of what is private but not secret: local absolute paths (the `plan:` fields, the archive path in
CLAUDE.md), the benchmark and origin texts, the Hungarian Log sections, the attribution trailers on every commit,
and the "built for the author and a few colleagues" framing. The outcome is a decision record (D-009): what gets
published (tasks/ as is, scrubbed, or left out) and whether the history is kept as is or rewritten before the
visibility switch.
Why: flipping a repo to public publishes its whole history at once and cannot be taken back; the project's own
rule is that no real message content ever enters the repo, and that has to be proven, not assumed. The first
M5 task, because D-009 shapes 023, 025 and 026.
Done when: the audit report is in this task's Log and D-009 is decided.
Decided 2026-10-18: 027 goes first; this task audits and decides only, the scrub is a follow-up task;
the small-audience ("2–5 users") framing is removed everywhere.

## Steps

1. [x] promote: task body, `plan:`, `tasks_check`
2. [x] secret scan: `gitleaks git` over every commit, `gitleaks dir` over the tree, redacted report in the scratchpad
3. [x] PII scan (anonilm): README, CLAUDE.md, docs/, tasks/, scripts/, the 33 commit messages
4. [x] inventory of private-but-not-secret content with `git grep` evidence; the 011 `.eml` name checked
5. [x] audit report in the Log: class · count · locations · cheapest fix · blocks publication?
6. [x] D-009 proposed (`/tasks decide`) and decided
7. [x] close: follow-up 030 (the scrub) filed, Verification, `tasks_check`, commit, push

## Log

- 2026-10-18 — plan approved; 027 moved from backlog/ to wip/
- 2026-10-18 — **secret scan**: gitleaks 8.30.1 (winget). `gitleaks git --redact` over the whole history: 33 commits,
  891 KB scanned, **no leaks found**, exit 0. `gitleaks dir --redact` over the working tree (6.15 GB, includes the
  ignored `target/`): 4 findings, all rule `generic-api-key` in `target/{debug,release}/deps/libmuda-*.rmeta`
  (compiled crate metadata, entropy 3.52, not tracked, false positives), exit 1. Reports in the scratchpad
  (`gitleaks-history.json`, `gitleaks-worktree.json`), not committed.
- 2026-10-18 — **PII scan** (anonilm 0.3.0, `anonymize_file` on per-directory concatenations with `### <path>`
  headers): README + CLAUDE.md → 0 findings; `docs/` (4 files) → 0; `scripts/` (5 files) → 0; `tasks/` (39 files)
  → TAJ 1 (a space-grouped byte count in `done/002` Log, false positive), MONEY 2 (scaled sizes, false positive),
  7 checksum-failed candidates skipped; the 33 commit messages (`%s%n%b`) → EMAIL 33 = the one `noreply@anthropic.com`
  trailer address, nothing else. Limitation noted: the scanner does not recognise a company name inside a path or a
  user name in `C:\Users\<name>`, so those two classes come from the manual inventory below.
- 2026-10-18 — **inventory** (`git grep`, counts per class; values stay out of this file):
  | # | class | where (count) | cheapest fix | blocks publication? |
  |---|---|---|---|---|
  | 1 | company / former-employer name in a local folder path | `docs/origin.md:17` (1), in history since `4a27c89` | edit + history rewrite to remove it from the past | **yes** — the only finding that names a third party |
  | 2 | the author's archive path (drive, backup folder, Takeout path) | `CLAUDE.md:17`, `docs/origin.md:9` (2) | edit; rewrite if 1 is rewritten anyway | no (local path, no secret) |
  | 3 | `C:\Users\<name>\.claude\plans\…` in `plan:` / `sources:` | 28 fields in `tasks/done/*`, `tasks/decisions/D-001…08`, this file (`git grep -l -E '^(plan\|sources):.*C:'`) | replace by `plan: <basename>` or `none`; rewrite optional | no (username only) |
  | 4 | other drive-letter paths | `docs/usage.md`, 3 scripts' `.EXAMPLE` (`D:\mail\…`, `C:\tmp\dist`), task Logs | none — generic examples | no |
  | 5 | benchmark aggregates of the mailbox (86 085 messages, 15 941 with attachment, hit counts of 4 generic words) | `docs/benchmarks/2026-10-17-14gb-takeout.md`, `done/002…004` Logs, commit bodies `08b340e`, `ab968e7` | keep (numbers, no content) | no |
  | 6 | a "Saved …" status line naming an `.eml` file | `tasks/done/011:61` | reword if real; the surrounding call is `{offset: 0, len: 2000}` = the stub backend's synthetic message, so the name is most likely the stub subject — **to be confirmed** | no |
  | 7 | "for the author and a handful of colleagues / other users / 2–5 users" framing | `README.md` (2), `tasks/roadmap.md` (1), `docs/origin.md` (1), `D-007` (1), `done/012` (4), `done/015` (1) | edit to a general-tool wording (decided 2026-10-18) | no, but goes before the switch |
  | 8 | commit trailers `Co-Authored-By: Claude …` (33) + `Claude-Session: https://claude.ai/code/…` (33) | every commit | keep, or `git filter-repo --message-callback` | no (style; the session links open for the account only) |
  | 9 | author e-mail (a personal Gmail address) as author and committer | 33 commits (66 fields) | keep, or GitHub noreply for future commits; past ones only by rewrite | no (identity the GitHub profile shows anyway) |
  | 10 | Hungarian prose (`[őűŐŰ]`) | 20 files: `done/*` Logs, `docs/origin.md` | language policy; origin.md rewrite in English | no |
  | 11 | e-mail addresses in code, fixtures, tests | only `example.com` (+ one `Example.com`) | none | no |
  No real message content, no credential, no token anywhere in 108 tracked files or 33 commits.
- 2026-10-18 — **D-009 proposed and decided** the same day: history rewritten with `git filter-repo`
  before the switch (class 1, 2, 3 and the 011 line out of every past blob); `Claude-Session:` trailers removed
  from the past and no longer written, `Co-Authored-By` stays; every commit gets the author's GitHub noreply
  address (`--mailmap` in the rewrite, `git config user.email` in this repo from this commit on — the first try
  with a real mailbox was rejected by GitHub's e-mail privacy protection, GH007); the 011 status
  line (class 6) is reworded on the safe side — whether the name was the stub's could not be established.
- 2026-10-18 — follow-up filed: `backlog/030-scrub-and-rewrite-history-per-d-09.md` (the whole scrub + rewrite, one
  task); `028` now waits for 030 as well. The `plan:` fields of the files touched in this round already carry the
  bare file name, as D-009 asks; the other 27 fields are 030's job.

## Deviations from the plan

- The plan expected D-009 to stay `proposed` until a later confirmation; it was decided in the same round, so
  027 closes with D-009 `decided` and 030 `ready` instead of `waiting`.
- The plan's PII step named the `scan` tool; `anonymize_file` on scratchpad concatenations was used instead (same
  detector, takes a path, so the 150 KB of task files did not have to pass through the conversation).

## Verification

- 2026-10-18 — `gitleaks git --redact` (8.30.1): "33 commits scanned … no leaks found", exit 0; `gitleaks dir
  --redact`: 4 findings, all under the ignored `target/` (`libmuda-*.rmeta`, `generic-api-key`), none in a tracked
  file.
- 2026-10-18 — anonilm 0.3.0 over README + CLAUDE.md, `docs/`, `scripts/`, `tasks/` and the 33 commit messages:
  the only true positive is the one `noreply@anthropic.com` trailer address (33×); TAJ 1 and MONEY 2 in `tasks/`
  are a byte count and scaled sizes.
- 2026-10-18 — inventory evidence: `git grep -l -E '^(plan|sources):.*C:' -- tasks` → 28 fields before this round;
  `git grep -c -E '[A-Z]:\\' -- ':!crates'` → 36 files, of which 2 carry the archive path and 1 the employer name;
  `git log --format=%b | grep -c '^Claude-Session'` → 33; `git log --format='%ae%n%ce' | sort -u` → one personal
  address (66 fields); framing grep → 10 lines in 6 files; e-mail addresses outside `example.com` → none.
- 2026-10-18 — D-009 written and decided in the same round (`status: decided`); 030 filed; `tasks_check`
  0 errors, 0 warnings.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — 030
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
