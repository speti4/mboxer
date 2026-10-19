# mboxer — development

How the repo is laid out, how CI and releases work, and what the helper scripts do. Build prerequisites, the
checks before a commit and the project rules are in [CONTRIBUTING.md](../CONTRIBUTING.md).

## Layout

- `crates/mboxer-core` — the streaming mbox reader (`MboxReader`, `read_at`), the message summary and MIME tree
  over `mail-parser`, and the sidecar index (`index/`: fingerprint, schema with the accent-folding tokenizer,
  one-pass build, search, the full sorted listing behind the viewer's list). Synthetic fixtures are under
  `tests/fixtures/`.
- `crates/mboxer-app` — the `mboxer` executable: the Tauri 2 window (`src/`: commands, state with the
  open-or-build flow, the message view, the label order, the macOS print windows), the static UI under `ui/`,
  the capability file and the icon.
- `crates/mboxer-cli` — the `mboxer-cli` executable, with its end-to-end tests under `tests/`.
- `scripts/` — helper scripts for measurements, the icon, the demo mbox, the Windows release zip, the macOS
  bundle, the changelog and the issue mirror.
- `docs/` — the usage guide that ships in the release zip, the full reference, this page, origin, research,
  dated benchmarks and the README screenshot.
- `tasks/` — the backlog, the work in flight, the history and the decisions (the task standard).
- `.github/workflows/` — CI (fmt, clippy, test, the dependency policy, the workflow lint, the commit rules), the
  tagged release and the issue sync; `.github/dependabot.yml` — the weekly dependency updates.
- `.githooks/` — the `commit-msg` hook over `scripts/check-commit.sh`, enabled per clone with
  `git config core.hooksPath .githooks` ([git-workflow.md](git-workflow.md)).
- `deny.toml` — the dependency policy `cargo deny` checks: licences, advisories, bans, sources (D-011);
  `about.toml` — the same licences for the third-party licence page of the two release zips.
- `CHANGELOG.md` — every release and the tasks it closed, generated from `tasks/done/` (see
  [The changelog](#the-changelog)).

## The UI

The UI is plain HTML, CSS and JavaScript embedded in the exe: no Node toolchain, no bundler, no Tauri CLI. A
change under `crates/mboxer-app/ui/` takes effect on the next `cargo run`. The pure modules (`query.js`,
`theme.js`, `files.js`, `print.js`) have Node tests under `crates/mboxer-app/tests/ui/`, which `cargo test`
runs through `tests/ui_query.rs` when `node` is on `PATH`. The application icon is generated once by
`scripts\make-icon.py` (Pillow) and committed: `icon.ico` for the Windows exe, `icon.png` for the window icon
on the other targets and the `icon.iconset/` PNGs that the macOS bundle turns into `icon.icns`.

Printing takes two paths. On Windows the print document goes into a hidden iframe whose `print()` opens the
WebView2 preview (task 011). WKWebView ignores `print()` on an iframe (tauri-apps/tauri #13451), so on macOS
`ui/print.js` inlines the cid images as `data:` URLs and the `print_message` command (`src/print.rs`, task 020)
opens a `print-<id>` window that loads the document from the `mboxer-print:` scheme, under a CSP that blocks
script and remote content, and calls `Webview::print()` once the page has loaded. The print window needs no
capability: it runs no script.

## CI and releases

`.github/workflows/ci.yml` runs five jobs on every push to `main` and every pull request: `test` (windows-latest:
`cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`), `lint` (ubuntu: `cargo fmt --check`,
`cargo clippy --target aarch64-apple-darwin --all-targets --locked -- -D warnings`, which compiles the macOS-only
code without a Mac, and `scripts/changelog.py --check`, skipped on a tag; see [The changelog](#the-changelog)),
`msrv` (ubuntu: `cargo check` for the macOS target with the toolchain that `rust-version` in `Cargo.toml` names,
so nothing newer than the declared minimum slips in), `deny` (`cargo deny check` against
`deny.toml` for the Windows and the two macOS targets: licences from the allowlist, no RustSec advisory or yanked
crate, crates.io as the only source, D-011; and `about.toml` accepts the same licences) and `workflow-lint`
(zizmor over the workflows and `dependabot.yml`; a finding accepted on purpose is listed with its reason in
`.github/zizmor.yml`, scoped to the lines it covers). The `test` job adds macos-latest only on a tag and on a
manual run: macOS runners are slower and scarcer, so the macOS tests run on tags and manual runs (task 042, set
while the repo was private). `.github/workflows/commit-rules.yml` runs `commit-rules` (`scripts/check-commit.sh`
over every pushed commit and the pull request text, the rules of [git-workflow.md](git-workflow.md)) on the same
pushes and pull requests. The workflows run with a read-only token by default; only the release job may write
contents and only the issue sync may write issues, and the sync runs only in `speti4/mboxer`, never in a fork.
Cargo runs with `--locked` in CI and in the release scripts, so a `Cargo.lock` out of step with the manifests
fails the build instead of being rewritten.

CodeQL runs as GitHub's default setup (Rust, JavaScript/TypeScript, Python, Actions), managed in the repository
settings, not as a workflow file. A ruleset protects `main`: changes arrive through a pull request with squash merge only,
the checks `test (windows-latest)`, `lint`, `msrv`, `deny`, `workflow-lint` and `commit-rules` must pass, the history
stays linear, and force pushes and deletion are blocked. Another ruleset keeps `v*` tags from being moved or
deleted.
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
`usage.html` and `THIRD-PARTY-LICENSES.html` into `dist\mboxer-<version>.zip`. `usage.html` is `docs\usage.md`
as one self-contained page (inline CSS, light and dark, no script), rendered by `scripts\render-usage-guide.py`
with the Python standard library, because Windows has no default app for `.md`; `bundle-macos.sh` packs the same
page. The renderer knows only the Markdown the guide uses and stops with the line number on anything else; the
lint job runs `python -I scripts/render-usage-guide.py --check` (its self-test, then the guide in memory). The version is the workspace
`Cargo.toml` one, the only place it is written: `tauri-build` stamps it into the exe's properties and the status
bar shows it. Next to the zip it writes `SHA256SUMS.txt` (the zip's SHA-256 in the `sha256sum -c` format) and
`release-notes-<version>.md`. The licence page comes from
[cargo-about](https://github.com/EmbarkStudios/cargo-about) over the crates compiled into the exe (it reads the
graph through `cargo metadata`, which wants the sources of every platform's crates, so the script runs `cargo
fetch` first and cargo-about offline after it): `about.toml`
accepts the same licences as `deny.toml` (a licence added to one is added to the other; the `deny` job compares
the two), `scripts/third-party-licenses.hbs` is the page, and every crate links to its source, which the MPL-2.0
crates ask for (D-011). cargo-about is a build-time tool, never part of the exe:

```powershell
cargo install cargo-about --version 0.9.2 --locked --features cli   # once; the version ci.yml pins
scripts\release.ps1                     # dist\: the zip, SHA256SUMS.txt, release-notes-<version>.md
```

Pushing a tag `v<version>` runs the four jobs, `test` on both platforms, and `licences-macos`, which renders the
licence page of the macOS zip on Linux; when they pass, two more: `bundle-macos` builds the macOS zip on a macOS
runner (see [The macOS bundle](#the-macos-bundle)) and hands it over as a one-day artifact, and the `release`
job checks that the tag is on `main`, installs the pinned cargo-about, runs `release.ps1`, verifies the macOS zip
against its checksum line, appends that line to `SHA256SUMS.txt`, attests the build provenance of both zips and
publishes them with the checksum file as a GitHub release titled `mboxer <version>`, with the notes file as its
text, through the runner's `gh` CLI. Both release jobs build without the Rust cache, so nothing a pull request run
cached reaches a zip. A downloaded zip is checked against its attestation with `gh attestation verify <zip> --repo
speti4/mboxer`.

### The macOS bundle

`scripts/bundle-macos.sh` is the macOS counterpart of `release.ps1` and runs on a Mac or on the `macos-latest`
runner (bash, the Xcode command line tools: `lipo`, `iconutil`, `codesign`, `plutil`, `ditto`). It builds the
release profile for `aarch64-apple-darwin` and `x86_64-apple-darwin`, joins the two binaries with `lipo` into
one universal binary and lays out `mboxer.app`: `Contents/MacOS/mboxer`, `Contents/Info.plist` (the
`io.github.speti4.mboxer` identifier, the `Cargo.toml` version as `CFBundleShortVersionString` and
`CFBundleVersion`, the `.mbox` document type, `LSMinimumSystemVersion` 10.13 as Tauri 2 defaults it) and
`Contents/Resources/icon.icns`, which `iconutil` makes from the committed `icons/icon.iconset/`. The bundle is
ad-hoc signed (`codesign -s -`) and not notarized (D-007): Apple Silicon loads it, Gatekeeper asks the user once.
`ditto` packs `mboxer.app`, `usage.html` (the rendered guide, as in the Windows zip) and
`THIRD-PARTY-LICENSES.html` into `dist/mboxer-<version>-macos.zip`, the way Finder's *Compress* does, and
`SHA256SUMS.txt` next to it holds the zip's line in the `shasum -a 256 -c` format. No Tauri CLI is involved (D-002); the bundle is the script.

The licence page is cargo-about with the `about.toml` and the template of `release.ps1`, over the crates of the
two darwin builds instead of the Windows one (`--target aarch64-apple-darwin --target x86_64-apple-darwin`), so
it lists `objc2` and the other macOS-only crates and none of the `windows-*` ones. A target's dependency graph
and the licence texts do not depend on the host, so in CI the `licences-macos` job renders the page on Linux
(`--licences-only`, without a build), in parallel with the checks, and `bundle-macos` packs that page
(`--licences`) instead of building cargo-about on the slower macOS runner. On a Mac without `--licences` the script renders the page itself, with cargo-about on `PATH`.

Without a local Mac, the script is verified by the CI job: a manual run of the workflow on a branch (`gh
workflow run ci.yml --ref <branch>`) runs the macOS tests, `licences-macos` and `bundle-macos` without a tag and
keeps the zip as the `macos-zip` artifact for a day, which is also how a build is obtained for a manual test on a
Mac. A pull request that touches macOS code gets such a run before its merge, because its own
runs test on Windows only. A change to the script is linted here with `bash -n scripts/bundle-macos.sh` and
shellcheck (`uvx --from shellcheck-py shellcheck -s bash scripts/bundle-macos.sh`).

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin   # once
cargo install cargo-about --version 0.9.2 --locked --features cli   # once
scripts/bundle-macos.sh                 # dist/: the zip and SHA256SUMS.txt
scripts/bundle-macos.sh --skip-build    # pack what target/<target>/release already holds
scripts/bundle-macos.sh --licences-only page.html   # the macOS licence page alone, on any OS (Git Bash too)
```

On Windows the darwin targets compile as far as `cargo check` goes, which catches a `cfg(windows)` gap before
CI does. The one build script that needs a C compiler for the darwin target (`objc2-exception-helper`, a
helper only the link step uses) is skipped with the `DOCS_RS` variable it honours, so no Apple SDK is needed:

```powershell
rustup target add aarch64-apple-darwin x86_64-apple-darwin   # once
$env:DOCS_RS = "1"; cargo check --target aarch64-apple-darwin -p mboxer-app -p mboxer-cli
```

### The changelog

`CHANGELOG.md` is generated from the task files by `scripts/changelog.py`: every file under `tasks/done/` is a
line (its H1, linked to the file) under its milestone, in the section of the first release tag whose tree holds
it under `tasks/done/`, or under *Unreleased* when no tag holds it yet. The tag trees decide rather than the
`closed` date, because tasks close both before and after a tag on the same day. The summary under a heading is
written by hand and kept by every run; the lists are never edited by hand. A task that changed the project rather
than the app (the repo, CI, the process) carries `changelog: housekeeping` in its frontmatter and is listed last,
under *Project housekeeping*. The release notes are one section of the changelog without that group (one line
counts it and links the changelog), with a footer on the checksum, the attestation and the licence page. A
release goes like this:

1. In a pull request: bump the version in `Cargo.toml`, write the summary under *Unreleased* in
   `CHANGELOG.md`, run `python -I scripts/changelog.py` and merge.
2. Tag the merge commit `v<version>` and push the tag; the release job takes the notes from the changelog (the
   new tag takes the *Unreleased* summary).
3. Afterwards, `python -I scripts/changelog.py` stamps the section with the version and the tag's date; commit
   it with the next change. Until then `--check` reads the newest tag without a section as not there yet and
   prints a notice, so the pull requests opened in between, dependabot's included, stay green.

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
idempotent and one-way (D-008); a release on the Releases page closes the milestones it ships. A dry run shows
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

Measured results are under `docs/benchmarks/`.

## The demo mbox

`scripts/make-demo-mbox.py` writes `target/demo/demo.mbox`: 50 made-up messages with Gmail labels,
conversations, an HTML newsletter, attachments and Hungarian accents, all under `example.*` domains. It is
deterministic and uses the standard library only. The screenshot in the README (`docs/images/viewer.png`) is
taken over it; never over a real archive.

```powershell
python -I scripts/make-demo-mbox.py
cargo run -p mboxer-app -- target\demo\demo.mbox
```
