---
status: idea
size: S
origin: wip/003-fulltext-sidecar-index-accent-folding.md
created: 2026-10-17
milestone: M3 Comfort
---

# Index fallback when the mbox folder is not writable

What: When `.mboxer/` cannot be created next to the mbox (DVD, read-only share, denied folder), build the sidecar under `%LOCALAPPDATA%\mboxer\<fingerprint hash>\` instead and look there on open.
Why: D-04 puts the index beside the mbox; a read-only location today fails the build outright, which the other 2–5 users may hit with an archive on a network share.
Done when: `mboxer index` on an mbox in a read-only folder succeeds, `mboxer search` finds the fallback index, and the task's test covers both locations.
