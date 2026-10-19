---
status: ready
size: M
origin: session 2026-10-19
created: 2026-10-19
milestone: M6 Hardening
---

# A colon in free text is read as a field name, so `Re: számla` finds nothing

What: the free text of a query goes to tantivy's lenient parser as is (`parse_text` in
`crates/mboxer-core/src/index/search.rs`), and the query grammar allows whitespace after `:`, so `Re: számla` becomes
a search of the unknown field `Re`, which is dropped, and `számla 10:30` or a URL loses words the same way. Escape
the grammar characters (`:`, `*`, `~`, `^`, brackets, braces) in every free-text word before the parse, or build the
term queries by hand; keep the quoted-phrase behaviour. Cover `Re: számla`, a URL, a time, a quoted phrase with a
colon, and a leading `-` in the core tests, from the CLI and through the UI's `query.js` tokenizer.
Why: `Re:` and `Fwd:` are the most common words in a subject line; a search that contains one silently matches
nothing.
Done when: the listed queries return the expected hits in the index tests and `cargo test` is green.
