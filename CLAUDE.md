# mboxer

Windows 11 viewer for large Gmail Takeout mbox archives: Rust core (streaming parser + sidecar
full-text index), Tauri 2 / WebView2 UI, one portable exe, nothing for the user to install.

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
- The author's mbox: a 14 GB Gmail Takeout export on a local drive (the path stays out of the repo)
- Index sidecars written next to any mbox (`*.idx`, `.mboxer/`); test fixtures are small synthetic samples only
  (`crates/mboxer-core/tests/fixtures/`)
- Any real message content: never paste it into task files, docs or commits

## Conventions
- Project conventions (language, names, docs to update) are in CONTRIBUTING.md; a new crate follows its
  "Dependencies: one exe" section (D-002): the one-line reason is a comment in `Cargo.toml`
- Git: branches, pull requests, commit rules and parallel sessions follow @docs/git-workflow.md

## Tasks
- `tasks/` follows the project-pal task standard (`tasks/README.md`); a task file's H1 and Context paragraph are
  English, because `scripts/sync-issues.py` mirrors them into GitHub issues (D-008), Log / Verification may be
  Hungarian; a decision stays `proposed` until the user confirms it
- The repo is public: a task file's `plan:` field holds the plan file's bare name (`plan: some-plan.md`), never
  its path, and no file carries a home folder path (`C:\Users\<name>\…`, `/Users/<name>/…`)
