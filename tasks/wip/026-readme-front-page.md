---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
milestone: M5 Public release
plan: peppy-imagining-pony.md
---

# 026 — README as a front page, the reference moved into docs

## Context

The README rewritten as a front page: a one-line pitch, badges (CI, latest release, licence), a screenshot of
the viewer over a synthetic mbox (never the author's archive), a feature list, download and quick start, and
links to usage, build and decisions. The current long reference moves to `docs/reference.md`, or into
`docs/usage.md` where it addresses the user. Why: today the README is a dense reference with no picture, and a
visitor decides within one screen. Done when: the README fits on two screens and every moved paragraph has a home
under `docs/`. Planning decisions 2026-10-18 (approved plan): the viewer and CLI reference goes to
`docs/reference.md`; layout, CI, release and scripts go to `docs/development.md`; build prerequisites and checks go
to `CONTRIBUTING.md` (023). The screenshot is taken over a demo mbox that `scripts/make-demo-mbox.py` generates
(stdlib only, deterministic, made-up mail under `example.*` domains), which can also serve the social preview in 028.

## Steps

1. [ ] `scripts/make-demo-mbox.py` → `target/demo/demo.mbox`; `mboxer-cli scan` / `index` / `search` rajta
2. [ ] Képernyőkép a release exe-ből a demo mboxon → `docs/images/viewer.png`
3. [ ] `docs/reference.md` (viewer + CLI referencia) és `docs/development.md` (layout, CI, release, scriptek)
4. [ ] Új `README.md` front page (pitch, badge-ek, kép, features, download, build, docs, status, licenc)
5. [ ] `docs/usage.md`: link a referenciára; `CLAUDE.md` Scope: az új docs fájlok
6. [ ] Ellenőrzés: sorszám, minden régi bekezdésnek megvan az új helye, relatív linkek; `tasks_check`; commit

## Log

- 2026-10-18 — plan approved; 026 promoted to wip (023-mal együtt)

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
