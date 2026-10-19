---
size: S
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-20
plan: none
milestone: M6 Hardening
changelog: housekeeping
---

# 061 — Small hygiene follow-ups: toolchain pin, ignores, arguments, cancel, logs, duplicates

## Context

What: `rust-toolchain.toml` says `stable` without a version, so a new Rust release with new clippy lints turns CI
red without a code change; `.gitignore` lacks `*.pdf` (a *Save as PDF* of real mail) and editor and anonymizer
sidecars; `initial_paths` in `crates/mboxer-app/src/commands.rs` uses `std::env::args()`, which panics on a
non-Unicode argument; `cancel.store(false)` runs per file, so a Cancel between two files of a batch is lost; the
`eprintln!` timing logs go; the `>From` unescaping exists in both `reader.rs` and `view.rs`; the virtual list's
spacer caps the listing near 1.2 million rows; `benchmark.ps1` needs `#Requires -Version 7` or a BOM; the
`zizmor.yml` ignores are by line number; the toolchain install step is copied five times in `ci.yml` and the
cargo-about version is pinned in five places; a scheduled `cargo deny` run would catch a new advisory; the
backend's query warnings fields are always empty since task 050.
Why: none of these blocks a user today; together they are a day of friction for the next contributor.
Done when: each point is fixed or dropped with a one-line reason in the Log.
Decided 2026-10-19: the toolchain is pinned to the current stable version with a comment that dependabot does not
bump it, so the bump is a conscious `chore:`; the warnings fields are removed, not given a use.

## Steps

1. [x] `rust-toolchain.toml`: the current stable version pinned (`rustc --version`), the comment says why;
   `docs/development.md` names the bump as a `chore:`
2. [x] `.gitignore`: `*.pdf`, the common editor folders, `.anonilm-map.json`
3. [x] `commands.rs`: `initial_paths` through `args_os` with a lossy conversion; `cancel.store(false)` once per
   batch, so a Cancel between two files stops the next; the `eprintln!` timing logs removed
4. [x] the `>From` unescaping in one place (`mboxer-core`), `view.rs` calls it
5. [x] the virtual list: the spacer height capped at the browser limit with a measured note, or the row count
   above 1.2 million paged; the choice in the Log (a test only where the UI module can be driven under Node)
6. [x] `benchmark.ps1`: `#Requires -Version 7`; `zizmor.yml` ignores by rule and job instead of line number if
   zizmor supports it, else a comment naming the lines; the toolchain install step as one reusable step or
   composite action in `ci.yml`; the cargo-about version in one place the others read
7. [x] a scheduled `cargo deny check advisories` job (weekly, `schedule:` + `workflow_dispatch`), not a required
   check
8. [x] the empty warnings fields (`SearchResult::warnings`, `Listing::warnings`, the app's `warnings` and the UI's
   status-bar text for them) removed
9. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check`,
   `uvx zizmor .github`; CONTRIBUTING and `docs/development.md` follow where a command or a job changed

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-20 — step 1, fixed: `rust-toolchain.toml` pins `1.99.1` (the installed stable, `rustc 1.99.1
  (58cf8f9c3 2026-10-13)`), its comment says why and that dependabot does not watch the file; `rust-version =
  "1.90"` in `Cargo.toml` stays. `docs/development.md` has a new section, "The Rust toolchain", with the bump as
  a `chore:` pull request; CONTRIBUTING says rustup installs the pinned version on the first `cargo` call.
- 2026-10-20 — step 2, fixed: `.gitignore` adds `*.pdf` (with the mail rules), `.vscode/`, `.idea/` and
  `.anonilm-map.json` (with the local files); no tracked file matches them.
- 2026-10-20 — step 3, fixed: `initial_paths` reads `std::env::args_os()` with `to_string_lossy`, so a
  non-Unicode argument becomes a path that fails to open instead of a panic. The cancel flag is cleared once at
  the start of `open_mbox_inner` and read before each file; a set flag ends the batch with the `Cancelled` event.
  Before, a Cancel after one file had loaded was cleared by the next file's reset. The four `eprintln!` lines of
  the open and the query are gone (`print.rs` keeps its error line).
- 2026-10-20 — step 4, fixed: `unquote_from_lines(&[u8]) -> Vec<u8>` in `crates/mboxer-core/src/envelope.rs`
  (exported from the crate root) is the one implementation; `eml_bytes` in `reader.rs` calls it (a one-line body,
  so the parallel core work in `reader.rs` meets a small diff), and the app's `unescape_from_lines` wraps it,
  converting back with `from_utf8_lossy`, which never replaces anything because only an ASCII `>` comes off.
  The new core test and the two existing tests (`eml_bytes_undo_the_mboxrd_quoting_only`,
  `mboxrd_quoting_comes_off_for_display_only`) pass.
- 2026-10-20 — step 5, decided: the limit is documented, the code stays. Measured in headless Edge 155 (the
  engine of WebView2): a spacer asked for 56 000 000 px is laid out at 33 554 428 px, the list's largest
  `scrollTop` is 33 553 828, and the last row that scrolls into view is row index 1 198 371, so a listing scrolls
  to 1 198 372 rows. WebKit uses the same 1/64-px layout unit (not measured, no Mac here). Reason: a listing that
  long is a single archive of more than 1.2 million messages, or as many conversations, without a label or a
  search; the scaled scroll position that would lift the limit cannot be driven by a Node test (it lives in
  `main.js`'s DOM code) or by a manual round on any archive at hand. `docs/reference.md` names the limit and the
  way around it (a label or a search).
- 2026-10-20 — step 6, fixed. `benchmark.ps1` says `#Requires -Version 7` after its help block (placed before
  it, the comment-based help was no longer found by `Get-Help`); Windows PowerShell 5.1 refuses to run it.
  zizmor 1.31.1 ignores by file, line and column only (its `rules.<id>.ignore`), not by job; an inline
  `# zizmor: ignore[cache-poisoning]` on a new `name:` line of each cache step covers the step it stands on
  ("anywhere in any span identified by a finding"), moves with it and keeps off the `uses:` line whose version
  comment dependabot rewrites, so `.github/zizmor.yml` is removed. The five toolchain install steps are one
  composite action, `.github/actions/install-rust-toolchain/action.yml`, with a `targets` input passed through
  `env`; the second, by-name install of the old step is dropped, because an exact version cannot be stood in for
  by an older preinstalled stable. zizmor's `self-repository` audit (new in 1.30) asks for the self-repository
  form, so the jobs use `uses: $/.github/actions/install-rust-toolchain`; the zizmor-action pin runs the same
  zizmor 1.31.1. The cargo-about version lives in `scripts/cargo-about-version.txt`: the two CI install steps
  read it (bash `$(cat …)`, pwsh `(Get-Content … -Raw).Trim()`), `release.ps1` and `bundle-macos.sh` stop with
  the install command when the cargo-about found is another version, and `docs/development.md` reads it in its
  two install commands; the workflow-level `CARGO_ABOUT_VERSION` is gone.
- 2026-10-20 — step 7, fixed: `.github/workflows/advisories.yml` runs `cargo deny check advisories` with the
  pinned `cargo-deny-action` every Monday at 06:17 UTC and on `workflow_dispatch`, read-only token, not in the
  ruleset; `docs/development.md` describes it.
- 2026-10-20 — step 8, fixed: `SearchResult::warnings`, `Listing::warnings()` and its field, `ListInfo.warnings`,
  the CLI's `warning:` loop (it never printed) and the UI's backend branch (`state.warnings`) are removed; the
  status bar's `showWarnings()` shows the field's own `query.js` warnings as before. The five test assertions on
  the empty lists went with the fields.
- 2026-10-20 — step 9: the checks below pass; CONTRIBUTING (the toolchain line, the darwin target once per pinned
  version) and `docs/development.md` (layout, CI, the toolchain, cargo-about, the advisories run) follow.
- 2026-10-20 — follow-ups, in the pull request description: a scaled scroll position for listings above
  1 198 372 rows, if a report shows one; a test of the open batch's cancel handling, which needs the loop out of
  the Tauri command.

## Deviations from the plan

- `.github/zizmor.yml` is removed rather than rewritten: the inline ignore on each cache step's name line scopes
  the acceptance to the step, which the file cannot do.
- The cargo-about version file is checked by the two release scripts, not only read by CI; a page rendered by
  hand with another cargo-about would differ from the released one.
- `docs/development.md` gets two subsection headings, "The Rust toolchain" and "The release", so the release
  text does not fall under the toolchain.

## Verification

- 2026-10-20, Windows 11, in the task worktree rebased on `main` with tasks 060 and 063, toolchain 1.99.1 from the
  pin: `cargo fmt --all --check` clean; `cargo clippy --all-targets --locked -- -D warnings` clean; `DOCS_RS=1
  cargo clippy --target aarch64-apple-darwin --all-targets --locked -- -D warnings` clean; `DOCS_RS=1 cargo +1.90
  check --target aarch64-apple-darwin --all-targets --locked` clean; `cargo test --locked` green (mboxer-app 52
  unit + 1 Node runner; mboxer-cli 4 + 7; mboxer-core 43 unit, 10 fixtures, 25 index, 6 index_build, 8
  index_quality, 1 tantivy_excluded_phrase); `node --test` over `crates/mboxer-app/tests/ui/` 53 passed; `node
  --check crates/mboxer-app/ui/main.js` clean.
- 2026-10-20: `cargo deny check` (advisories, bans, licenses, sources ok) and `cargo deny check advisories` ok;
  `uvx zizmor .github` (1.31.1) "No findings to report" with 2 ignored (the two cache steps) and 12 suppressed;
  `python -I scripts/render-usage-guide.py --check` passes; `python -I scripts/changelog.py --check` passes after
  the regeneration in the close commit.
- 2026-10-20: `bash -n` and shellcheck clean on `scripts/bundle-macos.sh`; `scripts/bundle-macos.sh
  --licences-only <scratch file>` in Git Bash rendered the macOS licence page (268 KB) through the new version
  check; `release.ps1`'s check, run alone, passes with cargo-about 0.9.2 and stops with "cargo-about 0.9.2 found,
  0.9.1 wanted: cargo install …" when the wanted version is changed; pwsh passes `(Get-Content
  scripts/cargo-about-version.txt -Raw).Trim()` as the one argument `0.9.2`. `Get-Help scripts/benchmark.ps1`
  shows its synopsis; `powershell.exe` 5.1 refuses the script with the `#requires` message.
- Not run here: the CI jobs, which are the pull request's checks (the composite action and the self-repository
  syntax on GitHub's runners); a manual Cancel between two files of a batch; the scheduled advisories run, which
  only runs from `main`.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
