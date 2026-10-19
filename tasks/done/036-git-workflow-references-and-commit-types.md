---
size: S
origin: done/035-claude-md-review-before-publication.md
created: 2026-10-18
started: 2026-10-19
plan: n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M5 Public release
closed: 2026-10-19
---

# 036 — Git workflow doc: current decision reference, the deps type, the expected index failure

## Context

What: `docs/git-workflow.md` cites D-009 for the trailer rule, which D-010 superseded (its point 3 carries the rule);
CONTRIBUTING's commit types leave out `deps:`, which the workflow doc lists; and the doc's section on parallel
work misses three rules learned in practice: the task-index check of a closing commit reports a stale index,
which is expected; the working tree must be clean before a task is reported done (035's close commit left the
file's edits behind); and a pull request is prepared only after the previous merge is pulled, so the same task
id is never handed out twice (032 waited for #52).
Why: found in the merges of pull requests #52 and #53; both files are what a contributor reads first.
Done when: the doc cites D-010, CONTRIBUTING lists `deps:`, and the workflow doc names the three rules.
Done in parallel with 038, 018 and 019.

## Steps

1. [x] `docs/git-workflow.md`: the trailer rule cites D-010 (point 3), not D-009
2. [x] `CONTRIBUTING.md`: the commit type list gains `deps:` (dependabot), matching the workflow doc
3. [x] Task-session section: the stale-index report of `tasks_check` without `write` is expected on the close
   commit; `git status` must be clean before the session reports
4. [x] Main-session section: a pull request is prepared only after the previous merge is pulled (`tasks_next_id`
   reads the checked-out tree)
5. [x] Pull request, CI green

## Log

- 2026-10-19 — promoted in the main session, branch `task/036-git-workflow-references`
- 2026-10-19 — task session: a `docs/git-workflow.md` trailer-szabálya D-010 3. pontjára hivatkozik; a CONTRIBUTING
  típuslistája `deps:`-szel bővült; a task-session szakasz új pontja a `tasks_check` várt stale-index jelentését
  és a tiszta `git status`-t írja le a jelentés előtt; a main-session szakasz PR-előkészítő pontja az előző merge
  `git pull`-ja után indul (`tasks_next_id` a kicsekkolt fát olvassa). `scripts/check-commit.sh` nem ellenőriz
  típus-prefixet, így `deps:` ott nem igényelt változást; a hibaüzenetei még D-009-re hivatkoznak, ez a PR
  Follow-ups szakaszába került. Lezárva a branchen, PR nyitva.

## Deviations from the plan

- none yet

## Verification

- 2026-10-19 — `git diff` a két fájlon: `D-009` már nem fordul elő egyikben sem (`grep -n D-009` üres); a
  `deps:` típus a CONTRIBUTING Conventions listájában; a task-session szakasz 3. pontja és a main-session szakasz
  PR-előkészítő pontja a Context minden mondatát tartalmazza. A sortördelés a doksi meglévő szélességén belül
  maradt (a leghosszabb új sor 111 bájt, a meglévő leghosszabb 113). A `commit-msg` hook a docs commitot
  átengedte (`core.hooksPath=.githooks`). `tasks_check` `write` nélkül: csak a várt stale-index jelentés a
  README-n. CI: a PR `commit-rules` és a többi job eredménye a PR leírásában.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log (one line in the
  PR description under `## Follow-ups`, filed by the main session)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (index: by the main session when it prepares the PR)
- [x] moved to `done/` with `closed: 2026-10-19`
