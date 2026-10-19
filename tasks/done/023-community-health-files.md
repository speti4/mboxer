---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: peppy-imagining-pony.md
---

# 023 — Community health files: licence, security policy, contributing guide, issue and PR templates

## Context

A `LICENSE` file (MIT, matching the workspace `Cargo.toml`); `SECURITY.md` naming the real attack surface (the
HTML sanitizer, the sandboxed iframe and its CSP, opening attachments and the Mark of the Web, the index files)
and how to report it; `CONTRIBUTING.md` (build prerequisites, the fmt / clippy / test checks, synthetic fixtures
only, the task standard, the D-002 dependency rule); `.github/ISSUE_TEMPLATE/` bug and feature forms (mboxer
version, Windows build, mbox size, a required "no real message content" checkbox) with `config.yml` turning blank
issues off; `pull_request_template.md`; `CODEOWNERS`. Why: these are the first things a visitor and GitHub's
community profile look for, and the people who use an mbox viewer hold private mail, so the templates must keep
it out of public issues. Done when: GitHub's Community Standards page is complete except for the items that need
a public repo. Planning decisions 2026-10-18 (approved plan): reports go through GitHub private vulnerability
reporting only, with no e-mail fallback, because no real mailbox is published (D-009); the copyright line names
the git author; CONTRIBUTING takes over the README's build prerequisites and checks (026).

## Steps

1. [x] `LICENSE` (MIT, `Copyright (c) 2026 Szőke Péter`)
2. [x] `SECURITY.md`: supported versions, támadási felület, bejelentés private vulnerability reportinggal
3. [x] `CONTRIBUTING.md`: issue first, prerequisites, checks, synthetic fixtures, D-002, task standard + mirrored issues (D-008), konvenciók, licenc
4. [x] `.github/ISSUE_TEMPLATE/bug-report.yml`, `feature-request.yml`, `config.yml`
5. [x] `.github/pull_request_template.md`, `.github/CODEOWNERS`
6. [x] `tasks/backlog/028-…`: a Community Standards oldal ellenőrzése a checklistre
7. [x] Ellenőrzés: YAML parse, linkek, `gh api …/community/profile` push után; `tasks_check`; commit

## Log

- 2026-10-18 — plan approved; 023 promoted to wip (026-tal együtt)
- 2026-10-18 — `gh api repos/speti4/mboxer/community/profile` privát repón is válaszol: push előtt
  `health_percentage: 28`, csak a README van meg; a `code_of_conduct` is a listán szerepel, ezért került be
- 2026-10-18 — 8 fájl megírva; a három issue form `yaml.safe_load`-dal parse-ol (name / description / labels /
  body, illetve blank_issues_enabled / contact_links). A CONTRIBUTING és a PR template a `docs/reference.md` és
  `docs/development.md` fájlokra hivatkozik, ezek a 026-ban születnek, ugyanabban a pushban

## Deviations from the plan

- `CODE_OF_CONDUCT.md` hozzáadva (nem volt a task szövegében): a community profile számon kéri. A Contributor
  Covenant 2.1-et linkkel veszi át, plusz egy szabály (más levelét / adatait nem posztoljuk); a bejelentés a
  private reportinggal vagy a GitHub *Report content* menüjével megy, e-mail nélkül
- `SECURITY.md`: nincs e-mailes tartalék (a terv szerint, D-009: nincs publikált valódi postafiók)

- 2026-10-18 — commit `bc9dc2b`, a user jóváhagyásával pusholva (`3c9ccb8..f640447`)
- 2026-10-18 — follow-up sweep: nincs nyitott tétel. A private vulnerability reporting bekapcsolása és a Community
  Standards oldal végső ellenőrzése a 028 checklistjén van

## Verification

- 2026-10-18 — push után `gh api repos/speti4/mboxer/community/profile`: `health_percentage` 28 → **100**;
  code_of_conduct, contributing, license, pull_request_template, readme mind megvan. Az `issue_template` mező `null`
  marad, mert az API csak a régi markdown template-eket nézi, a YAML formokat nem; a GraphQL `issueTemplates` is
  üres, a `contactLinks` viszont visszaadja a security linket (a `config.yml` tehát parse-olt)
- 2026-10-18 — Chrome, `issues/new/choose`: Bug report és Feature request form; Blank issue „Maintainers only”;
  „Report a security vulnerability” contact link. A `?template=bug-report.yml` oldal hiba nélkül renderel: minden
  mező, a mbox méret dropdown, a kötelező Privacy checkbox, a `bug` label előre beállítva, alul a contributing
  guidelines / security policy / code of conduct linkek

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
