# mboxer

A Windows 11 viewer for large Gmail Takeout mbox archives (14–50 GB): fast indexed search,
stable, usable UI. Replaces [eneam/mboxviewer](https://github.com/eneam/mboxviewer) for the author
and a handful of colleagues, each with their own mbox file.

- Stack: Rust, Tauri 2 on WebView2, one-time sidecar full-text index next to the mbox (see
  `tasks/decisions/`).
- Status and backlog: `tasks/README.md` (generated), milestones in `tasks/roadmap.md`.
- Why this exists: `docs/origin.md`; why build and not buy: `docs/research/`.

## Build

Prerequisites on Windows 11: Rust stable via rustup (the `x86_64-pc-windows-msvc` target) and the
Visual Studio 2022 C++ build tools plus a Windows 11 SDK, which the MSVC linker needs
(`winget install Rustlang.Rustup`; in the Visual Studio Installer add `MSVC v143 build tools` and
`Windows 11 SDK`).

```powershell
cargo build --release
target\release\mboxer.exe scan <file.mbox> [--sample 3] [--limit N] [--quiet]
```

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the
first messages, then a summary (message count, throughput, largest message, labels, parse failures).

Checks before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`. Peak memory of a run: `scripts\peak-memory.ps1 -- target\release\mboxer.exe scan <file.mbox> --quiet`.

## Layout

- `crates/mboxer-core` — streaming mbox reader (`MboxReader`, `read_at`), message summary and MIME
  tree over `mail-parser`; synthetic fixtures under `tests/fixtures/`.
- `crates/mboxer-cli` — the `mboxer` executable.
- `scripts/` — helper scripts for measurements.
