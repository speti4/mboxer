---
size: S
origin: wip/024-supply-chain-and-ci-hardening.md
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
plan: none
milestone: M5 Public release
---

# 033 — Install the Rust toolchain in CI without a third-party action

## Context

What: drop `dtolnay/rust-toolchain` from `ci.yml`; the runner's preinstalled rustup installs the toolchain and the
components that `rust-toolchain.toml` names (`rustup show` or the first `cargo` call), in the test and release jobs.
Why: the repo has no release tags, so the action is pinned to a `master` commit that dependabot cannot follow (its
run reports "Latest version is 1, no update needed") and the pin would go stale silently; zizmor's pedantic
persona flags it as `ref-version-mismatch`. One third-party action fewer in the release job.
Done when: CI is green without the action and the test log shows the toolchain from `rust-toolchain.toml`.
Run as the first task session under docs/git-workflow.md (task 034): its own worktree, a pull request, no new id.

## Steps

1. [x] `ci.yml`: replace `dtolnay/rust-toolchain` in the test and release jobs with a rustup step that installs
   what `rust-toolchain.toml` names; update the comments that mention the action or the toolchain
2. [x] `docs/development.md` and any other mention of the action, if there is one
3. [x] `uvx zizmor .github` clean; push the branch, open the pull request, CI green; the test log shows the
   toolchain and components from `rust-toolchain.toml`

## Log

- 2026-10-18 — promoted by the main session; worked in `.claude/worktrees/033-toolchain-without-action` on
  branch `task/033-toolchain-without-action`
- 2026-10-18 — `ci.yml`: a „Install the Rust toolchain from rust-toolchain.toml" bash lépés a test és a release
  jobban (`rustup toolchain install` argumentum nélkül, majd név szerint, `rustup show`, a test jobban
  `rustup component list --installed` is); workflow-szintű `env`: `CARGO_INCREMENTAL=0`, `CARGO_TERM_COLOR=always`;
  a fejléc-komment és `docs/development.md` mondja, honnan jön a toolchain. PR #50.

## Deviations from the plan

- Két rustup hívás kell, nem egy: az argumentum nélküli `rustup toolchain install` (rustup 1.29.1 helyben is)
  meglévő toolchainnél csak „using existing install"-t ír, a csatornát nem frissíti. A runner image stable-je
  régebbi lehet (az első futásban 1.99.0 volt, a legfrissebb 1.99.1), a dtolnay action mindig a legfrissebbet
  tette fel, ezért egy második `rustup toolchain install <név>` (a név a `rustup show active-toolchain`-ből) hozza
  naprakészre. Rögzített verziónál (`channel = "1.x.y"`) ez no-op.
- Az action a későbbi lépéseknek `CARGO_INCREMENTAL=0`-t, `CARGO_TERM_COLOR=always`-t és `CARGO_HOME`-ot is
  exportált. Az első futás nélkülük inkrementálisan fordított, és a rust-cache kulcsa (env-hash) is megváltozott;
  a kettőt most a workflow állítja be. A `CARGO_HOME` (a default útvonal) nem került vissza, egy runner-útvonal
  beégetése nem éri meg: a cache-kulcs egyszer változik, a merge utáni első `main` futás cache nélkül fordít.

## Verification

- 2026-10-18 — `uvx zizmor .github`: „No findings" (1 ignored, 6 suppressed); `--persona pedantic`: a dtolnay
  lépések két `superfluous-actions` találata eltűnt, csak `anonymous-definition` infók maradtak.
- 2026-10-18 — helyben a lépés szkriptje Git Bashben (rustup 1.29.1): az argumentum nélküli install
  „using existing install … overridden by '…\rust-toolchain.toml'", a név szerinti „syncing channel updates …
  unchanged - rustc 1.99.1"; a telepített komponensek közt `rustfmt` és `clippy`.
- 2026-10-18 — CI run 39148822817 (commit d5e2643), mind a négy job zöld. A test job logja (job 117738086624):
  nincs `dtolnay` sor; az új lépés: „it's active because: overridden by 'D:\a\mboxer\mboxer\rust-toolchain.toml'",
  „stable-x86_64-pc-windows-msvc updated - rustc 1.99.1 (from rustc 1.99.0)", `rustup show` active toolchain
  ugyanazzal az indokkal, `component list --installed`: `clippy-…`, `rustfmt-…`; fmt, clippy, test zöld. A cache
  kulcsa eltért a `main`-étől (nincs `CARGO_HOME`, `CARGO_INCREMENTAL`, `CARGO_TERM_COLOR` az env-ben) → cache miss,
  ~11 perc clippy + test.
- 2026-10-18 — CI run 39150020746 (commit 7cb471c, a visszaállított env): mind a négy job zöld, a test job env-jében
  `CARGO_INCREMENTAL: 0`, `CARGO_TERM_COLOR: always`; a cache kulcsa már csak a `CARGO_HOME` miatt tér el a
  `main`-étől (`bc559819` vs `e70cdc14`), a futás elmentette a sajátját.
- Release job: csak `v*` tagre fut, itt nem futott. A YAML-ból: ugyanaz a két `rustup toolchain install` + `rustup
  show` a checkout után, a workflow-szintű env ott is él; a `scripts/release.ps1` a `rust-toolchain.toml` által
  aktív toolchainnel fordít, a rustfmt/clippy komponens ott is felkerül (ártalmatlan).

## Closing checklist

- [x] every follow-up is listed in the pull request under `## Follow-ups` (the main session files it)
- [x] Verification written, dated
- [ ] `tasks_check` green, README index regenerated — a worktree-ből a check csak a régi README-indexre hibázik
  (a tool a session repóján kívül nem írja); a main session generálja újra a merge-nél
- [x] moved to `done/` with `closed: 2026-10-18`
