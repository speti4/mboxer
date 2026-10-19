---
size: S
origin: done/018-macos-build-bundle-and-ci.md
created: 2026-10-19
started: 2026-10-19
plan: C:\Users\peter\.claude\plans\n-zz-k-t-a-backlogot-toasty-dove.md
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
User decision 2026-10-19: part of the backlog wave 039 + 041 + 020 in parallel task sessions; 041 lands before
the first release tag (028), which builds the macOS zip too.

## Steps

1. [ ] how `scripts/release.ps1` and the `release` job render the page today (`about.toml`, the pinned
   `CARGO_ABOUT_VERSION`, the template), so the macOS side reuses the same config and version
2. [ ] the `bundle-macos` job installs the same cargo-about and renders the page for `aarch64-apple-darwin` and
   `x86_64-apple-darwin`; `scripts/bundle-macos.sh` copies it into the zip root next to `usage.md`
3. [ ] `deny.toml` / `about.toml` cover the darwin targets; `cargo deny check` passes with them
4. [ ] docs: `docs/development.md` (the macOS bundle), the header comment of `scripts/bundle-macos.sh`
5. [ ] local checks (CONTRIBUTING.md), `uvx zizmor .github`; a manual `bundle-macos` run on the branch shows the
   page in the zip with the darwin-only crates (objc2 …)

## Log

- 2026-10-19 — promoted on `task/041-third-party-licences-in-the-macos-zip`.

## Deviations from the plan

- none yet

## Verification

- 

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
