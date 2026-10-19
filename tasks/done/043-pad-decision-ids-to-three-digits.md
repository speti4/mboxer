---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: n-zz-k-t-a-backlogot-toasty-dove.md
milestone: M5 Public release
---

# 043 — Pad the decision ids to three digits before the repo goes public

## Context

What: `/tasks renumber` renames the ten two-digit decision files to three digits and rewrites the references inside
`tasks/`; the ~30 files outside it are fixed by hand (crate comments, docs, CLAUDE.md, README, CONTRIBUTING,
`scripts/check-commit.sh`, workflows). `scripts/sync-issues.py` keys the decision issues by their id
(`<!-- sync-issues: decision D-NN -->`), so it must match the old two-digit key to the new one instead of
opening ten new issues; `CHANGELOG.md` stays in line with `scripts/changelog.py --check`.
Why: mixed widths sort and read wrong; the public repo starts consistent (user decision 2026-10-19).
Done when: `tasks_check` has no narrow-id warning, a grep for two-digit decision ids finds only history that
must stay, and a sync run edits the ten decision issues in place.
Split (2026-10-19): the main session runs the renumber inside `tasks/` on this branch (the tool writes only in
the main checkout); the task session does everything outside `tasks/`.

## Steps

1. [x] `tasks_renumber` with `apply` (main session): ten decision files renamed, the references in `tasks/`
   rewritten, the index regenerated
2. [x] the references outside `tasks/` (the tool's `names` list and every bare two-digit decision id): code
   comments, docs, CLAUDE.md, README, CONTRIBUTING, scripts, workflows, the pull request template
3. [x] `scripts/sync-issues.py`: an existing issue whose marker carries the two-digit key is matched to the
   three-digit one and edited in place (title, body, marker); a dry run shows ten edits and no creates
4. [x] `CHANGELOG.md` agrees with `python -I scripts/changelog.py --check`
5. [x] the CONTRIBUTING.md checks; `scripts/check-commit.sh` still rejects a trailer and a session link
6. [ ] after the merge: the sync run on `main` edits the ten decision issues, opens none

## Log

- 2026-10-19 — promoted on `task/043-pad-decision-ids-to-three-digits`.
- 2026-10-19 — main session: `tasks_renumber` apply (10f2e49), tíz döntésfájl `D-0NN-…`, a `tasks/` hivatkozásai
  átírva, index újragenerálva.
- 2026-10-19 — task session, `tasks/` kívül: a `D-01` … `D-10` szóhatáros alakok → `D-001` … `D-010` egy
  scratch scripttel a git által követett fájlokon (`tasks/` és `CHANGELOG.md` kivételével): 53 csere 30
  fájlban — Rust/JS kommentek (15 fájl, csak komment), `Cargo.toml` (5), docs (`reference`, `development`,
  `git-workflow`, `research`: 10), README, CONTRIBUTING, CLAUDE.md, `scripts/` (`check-commit.sh`,
  `bundle-macos.sh`, `sync-issues.py`), workflows (2), PR template; a research fájl régi `tasks/decisions/D-01-….md` linkje is. A `scripts/__pycache__/` a `.gitignore`
  `__pycache__/` sora alatt van, nem forrás.
- 2026-10-19 — `scripts/sync-issues.py`: `match_issues` először a marker pontos kulcsát keresi, aztán azt az
  issue-t, amelynek markere ugyanazt a számot hordja más szélességben (`same_id`: a szám nullák nélkül, általános,
  nem lista; taskokra is áll). A meglévő issue-t helyben szerkeszti: a body az új markerrel együtt cserélődik, a
  title is; a log sor `was D-0N`-t ír. Új dry-run mód nem kellett, a meglévő `--dry-run` élesben (csak GET).
- 2026-10-19 — `CHANGELOG.md`: a `--check` már a task előtt is elavult volt (a 018–042 között zárt taskok
  hiányoztak; a changelog csak kiadás előtt frissül, CI nem ellenőrzi), így a futás az Unreleased listát egészében
  frissítette, a 030 sora `per D-009` lett. A 0.1.0 szekció nem hivatkozik döntésre, változatlan; kiadott történet
  nem módosult. A 030 és 039 fájlneve (`…-per-d-09.md`, `…-cite-d-10.md`) task fájlnév, a renumber nem nevezi át,
  a changelog linkje ezért marad a régi slugon.
- 2026-10-19 — maradék a `tasks/`-ban (nem a task session dolga, a PR Follow-ups listázza): `tasks/roadmap.md:8`
  `(D-08)`; idézet/történet: `D-007` 34. sor („D-07-t megerősítem”, a user szó szerinti idézete), 036 és 039 Log
  sorai (`D-09-re`, `D-10-re`, `D-09-es`) — a tool a magyar toldalékos alakot nem írta át.

## Deviations from the plan

- none

## Verification

- 2026-10-19 — `grep -rnE "D-(0[0-9]|10)\b"` `tasks/`, `target/`, `.git/`, `.claude/` nélkül: üres (kis-/nagybetű
  nélkül csak a `CHANGELOG.md` két linkje a 030/039 fájlnevére).
- 2026-10-19 — `python -I scripts/sync-issues.py --dry-run` a valódi repón (read-only): a tíz döntés-issue (#31–#40)
  `update (body, title, was D-0N)`, D-011/D-012 csak body; `created 0, updated 41, unchanged 14`, orphan nincs.
  A többi update a task-issue-k body-ja (a renumber átírta a task fájlok hivatkozásait).
- 2026-10-19 — `python -I scripts/changelog.py --check`: up to date.
- 2026-10-19 — `scripts/check-commit.sh` scratch üzenetekkel: tiszta → 0; `Claude-Session:` trailer → 1, „(D-010
  point 3)”; session link → 1, „(D-010 point 3)”; nem-noreply cím → 1 (D-010); a `.githooks/commit-msg` hookon át a
  trailer → 1.
- 2026-10-19 — `uvx zizmor .github`: no findings; `cargo deny check`: ok;
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`: mind zöld
  (a kód változása csak komment).

## Closing checklist

- [x] every follow-up is in the pull request's `## Follow-ups` (the main session files them; task session rule)
- [x] Verification written, dated
- [x] `tasks_check` green apart from the stale index; the README index is regenerated by the main session
- [x] moved to `done/` with `closed: 2026-10-19`
