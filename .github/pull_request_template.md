## What and why

<!-- What changes, and why. Link the issue (Fixes #N) or the task file (tasks/wip/NNN-….md). -->

## Follow-ups

<!-- Work this change found but did not do, one line each; the maintainer files them as tasks. "none" is fine. -->

## Checklist

- [ ] `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass
- [ ] The macOS-target clippy and the `rust-version` check pass (CONTRIBUTING, 'Checks before a commit')
- [ ] `python -I scripts/changelog.py --check` passes; a closed task brings its `CHANGELOG.md` line
- [ ] When `Cargo.toml`, `Cargo.lock`, `.github/` or the docs change: `cargo deny check`, `uvx zizmor .github`
      and `python -I scripts/render-usage-guide.py --check` pass, or none of them changed
- [ ] No real mail anywhere: fixtures, tests, screenshots and the description use made-up messages only
- [ ] A new crate has its one-line reason in `Cargo.toml`, so the shipped artifact stays one exe or app
      (CONTRIBUTING, 'Dependencies: one exe'), or there is none
- [ ] User-facing changes are described in `docs/usage.md` and `docs/reference.md`
- [ ] The commits pass the hook (`git config core.hooksPath .githooks`)
- [ ] Tested on: Windows / macOS / not platform-specific
