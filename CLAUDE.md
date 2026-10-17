# mboxer

Windows 11 viewer for large Gmail Takeout mbox archives: Rust core (streaming parser + sidecar
full-text index), Tauri 2 / WebView2 UI, one portable exe, nothing for the user to install.

## Scope
This repo is the source of truth for:
- the Rust workspace: `crates/mboxer-core` (streaming reader, message model, sidecar index),
  `crates/mboxer-app` (the `mboxer` exe: Tauri 2 window, static UI under `ui/`), `crates/mboxer-cli`
  (the `mboxer-cli` exe for scan / index / search). Build: `cargo build --release`, run the viewer:
  `cargo run -p mboxer-app -- [file.mbox]`, tests: `cargo test`; fixtures are synthetic files under
  `crates/mboxer-core/tests/fixtures/`
- `docs/` — origin, research, benchmarks, usage
- `tasks/` — the task standard: backlog, work in flight, history, decisions

## Data outside the repo (never commit)
- The author's mbox: `<archive path>` (14 GB)
- Index sidecars written next to any mbox (`*.idx`, `.mboxer/`); test fixtures are small synthetic samples only
- Any real message content: never paste it into task files, docs or commits

## Conventions
- Code, commits, docs, UI strings: English; names (files, folders, branches, crates) kebab-case
- Rust edition 2024, `cargo fmt` + `cargo clippy -D warnings` clean before a commit
- Runtime dependency rule (D-02): the shipped artifact is one exe; a new build-time crate needs a one-line justification in the task file
- Decisions with lasting effect go to `tasks/decisions/` (`/tasks decide`), proposed until the user confirms
- User communication: Hungarian, technical terms in English

## Tasks

- Tasks follow the project-pal task standard in `tasks/` (`backlog/ wip/ done/ dropped/ decisions/`, one `NNN-slug.md` per item, frontmatter, the index in `tasks/README.md` is generated): approved plan → `/tasks promote` (or `/tasks add` for later), closing files every follow-up with `/tasks add`, `/tasks check` before a task commit.
