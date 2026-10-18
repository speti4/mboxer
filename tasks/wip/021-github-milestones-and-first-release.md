---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
plan: local-repoban-profi-a-clever-pie.md
---

# 021 — GitHub milestones from the roadmap and the first release v0.1.0

## Context

A lokális task standard kész (roadmap négy milestone-nal, minden task `milestone:` mezővel), a GitHub repón viszont
nincs milestone, issue, tag, release, miközben a `docs/usage.md` a "GitHub releases page"-re küldi a usereket.
What: a négy roadmap-milestone tükrözése GitHub milestone-ként (azonos címek, M1–M3 closed, M4 open), `v0.1.0` tag,
amit a meglévő CI release jobja csomagol és publikál, kézzel írt release notes, egy-egy sor a docsban, D-08.
Why: a 2–5 usernek a release zip számít, a milestone önmagában üres lenne. User decision 2026-10-18: "milestone-ok +
v0.1.0 release" variáns; a tasks/ → issues tükröző script nem kell (project-pal feature lenne, csak GitHub-közönség
esetén). Done when: 4 milestone a megfelelő állapotban, a `v0.1.0` release egy `mboxer-0.1.0.zip` assettel, a zip
tartalma `mboxer.exe` (ProductVersion 0.1.0) + `usage.md`.

## Steps

1. [ ] GitHub milestones (`gh api`): `M1 Index core`, `M2 Viewer MVP`, `M3 Comfort` closed, `M4 macOS` open, leírás a roadmapből
2. [ ] Docs: `README.md` (milestone-tükör + Releases), `tasks/roadmap.md` (egy mondat), `docs/usage.md` (Releases URL)
3. [ ] D-08 proposed: a milestone-tükör szabálya
4. [ ] Commit, `v0.1.0` annotated tag, push; a CI `release` job zöld, a release megjelenik
5. [ ] Release notes `gh release edit v0.1.0 --notes-file`
6. [ ] Ellenőrzés: milestone lista, zip letöltése és tartalma, ProductVersion
7. [ ] `tasks_check`, close, commit

## Log

- 2026-10-18 — plan approved; 021 opened straight in wip/

## Deviations from the plan

- none yet

## Verification

- pending

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
