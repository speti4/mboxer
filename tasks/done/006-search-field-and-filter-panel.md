---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M2 Viewer MVP
plan: folytassuk-a-backlog-k-vetkez-purring-cupcake.md
---

# 006 — Search field and filter panel

## Context

A search field (Ctrl+F / Ctrl+K focus, the list refreshing while typing with a debounce) and a
filter panel (date range, sender, label, attachment) that combines with the text search; the
matched words highlighted in the list. This is the reason the project exists: search one keystroke away.
Done when: a user who knows no syntax finds a message by body word + sender in three interactions, at < 1 s
per keystroke.

The backend already knows everything (`QueryInput` → `SearchQuery` → `MboxIndex::list`, the `set_query` and
`position_of` commands exist); most of the work is under `ui/`, no new crate.

Decided 2026-10-17 (planning round): the label filter stays the folders
pane, the panel does not duplicate it (the label tree belongs to 009); the dates are local days, the "to"
is inclusive (the core `before` is exclusive, the CLI's UTC day stays); the list sorts by date for a text
search too, with a Sort: Newest first / Best match selector in the panel (`SortInput::Relevance` is
already in the backend); highlighting in the list (from + subject) and in the reader's plain-text body
too, 007 carries it into the iframe; JS-side folding (`NFD` + combining marks + lowercase), tokens
starting with `-` or containing `:` are not highlighted (the 010 syntax is untouched).

## Steps

1. [x] Backend `set_query` async (`spawn_blocking`, the state from the `AppHandle`), a stderr `query:` line for measuring; `list_rows`, `position_of` stay sync
2. [x] Rust tests: `state.rs` text + from + date window in one listing; `types.rs` the full `QueryInput` shape
3. [x] HTML: `#search` in the toolbar, `#filters-btn` with a badge, the `#filters` strip (from, after, before, attachments, sort, clear), `#list-empty`
4. [x] CSS: 4-row body grid, search field, filter strip, `.badge`, `mark`, `#list-empty`
5. [x] JS query model: `state.query`, `queryInput()`, `applyQuery()` token guard + 150 ms debounce; `selectLabel` through it; warnings, empty state, badge
6. [x] Selection kept after a narrowing (`position_of`, the reader does not clear)
7. [x] Highlighting: `foldText`, `highlightTerms`, `renderHighlighted` in the list and the reader
8. [x] Keys: Ctrl+F / Ctrl+K, Ctrl+Shift+F, Enter / ArrowDown / Escape in the field and the panel
9. [x] README: search, panel, highlighting, local days, keys
10. [x] Check on the 14 GB file (headless stderr + manual round) → Verification; notes into 007 and 010
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 006 promoted to wip
- 2026-10-17 — `set_query` is an async command now (`spawn_blocking`, the state reached through
  `AppHandle::state` inside the closure, the mutex never held across an await); the two new app tests
  (`text_and_filters_combine_in_one_listing`, `the_full_query_input_shape_maps_onto_a_search_query`)
  passed on the first run, 13 app unit tests green
- 2026-10-17 — UI written: search field + filter strip in `index.html`, 4-row body grid and the
  `mark` / badge / empty-list styles, `main.js` with the `state.query` model, token-guarded
  `runQuery`, selection kept through `position_of`, the folding highlighter. Node 24 is on the
  machine, so the highlighter's pure functions got a stub-DOM check (scratchpad script, 10 cases:
  folding, operator stripping, overlapping matches, emoji offsets): one failure — touching matches
  gave two adjacent marks — fixed by merging ranges; all 10 green afterwards. `node --check` clean
- 2026-10-17 — `cargo fmt` (one long assert reflowed), `cargo clippy --all-targets -- -D warnings`
  clean, `cargo test` 13 + 4 + 5 + 20 + 7 + 11 green; debug exe built and launched on the 14 GB
  archive with stderr captured for the manual round
- 2026-10-17 — manual round, first finding: `száml` → "No messages match", `számla` → hits. Whole-word
  index vs per-keystroke refresh; fixed with the prefix-of-the-last-word clause (see Deviations).
  tantivy's parser only accepts `*` on a multi-word phrase, so the single-word case is built by hand
  from `PhrasePrefixQuery`. New tests: 2 unit (`typing_word`) + 1 end-to-end (text and from
  prefixes, trailing space, one letter, the listing path) + the app's `QueryInput` shape; the CLI's
  explicit `SearchQuery` literal gained the field. fmt / clippy clean, 13 + 4 + 5 + 22 + 7 + 12 green.
  First-round stderr: 28 `query:` lines, median 0 ms, max 73 ms on the reused index
- 2026-10-17 — second manual round on the rebuilt exe passed. Follow-ups: no new task; two
  notes appended to existing backlog items — 007 (carry the body highlighting into the srcdoc iframe)
  and 010 (`state.query` is the sync point of the syntax and the panel; `typing_word` / `highlightTerms`
  already leave operator-looking words alone). Closed

## Deviations from the plan

- The word being typed matches as a prefix. The plan sent the text to the whole-word index as is;
  the first manual check showed `száml` giving "No messages match" until the last letter
  arrived, which defeats a list that refreshes per keystroke. Core: `SearchQuery.prefix_last_word`
  (the viewer sends it, the CLI leaves it off); `typing_word` splits off the last plain word when
  the text does not end in whitespace, the word carries no syntax and no phrase is open before it;
  the word's folded tokens become a `PhrasePrefixQuery` per search field (a single term turns
  into a bounded dictionary range, 200 expansions in dictionary order, so the shortest completions
  come first), AND-ed to the lenient parse of the rest. Applies to the text, from and to clauses;
  a one-letter word and a trailing space stay whole-word. The UI keeps the trailing space of the
  field so the backend can tell a finished word from one still being typed; the substring
  highlighter already marks the prefix inside the full word.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  13 app unit (2 new: text + sender + date window in one listing, the full `QueryInput` shape with
  `prefixLastWord`) + 4 + 5 CLI + 22 core unit (2 new `typing_word`) + 7 + 12 core end-to-end (1 new: the
  word being typed as a prefix on text and from, trailing space, one letter, the listing path) pass.
- 2026-10-17 — the highlighter's pure functions under Node 24 with a stub DOM (scratchpad script, 10 cases):
  folding, operator and quote stripping, overlapping and touching matches merged into one mark, emoji
  offsets; all green. `node --check ui/main.js` clean.
- 2026-10-17 — `target\debug\mboxer.exe "<14 GB mbox>"` with stderr captured, two rounds on the reused index
  (open 59 / 58 ms, 86 085 messages): 28 + 47 `query:` lines, median 0 / 2 ms, max 73 / 104 ms per
  keystroke in a debug build — the "< 1 s per keystroke" target holds with a wide margin.
- 2026-10-17 — manual rounds on the 14 GB archive. Round one found the whole-word gap (`száml` →
  no match); after the prefix fix round two passed every point: letter-by-letter typing refreshes the
  list with the prefix highlighted in the subject and the body, the From field narrows, the date window
  and the attachment filter narrow, a label click combines with the panel and Clear returns to All mail,
  the selected message stays selected through a narrowing, an empty result shows "No messages match", an
  unclosed quote shows a warning in the status bar, Esc / Enter / ArrowDown move the focus as described.
  Done-when met: a body word plus a sender in three interactions
  (Ctrl+F, the word, From), well under a second per keystroke.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — notes went into 007 and 010, no new item
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
