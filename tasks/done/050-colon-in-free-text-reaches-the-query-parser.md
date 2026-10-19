---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
closed: 2026-10-19
plan: none
milestone: M6 Hardening
---

# 050 — A colon in free text is read as a field name, so `Re: számla` finds nothing

## Context

What: the free text of a query goes to tantivy's lenient parser as is (`parse_text` in
`crates/mboxer-core/src/index/search.rs`), and the query grammar (`tantivy-query-grammar`, `field_name` allows
whitespace around `:`) turns `Re: számla` into a search of the unknown field `Re`, which the lenient parser drops;
`számla 10:30` or a URL loses words the same way. Escape the grammar characters (`:`, `*`, `~`, `^`, brackets,
braces) in every free-text word before the parse, or build the term queries by hand; keep the quoted-phrase
behaviour and the prefix-of-the-last-word behaviour of `text_clause`.
Why: `Re:` and `Fwd:` are the most common words in a subject line; a search that contains one silently matches
nothing.
Done when: `Re: számla`, a URL, a time, a quoted phrase with a colon and a leading `-` return the expected hits in
the index tests, from the CLI and through the UI's `query.js` tokenizer, and `cargo test` is green.
Decided 2026-10-19: filed by the post-release review; the Gmail-style operators stay in `ui/query.js`, the core
only ever sees free text, so the fix lives in the core.

## Steps

1. [x] failing tests in `crates/mboxer-core/tests/index.rs` over a fixture: `Re: számla`, `re:számla`, a URL with
   `https://`, `10:30`, `"Re: számla"` as a phrase, `-spam` alone, `számla*`; the expected hit set written down first
2. [x] the fix in `search.rs`: escape or hand-build the free-text terms so no word becomes a field, a range, a boost
   or a wildcard the user did not type; `typing_word` / `prefix_query` keep working for the word being typed
3. [x] `parse_query_lenient` warnings: an exclusion-only query (`-spam`) no longer surfaces `AllButQueryForbidden`
   to the CLI and the UI (either a match-all base clause or a clear message)
4. [x] a Node test for `ui/query.js` showing that `Re:` is not an operator and reaches the backend as free text
5. [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` says
   what a colon in free text means now

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.
- 2026-10-19 — new fixture `crates/mboxer-core/tests/fixtures/free-text-syntax.mbox`: five made-up
  `example.com` messages (`Re: számla február`, `Számla március` with `10:30` in the body, `Re: ebéd után` with
  `számla` and an `https://example.com/…` URL in the body, `Heti spam jelentés`, `Számlázás`). Three tests in
  `tests/index.rs` with the hit sets written first: `grammar_characters_in_free_text_are_plain_text`,
  `free_text_keeps_and_or_not_and_the_trailing_star`, `the_word_being_typed_after_a_colon_is_still_a_prefix`.
  Red before the fix: `Re: számla` returned the warning `Field does not exist: 'Re'`, and
  `számla ebéd OR március` matched four messages instead of two.
- 2026-10-19 — decided: build the term and phrase queries by hand (`crates/mboxer-core/src/index/free_text.rs`)
  instead of escaping the words for `QueryParser::parse_query_lenient`. Reason: tantivy's grammar has more
  readings than a colon (`[a TO b]` range, `^2` boost, `~2` slop, `/x/` regex, `'single quotes'`, `IN [...]`,
  a trailing `*` that fails as a one-term phrase prefix), and its escapes are not uniform (`interpret_escape`
  keeps the backslash before `*`, `~`, `!`, `+`), so every grammar change of a tantivy upgrade could reopen the
  bug. The hand-built grammar is small: words AND-ed, `"phrases"` (an unterminated quote runs to the end), `OR`
  binding tighter than the implicit AND, `-word`, `-"phrase"` and `NOT word` to exclude, `AND` accepted, a
  trailing `*` as a prefix; everything else is plain text. Each word goes through `fold_analyzer`, the index's
  own analyzer, so accent folding stays; a word that folds to several tokens (`re:számla`, `10:30`, a URL) is a
  phrase over them, as tantivy's parser did for `anna.pelda@example.com`. The subject boost (2.0) is a
  `BoostQuery` on the subject field; the word being typed is part of the same grammar (a prefix when the text
  does not end in whitespace, the word is neither quoted nor negated and ends in a letter or digit, so `Re:` is a
  finished word and `re:száml` completes as a phrase prefix). `typing_word`, `prefix_query`, `parse_text` and
  `text_clause` are gone from `search.rs`; the unit tests of the grammar sit beside it.
- 2026-10-19 — exclusions: a query with only exclusions gets an `AllQuery` base clause, so `-spam` lists every
  other message; text without a searchable word (`*`, `:`) is no condition. The parser no longer produces
  warnings, so `SearchResult::warnings` and `Listing::warnings` stay empty (the fields are kept for the
  viewer's status bar). `MboxIndex` no longer keeps the `Index` handle, which only the parser read.
- 2026-10-19 — tantivy 0.27 debug-asserts in `PhraseScorer::seek_danger` when its `Exclude` probes an excluded
  phrase with a document before the phrase's first match (`számla -"spam jelentés"` panicked in the debug test
  build). Excluded clauses are wrapped in `Excluded`, whose
  scorer answers `seek_danger` with the default `DocSet` implementation.
- 2026-10-19 — Node test in `crates/mboxer-app/tests/ui/query.test.js`: `Re: számla from:anna` keeps
  `Re: számla ` as free text and fills only `from`; `re:számla`, `Fwd: Re: 10:30`, `"Re: számla"` and a URL stay
  free text without a warning. `docs/reference.md` describes the free-text syntax and the colon as plain text,
  also for `mboxer-cli search`; `docs/usage.md` does not mention the colon and is unchanged.
- 2026-10-19 — CLI over a copy of the fixture: `mboxer-cli search <copy> "Re: számla"` → `hits: 2`
  (`Re: ebéd után`, `Re: számla február`), `"re:számla"` → 1, `"10:30"` → 1 (`Számla március`),
  `"https://example.com/invoice/2026"` → 1 (`Re: ebéd után`), `'"Re: számla"'` → 1, `"számla*"` → 5,
  `-- -spam` → 4; no `warning:` line in any of them.
- 2026-10-19 — follow-ups are listed in the pull request description for the main session to file: report the
  tantivy assert upstream and drop the wrapper after a fixed release, decide on the always-empty query
  warnings, and split the highlighted words of `ui/query.js` on punctuation the way the core does.

## Deviations from the plan

- `-spam` gets a match-all base clause rather than a message, and the colon-less grammar replaces tantivy's
  parser for the `from`, `to` and `subject` filters too (they shared `text_clause`).
- `OR` now binds tighter than the implicit AND (`a b OR c` is `a AND (b OR c)`); tantivy's parser read the mix
  differently. `AND`, `NOT`, parentheses and `+` are no longer tantivy syntax: `AND` and `NOT` are kept as
  keywords, parentheses and `+` are punctuation.
- A trailing `*` is a prefix (it was an error in tantivy's parser), so `számla*` finds `Számlázás`.
- The `Excluded` wrapper around excluded clauses works around a tantivy debug assertion; it was not planned.

## Verification

- 2026-10-19, Windows 11, in the task worktree: `cargo fmt --all --check` clean, `cargo clippy --all-targets --
  -D warnings` clean, `cargo test` green (mboxer-core: 33 unit tests, 10 fixture tests, 25 index tests;
  mboxer-cli: 4 + 6; mboxer-app: 42 unit tests and the Node tests of `ui/` through `ui_query`).
- 2026-10-19, the CLI round over a copy of `free-text-syntax.mbox` in the Log: every query returns the hit set of
  the index tests, `-spam` without a warning.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [x] moved to `done/` with `closed: YYYY-MM-DD`
