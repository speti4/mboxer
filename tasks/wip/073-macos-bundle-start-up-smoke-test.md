---
size: S
origin: backlog/040-manual-round-on-a-mac.md
created: 2026-11-16
started: 2026-11-16
plan: none
milestone: M4 macOS
---

# 073 — Start-up smoke test for the macOS bundle: the packed app opens a window

## Context

What: `scripts/smoke-macos.sh` launches the app unpacked from the release zip, waits for an on-screen window of
the process and keeps its stderr and a screenshot; the `bundle-macos` CI job runs it after packing.
Why: the manual round on a Mac (040) found that the app starts on macOS 27.0.1 with its Dock icon but shows no
window, and no automated check launches the bundle: CI runs the unit tests on macOS and packs the zip, nothing
more.
Done when: the smoke step passes on the macOS runner by manual dispatch (or reproduces the missing window there,
which is then fixed in this task), and the cause of the missing window on macOS 27 is fixed or filed as an item
of its own.

What is known at the start (2026-11-16): the `main` window is declared in `crates/mboxer-app/tauri.conf.json`
and nothing in `run()` (`crates/mboxer-app/src/lib.rs`) hides or delays it; release 0.3.0 locks tauri 2.12.4,
tao 0.37.1 and wry 0.57.0, and the changelogs of the later tao 0.37.2, wry 0.57.1, wry 0.57.2 and tauri 2.12.5
name no fix for a window that does not appear; the hosted runners have no macOS 27 image (`macos-latest` is
macOS 26 on Apple Silicon). So the first question the smoke test answers is whether the bundle opens a window
on any macOS at all.

## Docs that move

- `docs/development.md`: the smoke script next to the macOS bundle script, and the CI step.
- `CHANGELOG.md`: regenerated at the close.

## Not in this change

- A check of what the webview renders: the test asks for a window, not for its content.
- A macOS 27 runner: none is hosted yet; the evidence from macOS 27 comes from the manual round (040).
- The rest of the manual round of 040.

## Steps

1. [ ] `scripts/smoke-macos.sh <mboxer.app> [--out <folder>]`: start `Contents/MacOS/mboxer`, keep its stderr,
   wait up to 30 s for an on-screen window of the process (layer 0, not empty), take a screenshot, stop the
   app; fail when the process ends early or no window appears
2. [ ] `.github/workflows/ci.yml`, `bundle-macos`: unpack the zip with `ditto`, run the smoke script on the
   unpacked app, upload its stderr and screenshot whatever the result
3. [ ] manual dispatch on this branch; the result of the smoke step on macOS 26 in the Log
4. [ ] the missing window on macOS 27: the cause fixed here, or an item of its own with what is known
5. [ ] `docs/development.md`
6. [ ] the checks of CONTRIBUTING.md, "Checks before a commit"

## Log

- 2026-11-16 — filed and promoted in the main session, from the first point of the manual round (040).

## Deviations from the plan

- none yet

## Verification

- pending

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`, approved by the owner) or deleted with a reason in the Log
- [ ] every doc under *Docs that move* changed in this change
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
