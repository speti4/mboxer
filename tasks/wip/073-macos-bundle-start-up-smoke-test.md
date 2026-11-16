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
- `docs/reference.md`: the window opens as large as a smaller screen allows.
- `CHANGELOG.md`: regenerated at the close.

## Not in this change

- A check of what the webview renders: the test asks for a window, not for its content.
- A macOS 27 runner: none is hosted yet; the evidence from macOS 27 comes from the manual round (040).
- The rest of the manual round of 040.

## Steps

1. [x] `scripts/smoke-macos.sh <mboxer.app> [--out <folder>]`: start `Contents/MacOS/mboxer`, keep its stderr,
   wait up to 30 s for an on-screen window of the process (layer 0, not empty), take a screenshot, stop the
   app; fail when the process ends early or no window appears
2. [x] `.github/workflows/ci.yml`, `bundle-macos`: unpack the zip with `ditto`, run the smoke script on the
   unpacked app, upload its stderr and screenshot whatever the result
3. [x] manual dispatch on this branch; the result of the smoke step on macOS 26 in the Log
4. [ ] the missing window on macOS 27: the cause fixed here, or an item of its own with what is known
5. [x] `docs/development.md`
6. [ ] the checks of CONTRIBUTING.md, "Checks before a commit"

## Log

- 2026-11-16 — filed and promoted in the main session, from the first point of the manual round (040).
- 2026-11-16 — steps 1, 2 and 5: `scripts/smoke-macos.sh`, the "Start the packed app" step of `bundle-macos`
  with the `macos-smoke` artifact, and `docs/development.md`. `bash -n`, shellcheck and zizmor are clean. The
  app starts through `open`, not by running the binary, so LaunchServices launches it as a double click does.
- 2026-11-16 — step 3, two manual runs on the branch, three attempts. Run 43844726106: the step ended with "Permission
  denied", the script was committed without its executable bit; fixed in the next commit. Run 43847629898,
  first attempt: `test (macos-latest)` failed in
  `attachments::tests::clean_up_keeps_a_live_process_folder_and_removes_a_dead_one` (`assertion failed:
  !dead_copy.exists()`), a test that passed on the same Rust code in the run before; `bundle-macos` was
  skipped. Second attempt of the same run: every job green, and the smoke step printed `system: 26.7.2
  (arm64)` and `window: 1024x674`; stdout and stderr of the app are empty.
- 2026-11-16 — what the run shows. On macOS 26.7.2 the packed app opens its window, so the bundle itself works
  and the missing window of the manual round is specific to macOS 27 or to that machine. The screenshot shows
  a second fault: the runner's screen is 1024x768, smaller than the window the configuration asks for
  (1400x900, centred); the window is shrunk to the screen's width but placed as if it were 1400 wide, so its
  left part, the title bar buttons with it, lies off the screen. The smoke test passes on it, because it asks
  for a window on the screen, not for a whole one.
- 2026-11-16 — Decided 2026-11-16: the window's place is fixed in this task, and the smoke test asks for a
  window that stands whole on the main display (it measures the window a second after it appears). The flaky
  `clean_up_keeps_a_live_process_folder_and_removes_a_dead_one` gets no item of its own: one failure in five
  macOS test runs of this branch, noted here.
- 2026-11-16 — the stricter smoke test without a fix, run 43850387414: fails as it should, `window: 1024x674
  at -188,31`, "a part of the window lies off the main display". -188 is (1024 - 1400) / 2: the centred
  position is computed from the size the configuration asks for.
- 2026-11-16 — a first fix did not work (run 43850808260, the same -188,31): a `setup` hook that moved a
  window lying outside its monitor's work area. A run with a start-up log (43853820054) showed why: when
  `setup` runs the window still stands at 0,31, and the centred position is applied after it.
- 2026-11-16 — the fix: `"preventOverflow": true` on the main window in `crates/mboxer-app/tauri.conf.json`.
  Tauri then shrinks the window to the monitor's work area before it computes the centred position. The hook
  and its log are gone again; `lib.rs` is as on `main`. Run 43856410822: every job green, `window: 1024x674 at
  0,31`, and the screenshot shows the whole window with its title bar. On Windows, on a screen larger than
  the window, the viewer opens at the same place and size as before the change (1416x939 outer, centred).
- 2026-11-16 — Open: step 4. Whether this fault is what the manual round saw on macOS 27.0.1 is not known; the
  zip of run 43856410822 goes to the manual round, and its result decides.

## Deviations from the plan

- The plan had the smoke test and a fix that depended on its result. The result was a second fault, the window
  placed partly off a small screen, and its fix (one configuration line) is part of this change.

## Verification

- pending

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`, approved by the owner) or deleted with a reason in the Log
- [ ] every doc under *Docs that move* changed in this change
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
