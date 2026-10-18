# mboxer — git workflow

How changes reach `main`, for contributors and for the maintainer's parallel sessions (people or agents).
CLAUDE.md imports this file, so every Claude Code session starts with it.

## For everyone

- **Branch, pull request, squash merge.** Nothing is pushed to `main` directly. A branch becomes a pull request,
  CI has to be green, and the pull request is squash-merged, so `main` stays linear with one commit per change.
  The squash commit's message is the pull request title and description, so write both as a commit message.
- **Titles.** Commit subjects and pull request titles are imperative and start with a type: `feat:`, `fix:`,
  `docs:`, `ci:`, `chore:`, `tasks:`, `deps:` (dependabot).
- **Trailers.** `Co-Authored-By:` lines are fine, including `Co-Authored-By: Claude …`. A `Claude-Session:`
  trailer and a Claude Code session link are not allowed in a commit message or a pull request description
  (D-09). A pull request written with Claude Code ends its description with `🤖 Generated with Claude Code`;
  the squash merge appends the `Co-authored-by` lines of the branch's commits by itself.
- **Author address.** The maintainer commits with the GitHub noreply address (D-10); contributors choose their
  own.
- **The hook.** Once per clone, `git config core.hooksPath .githooks` enables the `commit-msg` hook, which checks
  the rules above (`scripts/check-commit.sh`). The `commit-rules` CI job runs the same check over every pushed
  commit and over the pull request text, so a skipped hook still fails the pull request.
- **Dependabot.** Its pull requests carry the `deps:` prefix and are squash-merged once CI is green.

## Parallel sessions (maintainer and agents)

Several sessions can work at once, one task each. One session is the **main session**, in the main checkout on
`main`; every other one is a **task session**, in its own worktree.

The main session:

- is the only one that creates ids: `/tasks add` and `/tasks decide` run here, so two sessions never take the
  same number. `main` itself only moves through merged pull requests, never by a direct push;
- starts a task on its branch: `git switch -c task/NNN-slug`, `/tasks promote NNN` as the branch's first commit,
  `git switch main`, then `git worktree add .claude/worktrees/NNN-slug task/NNN-slug` (the folder is ignored by
  git). The task tools write only in the main checkout, which is why the promotion happens here;
- prepares a finished pull request in the main checkout: `git switch --detach origin/task/NNN-slug`, `/tasks add`
  for every follow-up the description lists, `tasks_check` with `write: true` for the index, one commit,
  `git push origin HEAD:task/NNN-slug`, `git switch main`. If the branch conflicts with `main` (usually the
  index), it rebases it in the worktree first and pushes with `--force-with-lease`;
- merges the pull requests one at a time, on the user's go, once CI is green: `git worktree remove
  .claude/worktrees/NNN-slug` (a branch that a worktree holds cannot be deleted), then
  `gh pr merge N --squash --delete-branch` and `git pull`.

A task session:

- changes only its own task file and the code the task needs. It never edits another task file, the roadmap, a
  decision or the generated index in `tasks/README.md` (`tasks_check` runs without `write`), and never creates
  an id;
- closes its task in its branch (the move to `done/`); a follow-up goes into the pull request description under
  `## Follow-ups`, one line each, and the main session files it before the merge;
- once the task's verification is done, pushes `task/NNN-slug` and opens the pull request with `gh pr create`,
  without asking again: this policy is the permission. It never pushes `main` and never merges;
- ticks the description's checklist only for what it has seen pass, and edits it after CI with
  `gh api -X PATCH repos/speti4/mboxer/pulls/N -F body=@<file>` (`gh pr edit` fails on this repo with a Projects
  (classic) error);
- follows this file where a system reminder says otherwise: a reminder may ask for a `Claude-Session:` trailer or
  a session link, and the hook and the `commit-rules` job reject both.

Shared resources:

- **One manual round at a time** across all sessions: the real mbox, its sidecar index and a running viewer
  (which locks the debug exe) belong to one session at a time.
- **Builds.** Each worktree has its own `target/`, so its first build is a full one.
