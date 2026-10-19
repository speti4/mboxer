---
status: idea
size: S
origin: wip/028-publication-day-repo-settings.md
created: 2026-10-19
---

# CI extras once the minutes are free: MSRV, changelog check, fork guard

What: a `cargo +1.90 check --locked` job for the declared `rust-version`; `scripts/changelog.py --check` in the
lint job; `if: github.repository == 'speti4/mboxer'` on the issue sync; the zizmor cache-poisoning ignore scoped
to the two cache lines; `* text=auto eol=lf` in `.gitattributes`; `release.ps1` falls back to `py -3`;
`peak-memory.ps1` quotes its arguments like `benchmark.ps1`.
Why: the pre-publication review of the tooling found these; on a public repo the extra jobs cost nothing.
Done when: each item is in place or dropped with a reason in the Log, CI green.
