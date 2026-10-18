---
status: decided
date: 2026-10-17
cost: cheap
sources: [wip/014-index-fallback-for-read-only-mbox-locations.md, D-04-sidecar-index-location-and-freshness.md, 12-14-15-taskokat-fejlessz-k-le-keen-whale.md]
---

# D-05 — When `.mboxer/` beside the mbox cannot be created, the index lives under `%LOCALAPPDATA%\mboxer\index\<name>-<path hash>\`

## Context

D-04 puts the sidecar index beside the mbox and listed `%LOCALAPPDATA%` as a not-chosen default,
kept as a fallback idea. Task 014 makes that fallback real: an archive on a read-only share, a
DVD or a denied folder failed the build outright, which any user may hit. The rule
has a lasting effect on where mboxer writes on a user's machine, so it is recorded here.

## Decision

- Primary location stays D-04: `<folder of the mbox>/.mboxer/<file name>/`.
- Trigger: any I/O error while preparing the primary `.building` directory (remove a stale one,
  create it). No permission probe: on Windows the read-only folder attribute does not block
  creation, a deny ACL, a read-only share or a stray file named `.mboxer` does, and the build
  finds out by trying.
- Fallback location: `<index home>/<file name>-<16 hex>/`, the hex being FNV-1a 64 over the
  canonical absolute path of the mbox. The index home is `MBOXER_INDEX_HOME` when set, else
  `%LOCALAPPDATA%\mboxer\index`, else `<temp>\mboxer\index`.
- Keyed by the path, not by the content fingerprint: one location maps to one directory, a new
  export dropped at the same place replaces its index, nothing is orphaned. Freshness is still the
  `mboxer.json` fingerprint of D-04 in either location.
- Every open looks in both places: a fresh primary wins, then a fresh fallback; a stale index is
  reported from the primary first. A build clears the location it did not use, so one index
  exists per mbox. The CLI prints the fallback on `index`, the viewer shows the index directory
  in the status bar tooltip.

Proposed by the session on 2026-10-17 (task 014); confirmed by the user on 2026-10-18 ("Megerősítem").

## Not chosen

- Keying the fallback by the content fingerprint (the backlog wording): a moved mbox would keep
  its index, but a re-export at the same read-only place would leave the old directory behind.
- A write probe or ACL check before the build: more code than trying, and still wrong for a
  share that accepts a directory but not a file.
- Always indexing under `%LOCALAPPDATA%`: D-04's reasons stand (visible, deletable, moves with
  the mbox).
