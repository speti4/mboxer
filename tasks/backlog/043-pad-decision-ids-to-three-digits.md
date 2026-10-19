---
status: ready
size: M
origin: session 2026-10-19
created: 2026-10-19
milestone: M5 Public release
---

# Pad the decision ids to three digits before the repo goes public

What: `/tasks renumber` renames `D-01` … `D-10` to `D-001` … `D-010` and rewrites the references inside `tasks/`
(43 files); the ~30 files outside it are fixed by hand (crate comments, docs, CLAUDE.md, README, CONTRIBUTING,
`scripts/check-commit.sh`, workflows). `scripts/sync-issues.py` keys the decision issues by their id
(`<!-- sync-issues: decision D-08 -->`), so it must match the old two-digit key to the new one instead of
opening ten new issues; `CHANGELOG.md` stays in line with `scripts/changelog.py --check`.
Why: mixed widths (`D-10` next to `D-011`) sort and read wrong; the public repo starts consistent (user decision
2026-10-19). Done when: `tasks_check` has no narrow-id warning, `grep -rE "D-(0[0-9]|10)\b"` finds only
history that must stay, and a sync run edits the ten decision issues in place.
