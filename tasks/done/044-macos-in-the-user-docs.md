---
size: M
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M5 Public release
---

# 044 — macOS in the user docs, the security policy and the bug report form

## Context

What: the README (tagline, Download with the macOS preview zip and the Gatekeeper *Open Anyway* step, `gh
attestation verify`, Requirements, how to report a bug, Build from source on a Mac), `docs/usage.md` (an "On a
Mac" part, `shasum`, the write locations on a Mac, WebView2 fallback, plainer wording), `docs/reference.md` (the
quarantine attribute and the blocked types on a Mac, the CLI is not in the zips), `SECURITY.md` (macOS scope) and
the bug report form ("Operating system and version", the Mac index location).
Why: 0.2.0 ships a macOS zip, but every user-facing text still reads as Windows-only (pre-publication review).
Done when: a Mac user can download, verify, start and report a bug from the docs alone, before the `v0.2.0` tag.
Decided 2026-10-19: the macOS zip is described as a preview that has not yet been tested on a Mac.

## Steps

1. [x] README: tagline, Requirements, Download and start (Windows and macOS), checking the download, reporting a
   bug, Build from source on a Mac, the theme line, D-id links, the outdated "macOS build is on the roadmap"
   sentence
2. [x] `docs/usage.md`: title and headings, the Mac steps, checksums and attestation, write locations, search
   wording, WebView2 fallback, reporting a problem
3. [x] `docs/reference.md`: attachments on a Mac, the CLI outside the zips, D-id links, the shortcut wording
4. [x] `SECURITY.md`: quarantine attribute, Mac index fallback, Gatekeeper and WKWebView out of scope
5. [x] `.github/ISSUE_TEMPLATE/bug-report.yml`: operating system field, Mac index location option
6. [x] every statement checked against the code, links resolve

## Log

- 2026-10-19 — promoted
- 2026-10-19 — the facts behind the new text, read from the code: `attachments.rs` (`temp_root` is
  `std::env::temp_dir()/mboxer`, so `$TMPDIR/mboxer/` on a Mac; `com.apple.quarantine` written with `xattr`; one
  `BLOCKED_EXTENSIONS` list with the macOS types `command`, `tool`, `pkg`, `mpkg`, `dmg`, `app`, `sh`, `scpt`,
  `applescript`, `workflow`, `terminal`, `webloc`, `inetloc`, `fileloc`); `fingerprint.rs` (the fallback root
  under `~/Library/Application Support`); `bundle-macos.sh` (universal binary through `lipo`,
  `LSMinimumSystemVersion` 10.13); `release.ps1` and `bundle-macos.sh` (the zips hold the app, `usage.md` and
  `THIRD-PARTY-LICENSES.html`, no `mboxer-cli`); `commands.rs` (`RunEvent::Opened` for Finder and Dock opens)
- 2026-10-19 — README: Requirements section, Download and start per platform, one "Check the download" paragraph
  (`Get-FileHash`, `shasum -a 256`, `gh attestation verify`), "Found a bug?" line, the Mac build in Build from
  source; D-002 and D-008 linked to their files. `docs/usage.md`: retitled "user guide", the Mac download and
  Gatekeeper steps, drag-and-drop wording, a Windows/Mac table for the write locations, "A message must contain
  every word you type", the WebView2 runtime fallback and an issue link under "When something is off".
  `docs/reference.md`: the Mac attachment folder, blocked types and quarantine mark, D-004/D-005/D-006 linked,
  `mboxer-cli` is a developer tool outside the zips. `SECURITY.md`: macOS paths and marks in scope, Gatekeeper
  and WKWebView/macOS out of scope. `CONTRIBUTING.md`: the Mac build prerequisites. Bug report form: `os` field
  for both systems, a macOS index location option

## Deviations from the plan

- `CONTRIBUTING.md` got the Mac build prerequisites too (the review listed it next to the README build section).
- The web view's own data on a Mac is described as "the folders named `io.github.speti4.mboxer` under
  `~/Library/`", without exact paths: the exact WebKit and cache folders are for the manual test on a Mac (040)
  to confirm.

## Verification

- 2026-10-19 — every relative link in `README.md`, `docs/usage.md`, `docs/reference.md`, `SECURITY.md` and
  `CONTRIBUTING.md` resolves to an existing file, and every `#anchor` to an existing heading (script over the
  Markdown links: 0 missing); `bug-report.yml` parses as YAML (`yaml.safe_load`); every macOS statement matches
  the code lines listed in the Log; `tasks_check` without write: only the expected stale-index report

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log: none left; the exact
  macOS web view data folders belong to the 040 manual test, which already covers the Mac runtime paths
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (index by the main session)
- [x] moved to `done/` with `closed: 2026-10-19`
