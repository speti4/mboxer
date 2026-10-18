---
status: ready
size: M
origin: wip/027-pre-publish-history-and-content-audit.md
created: 2026-10-18
milestone: M5 Public release
---

# Scrub the repo and rewrite the history per D-09

What: the in-place edits D-09 point 1 lists (origin.md rewritten in English, the archive path out of CLAUDE.md,
27 `plan:` / `sources:` paths reduced to bare names, the colleagues framing reworded in README / roadmap / D-07 /
done 010 012 015, the 011 Log line, a CLAUDE.md line on commit trailers), then one `git filter-repo` run
(`--replace-text` for the four strings, `--mailmap` to the GitHub noreply address, `--message-callback` dropping
`Claude-Session:`), `v0.1.0` re-tagged, force push, re-clone; the 027 Log has the inventory with locations.
Why: D-09 — one third-party name sits in the history since the first commit; the rewrite is cheap while the repo
is private and unforked, impossible to do cleanly later. 028 waits for this task.
Done when: `git log -p --all` finds none of the four strings, every commit carries the noreply address and no
`Claude-Session:` line, `gh release view v0.1.0` still shows the asset, `tasks_check`, CI and the sync are green.
