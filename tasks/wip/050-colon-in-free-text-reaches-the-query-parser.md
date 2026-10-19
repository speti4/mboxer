---
size: M
origin: session 2026-10-19
created: 2026-10-19
started: 2026-10-19
plan: reviewold-a-repo-jelenlegi-fancy-kite.md
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

1. [ ] failing tests in `crates/mboxer-core/tests/index.rs` over a fixture: `Re: számla`, `re:számla`, a URL with
   `https://`, `10:30`, `"Re: számla"` as a phrase, `-spam` alone, `számla*`; the expected hit set written down first
2. [ ] the fix in `search.rs`: escape or hand-build the free-text terms so no word becomes a field, a range, a boost
   or a wildcard the user did not type; `typing_word` / `prefix_query` keep working for the word being typed
3. [ ] `parse_query_lenient` warnings: an exclusion-only query (`-spam`) no longer surfaces `AllButQueryForbidden`
   to the CLI and the UI (either a match-all base clause or a clear message)
4. [ ] a Node test for `ui/query.js` showing that `Re:` is not an operator and reaches the backend as free text
5. [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; `docs/reference.md` says
   what a colon in free text means now

## Log

- 2026-10-19 — promoted from the backlog in the main session; the work runs in its own worktree.

## Deviations from the plan

- none yet

## Verification

- (pending)

## Closing checklist

- [ ] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [ ] Verification written, dated
- [ ] `tasks_check` green, README index regenerated (the index is the main session's, see docs/maintainer-sessions.md)
- [ ] moved to `done/` with `closed: YYYY-MM-DD`
