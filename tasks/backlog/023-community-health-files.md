---
status: ready
size: S
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Community health files: licence, security policy, contributing guide, issue and PR templates

What: a `LICENSE` file (MIT, matching the workspace `Cargo.toml`); `SECURITY.md` naming the real attack surface
(the HTML sanitizer, the sandboxed iframe and its CSP, opening attachments and the Mark of the Web, the index
files) and how to report (a private security advisory once public, e-mail until then); `CONTRIBUTING.md` (build
prerequisites, the fmt / clippy / test checks, synthetic fixtures only, the task standard, the D-02 dependency
rule); `.github/ISSUE_TEMPLATE/` bug and feature forms (mboxer version, Windows build, mbox size, a required
"no real message content" checkbox) with `config.yml` turning blank issues off; `pull_request_template.md`;
`CODEOWNERS`. Inspired by serionist/docket, written for a mail viewer.
Why: the first thing a visitor and GitHub's community profile look for; an mbox viewer's users hold private
mail, so the templates must keep it out of public issues.
Done when: GitHub's Community Standards page is complete except for the items that need a public repo.
