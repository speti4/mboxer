---
size: M
origin: wip/027-pre-publish-history-and-content-audit.md
created: 2026-10-18
started: 2026-10-18
plan: folytassuk-a-030-task-replicated-fairy.md
milestone: M5 Public release
---

# 030 — Scrub the repo and rewrite the history per D-09

## Context

What: the in-place edits D-09 point 1 lists (origin.md rewritten in English, the archive path out of CLAUDE.md,
27 `plan:` / `sources:` paths reduced to bare names, the colleagues framing reworded in README / roadmap /
decisions / done tasks, the 011 Log line, a CLAUDE.md line on commit trailers), then one `git filter-repo` run
(`--replace-text` for the four strings, `--mailmap` to the GitHub noreply address, `--replace-message` dropping
`Claude-Session:`), and the GitHub repo recreated from the rewritten history (D-10); the 027 Log has the
inventory with locations.
Why: D-09 — one third-party name sits in the history since the first commit; the rewrite is cheap while the repo
is private and unforked, impossible to do cleanly later. A force push would leave the old commits reachable by
SHA on GitHub, hence D-10. 028 waits for this task.
Done when: `git log -p --all` on a fresh clone finds none of the four strings, every commit carries the noreply
address and no `Claude-Session:` line, `gh release view v0.1.0` shows the asset and the restored notes, issue #N
is task N for 001–030, M1–M3 are closed, `tasks_check`, CI and the sync are green.
User decisions 2026-10-18: D-09 (scrub in place, rewrite once, `Claude-Session:` out, `Co-Authored-By` stays,
noreply address past and future, the 011 status line reworded on the safe side); in the planning round of this
task: back up, delete and recreate the GitHub repo instead of force-pushing (D-10).

## Steps

Phase A — in-place scrub, one commit

1. [x] `docs/origin.md` rewritten in English as the origin story, without paths, names or a user count
2. [x] `CLAUDE.md`: the archive path replaced by a generic description; one Conventions line on commit trailers
   and the noreply author address
3. [x] the 27 `plan:` / `sources:` fields reduced to the bare file name (`git grep -n -E '^(plan|sources):.*C:'` empty)
4. [x] colleagues framing reworded: README, roadmap (2 lines), D-01, D-05, D-07 (2), done 010, 012 (H1, Context,
   2 Hungarian lines), 014, 015, 021
5. [x] `done/011` status line and the `done/027` inventory row quoting it reworded
6. [x] D-10 written (supersedes D-09, D-09 `superseded`)
7. [x] `tasks_check` green; scrub committed, not pushed

Phase B — rewrite in a fresh clone

8. [ ] backup outside the repo: a bundle of every ref + the GitHub side exported (release notes, issues,
   milestones, labels, repo settings)
9. [ ] `git filter-repo --replace-text --replace-message --mailmap` in a `--no-local` clone; inputs in the
   scratchpad, never committed
10. [ ] verify the clone: the four strings, the personal mailbox and `Claude-Session:` → 0 in `git log -p --all`,
    the messages and the tag; only the noreply address; `HEAD^{tree}` equal to the scrubbed working copy
11. [ ] commit SHAs cited in task files remapped through `.git/filter-repo/commit-map`, one commit

Phase C — recreate the remote

12. [ ] `delete_repo` scope (user), last confirmation, `gh repo delete`, `gh repo create` (private, same description)
13. [ ] push `main` → CI and sync green, 40 issues; M1–M3 closed
14. [ ] push `v0.1.0` → release job green; title and notes restored

Phase D — working copy and close

15. [ ] working copy: fetch, hard reset, reflog expire, gc; the step-10 greps → 0 here too
16. [ ] Verification on a fresh clone from GitHub; close (follow-ups, memory note), commit, push

## Log

- 2026-10-18 — promoted from backlog/ to wip/
- 2026-10-18 — plan approved (`folytassuk-a-030-task-replicated-fairy.md`); the user replaced the force push by a
  recreated GitHub repo (D-10). Planning facts: `git filter-repo a40bce548d2c` installed; 34 commits, one branch,
  annotated `v0.1.0`, 0 forks, 0 PRs, no secrets; 39 issues with 0 comments; 15 Actions runs. The 011 status
  line's file name is the synthetic fixture subject (asserted in `crates/mboxer-core/tests/fixtures.rs`), so the
  rewrite replaces the exact status string only, never the subject.
- 2026-10-18 — Phase A: `origin.md` rewritten; `strip_plan_paths.py` (scratchpad) reduced 27 fields in 27 files;
  the framing sweep found 6 more lines than the 027 inventory (D-01, D-05, D-07 Context, done 014, 021, roadmap
  M3), all reworded; D-06's "the others" means other files and stays.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
