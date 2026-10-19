# Contributing to mboxer

Thank you for taking the time. mboxer is a small project with a few firm rules: mail stays private, the
shipped artifact stays one exe, and the task files are the plan. Everything below follows from these.

## Start with an issue

Open an issue before a pull request: a [bug report](https://github.com/speti4/mboxer/issues/new?template=bug-report.yml)
or a [feature request](https://github.com/speti4/mboxer/issues/new?template=feature-request.yml). A short
exchange about the approach saves a rewrite. Security problems go to a private report instead
([SECURITY.md](SECURITY.md)).

## No real mail, anywhere

People run mboxer on their own mail. Real message content, addresses and names never go into an issue, a
pull request, a commit, a test fixture or a screenshot. Reproduce a problem with a made-up message: a few
lines of mbox with `example.com` addresses. The fixtures under `crates/mboxer-core/tests/fixtures/` are all
synthetic and show the shapes that matter (Gmail labels and threads, encoded headers, attachments,
Hungarian charsets, CRLF). `.gitignore` keeps `*.mbox` files and `.mboxer/` index folders out of the repo,
with the fixtures as the only exception.

## Build prerequisites

On Windows 11 you need:

- Rust stable via rustup, with the `x86_64-pc-windows-msvc` target. `rust-toolchain.toml` pins the channel.
- The Visual Studio 2022 C++ build tools and a Windows 11 SDK, which the MSVC linker needs.
- Node.js (optional). `cargo test` runs the UI's Node tests when `node` is on `PATH` and skips them
  otherwise.

```powershell
winget install Rustlang.Rustup
# Visual Studio Installer: add "MSVC v143 build tools" and "Windows 11 SDK"
cargo build --release
cargo run -p mboxer-app -- [file.mbox]      # the viewer, during development
```

The UI under `crates/mboxer-app/ui/` is plain HTML, CSS and JavaScript embedded in the exe, with no bundler
and no Tauri CLI. A change there takes effect on the next `cargo run`. The layout, the CLI, the release and
the helper scripts are described in [docs/development.md](docs/development.md).

## Branches, pull requests and commits

Work on a branch and open a pull request; it is squash-merged once CI is green, so its title and description
become the commit message. The commit rules (no `Claude-Session:` trailer, no Claude Code session link) and how
the maintainer's parallel sessions work are in [docs/git-workflow.md](docs/git-workflow.md). Enable the hook
that checks them once per clone:

```powershell
git config core.hooksPath .githooks
```

## Checks before a commit

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

A change to `Cargo.toml`, `Cargo.lock` or `.github/` also runs `cargo deny check` and `uvx zizmor .github`
(see [docs/development.md](docs/development.md#ci-and-releases)). CI (`.github/workflows/ci.yml`) runs all of
them on every push and pull request that changes code, the macOS tests on a release tag or a manual run. A
change in behaviour comes with a test: Rust tests sit beside the code or under each crate's `tests/`, and the UI
modules have Node tests under `crates/mboxer-app/tests/ui/`.

## Dependencies: one exe

The shipped artifact is one portable exe, with nothing for the user to install (decision D-02 in
`tasks/decisions/`). WebView2 is part of Windows 11, everything else is compiled in. A new crate is fine when
it earns its place. Add it to `[workspace.dependencies]` in `Cargo.toml` with a one-line comment saying why
it is needed and why the existing ones do not cover it. Crates that compile C code or pull a runtime
download need a strong reason.
The crate's licence has to be on the allowlist in `deny.toml` and its source has to be crates.io (D-011);
`cargo deny check` says so before CI does.

## Tasks, decisions and the mirrored issues

The plan lives in `tasks/`: one file per item under `backlog/`, `wip/`, `done/` and `dropped/`, decisions
under `decisions/`, milestones in `tasks/roadmap.md`, and a generated index in `tasks/README.md`. These
files are mirrored one way into GitHub issues and milestones by `scripts/sync-issues.py` (decision D-08).
The issues labelled `mirrored` are generated: comment on them, but do not edit them, because the next sync
overwrites the change. The maintainer turns an accepted issue into a task file.

## Conventions

- Rust edition 2024. `cargo fmt` and `clippy -D warnings` are clean.
- Code, comments, commit messages, docs and UI strings are in English.
- File, folder, branch and crate names are kebab-case and descriptive, without ambiguous abbreviations.
- User-facing changes update `docs/usage.md` (it ships in the release zip) and `docs/reference.md`.
- Commit subjects are imperative and start with a type: `feat:`, `fix:`, `docs:`, `ci:`, `chore:`, `tasks:`,
  `deps:` (dependabot).

## Licence

mboxer is MIT-licensed ([LICENSE](LICENSE)). By contributing, you agree that your contribution is licensed
under the same terms.
