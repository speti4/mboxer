---
status: ready
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
milestone: M4 macOS
---

# `THIRD-PARTY-LICENSES.html` in the macOS zip

What: the Windows zip carries the licence page that cargo-about renders (025, D-012); the macOS zip of
`scripts/bundle-macos.sh` does not. Run cargo-about over the two darwin targets in the `bundle-macos` job (the
macOS dependency graph adds objc2 and friends) and put the page into the zip next to `usage.md`.
Why: the licence page is part of what a release ships (D-012); the macOS zip must not be the exception.
Done when: the macOS zip holds a licence page that lists the darwin-only crates, and `cargo deny check` still
passes with the macOS targets.
