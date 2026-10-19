# mboxer

Viewer for large Gmail Takeout mbox archives on Windows 11 and macOS: Rust core (streaming parser + sidecar
full-text index), Tauri 2 UI (WebView2 / WKWebView), one portable exe or app, nothing to install.

## Layout
- `crates/mboxer-core` (streaming reader, message model, sidecar index), `crates/mboxer-app` (the `mboxer`
  exe: Tauri 2 window, static UI under `ui/`), `crates/mboxer-cli` (the `mboxer-cli` exe: scan / index / search)
- `docs/` (`usage.md` ships in the release zip; `images/` holds screenshots over the demo mbox only), `tasks/`,
  the community files at the root and under `.github/`; the full layout is in `docs/development.md`

## Commands
```powershell
cargo build --release
cargo run -p mboxer-app -- [file.mbox]   # the viewer
cargo test
```
Before a commit: the checks in CONTRIBUTING.md, "Checks before a commit" (more of them when `Cargo.toml`,
`Cargo.lock` or `.github/` change).

## Data outside the repo (never commit)
- Index sidecars written next to any mbox (`.mboxer/`); test fixtures are small synthetic samples only
  (`crates/mboxer-core/tests/fixtures/`)
- Any real message content: never paste it into task files, docs or commits

## Conventions
- Project conventions (language, names, docs to update) are in CONTRIBUTING.md; a new crate follows its
  "Dependencies: one exe" section (D-002): the one-line reason is a comment in `Cargo.toml`
- Git: branches, pull requests and commit rules follow @docs/git-workflow.md

## Tasks
- `tasks/` follows the project-pal task standard (`tasks/README.md`); `scripts/sync-issues.py` mirrors every
  task and decision file into a GitHub issue (D-008), so a task file reads as public text: it states decisions
  and results, not conversations; a decision stays `proposed` until the maintainer decides it
- The repo is public: a task file's `plan:` field holds the plan file's bare name (`plan: some-plan.md`), never
  its path, and no file carries a home folder path (`C:\Users\<name>\…`, `/Users/<name>/…`)
