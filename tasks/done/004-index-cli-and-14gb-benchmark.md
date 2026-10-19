---
size: S
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M1 Index core
plan: folytassuk-a-004-task-scalable-trinket.md
---

# 004 — Index CLI filters and the 14 GB benchmark

## Context

003 brought in the `mboxer index` / `mboxer search` commands with progress and open/search timings; the
numbers of the 14 GB run are in the 003 Verification (122 s build, 314 MB peak, 129 MB index,
search < 1 ms). What is left of 004 (decided 2026-10-17 during the 003 planning): the `search`
filter flags (`--from --to --after --before --label --has-attachment`, onto the already existing fields
of the core `SearchQuery`), a repeatable benchmark (build time, index size, peak RAM, query latency in
one table) and the dated benchmark table in `docs/benchmarks/`, which closes M1.
Done when: the dated benchmark table of the 14 GB mbox is in `docs/`, and the numbers meet the M1
roadmap targets (reopen < 1 s, body search < 1 s, build in acceptable time).

No new build-time crate: the date is parsed by the already pulled-in `mail_parser::DateTime`, the benchmark
PowerShell script follows the pattern of `scripts/peak-memory.ps1` (the exe stays thin, D-002).

## Steps

1. [x] CLI filters in `crates/mboxer-cli/src/main.rs`: `query` optional; `--from --to --after --before --label --has-attachment/--no-attachment --offset --count`; a `filters:` line in the output
2. [x] Tests: `parse_date` unit; `crates/mboxer-cli/tests/cli.rs` (`CARGO_BIN_EXE_mboxer`, fixture copied into a temp folder, filters, error without an index)
3. [x] `scripts/benchmark.ps1`: environment, `index --force` under the peak WS sampler, re-open, median of a query set → markdown
4. [x] Run on the 14 GB file → `docs/benchmarks/2026-10-17-14gb-takeout.md` (hand-written header: M1 targets vs measured)
5. [x] `README.md`: `index`/`search` usage, benchmark script, Layout
6. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 004 promoted to wip
- 2026-10-17 — filters, `parse_date` (mail-parser `parse_rfc3339`, a day gets `T00:00:00Z`) and the CLI end-to-end
  tests compiled and passed on the first `cargo test` (4 unit + 5 integration); `tempfile` added as a CLI dev-dep
  (already in the build)
- 2026-10-17 — `scripts/benchmark.ps1` smoke-tested on a fixture copy; two fixes before the real run: a phrase
  argument that already carries quotes must not be quoted again, and an absolute `-Out` path must not be joined
  to the cwd
- 2026-10-17 — first 14 GB run: the phrase row showed 84 hits where 003 had measured 7. Cause: `Start-Process`
  hands `"számla melléklet"` to the Windows argv parser, which strips the quotes, so the exe ran the AND of two
  words. Fix: embedded quotes are escaped (`\"`) in the command line; verified directly (`"számla melléklet"` → 7,
  `számla melléklet` → 84) and the benchmark re-run. The report's commit field now gets `-dirty` when the working
  tree has changes.
- 2026-10-17 — second 14 GB run built the index in 35 s instead of 125 s: the 13 GB file was in the OS file cache
  (31 GB RAM) after the first run, so the build became CPU-bound (401 MB/s). Both numbers are in the benchmark
  doc's header; the 003 target (< 1 GB peak) holds at 314 MB in both runs. No follow-up: a cold-cache open was not
  measurable without dropping the Windows file cache, recorded as an expectation in the doc instead. Closed.

## Deviations from the plan

- The re-open latency is measured with `Measure-Command` around the whole `mboxer index` process, not under the
  peak-memory sampler: the 200 ms sampling loop quantised a 17 ms run to 224 ms.
- No peak working set per search: a `mboxer search` process lives ~20 ms, one sample at most, so the column would
  have shown the process start-up footprint (5 MB) rather than anything about the index. The build's peak memory
  (a 2-minute process, sampled every 20 ms) stays.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  4 CLI unit (`parse_date`, `describe_filters`) + 5 CLI end-to-end (`tests/cli.rs`: missing index → exit 1 with
  the hint, accent folding + timings lines, every filter alone and combined, `--count`/`--offset`, bad date and
  conflicting flags → exit 2) + the unchanged 20 + 7 + 9 core tests pass.
- 2026-10-17 — `scripts\benchmark.ps1 -Repeat 5` on the real 14 GB archive, report in
  `docs/benchmarks/2026-10-17-14gb-takeout.md` (numbers only): build 124.9 s cold cache / 35.4 s warm, peak working
  set 314.5 MB, index 128.8 MB (0.95 %), re-open 20 ms (whole process), open 7 ms + search 0.5–0.8 ms per query over
  86 085 messages; `szamla` = `számla` = 1137 hits, phrase 7, `--has-attachment --count` 15 941 (= the 002 scan's
  count). Every M1 target met.
- 2026-10-17 — manual check on the real index: `számla --after 2020-01-01 --before 2021-01-01` 173 hits, with
  `--has-attachment` fewer; `--from <a real address>` returns hits (count only, not recorded).
- 2026-10-17 — `tasks_check` (write): 0 errors, 0 warnings; README index regenerated.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none left
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
