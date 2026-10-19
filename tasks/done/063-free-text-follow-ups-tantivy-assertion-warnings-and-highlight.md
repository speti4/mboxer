---
size: S
origin: done/050-colon-in-free-text-reaches-the-query-parser.md
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 063 — Free-text follow-ups: the highlight split and the tantivy exclusion assertion

## Context

What: two leftovers of task 050. `highlightTerms` in `ui/query.js` highlights whole free words, while the core
analyzer splits a word on punctuation, so `re:számla` finds `Re: számla` but does not highlight it; split the same
way. `free_text.rs` wraps an excluded phrase in an `Excluded` query because tantivy 0.27's
`PhraseScorer::seek_danger` trips a debug assertion under `Exclude`; prepare the upstream report with the fixture
that shows it, so the wrapper can go once a fixed tantivy is in.
Why: an unhighlighted hit looks like a false positive, and a workaround without an upstream ticket outlives the bug.
Done when: a highlight test covers `re:számla` against `Re: számla`, and the upstream report (title, minimal
reproduction, the tantivy version) is ready in the task file for the maintainer to post.
Decided 2026-10-19: the upstream issue is posted by the maintainer, not by the task; the task delivers the text and
a standalone reproduction (a small Rust test against tantivy alone, or the fixture and the query that trips it).

## Steps

1. [x] a failing Node test in `crates/mboxer-app/tests/ui/query.test.js`: the highlight of `re:számla`, `10:30`
   and a URL covers the same pieces the core analyzer indexes (`Re`, `számla`, `10`, `30`, …), accent-folded
2. [x] `highlightTerms` splits a free word on the punctuation the core's tokenizer splits on (read
   `crates/mboxer-core/src/index/schema.rs` and `free_text.rs` for the exact rule) and keeps phrases whole
3. [x] the tantivy reproduction: a minimal test that builds a two-document index and runs an excluded phrase
   query in a debug build, showing the assertion; its text, the tantivy version and the suggested title in the
   Log; a comment at the `Excluded` wrapper names the report
4. [x] `cargo test` (Node tests included), `node --check ui/query.js`, `cargo fmt --check`, `cargo clippy
   --all-targets -- -D warnings`

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — the split rule. `fold_analyzer` in `schema.rs` is tantivy's `SimpleTokenizer` (a piece is a run of
  `char::is_alphanumeric` characters, everything else cuts), then `RemoveLongFilter` (40 bytes), `LowerCaser` and
  `AsciiFoldingFilter`. `free_text.rs` searches a word or a quoted phrase as its pieces in order, one position
  apart. The UI mirrors it in `query.js` (one comment above the highlighting section names `schema.rs`): a term is
  the folded pieces of a word or phrase joined by one space (`re:számla` → `re szamla`, `10:30` → `10 30`,
  `https://example.com/a-b` → `https example com a b`), split on `[^\p{Alphabetic}\p{N}]+`, the JS form of
  `!char::is_alphanumeric`. The new `termRanges(folded, terms)` finds a one-piece term with `indexOf` as before
  and a term of several pieces with a cached pattern that allows any run of non-alphanumeric characters between
  the pieces, so `re:számla` marks `Re: számla` and `RE:SZAMLA` but not a lone `számla`; the last piece stays a
  substring, so `re:száml` still marks while typing. `main.js` takes its ranges from `termRanges`.
- 2026-10-19 — red before the change: the two new Node tests failed (`highlightTerms("re:számla")` gave
  `['re:szamla']`, `highlightTerms('"Re: számla"')` gave `['re: szamla']`, and `termRanges` did not exist); green
  after it, 53 Node tests pass.
- 2026-10-19 — the tantivy side. `Cargo.lock` pins tantivy 0.27.0. `PhraseScorer::new_with_offset` moves to the
  phrase's first match when it is built; `Exclude::new` (and `advance` / `seek`) asks the exclusion scorer
  `seek_danger(candidate)` for every candidate of the include side, so a candidate before the phrase's first match
  is a target behind `doc()`, and `PhraseScorer::seek_danger` opens with
  `debug_assert!(target >= self.doc(), "target ({}) should be greater than or equal to doc ({})", …)`. The
  default `DocSet::seek_danger` answers such a target with `SeekLowerBound(doc)`, which is why an excluded term is
  fine and why the `Excluded` wrapper, whose scorer falls back to the default, works. `PhrasePrefixScorer` calls the
  same `seek_danger`. Release builds compile the assertion out and count right.
- 2026-10-19 — upstream state, read-only through the GitHub API: tantivy `main` at `67889269ca` (2026-10-17) has
  `phrase_scorer.rs` and `exclude.rs` byte-identical to the 0.27.0 crate; the last commits on `phrase_scorer.rs`
  are `79eedf4a81` (2026-09-03, "Added the capacity to build scorer and seek_danger in a joint manner. (#3070)",
  already in 0.27.0) and `3f448ecf79` (2026-01-27, "Bugfix on intersection. (#2812)"). So `main` has not changed
  `seek_danger`. An issue on exactly this already exists and is open: quickwit-oss/tantivy#3193, "`PhraseScorer::
  seek_danger` debug assertion fails when a phrase is excluded (`MustNot`) and its first match is after the first
  candidate", filed 2026-10-15 against 0.26.2 with the same analysis and a proposed fix (answer
  `SeekLowerBound(self.doc())` when `target < self.doc()`), no comments yet. Issue #2811 (closed 2026-01-27)
  covered a backward `seek` on the same scorer, not `seek_danger`.
- 2026-10-19 — the report, ready for the maintainer. As #3193 exists, the useful post is a comment on it that
  confirms 0.27.0 and `main`, rather than a new issue; the title below is for a new issue only if #3193 is closed
  without a fix.
  - Suggested title: "`PhraseScorer::seek_danger` debug assertion fails when a phrase is excluded (`MustNot`)
    and its first match is after the first candidate (still in 0.27.0)"
  - Text: "Still reproduces on tantivy 0.27.0 and on `main` at 67889269ca (`phrase_scorer.rs` and `exclude.rs`
    unchanged). Minimal reproduction below, tantivy alone, `cargo run` in a debug build. We work around it by
    wrapping the excluded query in a scorer that falls back to the default `DocSet::seek_danger`."
  - Panic in a debug build (`cargo run`), file and line from the 0.27.0 crate:
    `panicked at …/tantivy-0.27.0/src/query/phrase_query/phrase_scorer.rs:578:9:` /
    `target (0) should be greater than or equal to doc (1)`. A release build prints `1`.
  - `Cargo.toml`:

    ```toml
    [package]
    name = "tantivy-excluded-phrase"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    tantivy = "=0.27.0"
    ```

  - `src/main.rs`:

    ```rust
    use tantivy::collector::Count;
    use tantivy::query::{BooleanQuery, Occur, PhraseQuery, Query, TermQuery};
    use tantivy::schema::{IndexRecordOption, Schema, TEXT};
    use tantivy::{Index, IndexWriter, Term, doc};

    fn main() -> tantivy::Result<()> {
        let mut schema = Schema::builder();
        let text = schema.add_text_field("text", TEXT);
        let index = Index::create_in_ram(schema.build());
        let mut writer: IndexWriter = index.writer_with_num_threads(1, 15_000_000)?;
        writer.add_document(doc!(text => "invoice for march"))?; // doc 0: a candidate
        writer.add_document(doc!(text => "weekly spam report invoice"))?; // doc 1: the phrase's first match
        writer.commit()?;

        let term = |word: &str| Term::from_field_text(text, word);
        let query = BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(TermQuery::new(term("invoice"), IndexRecordOption::Basic)) as Box<dyn Query>,
            ),
            (
                Occur::MustNot,
                Box::new(PhraseQuery::new(vec![term("spam"), term("report")])),
            ),
        ]);
        let searcher = index.reader()?.searcher();
        // Debug build: panics in PhraseScorer::seek_danger. Release build: prints 1.
        println!("{}", searcher.search(&query, &Count)?);
        Ok(())
    }
    ```

- 2026-10-19 — the same reproduction lives in the repo as `crates/mboxer-core/tests/tantivy_excluded_phrase.rs`
  (tantivy alone, no mboxer code): in a debug build it expects the panic (`#[should_panic]` with the message
  above), in a release build it checks the count of 1. A tantivy that fixes the assertion makes the debug test
  fail, which is the signal to drop the `Excluded` wrapper and the test; the comment at the wrapper in
  `free_text.rs` names #3193, its title and this task.

## Deviations from the plan

- Step 1 named the pieces (`re`, `számla`, `10`, `30`, …) as what the highlight covers. The highlight matches the
  pieces of one word in order instead of each piece anywhere, as the core's phrase does: a lone piece like `a` (from
  `a-b`) or `10` would otherwise mark every such letter or number in a message. A match marks the span from the
  first piece to the last, so `Re: számla` is one mark, like a phrase before.
- A quoted phrase follows the same rule (its pieces in order, any punctuation between), as the core analyzes it the
  same way: `"big bad"` now also marks `Big, bad`, where it marked only `big bad` before.
- `main.js` changed at its one call site (`highlightFragments` takes its ranges from `termRanges`), and
  `docs/reference.md` gained one clause on how a punctuated word is highlighted.
- The reproduction is a passing test (`#[should_panic]` in debug, a count check in release) rather than an
  `#[ignore]`d one, so a fixed tantivy shows up as a failing test.
- The upstream report is prepared as a comment on the existing #3193, with a title kept for a new issue.

## Verification

- 2026-10-19 — in the worktree, Windows 11: `node --check` on `ui/query.js`, `ui/main.js` and
  `tests/ui/query.test.js`; `node --test crates/mboxer-app/tests/ui/*.test.js`: 53 pass, the two new tests failed
  before the change; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test`: every
  suite green, `ui_query` (the Node tests) and `tantivy_excluded_phrase` included;
  `cargo test --release -p mboxer-core --test tantivy_excluded_phrase`: the release variant passes (count 1);
  `cargo test -p mboxer-core --test tantivy_excluded_phrase -- --nocapture` shows the panic text quoted in the Log.
  With `DOCS_RS=1`: `cargo clippy --target aarch64-apple-darwin --all-targets --locked -- -D warnings` and
  `cargo +1.90 check --target aarch64-apple-darwin --all-targets --locked` pass; `cargo deny check`,
  `uvx zizmor .github` and `python -I scripts/render-usage-guide.py --check` pass (`docs/reference.md` changed).
  No manual round in the viewer: the change is the pure functions in `query.js` and one call site.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
