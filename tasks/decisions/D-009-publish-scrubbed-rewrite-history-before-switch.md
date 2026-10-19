---
status: superseded
date: 2026-10-18
cost: one-way
superseded-by: D-010-publish-scrubbed-rewritten-history-in-a-recreated-repo.md
sources: [done/027-pre-publish-history-and-content-audit.md, a-task-021-hez-kapcsol-d-an-mellow-hearth.md]
---

# D-009 — The repo goes public with `tasks/` and `docs/` scrubbed in place and the history rewritten once before the switch

## Context

Task 027 audited the repo before the visibility switch (M5). gitleaks over all 33 commits found no secret; the
PII scan found no real message content and no personal identifier beyond the author's own. The manual inventory
found one finding that names a third party: a local folder path in `docs/origin.md` carries a former employer's
name, present since the first commit. The rest is the author's own trace: the archive path, `C:\Users\<name>`
plan paths in 28 frontmatter fields, aggregate benchmark numbers of the mailbox, Hungarian Logs, Claude
attribution trailers and a personal Gmail address on every commit, and the "built for a few colleagues" framing.
A public repo exposes every past commit, so an edit alone leaves the third-party name reachable through the
history. Today the rewrite is cheap: 33 commits, one author, one tag, one release, no fork, no clone besides the
author's.

## Decision

1. **`tasks/` and `docs/` are published, scrubbed in place** (task 030): the two local paths out of `origin.md`
   and `CLAUDE.md`; `plan:` / `sources:` plan paths become the bare file name; the colleagues framing becomes a
   general-tool wording everywhere the inventory lists; `docs/origin.md` is rewritten in English as the project's
   origin story without paths or names (the Hungarian draft is not kept). Benchmark numbers stay: aggregates, no
   content. Hungarian Log sections of closed tasks stay as they are (history; the Context lines that reach the
   issues are English already).
2. **The history is rewritten once, before the switch**, with `git filter-repo --replace-text`: the employer
   name, the archive path, `C:\Users\<name>` and the one status line of the 011 Log that may carry a real file
   name are replaced in every past blob; `v0.1.0` is re-tagged on the rewritten commit, `main` and the tag are
   force-pushed, the author re-clones. The GitHub release and its asset, the issues and the milestones are
   untouched by a rewrite.
3. **Commit trailers**: the `Claude-Session:` lines are removed in the same rewrite and no longer written (they
   open only for the author's account); `Co-Authored-By: Claude …` stays as the honest record and keeps being
   written.
4. **Author identity**: every commit, past and future, carries the author's GitHub noreply address
   (`<id>+<login>@users.noreply.github.com`, the one `git config user.email` holds in this repo from now on), not
   a personal mailbox; GitHub still attributes the commits to the profile. The rewrite maps the old mailbox to it
   with `--mailmap`.
5. The scrub (1) and the rewrite (2–4) are one task (030) and land before 028 flips the visibility; 028 waits
   for 030.

Proposed by the session on 2026-10-18 from the 027 audit; the user confirmed points 1–3 on 2026-10-18 and, after
GitHub's e-mail privacy protection rejected a push with their developer mailbox, chose the noreply address for
point 4 the same day.

## Not chosen

- Publish as is: leaves a third party's name in a public history for a private project that never needed it.
- Edit in place without a rewrite: the name stays one `git log -p` away; the rewrite costs minutes today and is
  impossible to do cleanly once anyone has cloned.
- Leave `tasks/` out of the public repo: the task standard, the decisions and the mirror into issues (D-008) are
  the project's working method and part of what it shows; the private bits are two paths and a framing, not the
  files.
- A fresh repo from a squashed snapshot: loses the 37 issues, the milestones and the release for nothing a
  rewrite does not already give.
- Keeping the personal address, or a second real mailbox: both end up harvestable in a public history, and the
  account's "block pushes that expose my email" protection rejects them unless the address is made public.
- Dropping `Co-Authored-By` too (as docket does): the user keeps it as the record of how the code was written.
- Rewriting the Hungarian Logs into English: history of closed work, nobody reads it from the issue side, and the
  translation would be busywork with no reader.
