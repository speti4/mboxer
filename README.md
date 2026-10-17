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
target\release\mboxer.exe index <file.mbox> [--force] [--threads N] [--quiet]
target\release\mboxer.exe search <file.mbox> [query words] [--from WORDS] [--to WORDS]
    [--after YYYY-MM-DD] [--before YYYY-MM-DD] [--label NAME]... [--has-attachment | --no-attachment]
    [--limit 20] [--offset N] [--count] [--relevance]
```

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the
first messages, then a summary (message count, throughput, largest message, labels, parse failures).

`index` builds the sidecar full-text index in `.mboxer\<file name>\` next to the mbox (a cache:
delete it any time) and skips the build when the index already matches the file. `search` needs
that index: query words are AND-ed, `"quotes"` make a phrase, accents and case are folded on
both sides (`szamla` finds `Számla`), the filters narrow the result, and every hit line carries
the message's byte offset and length in the mbox. Dates are UTC days; `--before` is exclusive.

Checks before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`. Peak memory of a run: `scripts\peak-memory.ps1 -- target\release\mboxer.exe scan <file.mbox> --quiet`.
The full benchmark (build time, peak memory, index size, re-open and search latency as a Markdown
report): `scripts\benchmark.ps1 -Mbox <file.mbox> -Out docs\benchmarks\<date>.md`; results for the
author's archive are under `docs/benchmarks/`.

## Layout

- `crates/mboxer-core` — streaming mbox reader (`MboxReader`, `read_at`), message summary and MIME
  tree over `mail-parser`, and the sidecar index (`index/`: fingerprint, schema with the accent-folding
  tokenizer, one-pass build, search); synthetic fixtures under `tests/fixtures/`.
- `crates/mboxer-cli` — the `mboxer` executable; its end-to-end tests under `tests/`.
- `scripts/` — helper scripts for measurements.
- `docs/` — origin, research, dated benchmarks.
