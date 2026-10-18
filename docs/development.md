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
- `scripts/` — helper scripts for measurements, the icon, the demo mbox, the release zip and the issue mirror.
- `docs/` — the usage guide that ships in the release zip, the full reference, this page, origin, research,
  dated benchmarks and the README screenshot.
- `tasks/` — the backlog, the work in flight, the history and the decisions (the task standard).
- `.github/workflows/` — CI (fmt, clippy, test), the tagged release and the issue sync.

## The UI

The UI is plain HTML, CSS and JavaScript embedded in the exe: no Node toolchain, no bundler, no Tauri CLI. A
change under `crates/mboxer-app/ui/` takes effect on the next `cargo run`. The pure modules (`query.js`,
`theme.js`, `files.js`) have Node tests under `crates/mboxer-app/tests/ui/`, which `cargo test` runs through
`tests/ui_query.rs` when `node` is on `PATH`. The application icon is generated once by `scripts\make-icon.py`
(Pillow) and committed.

## CI and releases

`.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and
`cargo test` on every push and pull request (windows-latest).

`scripts\release.ps1` builds the release profile (fat LTO, stripped) and packs `target\release\mboxer.exe` and
`docs\usage.md` into `dist\mboxer-<version>.zip`. The version is the workspace `Cargo.toml` one, the only place
it is written: `tauri-build` stamps it into the exe's properties and the status bar shows it. Pushing a tag
`v<version>` runs the tests and publishes that zip as a GitHub release.

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
