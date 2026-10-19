---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: none
changelog: housekeeping
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
reporting only, with no e-mail fallback, because no real mailbox is published (D-009/D-010); the copyright line names
the git author; CONTRIBUTING takes over the README's build prerequisites and checks (026).

## Steps

1. [x] `LICENSE` (MIT, `Copyright (c) 2026 Szőke Péter`)
2. [x] `SECURITY.md`: supported versions, attack surface, reporting through private vulnerability reporting
3. [x] `CONTRIBUTING.md`: issue first, prerequisites, checks, synthetic fixtures, D-002, task standard + mirrored issues (D-008), conventions, licence
4. [x] `.github/ISSUE_TEMPLATE/bug-report.yml`, `feature-request.yml`, `config.yml`
5. [x] `.github/pull_request_template.md`, `.github/CODEOWNERS`
6. [x] `tasks/backlog/028-…`: the Community Standards page check added to its checklist
7. [x] Check: YAML parse, links, `gh api …/community/profile` after the push; `tasks_check`; commit

## Log

- 2026-10-18 — plan approved; 023 promoted to wip (together with 026)
- 2026-10-18 — `gh api repos/speti4/mboxer/community/profile` answers on a private repo too: before the push
  `health_percentage: 28`, only the README present; `code_of_conduct` is on the list as well, which is why it was added
- 2026-10-18 — 8 files written; the three issue forms parse with `yaml.safe_load` (name / description / labels /
  body, and blank_issues_enabled / contact_links). CONTRIBUTING and the PR template refer to `docs/reference.md` and
  `docs/development.md`, which 026 creates in the same push

## Deviations from the plan

- `CODE_OF_CONDUCT.md` added (not in the task text): the community profile asks for it. It adopts the Contributor
  Covenant 2.1 by link, plus one rule (no posting of other people's mail or data); reports go through private
  reporting or GitHub's *Report content* menu, without e-mail
- `SECURITY.md`: no e-mail fallback (as planned, D-009: no real mailbox is published)

- 2026-10-18 — commit `bc9dc2b`, pushed (`3c9ccb8..f640447`)
- 2026-10-18 — follow-up sweep: no open item. Turning on private vulnerability reporting and the final check of the
  Community Standards page are on 028's checklist

## Verification

- 2026-10-18 — after the push `gh api repos/speti4/mboxer/community/profile`: `health_percentage` 28 → **100**;
  code_of_conduct, contributing, license, pull_request_template, readme all present. The `issue_template` field
  stays `null`, because the API looks only at the old markdown templates, not the YAML forms; GraphQL
  `issueTemplates` is empty too, but `contactLinks` returns the security link (so `config.yml` parsed)
- 2026-10-18 — Chrome, `issues/new/choose`: Bug report and Feature request forms; Blank issue "Maintainers only";
  the "Report a security vulnerability" contact link. The `?template=bug-report.yml` page renders without errors:
  every field, the mbox size dropdown, the required Privacy checkbox, the `bug` label preset, and the contributing
  guidelines / security policy / code of conduct links at the bottom

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
