---
size: S
origin: done/025-release-notes-changelog-and-checksums.md
created: 2026-10-18
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-gentle-goose.md
milestone: M5 Public release
closed: 2026-10-19
---

# 038 — README links the changelog next to the releases

## Context

What: the README's download step and its docs list point to the releases but not to `CHANGELOG.md`, which 025 added
(D-012); add the link, and a word on checking the zip against `SHA256SUMS.txt` where the download step is.
Why: a visitor reading the front page should find what changed and how to verify the download without the docs.
Done when: the README links `CHANGELOG.md` and names the checksum file at the download step.
Run as a task session under docs/git-workflow.md, in parallel with 036, 018 and 019 (plan 2026-10-19).

## Steps

1. [x] `README.md`: the download step links `CHANGELOG.md` and says how to check the zip against `SHA256SUMS.txt`
2. [x] `README.md`: the docs list gains the changelog entry
3. [x] Pull request, CI green

## Log

- 2026-10-19 — promoted in the main session, branch `task/038-readme-changelog-link`
- 2026-10-19 — task session a `.claude/worktrees/038-readme-changelog-link` worktree-ben: a README letöltési
  lépése a `CHANGELOG.md`-re mutat és egy mondatban adja a `Get-FileHash` / `SHA256SUMS.txt` összevetést (ugyanaz
  az egysoros, amit a `docs/usage.md` is ír, kódblokk nélkül, mert a lépés listája sem használ ilyet); a
  dokumentációs lista új sora a changelog és a releases oldal; lezárva a branchen

## Deviations from the plan

- none

## Verification

- 2026-10-19 — `README.md` minden relatív linkje (14 db, köztük az új `CHANGELOG.md`) létező fájlra vagy mappára
  mutat (shell ciklus `[ -e ]`-vel a worktree gyökeréből); 112 karakternél hosszabb sor csak a három meglévő
  badge / kép sor; a `Get-FileHash` mondat a `docs/usage.md` 20. sorával egyező parancsot ad
- 2026-10-19 — `tasks_check` a worktree-n: csak a README index „stale” jelzése (a main session regenerálja a
  merge előtt)

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the main session, before the merge)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
