# mboxer

A Windows 11 viewer for large Gmail Takeout mbox archives (14–50 GB): fast indexed search,
stable, usable UI. Replaces [eneam/mboxviewer](https://github.com/eneam/mboxviewer) for the author
and a handful of colleagues, each with their own mbox file.

- Stack: Rust, Tauri 2 on WebView2, one-time sidecar full-text index next to the mbox (see
  `tasks/decisions/`).
- Status and backlog: `tasks/README.md` (generated), milestones in `tasks/roadmap.md`.
- Why this exists: `docs/origin.md`; why build and not buy: `docs/research/`.
- For the people who only run it: `docs/usage.md` (ships in the release zip).

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
    [--subject WORDS] [--after YYYY-MM-DD] [--before YYYY-MM-DD] [--label NAME]... [--has-attachment | --no-attachment]
    [--limit 20] [--offset N] [--count] [--relevance]
```

The viewer (`mboxer.exe`, or `cargo run -p mboxer-app -- [file.mbox]` during development) opens an
mbox through a file dialog or from its first argument, builds the sidecar index on the first open
(with progress and a cancel button) and reuses it afterwards, then shows the archive in three
panes: the Gmail label tree with counts, the message list (newest first, virtualised, so a
100 000-message archive scrolls like a short one) and a reader with the headers and the body.

Labels: the folder pane shows the Gmail labels as a tree (`Work/Clients` sits under `Work`, the
Takeout `Category …` labels sit under *Categories*, the system labels come first), each with the
number of messages carrying it; a parent that only groups sub-labels shows their total in italics.
An RFC 2047 encoded label name (`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) is decoded before it is indexed.
Clicking a label narrows the list to that label *and* every label under it; a chevron (or
`ArrowLeft` / `ArrowRight` on the selected node) folds and unfolds a branch.

Conversations: by default the list shows one row per Gmail conversation (`X-GM-THRID`): the
newest message with its subject, a count badge and a chevron. The chevron, `ArrowRight` or a
click opens the conversation under its row, oldest message first; `ArrowLeft` closes it again,
also from one of its messages. Only the messages that match the current search belong to a
conversation row, so a search can show a thread with fewer messages than it has; a thread with
one matching message is a plain row. The *Conversations* checkbox in the filter panel switches
back to one row per message; the choice is remembered. The status bar counts both:
`N conversations · M messages`.

Reader: an HTML message renders as HTML (the *HTML* / *Text* / *Source* buttons switch between the
rendered body, the plain-text part and the raw source of the message; HTML or Text is remembered).
The HTML is sanitized in the webview before it is shown (scripts, frames, forms, event handlers
and `javascript:` links are removed) and rendered in a sandboxed iframe whose own content security
policy allows no network request at all: remote images and styles are held back and a yellow bar
counts them, *Show* loads them for that one message. Links in the body open in the system browser
or mail client (`http`, `https`, `mailto` only). *Copy* puts the text of the current view on the
clipboard. In the Text view the mboxrd quoting of body lines (`>From `) is undone; the Source view
shows the bytes as they are in the file.

Attachments: a strip under the headers lists them (name, size, an `inline` tag on images that
belong to the HTML body). A click on a chip decodes the attachment from the mbox into a temporary
file under `%TEMP%\mboxer\` and opens it with the default application; *Save…* asks where to put
it, *Save all…* writes every attachment of the message into a folder without overwriting
(`name (2).ext`). Executable attachments (`.exe`, `.bat`, `.ps1`, `.js`, `.lnk` and the like) can
only be saved. Every written file carries the Mark of the Web, so Office opens it in Protected
View, as with a file downloaded from a mail client; the temporary copies are removed at the next
start. Inline images referenced by `cid:` are shown inside the HTML body; text attachments (csv,
ics) are saved byte for byte in their original charset.

Search: the field in the toolbar (`Ctrl+F` or `Ctrl+K`) narrows the list while you type (words are
AND-ed, the word you are still typing counts as a prefix until you type a space, `"quotes"` make a
phrase, `OR` and `-word` work, accents and case do not matter). The field also takes Gmail-style
operators, and it is the one source of truth: the *Filters* panel (`Ctrl+Shift+F`) and a click on a
label in the folder pane write their operator into the field, and an operator you type shows up in
the panel and in the folder selection.

| Operator | Meaning |
|---|---|
| `from:anna`, `from:"anna példa"` | words in the sender (several `from:` add up) |
| `to:konyveles` | words in a recipient (To or Cc) |
| `subject:számla` | words in the subject |
| `after:2026-01-01`, `before:2026-01-31` | local days, both inclusive (`before:` includes the day, unlike Gmail; `2026/1/31` is accepted) |
| `has:attachment`, `-has:attachment` | with, without an attachment |
| `label:Inbox`, `label:"Work/Invoices"` | the first `label:` is the folder-pane node (it and its sub-labels), further ones are exact labels that must all be present; a container name (`label:Work`, `label:Categories`) selects the container |

Operator names are case-insensitive; `-` only negates `has:`. An unknown `foo:bar` is passed to the
search engine as is (it warns in the status bar), a bad date or `has:` value is a warning and the
token is ignored. The panel adds the sort order (newest first, or best match when there are words to
score) and the *Conversations* view. All of it is one query; the words you searched for (and the
`from:` / `subject:` values) are highlighted in the list and in the message body, the selected
message stays selected when it is still in the result, and *Clear* or `Esc` in the field empties the
whole query, label included. Keyboard: `Ctrl+O` opens, `Enter` or
`ArrowDown` in the search field moves to the list, `Esc` clears the query, arrows / `PageUp` /
`PageDown` / `Home` / `End` move in the list, `Enter` focuses the reader and `Esc` returns. The
UI is plain HTML/CSS/JS embedded in the exe: no Node toolchain, no
bundler, no Tauri CLI needed; a change under `crates/mboxer-app/ui/` takes effect on the next
`cargo run`. The application icon is generated once by `scripts\make-icon.py` (Pillow) and committed.

`scan` streams the whole file once with a bounded buffer, prints the headers and MIME tree of the
first messages, then a summary (message count, throughput, largest message, labels, parse failures).

`index` builds the sidecar full-text index in `.mboxer\<file name>\` next to the mbox (a cache:
delete it any time) and skips the build when the index already matches the file. When that folder
is not writable (a read-only share, a DVD), the index goes under `%LOCALAPPDATA%\mboxer\index\`
(`MBOXER_INDEX_HOME` overrides the root), and every open looks in both places. `search` needs
that index: query words are AND-ed, `"quotes"` make a phrase, accents and case are folded on
both sides (`szamla` finds `Számla`), the filters narrow the result, and every hit line carries
the message's byte offset and length in the mbox. Dates are UTC days; `--before` is exclusive.

Checks before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`; `.github/workflows/ci.yml` runs the same three on every push and pull request
(windows-latest). Release: `scripts\release.ps1` builds the release profile (fat LTO, stripped) and
packs `target\release\mboxer.exe` + `docs\usage.md` into `dist\mboxer-<version>.zip`, the version
being the workspace `Cargo.toml` one (the only place it is written; `tauri-build` stamps it into
the exe's properties and the status bar shows it). Pushing a tag `v<version>` runs the tests and
publishes that zip as a GitHub release. Peak memory of a run: `scripts\peak-memory.ps1 -- target\release\mboxer-cli.exe scan <file.mbox> --quiet`.
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
- `scripts/` — helper scripts for measurements, the icon and the release zip.
- `docs/` — origin, research, dated benchmarks, the usage guide for the other users.
- `.github/workflows/` — CI (fmt, clippy, test) and the tagged release.
