---
size: S
origin: done/058-dead-task-references-plan-slugs-and-milestone-closing.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 066 — The `plan:` field is `none` in every task file, the rule is written down

## Context

What: 43 task files carried a `plan:` value that was the bare name of a plan file outside the repo, and most of
those names were slugs derived from prompts (task 058 removed such names from the decisions' `sources:`, the `plan:`
fields stayed). The field is public text through the issue mirror, and a prompt slug is a trace of a conversation.
Why: the repo is public; a reader of an issue should see nothing that is not about the work.
Done when: every task file carries `plan: none`, CLAUDE.md "Tasks" states the rule, and a new promote writes `none`.
Decided 2026-10-19: `plan: none` for every task, new and closed alike; the closed files are rewritten in one
`tasks:` commit rather than kept as history, because the names carry no meaning for a reader.

## Steps

1. [x] every `plan:` line under `tasks/` set to `plan: none` (the three tasks in flight follow in their own
   pull requests)
2. [x] CLAUDE.md "Tasks": the rule says `plan: none`, not a bare file name
3. [x] `tasks_check` green, the issue mirror's dry run shows only the expected body updates

## Log

- 2026-10-19 — promoted and done in the main session in one round with the backlog trim (065 dropped, the 063
  warnings point moved into 061). 53 files rewritten with one substitution over `tasks/` (`plan: <name>` →
  `plan: none`); 054, 051 and 055, in flight on their branches, get the same line when their pull requests are
  prepared.

## Deviations from the plan

- none

## Verification

- 2026-10-19 — `grep -rh "^plan: " tasks | sort | uniq -c`: 54 lines, all `plan: none`. `tasks_check` 0 errors,
  0 warnings. `python -I scripts/sync-issues.py --dry-run`: 4 updates (065 dropped, 066 closed, 061 and 063
  edited), 75 unchanged: the mirror never publishes `plan`, so the rewrite changes no issue body.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-19`
