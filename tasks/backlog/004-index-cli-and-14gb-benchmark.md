---
status: ready
size: S
origin: session 2026-10-17
created: 2026-10-17
milestone: M1 Index core
---

# Index CLI and the 14 GB benchmark

What: A thin CLI over 002 + 003: `mboxer index <mbox>`, `mboxer search <mbox> <query> [--from --after --before --has-attachment]`, with progress output and timings.
Why: Proves M1 without any UI and gives a repeatable benchmark (build time, index size, RAM, query latency) to protect against regressions.
Done when: A dated benchmark table for the real 14 GB mbox sits in `docs/` and the numbers meet the M1 roadmap targets.
