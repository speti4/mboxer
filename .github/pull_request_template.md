## What and why

<!-- What changes, and why. Link the issue (Fixes #N) or the task file (tasks/wip/NNN-….md). -->

## Follow-ups

<!-- Work this change found but did not do, one line each; the maintainer files them as tasks. "none" is fine. -->

## Checklist

- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass
- [ ] No real mail anywhere: fixtures, tests, screenshots and the description use made-up messages only
- [ ] A new crate has its one-line justification in `Cargo.toml` (one exe, D-02), or there is none
- [ ] User-facing changes are described in `docs/usage.md` and `docs/reference.md`
- [ ] The commits pass the hook (`git config core.hooksPath .githooks`); no session link in this description
