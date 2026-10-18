---
status: ready
size: S
origin: wip/024-supply-chain-and-ci-hardening.md
created: 2026-10-18
milestone: M5 Public release
---

# Install the Rust toolchain in CI without a third-party action

What: drop `dtolnay/rust-toolchain` from `ci.yml`; the runner's preinstalled rustup installs the toolchain and the
components that `rust-toolchain.toml` names (`rustup show` or the first `cargo` call), in the test and release jobs.
Why: the repo has no release tags, so the action is pinned to a `master` commit that dependabot cannot follow (its
run reports "Latest version is 1, no update needed") and the pin would go stale silently; zizmor's pedantic
persona flags it as `ref-version-mismatch`. One third-party action fewer in the release job.
Done when: CI is green without the action and the test log shows the toolchain from `rust-toolchain.toml`.
