---
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M4 macOS
---

# 041 — `THIRD-PARTY-LICENSES.html` in the macOS zip

## Context

What: the Windows zip carries the licence page that cargo-about renders (025, D-012); the macOS zip of
`scripts/bundle-macos.sh` does not. Run cargo-about over the two darwin targets in the `bundle-macos` job (the
macOS dependency graph adds objc2 and friends) and put the page into the zip next to `usage.md`.
Why: the licence page is part of what a release ships (D-012); the macOS zip must not be the exception.
Done when: the macOS zip holds a licence page that lists the darwin-only crates, and `cargo deny check` still
passes with the macOS targets.
Decided 2026-10-19: done in parallel with 039 and 020; 041 lands before
the first release tag (028), which builds the macOS zip too.

## Steps

1. [x] how `scripts/release.ps1` and the `release` job render the page today (`about.toml`, the pinned
   `CARGO_ABOUT_VERSION`, the template), so the macOS side reuses the same config and version
2. [x] the `bundle-macos` job installs the same cargo-about and renders the page for `aarch64-apple-darwin` and
   `x86_64-apple-darwin`; `scripts/bundle-macos.sh` copies it into the zip root next to `usage.md` (deviation:
   a Linux job renders it, see below)
3. [x] `deny.toml` / `about.toml` cover the darwin targets; `cargo deny check` passes with them
4. [x] docs: `docs/development.md` (the macOS bundle), the header comment of `scripts/bundle-macos.sh`
5. [x] local checks (CONTRIBUTING.md), `uvx zizmor .github`; a manual `bundle-macos` run on the branch shows the
   page in the zip with the darwin-only crates (objc2 …)

## Log

- 2026-10-19 — promoted on `task/041-third-party-licences-in-the-macos-zip`.
- 2026-10-19 — Survey: `release.ps1` renders the page with `cargo about generate --frozen --fail`, `about.toml`
  (`targets = ["x86_64-pc-windows-msvc"]`) and the `scripts/third-party-licenses.hbs` template; the `release` job
  builds cargo-about 0.9.2 with `cargo install --locked`. The `generate --target` switch of cargo-about 0.9.2
  accepts several targets and overrides the config's `targets`, so the macOS page is made with the same
  `about.toml`, without a second config. The `deny.toml` `[graph] targets` has held all three targets since 018,
  nothing to do there.
- 2026-10-19 — Local test on Windows: `cargo about generate --frozen --fail --target aarch64-apple-darwin
  --target x86_64-apple-darwin …` runs without errors, 272 crates (Windows: 278); only on the macOS page: objc2,
  objc2-app-kit, objc2-foundation, objc2-web-kit, objc2-core-foundation, objc2-quartz-core,
  objc2-exception-helper, objc2-encode, block2, dispatch2, core-foundation(-sys), core-graphics(-types),
  foreign-types*, swift-rs, embed_plist, rustix, errno, base64 0.21; only on the Windows page the `windows-*`,
  `webview2-com*`, `winapi*`, softbuffer, tracing* crates. So the graph does not depend on the host.
- 2026-10-19 — Decision: where cargo-about runs. On the macOS runner `cargo install cargo-about` is a
  few-minute build at 10× the price (042), on every tag and manual run; the page, however, depends only on the
  lock file and the crate sources, not on the host. So a new `licences-macos` job renders it on ubuntu-latest
  (`cargo fetch --locked` for the two darwin targets, then an offline `--frozen` generation) and hands it over as a
  one-day artifact, which `bundle-macos` downloads and packs. The job starts in parallel with the checks (no
  `needs`), so `bundle-macos` does not wait for it separately. The command lives in one place:
  `scripts/bundle-macos.sh --licences-only <file>` (on any OS, needs cargo and cargo-about); packing takes it with
  `--licences <file>`; without a switch, on a Mac, the script renders the page itself. The `CARGO_ABOUT_VERSION`
  pin moved into the workflow-level `env`, the `release` and `licences-macos` jobs use the same one.
- 2026-10-19 — The template's introduction was Windows-specific (`mboxer.exe`, "WebView2, part of Windows 11");
  it is now platform-neutral (exe or app, WebView2 or WKWebView). The release notes' "Verify the download" text
  (`scripts/changelog.py`) now refers to both zips as well.
- 2026-10-19 — First manual run (run 39260328239): `licences-macos` failed: `cargo metadata` exited with an
  error: failed to download `android_system_properties v0.1.6` … `--frozen was specified`. Cause: cargo-about
  reads the graph with an unfiltered `cargo metadata`, which wants the sources of every platform's crates, while
  `cargo fetch --target …` (and a build) downloads only its own target's. Locally it worked because
  `~/.cargo/registry` already held everything. Reproduced with a fresh `CARGO_HOME`: after `cargo fetch --locked
  --target x86_64-pc-windows-msvc` the Windows page fails the same way — so the `release` job (025) would also
  have failed on the first tag on a clean Windows runner (it never ran on a tag; 025 was checked only with a local
  dry run). Fix in both scripts: `cargo fetch --locked` (all targets) right before the `--frozen` generation; with
  a fresh `CARGO_HOME` both the macOS and the Windows page are then produced without errors.

## Deviations from the plan

- Step 2: cargo-about does not run in the `bundle-macos` job (macOS runner) but in a Linux job, and
  `bundle-macos` only packs the page — the same result without the macOS minutes (reasons in the Log).
- The template and the release notes text changed too (the page is no longer about the exe only).
- `scripts/release.ps1` also got a `cargo fetch --locked` before cargo-about: the same error would have broken the
  Windows release job on the first tag as well (Log); 041 lands before 028's first release.

## Verification

- 2026-10-19 — locally: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green;
  `cargo deny check`: advisories, bans, licenses, sources ok (`deny.toml` with all three targets);
  `uvx zizmor .github`: No findings; `actionlint` (actionlint-py, with shellcheck-py) clean; `bash -n` and
  shellcheck clean on `scripts/bundle-macos.sh`; `release.ps1` parses without errors.
- 2026-10-19 — with a fresh `CARGO_HOME`: after a host-only fetch the Windows page fails with `--frozen` (the old
  `release.ps1` path); `scripts/bundle-macos.sh --licences-only` and then the Windows page succeed.
- 2026-10-19 — manual CI run on the branch (run 39262031629): every job green; `licences-macos` 3m32s (ubuntu, in
  parallel with the checks), `bundle-macos` 12m. The downloaded `macos-zip` artifact: `mboxer-0.1.0-macos.zip`
  holds `usage.md`, `THIRD-PARTY-LICENSES.html` (273 KB) and `mboxer.app` at its root; the zip's SHA-256 matches
  the `SHA256SUMS.txt` line. The page lists 272 crates, among them objc2, objc2-app-kit, objc2-foundation,
  objc2-web-kit, objc2-core-foundation, objc2-quartz-core, objc2-exception-helper, objc2-encode, block2,
  dispatch2, core-foundation(-sys), core-graphics(-types), swift-rs, embed_plist; no `windows*` / `webview2*` /
  `winapi*`; the introduction is the new platform-neutral text. Apart from line endings it is byte-identical to the
  locally rendered page.
- The Windows `release` job's `cargo fetch` path runs only on a tag; the first tag (028) proves it live.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — no follow-up
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated — in the task session `tasks_check` without `write`; the main
  session regenerates the index when it prepares the pull request
- [x] moved to `done/` with `closed: 2026-10-19`
