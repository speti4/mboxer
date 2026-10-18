---
status: ready
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M4 macOS
---

# macOS build, `.app` bundle and CI

What: a `scripts/bundle-macos.sh` that builds `aarch64-apple-darwin` + `x86_64-apple-darwin`, joins them with
`lipo`, assembles the `.app` (`Info.plist` with the Cargo.toml version and the `io.github.speti4.mboxer`
identifier, `.mbox` document type, `icon.icns` via `iconutil` from PNGs that `make-icon.py` also emits),
ad-hoc signs it (`codesign -s -`) and packs `dist/mboxer-<version>-macos.zip` (`.app` + `usage.md`) with
`ditto`; `ci.yml` gets `macos-latest` in the `test` matrix and a second release artefact, `deny.toml` the two
macOS targets in `[graph] targets` (D-011). No Tauri CLI (D-02).
Why: the first of three macOS tasks (plan 2026-10-18); it proves the workspace compiles and tests pass on macOS
before 019 and 020. First step on Windows: `cargo check --target aarch64-apple-darwin -p mboxer-app`.
Done when: the `macos-latest` CI job is green and the release zip holds an `.app` that starts on a Mac
(manual round by a Mac user; the author has no Mac, the Gatekeeper step is D-07).
