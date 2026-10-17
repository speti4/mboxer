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

Egy keresőmező (Ctrl+F / Ctrl+K fókusz, gépelés közben frissülő lista debounce-szal) és egy
szűrőpanel (dátumtartomány, feladó, címke, melléklet), ami a szöveges kereséssel kombinálódik; a
találati szavak kiemelve a listában. Ez a projekt létezésének oka (az mboxviewer Advanced Find
fájdalma). Done when: egy szintaxist nem ismerő user három interakcióval megtalál egy levelet
body-szó + feladó alapján, billentyűleütésenként < 1 s.

A backend már tud mindent (`QueryInput` → `SearchQuery` → `MboxIndex::list`, a `set_query` és
`position_of` parancs megvan); a munka zöme a `ui/` alatt van, nincs új crate.

User döntések 2026-10-17 (tervezési kör, a terv jóváhagyásával): a címke-szűrő a folders pane
marad, a panel nem duplikálja (a címkefa a 009-é); a dátumok helyi napok, a "to" inkluzív
(a core `before` exkluzív, a CLI UTC-napja marad); a lista szöveges keresésnél is dátum szerint
rendez, a panelben Sort: Newest first / Best match választó (a `SortInput::Relevance` már a
backendben van); a kiemelés a listában (from + subject) és a reader plain-text body-jában is, a
007 viszi át az iframe-be; JS-oldali folding (`NFD` + combining jelek + lowercase), a `-`-szal
kezdődő és `:`-ot tartalmazó tokenek nem kerülnek kiemelésre (a 010 szintaxisa érintetlen).

## Steps

1. [x] Backend `set_query` async (`spawn_blocking`, state az `AppHandle`-ból), stderr `query:` sor a méréshez; `list_rows`, `position_of` sync marad
2. [x] Rust tesztek: `state.rs` text + from + dátumablak egy listingben; `types.rs` teljes `QueryInput` alak
3. [x] HTML: `#search` a toolbarban, `#filters-btn` jelvénnyel, `#filters` sáv (from, after, before, attachments, sort, clear), `#list-empty`
4. [x] CSS: 4 soros body grid, keresőmező, szűrősáv, `.badge`, `mark`, `#list-empty`
5. [x] JS query-modell: `state.query`, `queryInput()`, `applyQuery()` token-guard + 150 ms debounce; `selectLabel` ezen át; warnings, empty state, jelvény
6. [x] Kijelölés megtartása szűkítés után (`position_of`, reader nem ürül)
7. [x] Kiemelés: `foldText`, `highlightTerms`, `renderHighlighted` a listában és a readerben
8. [x] Billentyűk: Ctrl+F / Ctrl+K, Ctrl+Shift+F, Enter / ArrowDown / Escape a mezőben és a panelen
9. [x] README: keresés, panel, kiemelés, helyi napok, billentyűk
10. [x] Ellenőrzés a 14 GB-on (headless stderr + user kézi kör) → Verification; jegyzet a 007-be és 010-be
11. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test` zöld; `tasks_check`; commit

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
- 2026-10-17 — second manual round on the rebuilt exe: "minden működik!". Follow-ups: no new task; two
  notes appended to existing backlog items — 007 (carry the body highlighting into the srcdoc iframe)
  and 010 (`state.query` is the sync point of the syntax and the panel; `typing_word` / `highlightTerms`
  already leave operator-looking words alone). Closed

## Deviations from the plan

- The word being typed matches as a prefix. The plan sent the text to the whole-word index as is;
  the user's first manual check showed `száml` giving "No messages match" until the last letter
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
- 2026-10-17 — user's manual rounds on the 14 GB archive. Round one found the whole-word gap (`száml` →
  no match); after the prefix fix round two passed every point: letter-by-letter typing refreshes the
  list with the prefix highlighted in the subject and the body, the From field narrows, the date window
  and the attachment filter narrow, a label click combines with the panel and Clear returns to All mail,
  the selected message stays selected through a narrowing, an empty result shows "No messages match", an
  unclosed quote shows a warning in the status bar, Esc / Enter / ArrowDown move the focus as described.
  Reported as "minden működik!". Done-when met: a body word plus a sender in three interactions
  (Ctrl+F, the word, From), well under a second per keystroke.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — notes went into 007 and 010, no new item
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
