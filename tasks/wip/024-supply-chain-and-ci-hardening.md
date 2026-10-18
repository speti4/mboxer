---
status: ready
size: M
origin: session 2026-10-18
created: 2026-10-18
milestone: M5 Public release
---

# Supply chain and CI hardening: dependabot, cargo-deny, pinned actions

What: `.github/dependabot.yml` for cargo and github-actions (weekly, with a cooldown); every action pinned by
commit SHA with a version comment, on releases past the Node 20 deprecation; `permissions: contents: read` as the
workflow default, `concurrency` and `timeout-minutes` on every job; `cargo-deny` (licences, advisories, bans,
sources) as a CI job with a `deny.toml`; a workflow linter (`zizmor`) in CI. Build-time tools only, each with
its D-02 justification line.
Why: a public repo that ships an exe needs a visible, automatic answer to "are the dependencies and the
pipeline trustworthy", and the licence check is a precondition for the third-party notices (025).
Done when: CI is green with `cargo deny check` and the workflow lint, and the first dependabot PR has arrived.
