---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: cuddly-painting-trinket.md
milestone: M5 Public release
---

# 034 — Git workflow policy for contributors and parallel work, with the commit rules enforced

## Context

What: `docs/git-workflow.md` (contributors: branch → PR → squash merge, commit format, dependabot PRs; maintainer and
agent sessions: one worktree per task, new ids only in the main session, the task index regenerated on main after a
merge, one manual round at a time, push only on the maintainer's go), imported into CLAUDE.md with `@` and linked from
CONTRIBUTING; a versioned `commit-msg` git hook (`.githooks/`, `core.hooksPath`) that rejects a `Claude-Session:`
trailer and a non-noreply author, plus the same check as a CI job for pushes and PRs that bypass the local hook.
Why: parallel sessions need the rules in context at start (decided 2026-10-18: a policy file, not a decision record);
an unchecked rule is easy to break, so a hook and a CI job enforce the trailer rule (D-009). Branch ruleset on main: 028.
Done when: a commit with the trailer fails locally and in CI, and a second session in a worktree follows the file.

## Steps

1. [x] `scripts/check-commit.sh MSG_FILE NAME EMAIL` (POSIX sh): rejects a `Claude-Session:` line and a
   `claude.ai/code/session_` link; the maintainer name requires the noreply address
2. [x] `.githooks/commit-msg` (executable, LF via `.gitattributes`) calls the script with `git var GIT_AUTHOR_IDENT`
3. [x] `git config core.hooksPath .githooks` in this clone
4. [x] `ci.yml`: `commit-rules` job — PR range + PR title/body, push range, tag push skipped; values only via `env:`
5. [x] `docs/git-workflow.md`: everyone (branch → PR → squash, trailers, hooks, dependabot), main session vs task
   session (worktrees under `.claude/worktrees/`, ids only on main, follow-ups in the PR body), shared resources
6. [x] `CLAUDE.md`: `@docs/git-workflow.md` import, the Commits line shrinks to a pointer
7. [x] `.gitignore`: `.claude/worktrees/`
8. [x] `CONTRIBUTING.md`, `.github/pull_request_template.md`, `docs/development.md` point to the policy
9. [x] Repo merge settings (`gh api`): squash only, squash message = PR title + description, delete head branches
10. [x] 034 itself through the flow: branch `task/034-git-workflow-policy`, PR #48, CI red with a trailer commit,
    green without, squash merge on the maintainer's go
11. [x] `cargo fmt --check`, clippy, test, `uvx zizmor .github`
12. [x] Done-when, second half: 033 runs in a worktree session and follows the file (PR, no new id, follow-ups in
    the PR body)

## Log

- 2026-10-18 — promoted from backlog; plan approved (decided: the noreply rule for the maintainer identity only;
  worktrees under `.claude/worktrees/`; task sessions open a PR, the main session squash-merges it)
- 2026-10-18 — branch `task/034-git-workflow-policy`: `scripts/check-commit.sh`, `.githooks/commit-msg`
  (100755, `i/lf w/lf`), `commit-rules` job, `docs/git-workflow.md`, CLAUDE.md import, CONTRIBUTING / PR template /
  development.md; `core.hooksPath` set. The hook in a scratch worktree: trailer, session link, non-noreply
  maintainer → exit 1; contributor (other name, own address) → exit 0; a link in the diff of `commit -v -e` does
  not trigger (scissors cut it off), a trailer in the edited message does; it also catches one in a CRLF PR body.
  An identical author/committer address is reported only once.
- 2026-10-18 — repo merge settings (`gh api -X PATCH`): `allow_merge_commit` / `allow_rebase_merge` false,
  `squash_merge_commit_title=PR_TITLE`, `squash_merge_commit_message=PR_BODY`, `delete_branch_on_merge` true.
  Hence the `Co-Authored-By` trailer has to go at the end of the PR body — written into git-workflow.md.
- 2026-10-18 — `cargo fmt --check`, `clippy -D warnings`, `cargo test` green; `uvx zizmor .github`: no findings.
- 2026-10-18 — PR #48 squash-merged (`dda4e86`), main CI green. Two surprises in the squash commit: GitHub appends
  the commits' `Co-authored-by` lines after `---------` by itself (so the trailer in the PR body appeared twice),
  and the author is `speti4 <noreply>`, not `Szőke Péter`. Fix on the `task/034-commit-rule-fixes` branch:
  git-workflow.md no longer asks for the trailer in the PR body; `check-commit.sh` also treats the name `speti4`
  as the maintainer (speti4 + other address → exit 1, speti4 + noreply → 0, other name → 0). PR #49 → `1f52077`.
- 2026-10-18 — 033 as the first task session: the main session promoted it (`109cf4a`, the task branch's first
  commit), worktree `.claude/worktrees/033-toolchain-without-action`, a background subagent in it following the
  policy: PR #50, no new id, follow-ups: none, not merged. Four gaps in the policy (from its report and the merge):
  `tasks_check` does not write from a worktree (index); promoting "on main" clashed with the "main only through a
  PR" rule (`/tasks add` for follow-ups too); updating the PR checklist after CI (`gh pr edit` fails with a
  Projects (classic) error); `gh pr merge --delete-branch` does not delete a local branch a worktree holds. The
  index was made before the merge from the main checkout, on a detached HEAD (`c2135cb`); #50 → `398e87b`. Fixed in
  git-workflow.md on this branch (`task/034-close-git-workflow`): promote on the task's own branch in the main
  checkout, the pre-merge preparation (follow-ups + index) on a detached HEAD, `git worktree remove` before the
  merge, the task session does not touch the index, the PR body via `gh api -X PATCH`, the reminder-trailer
  sentence.
- 2026-10-18 — follow-up sweep: nothing open. The Context sentences "index regenerated on main after a merge" and
  "push only on the maintainer's go" were superseded by the pre-merge preparation above and the "main moves only
  through PRs" rule.

## Deviations from the plan

- The PR description ends with `🤖 Generated with Claude Code` only, as planned; a detour through a
  `Co-Authored-By` trailer in the body was reverted after PR #48 showed that GitHub appends the commits'
  co-authors to the squash message itself.
- The maintainer check knows two names: `Szőke Péter` (git config) and `speti4` (the author of a web merge).
- The 034 promote commit lives only on the task branch; local `main` is set back to `origin/main` before the push,
  so the squash merge is the one commit `main` gets. The policy now says so: `main` moves only through merged
  pull requests; the promotion, the follow-ups and the index are commits on the task branch, made in the main
  checkout because the task tools write only there.
- The second session of the done-when was a background subagent in the 033 worktree, not a separately opened
  session (decided 2026-10-18).

## Verification

- 2026-10-18 — the hook locally (scratch worktree, `scratch/hook-test`, deleted afterwards): `Claude-Session:`
  trailer, session link, `-c user.email=x@example.com` with the maintainer's name → each exit 1 with its own
  message; other name + own address → exit 0; a clean commit passes (c04a34d, 3deb730 were made with it).
- 2026-10-18 — CI on PR #48: first run (39145607833) all green, `commit-rules` 4 s. A temporary commit with a
  trailer (`fdfb64a`, `--no-verify`) → `commit-rules` failure (39145901495): "no Claude-Session: trailer … in
  fdfb64a…", exit 1. Dropped (`reset --hard 3deb730`, `--force-with-lease`) → all green again (39145983395).
- 2026-10-18 — second session in the worktree (033): the subagent worked by `docs/git-workflow.md` — only its own
  task file + `ci.yml` + `docs/development.md`, no new id, commits passing the hook (no trailer, no session link),
  PR #50 per the template with `## Follow-ups: none` and only the `🤖 Generated with Claude Code` closing line, CI
  green (39151233114, then 39151560485 after the index commit), no merge. Main CI green after #48, #49 and #50 (the
  last one 39151806245, with a cold cache because of the cache-key change); `commit-rules` passed all three times
  on the `speti4 <noreply>` squash commits.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: YYYY-MM-DD`
