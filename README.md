# mboxer

A fast, portable viewer for big Gmail Takeout mbox archives on Windows 11: search years of mail in well under a
second, one exe, nothing to install.

[![CI](https://github.com/speti4/mboxer/actions/workflows/ci.yml/badge.svg)](https://github.com/speti4/mboxer/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/speti4/mboxer)](https://github.com/speti4/mboxer/releases/latest)
[![Licence: MIT](https://img.shields.io/github/license/speti4/mboxer)](LICENSE)

![mboxer showing a made-up Gmail archive: the label tree, a conversation opened in the list, and a message with an attachment in the reader](docs/images/viewer.png)

## Features

- **Full-text search in milliseconds.** The first open builds an index beside the mbox (about two minutes and
  135 MB for a 14 GB archive); every later open takes under a second. Accents and case do not matter:
  `szamla` finds `Számla`.
- **Gmail as you know it.** Labels as a tree with counts, conversations, and Gmail-style operators (`from:`,
  `to:`, `subject:`, `after:`, `before:`, `has:attachment`, `label:`) with a filter panel that writes them.
- **Safe HTML mail.** Messages are sanitized and rendered in a sandbox that cannot reach the network; remote
  images load only when you ask.
- **Attachments** open with their default application or are saved, marked as downloaded from the internet;
  executables can only be saved.
- **Hand a message on** as an `.eml` file, print it, or save it as PDF.
- **Several archives at once**, for a split Takeout export or more than one export.
- **Light and dark**, following Windows or your choice.
- **Never touches your mail.** The mbox is only read; the index is a cache you can delete any time.

## Download and start

1. Download `mboxer-<version>.zip` from the [latest release](https://github.com/speti4/mboxer/releases/latest)
   and unzip it anywhere; [CHANGELOG.md](CHANGELOG.md) says what each release changed. To check the download,
   compare `(Get-FileHash mboxer-<version>.zip).Hash` in PowerShell with the value in `SHA256SUMS.txt` from the
   same release.
2. Start `mboxer.exe`. The exe is not code-signed, so SmartScreen may warn on the first start: *More info*,
   then *Run anyway*.
3. *Open* (`Ctrl+O`) your Takeout `.mbox` file and wait for the first index build.

The [usage guide](docs/usage.md) (also inside the zip) covers the rest; the [reference](docs/reference.md)
describes every feature, the search syntax and the `mboxer-cli` command line.

## Build from source

mboxer is a Rust core with a Tauri 2 window on WebView2 ([D-02](tasks/decisions/)). You need Rust stable with
the MSVC toolchain and the Visual Studio 2022 C++ build tools (details in [CONTRIBUTING.md](CONTRIBUTING.md)):

```powershell
cargo build --release
target\release\mboxer.exe [file.mbox]
```

## Documentation

- [Usage guide](docs/usage.md): for the people who only run it
- [Reference](docs/reference.md): the viewer, the search syntax, the index and the CLI
- [Development](docs/development.md): layout, CI, releases and helper scripts
- [Changelog](CHANGELOG.md): every release and the tasks it closed, with the zips and checksums on the
  [releases page](https://github.com/speti4/mboxer/releases)
- [Contributing](CONTRIBUTING.md) and the [security policy](SECURITY.md)
- [Why mboxer exists](docs/origin.md), [the alternatives it was measured against](docs/research/),
  [benchmarks](docs/benchmarks/)
- [Decisions](tasks/decisions/): the stack, the index, distribution, and why each

## Project status

The plan lives in [`tasks/`](tasks/README.md): the backlog, the work in flight and the history, one file per
item, with milestones in [`tasks/roadmap.md`](tasks/roadmap.md). The files are mirrored one way into
[issues](https://github.com/speti4/mboxer/issues) and [milestones](https://github.com/speti4/mboxer/milestones),
so the task files stay the only place to edit (D-08). Windows comes first; a macOS build is on the roadmap.

mboxer started as a faster, steadier alternative to [eneam/mboxviewer](https://github.com/eneam/mboxviewer) for
anyone who keeps an old mailbox as a Takeout export.

## Licence

[MIT](LICENSE)
