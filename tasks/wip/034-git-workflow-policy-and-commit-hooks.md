---
size: M
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
plan: cuddly-painting-trinket.md
milestone: M5 Public release
---

# 034 — Git workflow policy for contributors and parallel sessions, with the commit rules enforced

## Context

What: `docs/git-workflow.md` (contributors: branch → PR → squash merge, commit format, dependabot PRs; maintainer and
agent sessions: one worktree per task, new ids only in the main session, the task index regenerated on main after a
merge, one manual round at a time, push only on the user's go), imported into CLAUDE.md with `@` and linked from
CONTRIBUTING; a versioned `commit-msg` git hook (`.githooks/`, `core.hooksPath`) that rejects a `Claude-Session:`
trailer and a non-noreply author, plus the same check as a CI job for pushes and PRs that bypass the local hook.
Why: parallel sessions need the rules in context at start (user decision 2026-10-18: a policy file, not a decision);
the trailer rule (D-09) is lost to the system reminder unless something checks it. Branch ruleset on main: 028.
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
10. [ ] 034 itself through the flow: branch `task/034-git-workflow-policy`, PR #48, CI red with a trailer commit,
    green without (done), squash merge on the user's go
11. [x] `cargo fmt --check`, clippy, test, `uvx zizmor .github`
12. [ ] Done-when, second half: 033 runs in a worktree session and follows the file (PR, no new id, follow-ups in
    the PR body)

## Log

- 2026-10-18 — promoted from backlog; plan approved (user decisions: noreply rule for the maintainer identity
  only; worktrees under `.claude/worktrees/`; task sessions open a PR, the main session squash-merges it)
- 2026-10-18 — branch `task/034-git-workflow-policy`: `scripts/check-commit.sh`, `.githooks/commit-msg`
  (100755, `i/lf w/lf`), `commit-rules` job, `docs/git-workflow.md`, CLAUDE.md import, CONTRIBUTING / PR template /
  development.md; `core.hooksPath` beállítva. A hook egy scratch worktree-ben: trailer, session link, nem-noreply
  maintainer → exit 1; contributor (más név, saját cím) → exit 0; `commit -v -e` diffjében lévő link nem riaszt
  (scissors levágva), a szerkesztett üzenetben lévő trailer igen; CRLF-es PR body-ban is elkapja. Az azonos
  author/committer címet csak egyszer jelenti.
- 2026-10-18 — repo merge settings (`gh api -X PATCH`): `allow_merge_commit` / `allow_rebase_merge` false,
  `squash_merge_commit_title=PR_TITLE`, `squash_merge_commit_message=PR_BODY`, `delete_branch_on_merge` true.
  Ezért a PR body végére kell a `Co-Authored-By` trailer — beírva a git-workflow.md-be.
- 2026-10-18 — `cargo fmt --check`, `clippy -D warnings`, `cargo test` zöld; `uvx zizmor .github`: no findings.
- 2026-10-18 — PR #48 squash-merge-ölve (`dda4e86`), main CI zöld. A squash commitban két meglepetés: a GitHub
  `---------` után magától hozzáfűzi a commitok `Co-authored-by` sorait (a PR body-ban lévő trailer így duplán
  jelent meg), és az author `speti4 <noreply>`, nem `Szőke Péter`. Javítás a `task/034-commit-rule-fixes`
  branchen: a git-workflow.md már nem kéri a trailert a PR body-ba; a `check-commit.sh` a `speti4` nevet is
  maintainernek veszi (speti4 + idegen cím → exit 1, speti4 + noreply → 0, más név → 0).

## Deviations from the plan

- The PR description ends with `🤖 Generated with Claude Code` only, as planned; a detour through a
  `Co-Authored-By` trailer in the body was reverted after PR #48 showed that GitHub appends the commits'
  co-authors to the squash message itself.
- The maintainer check knows two names: `Szőke Péter` (git config) and `speti4` (the author of a web merge).
- The 034 promote commit lives only on the task branch; local `main` is set back to `origin/main` before the push,
  so the squash merge is the one commit `main` gets.

## Verification

- 2026-10-18 — hook helyben (scratch worktree, `scratch/hook-test`, utána törölve): `Claude-Session:` trailer,
  session link, `-c user.email=x@example.com` a maintainer nevével → mind exit 1 a saját üzenetével; más név +
  saját cím → exit 0; tiszta commit → átmegy (c04a34d, 3deb730 ezzel készült).
- 2026-10-18 — CI a PR #48-on: első futás (39145607833) mind zöld, `commit-rules` 4 s. Ideiglenes commit trailerrel
  (`fdfb64a`, `--no-verify`) → `commit-rules` failure (39145901495): "no Claude-Session: trailer … in fdfb64a…",
  exit 1. Eldobva (`reset --hard 3deb730`, `--force-with-lease`) → újra minden zöld (39145983395).

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
