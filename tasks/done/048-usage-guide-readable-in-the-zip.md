---
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M5 Public release
---

# 048 — The usage guide in the zips in a format that opens with a double-click

## Context

What: `release.ps1` and `bundle-macos.sh` put `usage.md` into the zips; Windows has no default app for `.md` and
the tables read poorly as plain text. Render it to `usage.html` next to `THIRD-PARTY-LICENSES.html` (or ship
`.txt`), without a new runtime dependency.
Why: the guide is for people who only run the program.
Done when: both zips hold a guide that opens in the default browser or text editor and reads well.
Decided 2026-10-19: merged after the `v0.2.0` release, so the first public release run is not changed.

## Steps

1. [x] the renderer: how `usage.md` becomes `usage.html` at release time with tools the release jobs already
   have (Python standard library, the cargo-about step, or a small converter in `scripts/`); the choice and the
   reason in the Log
2. [x] `scripts/release.ps1` and `scripts/bundle-macos.sh` pack the rendered guide; the macOS licence-only path
   and the CI artifact paths still work
3. [x] the docs that name the zip contents (README, `docs/usage.md`, `docs/development.md`, the release notes
   footer in `scripts/changelog.py`) follow
4. [x] a local `release.ps1` run (or its packing part) shows the guide in the zip and it renders in a browser;
   the macOS script checked with `bash -n` and a dry path where possible

## Log

- 2026-10-19 — renderer: a new `scripts/render-usage-guide.py`, Python standard library only. Both release jobs
  already run Python (`release.ps1` resolves `python` / `py -3` for the notes; the macOS runner has `python3`),
  so it adds no tool and no crate. cargo-about's Handlebars template was not chosen: it renders the crate graph,
  not a Markdown file. A `.txt` copy was not chosen: the guide's two tables and its code spans read poorly as
  plain text. The renderer covers exactly the Markdown the guide uses (headings, paragraphs, flat bullet and
  numbered lists with indented continuation lines, pipe tables, fenced code, inline code, emphasis, strong,
  links and bare URLs). It stops with the line number on anything else (nested list, block quote, raw HTML,
  `*`/`+` bullets, indented code, a rule), so a guide edit it would render wrongly fails instead of shipping.
  The page is self-contained: inline CSS, no script, no remote font, light and dark through
  `prefers-color-scheme`, `<h2>` ids for anchors.
- 2026-10-19 — `--check` runs a built-in self-test (every supported construct, HTML escaping in text, code and
  attributes, URL trailing punctuation, seven unsupported-input errors) and then renders `docs/usage.md` in
  memory; the lint job runs it after the changelog check. The step goes after line 102 of `ci.yml`, so the
  zizmor ignores scoped to `ci.yml:67` and `ci.yml:91` (task 046) still point at the two rust-cache lines; the
  header comment was edited without changing its line count.
- 2026-10-19 — `release.ps1` renders the guide into the stage as `usage.html` instead of copying `usage.md`;
  `bundle-macos.sh` does the same with `python3` and lists `python3` among the tools it checks. `--licences-only`
  and the artifact paths in `ci.yml` (the zips, `SHA256SUMS.txt`, the licence page) are untouched. The zips hold
  only `usage.html`; the Markdown source stays readable on GitHub.
- 2026-10-19 — docs: README, `docs/usage.md` ("this guide as `usage.html`"), `docs/reference.md`,
  `docs/development.md` (the release paragraph, the renderer, the lint step, the macOS bundle paragraph),
  CONTRIBUTING, CLAUDE.md and the release notes footer in `scripts/changelog.py` name `usage.html`.

## Deviations from the plan

- `ci.yml` got a lint step (`render-usage-guide.py --check`), which the steps did not name; it keeps a guide edit
  from breaking a release run.

## Verification

- 2026-10-19 — `python -I scripts/render-usage-guide.py --check`: "usage.md renders (12 KB); self-test passed".
  The rendered page parses with balanced tags (stdlib `html.parser`: no unbalanced end tag, nothing left open);
  all four bare URLs of the guide became links, with the closing `)`, `,` and `.` left outside.
- 2026-10-19 — full `scripts\release.ps1 -Out <scratch>\dist` run in this worktree (release build 2 m 57 s,
  cargo-about 0.9.2, exit 0): `mboxer-0.2.0.zip` holds `mboxer.exe` (15 151 616 B), `THIRD-PARTY-LICENSES.html`
  (272 245 B) and `usage.html` (12 460 B); the zipped `usage.html` is byte-identical to a fresh render of
  `docs/usage.md`; `SHA256SUMS.txt` matches `Get-FileHash` of the zip
  (`e47039ff…fdabf8`); `release-notes-0.2.0.md` ends with the new footer.
- 2026-10-19 — headless Edge (`--headless=new --screenshot`, no window on screen) rendered the page in the dark
  and, with `--blink-settings=preferredColorScheme=1`, the light scheme: headings, both lists, the code spans,
  emphasis and the links read as intended.
- 2026-10-19 — `bash -n scripts/bundle-macos.sh` passes; its `--help` range (lines 2–30) still covers the whole
  header comment. The script itself runs only on macOS: its first real run is the next `bundle-macos` job (a tag
  or a manual dispatch).
- 2026-10-19 — `uvx zizmor .github`: no findings; `ci.yml:67` and `ci.yml:91` are still the rust-cache lines.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: 2026-10-19`
