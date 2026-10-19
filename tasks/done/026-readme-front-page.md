---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: peppy-imagining-pony.md
---

# 026 — README as a front page, the reference moved into docs

## Context

The README rewritten as a front page: a one-line pitch, badges (CI, latest release, licence), a screenshot of
the viewer over a synthetic mbox (never a real archive), a feature list, download and quick start, and
links to usage, build and decisions. The current long reference moves to `docs/reference.md`, or into
`docs/usage.md` where it addresses the user. Why: today the README is a dense reference with no picture, and a
visitor decides within one screen. Done when: the README fits on two screens and every moved paragraph has a home
under `docs/`. Planning decisions 2026-10-18 (approved plan): the viewer and CLI reference goes to
`docs/reference.md`; layout, CI, release and scripts go to `docs/development.md`; build prerequisites and checks go
to `CONTRIBUTING.md` (023). The screenshot is taken over a demo mbox that `scripts/make-demo-mbox.py` generates
(stdlib only, deterministic, made-up mail under `example.*` domains), which can also serve the social preview in 028.

## Steps

1. [x] `scripts/make-demo-mbox.py` → `target/demo/demo.mbox`; `mboxer-cli scan` / `index` / `search` over it
2. [x] Screenshot over the demo mbox → `docs/images/viewer.png`
3. [x] `docs/reference.md` (viewer + CLI reference) and `docs/development.md` (layout, CI, release, scripts)
4. [x] New `README.md` front page (pitch, badges, picture, features, download, build, docs, status, licence)
5. [x] `docs/usage.md`: link to the reference; `CLAUDE.md` Scope: the new docs files
6. [x] Check: line count, every old paragraph has its new home, relative links; `tasks_check`; commit

## Log

- 2026-10-18 — plan approved; 026 promoted to wip (together with 023)
- 2026-10-18 — `make-demo-mbox.py`: 50 messages, 42 862 bytes, the SHA-256 of two runs identical (`069f772c…`);
  `mboxer-cli scan`: 50 messages, 11 with attachments, 10 labels, 0 parse failures; `index` 0.1 s; `search kave` →
  1 hit ("kávé" in the v2.0.0 release mail), `--label Work/Invoices --count` → 4
- 2026-10-18 — screenshot: scratchpad `capture-viewer.ps1` (debug exe, `cargo build` first; fresh since the
  crates' last commit). Protection against real data: a throwaway WebView2 profile (`WEBVIEW2_USER_DATA_FOLDER`,
  deleted before every run), so the remembered file list (the 14 GB archive) does not open; the newest file of
  the real profile is the same before and after (04:20:55). `--lang=en-US` (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`)
  for English dates; the mbox under `subst M:` on a Takeout-like path
  (`M:\Takeout\Mail\All mail Including Spam and Trash.mbox`), so the status bar shows no local path; the subst
  removed afterwards. 1440×860, the list/reader splitter dragged, *Light* theme, the Go-live conversation open,
  the message with the attachment selected, the title bar cropped (31 px) → 1426×822, 105 KB
- 2026-10-18 — two small UI issues seen while taking the picture went to the backlog: 031 (the native title bar
  stays dark under the Light override on a dark Windows), 032 (the date column shifts between conversation rows
  and plain rows)
- 2026-10-18 — `README.md` 146 → 71 lines; scratchpad `check-docs.py`: 0 broken relative links in 8 files; of 50
  characteristic phrases of the old README, 49 are verbatim in their new home, the 50th ("one-way into") reworded
  ("mirrored one way into")

## Deviations from the plan

- The screenshot comes from the debug exe, not the release one: the `target\release` exe predates 016/017; the UI
  is the same, so is the version in the status bar (v0.1.0)
- The title bar is cropped from the picture (because of the 031 bug, and on a front page the native frame is
  noise anyway)
- `docs/development.md` also got a "The demo mbox" section (the script and the screenshot rule)

## Verification

- 2026-10-18 — `README.md` 71 lines (within the "two screens" limit); 0 broken relative links in the 8 files
  touched; every paragraph of the old README has a home (`docs/reference.md`, `docs/development.md`,
  `CONTRIBUTING.md`), checked by a 50-phrase grep
- 2026-10-18 — README and screenshot reviewed and approved; commit `f640447`, pushed, the `sync-issues` run green
- The badges do not render on the private repo; this went onto the 028 checklist (README badges after the switch)

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
