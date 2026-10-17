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
target\release\mboxer.exe [file.mbox]                  # the viewer (Tauri 2 / WebView2)
target\release\mboxer-cli.exe scan <file.mbox> [--sample 3] [--limit N] [--quiet]
target\release\mboxer-cli.exe index <file.mbox> [--force] [--threads N] [--quiet]
target\release\mboxer-cli.exe search <file.mbox> [query words] [--from WORDS] [--to WORDS]
    [--after YYYY-MM-DD] [--before YYYY-MM-DD] [--label NAME]... [--has-attachment | --no-attachment]
    [--limit 20] [--offset N] [--count] [--relevance]
```

The viewer (`mboxer.exe`, or `cargo run -p mboxer-app -- [file.mbox]` during development) opens an
mbox through a file dialog or from its first argument, builds the sidecar index on the first open
(with progress and a cancel button) and reuses it afterwards, then shows the archive in three
panes: Gmail labels with counts, the message list (newest first, virtualised, so a 100 000-message
archive scrolls like a short one) and a reader with the headers and the body.

Reader: an HTML message renders as HTML (the *HTML* / *Text* / *Source* buttons switch between the
rendered body, the plain-text part and the raw source of the message; HTML or Text is remembered).
The HTML is sanitized in the webview before it is shown (scripts, frames, forms, event handlers
and `javascript:` links are removed) and rendered in a sandboxed iframe whose own content security
policy allows no network request at all: remote images and styles are held back and a yellow bar
counts them, *Show* loads them for that one message. Links in the body open in the system browser
or mail client (`http`, `https`, `mailto` only). *Copy* puts the text of the current view on the
clipboard. In the Text view the mboxrd quoting of body lines (`>From `) is undone; the Source view
shows the bytes as they are in the file.

Search: the field in the toolbar (`Ctrl+F` or `Ctrl+K`) narrows the list while you type (words are
AND-ed, the word you are still typing counts as a prefix until you type a space, `"quotes"` make a
phrase, accents and case do not matter); the *Filters* panel
(`Ctrl+Shift+F`) adds a sender, a date range (local days, both ends inclusive), an attachment
filter and the sort order (newest first, or best match when there are words to score); clicking a
label in the folder pane narrows further. All of it combines into one query; the words you searched
for are highlighted in the list and in the message body, the selected message stays selected when
it is still in the result, and *Clear* returns to all mail. Keyboard: `Ctrl+O` opens, `Enter` or
`ArrowDown` in the search field moves to the list, `Esc` clears the field, arrows / `PageUp` /
`PageDown` / `Home` / `End` move in the list, `Enter` focuses the reader and `Esc` returns. The
UI is plain HTML/CSS/JS embedded in the exe: no Node toolchain, no
bundler, no Tauri CLI needed; a change under `crates/mboxer-app/ui/` takes effect on the next
`cargo run`. The application icon is generated once by `scripts\make-icon.py` (Pillow) and committed.

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the
first messages, then a summary (message count, throughput, largest message, labels, parse failures).

`index` builds the sidecar full-text index in `.mboxer\<file name>\` next to the mbox (a cache:
delete it any time) and skips the build when the index already matches the file. `search` needs
that index: query words are AND-ed, `"quotes"` make a phrase, accents and case are folded on
both sides (`szamla` finds `Számla`), the filters narrow the result, and every hit line carries
the message's byte offset and length in the mbox. Dates are UTC days; `--before` is exclusive.

Checks before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`. Peak memory of a run: `scripts\peak-memory.ps1 -- target\release\mboxer-cli.exe scan <file.mbox> --quiet`.
The full benchmark (build time, peak memory, index size, re-open and search latency as a Markdown
report): `scripts\benchmark.ps1 -Mbox <file.mbox> -Out docs\benchmarks\<date>.md`; results for the
author's archive are under `docs/benchmarks/`.

## Layout

- `crates/mboxer-core` — streaming mbox reader (`MboxReader`, `read_at`), message summary and MIME
  tree over `mail-parser`, and the sidecar index (`index/`: fingerprint, schema with the accent-folding
  tokenizer, one-pass build, search, the full sorted listing behind the viewer's list); synthetic
  fixtures under `tests/fixtures/`.
- `crates/mboxer-app` — the `mboxer` executable: the Tauri 2 window (`src/`: commands, state with
  the open-or-build flow, the message view, the label order), the static UI under `ui/`, the
  capability file and the icon.
- `crates/mboxer-cli` — the `mboxer-cli` executable; its end-to-end tests under `tests/`.
- `scripts/` — helper scripts for measurements and the icon.
- `docs/` — origin, research, dated benchmarks.
