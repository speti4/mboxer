# mboxer — development

How the repo is laid out, how CI and releases work, and what the helper scripts do. Build prerequisites, the
checks before a commit and the project rules are in [CONTRIBUTING.md](../CONTRIBUTING.md).

## Layout

- `crates/mboxer-core` — the streaming mbox reader (`MboxReader`, `read_at`), the message summary and MIME tree
  over `mail-parser`, and the sidecar index (`index/`: fingerprint, schema with the accent-folding tokenizer,
  one-pass build, search, the full sorted listing behind the viewer's list). Synthetic fixtures are under
  `tests/fixtures/`.
- `crates/mboxer-app` — the `mboxer` executable: the Tauri 2 window (`src/`: commands, state with the
  open-or-build flow, the message view, the label order), the static UI under `ui/`, the capability file and
  the icon.
- `crates/mboxer-cli` — the `mboxer-cli` executable, with its end-to-end tests under `tests/`.
- `scripts/` — helper scripts for measurements, the icon, the demo mbox, the release zip, the changelog and the
  issue mirror.
- `docs/` — the usage guide that ships in the release zip, the full reference, this page, origin, research,
  dated benchmarks and the README screenshot.
- `tasks/` — the backlog, the work in flight, the history and the decisions (the task standard).
- `.github/workflows/` — CI (fmt, clippy, test, the dependency policy, the workflow lint, the commit rules), the
  tagged release and the issue sync; `.github/dependabot.yml` — the weekly dependency updates.
- `.githooks/` — the `commit-msg` hook over `scripts/check-commit.sh`, enabled per clone with
  `git config core.hooksPath .githooks` ([git-workflow.md](git-workflow.md)).
- `deny.toml` — the dependency policy `cargo deny` checks: licences, advisories, bans, sources (D-011);
  `about.toml` — the same licences for the third-party licence page of the release zip.
- `CHANGELOG.md` — every release and the tasks it closed, generated from `tasks/done/` (see
  [The changelog](#the-changelog)).

## The UI

The UI is plain HTML, CSS and JavaScript embedded in the exe: no Node toolchain, no bundler, no Tauri CLI. A
change under `crates/mboxer-app/ui/` takes effect on the next `cargo run`. The pure modules (`query.js`,
`theme.js`, `files.js`) have Node tests under `crates/mboxer-app/tests/ui/`, which `cargo test` runs through
`tests/ui_query.rs` when `node` is on `PATH`. The application icon is generated once by `scripts\make-icon.py`
(Pillow) and committed.

## CI and releases

`.github/workflows/ci.yml` runs four jobs on every push and pull request: `test` (windows-latest: `cargo fmt
--check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`), `deny` (`cargo deny check` against
`deny.toml`: licences from the allowlist, no RustSec advisory or yanked crate, crates.io as the only source,
D-011; and `about.toml` accepts the same licences), `workflow-lint` (zizmor over the workflows and `dependabot.yml`; a finding accepted on purpose is
listed with its reason in `.github/zizmor.yml`) and `commit-rules` (`scripts/check-commit.sh` over every pushed
commit and the pull request text, the rules of [git-workflow.md](git-workflow.md)). The workflows run with a
read-only token by default; only the release job may write contents and only the issue sync may write issues.
The Rust toolchain comes from the runner's own rustup, which installs the channel and the components that
`rust-toolchain.toml` names, so no action is involved.

Every action is pinned by commit SHA with the version in a comment. Dependabot proposes the updates for the
actions and for `Cargo.lock` once a week, a release only after it is a week old; a pin is changed through such a
pull request, never by retagging by hand. Before a change to the dependencies or the workflows, the same
checks run locally:

```powershell
cargo install cargo-deny --locked   # once
cargo deny check
uvx zizmor .github
```

`scripts\release.ps1` builds the release profile (fat LTO, stripped) and packs `target\release\mboxer.exe`,
`docs\usage.md` and `THIRD-PARTY-LICENSES.html` into `dist\mboxer-<version>.zip`. The version is the workspace
`Cargo.toml` one, the only place it is written: `tauri-build` stamps it into the exe's properties and the status
bar shows it. Next to the zip it writes `SHA256SUMS.txt` (the zip's SHA-256 in the `sha256sum -c` format) and
`release-notes-<version>.md`. The licence page comes from
[cargo-about](https://github.com/EmbarkStudios/cargo-about) over the crates compiled into the exe: `about.toml`
accepts the same licences as `deny.toml` (a licence added to one is added to the other; the `deny` job compares
the two), `scripts/third-party-licenses.hbs` is the page, and every crate links to its source, which the MPL-2.0
crates ask for (D-011). cargo-about is a build-time tool, never part of the exe:

```powershell
cargo install cargo-about --version 0.9.2 --locked --features cli   # once; the version the release job pins
scripts\release.ps1                     # dist\: the zip, SHA256SUMS.txt, release-notes-<version>.md
```

Pushing a tag `v<version>` runs the three jobs and, when they pass, the release job installs the pinned
cargo-about, runs the script and publishes the zip and `SHA256SUMS.txt` as a GitHub release titled
`mboxer <version>`, with the notes file as its text, through the runner's `gh` CLI. The release job builds
without the Rust cache, so nothing a pull request run cached reaches the zip.

### The changelog

`CHANGELOG.md` is generated from the task files by `scripts/changelog.py`: every file under `tasks/done/` is a
line (its H1, linked to the file) under its milestone, in the section of the first release tag whose tree holds
it under `tasks/done/`, or under *Unreleased* when no tag holds it yet. The tag trees decide rather than the
`closed` date, because tasks close both before and after a tag on the same day. The summary under a heading is
written by hand and kept by every run; the lists are never edited by hand. The release notes are one section of
it, with a footer on the checksum and the licence page. A release goes like this:

1. In a pull request: bump the version in `Cargo.toml`, write the summary under *Unreleased* in
   `CHANGELOG.md`, run `python -I scripts/changelog.py` and merge.
2. Tag the merge commit `v<version>` and push the tag; the release job takes the notes from the changelog (the
   new tag takes the *Unreleased* summary).
3. Afterwards, `python -I scripts/changelog.py` stamps the section with the version and the tag's date; commit
   it with the next change.

```powershell
python -I scripts/changelog.py                 # rewrite CHANGELOG.md
python -I scripts/changelog.py --check         # exit 1 when it is out of date
python -I scripts/changelog.py --notes 0.2.0   # one release's notes (a version without a tag: Unreleased)
```

Between releases the committed *Unreleased* list may lag behind `tasks/done/`: closing a task does not touch
the changelog, so parallel task branches do not conflict over it.

## The issue mirror

`.github/workflows/sync-issues.yml` runs `scripts/sync-issues.py` on every push to `main` that touches `tasks/`.
It keeps one GitHub issue per task and decision file (the id in the title, the Context prose, labels, the
milestone, open or closed by the folder) and the milestones from the `tasks/roadmap.md` headings. It is
idempotent and one-way (D-08); a release on the Releases page closes the milestones it ships. A dry run shows
what a run would change:

```powershell
python -I scripts/sync-issues.py --dry-run
```

## Measurements

Peak memory of a run:

```powershell
scripts\peak-memory.ps1 -- target\release\mboxer-cli.exe scan <file.mbox> --quiet
```

The full benchmark (build time, peak memory, index size, re-open and search latency) as a Markdown report:

```powershell
scripts\benchmark.ps1 -Mbox <file.mbox> -Out docs\benchmarks\<date>.md
```

The results for the author's archive are under `docs/benchmarks/`.

## The demo mbox

`scripts/make-demo-mbox.py` writes `target/demo/demo.mbox`: 50 made-up messages with Gmail labels,
conversations, an HTML newsletter, attachments and Hungarian accents, all under `example.*` domains. It is
deterministic and uses the standard library only. The screenshot in the README (`docs/images/viewer.png`) is
taken over it; never over a real archive.

```powershell
python -I scripts/make-demo-mbox.py
cargo run -p mboxer-app -- target\demo\demo.mbox
```
