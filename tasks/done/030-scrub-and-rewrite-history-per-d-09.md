---
size: M
origin: done/027-pre-publish-history-and-content-audit.md
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
milestone: M5 Public release
changelog: housekeeping
---

# 030 — Scrub the repo and rewrite the history per D-009 / D-010

## Context

What: the in-place edits D-009 point 1 lists (origin.md rewritten in English, the archive path out of CLAUDE.md,
27 `plan:` / `sources:` paths reduced to bare names, the colleagues framing reworded in README / roadmap /
decisions / done tasks, the 011 Log line, a CLAUDE.md line on commit trailers), then one `git filter-repo` run
(`--replace-text` for the four strings, `--mailmap` to the GitHub noreply address, `--replace-message` dropping
`Claude-Session:`), and the GitHub repo recreated from the rewritten history (D-010); the 027 Log has the
inventory with locations.
Why: D-009 — one third-party name sits in the history since the first commit; the rewrite is cheap while the repo
is private and unforked, impossible to do cleanly later. A force push would leave the old commits reachable by
SHA on GitHub, hence D-010. 028 waits for this task.
Done when: `git log -p --all` on a fresh clone finds none of the four strings, every commit carries the noreply
address and no `Claude-Session:` line, `gh release view v0.1.0` shows the asset and the restored notes, issue #N
is task N for 001–030, M1–M3 are closed, `tasks_check`, CI and the sync are green.
Decided 2026-10-18: D-009 (scrub in place, rewrite once, `Claude-Session:` out, `Co-Authored-By` stays,
noreply address past and future, the 011 status line reworded on the safe side); in the planning round of this
task: back up, delete and recreate the GitHub repo instead of force-pushing (D-010).

## Steps

Phase A — in-place scrub, one commit

1. [x] `docs/origin.md` rewritten in English as the origin story, without paths, names or a user count
2. [x] `CLAUDE.md`: the archive path replaced by a generic description; one Conventions line on commit trailers
   and the noreply author address
3. [x] the 27 `plan:` / `sources:` fields reduced to the bare file name (`git grep -n -E '^(plan|sources):.*C:'` empty)
4. [x] colleagues framing reworded: README, roadmap (2 lines), D-001, D-005, D-007 (2), done 010, 012 (H1, Context,
   2 Hungarian lines), 014, 015, 021
5. [x] `done/011` status line and the `done/027` inventory row quoting it reworded
6. [x] D-010 written (supersedes D-009, D-009 `superseded`)
7. [x] `tasks_check` green; scrub committed, not pushed

Phase B — rewrite in a fresh clone

8. [x] backup outside the repo: a bundle of every ref + the GitHub side exported (release notes, issues,
   milestones, labels, repo settings)
9. [x] `git filter-repo --replace-text --replace-message --mailmap` in a `--no-local` clone; inputs in the
   scratchpad, never committed
10. [x] verify the clone: the four strings, the personal mailbox and `Claude-Session:` → 0 in `git log -p --all`,
    the messages and the tag; only the noreply address; `HEAD^{tree}` equal to the scrubbed working copy
11. [x] commit SHAs cited in task files remapped through `.git/filter-repo/commit-map`, one commit

Phase C — recreate the remote

12. [x] `delete_repo` scope (maintainer's token), last confirmation, `gh repo delete`, `gh repo create` (private, same description)
13. [x] push `main` → CI and sync green, 40 issues; M1–M3 closed
14. [x] push `v0.1.0` → release job green; title and notes restored

Phase D — working copy and close

15. [x] working copy: fetch, hard reset, reflog expire, gc; the step-10 greps → 0 here too
16. [x] Verification on a fresh clone from GitHub; close (follow-ups), commit, push

## Log

- 2026-10-18 — promoted from backlog/ to wip/
- 2026-10-18 — plan approved (`folytassuk-a-030-task-replicated-fairy.md`); the force push was replaced by a
  recreated GitHub repo (D-010). Planning facts: `git filter-repo a40bce548d2c` installed; 34 commits, one branch,
  annotated `v0.1.0`, 0 forks, 0 PRs, no secrets; 39 issues with 0 comments; 15 Actions runs. The 011 status
  line's file name is the synthetic fixture subject (asserted in `crates/mboxer-core/tests/fixtures.rs`), so the
  rewrite replaces the exact status string only, never the subject.
- 2026-10-18 — Phase A: `origin.md` rewritten; `strip_plan_paths.py` (scratchpad) reduced 27 fields in 27 files;
  the framing sweep found 6 more lines than the 027 inventory (D-001, D-005, D-007 Context, done 014, 021, roadmap
  M3), all reworded; D-006's "the others" means other files and stays.
- 2026-10-18 — Phase B: backup in a folder outside the repo (never pushed):
  `history.bundle` (370 KB, `main`, `origin/main`, `v0.1.0`), the release notes and JSON exports of the 39
  issues, milestones, labels, repo settings and 15 runs. `git filter-repo --replace-text` (10 rules: the two
  local paths, the plan-path prefix, the user folder, the 011 status line in its two quoted forms plus a
  fallback, three case-insensitive safety nets) `--replace-message` (the `Claude-Session:` line) `--mailmap`
  in a `--no-local` clone: 35 commits rewritten in 0.4 s, the annotated tag with them. Checks on the clone:
  employer name, folder name, archive folder, `Users\<name>`, `Saved D:\mail\`, the personal mailbox,
  `gmail.com` → 0 in `git log -p --all` + messages + tag; 0 `Claude-Session:` lines and 0 session URLs in
  messages (16 hits of the bare word are the rule's own text in D-009, D-010, CLAUDE.md, 027, 030); one identity,
  the noreply address, tagger included; `Co-Authored-By` 35/35; `HEAD^{tree}` `7fd061e` equal to the scrubbed
  working copy. `remap_shas.py` (scratchpad): 11 cited SHAs in 9 files remapped, none ambiguous.
- 2026-10-18 — Phase C: the `delete_repo` scope added and the delete confirmed by the maintainer; `gh repo delete` +
  `gh repo create --private --disable-wiki` with the saved description (new repo: size 0, 0 issues). Push of
  `main` (`7ae3275`): `ci` 39133653751 and `sync-issues` 39133653556 green; the sync created the labels, M1–M5 and
  40 issues, #1–#30 = tasks 001–030 (0 mismatches), #31–#40 = D-001–D-010; M1–M3 closed by `gh api -X PATCH`. Push
  of `v0.1.0` (tag object `d28c3de`): `ci` 39134794224 test + release green, `mboxer-0.1.0.zip` published;
  `gh release edit` restored the title `mboxer 0.1.0` and the saved notes (byte-equal after trim).
- 2026-10-18 — Phase D: working copy `git fetch --force --prune --tags` (main forced, tag updated), `reset --hard
  origin/main`, `reflog expire --expire=now --all`, `gc --prune=now`: 0 loose objects, `fsck --unreachable` 0, the
  repo's `user.email` (noreply) and `target/` kept. Greps as in Phase B → 0, except two lines of this Log that list
  the search patterns themselves (a bare `D:\mail\` prefix and a bare domain, no file name, no address).
- 2026-10-18 — follow-ups: deleting the backup folder after the switch → appended to 028 (its `waits-for` no longer
  names 030); nothing else open. The `#N` issue numbers in the `done/021` and `done/022` Logs describe the old repo
  and stay as history.

## Deviations from the plan

- The remote was recreated instead of force-pushed: decided in the planning round (D-010), so the plan
  already carried it; against the backlog item and D-009 this is the deviation, recorded in D-010.
- The working copy was reset and pruned in place instead of re-cloned: same object set as a clone (0 unreachable),
  keeps the 6 GB `target/` cache and the repo-local noreply `user.email`.
- Issue numbers changed: D-001–D-008 moved from #23–#30 to #31–#38, tasks 023–029 from #31–#37 to #23–#29; nothing
  outside the repo referenced them.
- The framing sweep reached 6 lines the 027 inventory did not list (see Log); the plan's step 4 list was extended.

## Verification

- 2026-10-18 — fresh `git clone https://github.com/speti4/mboxer.git`: 36 commits; employer name, its folder name,
  the archive folder, `Users\<name>`, the full 011 file name, any `@gmail.com` address, `Claude-Session:` lines and
  `claude.ai/code/session` URLs → 0 each in `git log -p --all` + messages + the tag; one identity (the noreply
  address) for author and committer, tagger likewise; `Co-Authored-By` 36/36.
- 2026-10-18 — `gh release view v0.1.0`: not a draft, title `mboxer 0.1.0`, notes equal to the saved ones, asset
  `mboxer-0.1.0.zip` 5 341 404 byte; downloaded and unpacked: `mboxer.exe` 15 118 336 byte (ProductVersion 0.1.0) +
  `usage.md` 6 299 byte.
- 2026-10-18 — `python -I scripts/sync-issues.py --check`: created 0, updated 0, unchanged 40 (30 tasks, 10
  decisions), exit 0; milestones M1–M3 closed, M4–M5 open; `gh run list`: 3 runs, all of the new history, all
  success; `tasks_check` 0 errors, 0 warnings.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — the backup cleanup
  went into 028
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
