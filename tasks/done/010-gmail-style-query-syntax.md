---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: none
---

# 010 — Gmail-style query syntax in the search field

## Context

Today the search field's text goes unchanged into tantivy's lenient parser: `"phrase"`, `OR` and
`-word` work because of that, and `from:anna` only because `from` is a schema field name. `label:`,
`has:attachment`, `subject:`, `after:` / `before:` do nothing, and an operator typed into the field does
not show up on the filter panel or in the label tree. 010 makes the field a real query language for
keyboard users, the panel stays for everyone else. Why: power-user speed. Done when: every operator has a
test; a typed operator shows up on the panel and vice versa.

Decided 2026-10-17 (planning round): **the field's text is the single source of truth** — the panel and
a label click rewrite the text (`from:anna`, `after:2026-01-01`, `label:Inbox`), typing updates the panel
and the label selection; it follows that Esc and Clear empty the whole query, the label too.
**`before:` is an inclusive** day, the same the panel's "to" field shows (differs from Gmail,
documented in the README). The parser lives in the UI (`ui/query.js`, it has to be there for the two-way
sync), the CLI keeps its flags; the core's only change is the `subject` filter. No new runtime crate (D-002).
Size S → M: the field became the model, which touches every panel handler.

Note from 006: `state.query` is the shared model of the field, the panel and the folders pane; the core
`typing_word` / `prefix_last_word` and the UI `highlightTerms` already leave operator-looking words
alone.

## Steps

1. [x] Core `SearchQuery.subject` + Must clause (`search.rs`), end-to-end test (`tests/index.rs`); app
   `QueryInput.subject` + mapping test (`types.rs`); CLI `--subject` + `describe_filters` + test
2. [x] `ui/query.js`: fold helpers moved over, `tokenize`, `parse`, `setOperator`, `highlightTerms`;
   `index.html` script tag
3. [x] `crates/mboxer-app/tests/ui/query.test.js` green under `node --test`; `tests/ui_query.rs` wrapper
   (skipped without node)
4. [x] `main.js`: `state.parsed` + `syncFromText` + `renderQueryViews`, `queryInput`, the panel handlers and
   `selectLabel` through `setOperator`, badge / narrowing / warnings, reset
5. [x] README (operator table, `before:` inclusive, label click → `label:`, Esc / Clear, CLI `--subject`);
   `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --check`
6. [x] Chrome check of the DOM sync (stubbed `invoke`), then the debug exe on the 14 GB file: manual round →
   Verification; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 010 promoted to wip
- 2026-10-17 — core: `SearchQuery.subject` + a Must clause on the existing `text_clause` (the prefix applies to it
  too), end-to-end test (`szamla` 1, `köszönöm` only in a body → 0, `janu` prefix 1); app `QueryInput.subject` +
  shape test; CLI `--subject` + `describe_filters`. `cargo test` green on the first run
- 2026-10-17 — UI: new `ui/query.js` (UMD: `window.MboxerQuery` / CommonJS): `tokenize` (quote grouping,
  `from:"a b"`), `classify`, `normalizeDate`, `parse` (the last token without a trailing space is "being typed":
  its value does not count, no warning), `setOperator` (replaces in place, removes duplicates, appends a missing
  one, an empty value deletes it along with one whitespace run), `highlightTerms` (a phrase is one term, the
  values of `from:`/`to:`/`subject:` too), the fold helpers moved here. `tests/ui/query.test.js` 16 cases under
  `node --test`, green on the first run; `tests/ui_query.rs` runs it under cargo (skipped without node)
- 2026-10-17 — `main.js`: `state.query` = `{text, groupThreads, sort}`, `state.parsed` from `syncFromText`;
  `rewrite(name, value)` behind the panel handlers and `selectLabel`; `indexLabelTree` (the whole tree, collapsed
  nodes too → `labelsUnder` no longer falls back to the exact term for a collapsed child), `findLabelKey`,
  `labelName` (`Work#/Clients#` → `Work/Clients`); `showWarnings` puts the syntax warnings before the backend's.
  `node --check` clean, `cargo fmt` (one assert reflowed), clippy `-D warnings` clean, 96 tests green
- 2026-10-17 — Chrome check: the real `index.html` + `query.js` + `main.js` from 127.0.0.1:8766, the stub
  `__TAURI__.core.invoke` returns a tree of 4 labels (`Work` and `Categories` containers); 36 checks from
  `javascript_tool`, all green: a typed operator → panel + label selection, panel From/Attachments/date → the
  text rewritten in place, label click → `label:` (a name with a space in quotes, a container name without `#`),
  All mail clears it, a bad date warns in the status bar, a half-typed date does not, two `label:` → anyLabels +
  labels, `subject:` reaches the backend, Esc clears everything, badge 3. Debug exe built for the manual round
- 2026-10-17 — manual round on the 14 GB file passed. Follow-up sweep: no open step; `to:` and `subject:` stay
  without a panel field as planned (syntax only, documented in the README) — no new backlog item. Closed

## Deviations from the plan

- none: the planned files and behaviour; one small extra is the `labelsUnder` fix (index of the whole tree,
  collapsed nodes too), because a typed `label:` can reach a collapsed container

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`
  96 green: 30 app unit (the `QueryInput` shape with `subject`) + 1 `ui_query` wrapper + 4 + 5 CLI (`--subject`
  2 cases) + 29 + 8 + 19 core (`subject` filter + prefix). `node --test` 16 cases (every operator, tokenizer,
  `setOperator`, round trip, `highlightTerms`, `normalizeDate`) green; `node --check` clean on both UI files.
- 2026-10-17 — Chrome (127.0.0.1:8766, stub backend, the real `index.html` + `query.js` + `main.js`): 36
  DOM sync checks, all green (see the Log entry).
- 2026-10-17 — `target\debug\mboxer.exe` on the 14 GB archive (86 085 messages, index reused, open
  183 ms), with stderr: 47 `query:` lines, median 1 ms, max 843 ms (a list of 12 099 hits, debug build) — the
  "< 1 s per keystroke" target holds. Manual round: a typed operator → the panel and the label tree follow, panel
  → the field rewritten in place, label click → `label:`, Esc / Clear empty everything, the letter-by-letter
  prefix refresh still works. Done-when met: every operator has a test, the field and the panel stay in sync in
  both directions.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none found
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
