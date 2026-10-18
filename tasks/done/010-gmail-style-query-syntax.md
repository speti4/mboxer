---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: j-backlog-tletek-expressive-breeze.md
---

# 010 — Gmail-style query syntax in the search field

## Context

Today the search field's text goes unchanged into tantivy's lenient parser: `"frázis"`, `OR` and
`-szó` work because of that, and `from:anna` only because `from` is a schema field name. `label:`,
`has:attachment`, `subject:`, `after:` / `before:` do nothing, and an operator typed into the field does
not show up on the filter panel or in the label tree. 010 makes the field a real query language for the
main user, the panel stays for the others. Why: power-user speed. Done when: every operator has a
test; a typed operator shows up on the panel and vice versa.

User decisions 2026-10-17 (planning round): **the field's text is the single source of truth** — the panel and
a label click rewrite the text (`from:anna`, `after:2026-01-01`, `label:Inbox`), typing updates the panel
and the label selection; it follows that Esc and Clear empty the whole query, the label too.
**`before:` is an inclusive** day, the same the panel's "to" field shows (differs from Gmail,
documented in the README). The parser lives in the UI (`ui/query.js`, it has to be there for the two-way
sync), the CLI keeps its flags; the core's only change is the `subject` filter. No new runtime crate (D-02).
Size S → M: the field became the model, which touches every panel handler.

Note from 006: `state.query` is the shared model of the field, the panel and the folders pane; the core
`typing_word` / `prefix_last_word` and the UI `highlightTerms` already leave operator-looking words
alone.

## Steps

1. [x] Core `SearchQuery.subject` + Must-klauzula (`search.rs`), end-to-end teszt (`tests/index.rs`); app
   `QueryInput.subject` + mapping teszt (`types.rs`); CLI `--subject` + `describe_filters` + teszt
2. [x] `ui/query.js`: fold helperek átköltöztetve, `tokenize`, `parse`, `setOperator`, `highlightTerms`;
   `index.html` script tag
3. [x] `crates/mboxer-app/tests/ui/query.test.js` zöld `node --test` alatt; `tests/ui_query.rs` wrapper
   (node nélkül skip)
4. [x] `main.js`: `state.parsed` + `syncFromText` + `renderQueryViews`, `queryInput`, panel-handlerek és
   `selectLabel` a `setOperator`-on át, badge / narrowing / warnings, reset
5. [x] README (operátor-táblázat, `before:` inkluzív, címkekattintás → `label:`, Esc / Clear, CLI `--subject`);
   `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `node --check`
6. [x] Chrome-ellenőrzés a DOM-szinkronra (stubbed `invoke`), majd debug exe a 14 GB-on: user kézi kör →
   Verification; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 010 promoted to wip
- 2026-10-17 — core: `SearchQuery.subject` + Must-klauzula a meglévő `text_clause`-on (prefix is megy rá),
  end-to-end teszt (`szamla` 1, `köszönöm` csak body-ban → 0, `janu` prefix 1); app `QueryInput.subject` +
  shape-teszt; CLI `--subject` + `describe_filters`. `cargo test` első futásra zöld
- 2026-10-17 — UI: új `ui/query.js` (UMD: `window.MboxerQuery` / CommonJS): `tokenize` (idézőjel-csoportosítás,
  `from:"a b"`), `classify`, `normalizeDate`, `parse` (az utolsó, szóköz nélküli token "gépelés alatt": az
  értéke nem számít, nincs warning), `setOperator` (helyben csere, duplikátumok törlése, hiányzó a végére,
  üres érték = törlés egy whitespace-futammal), `highlightTerms` (frázis egy term, `from:`/`to:`/`subject:`
  értékei is), a fold helperek átköltöztek ide. `tests/ui/query.test.js` 16 eset `node --test` alatt, első
  futásra zöld; `tests/ui_query.rs` futtatja cargo alól (node nélkül skip)
- 2026-10-17 — `main.js`: `state.query` = `{text, groupThreads, sort}`, `state.parsed` a `syncFromText`-ből;
  `rewrite(name, value)` a panel-handlerek és a `selectLabel` mögött; `indexLabelTree` (teljes fa, összecsukott
  csomópontok is → `labelsUnder` már nem esik vissza exact termre egy összecsukott gyereknél), `findLabelKey`,
  `labelName` (`Work#/Clients#` → `Work/Clients`); `showWarnings` a szintaxis-warningokat a backendé elé teszi.
  `node --check` tiszta, `cargo fmt` (egy assert reflow), clippy `-D warnings` tiszta, 96 teszt zöld
- 2026-10-17 — Chrome-ellenőrzés: a valódi `index.html` + `query.js` + `main.js` 127.0.0.1:8766-ról, a stub
  `__TAURI__.core.invoke` 4 címkés fát ad vissza (`Work` és `Categories` konténer); 36 ellenőrzés
  `javascript_tool`-ból mind zöld: gépelt operátor → panel + címkekijelölés, panel From/Attachments/dátum →
  szöveg helyben átírva, címkekattintás → `label:` (szóközös név idézőjelben, konténer név `#` nélkül), All
  mail töröl, rossz dátum warning a státuszsorban, félkész dátum nem, két `label:` → anyLabels + labels,
  `subject:` eljut a backendig, Esc mindent ürít, badge 3. Debug exe buildelve a kézi körhöz
- 2026-10-17 — user kézi kör a 14 GB-on: "fasza minden". Follow-up sweep: nincs nyitott lépés, a `to:` és a
  `subject:` panel-mező nélkül marad a terv szerint (szintaxis-only, README-ben dokumentálva) — nincs új
  backlog tétel. Closed

## Deviations from the plan

- none: a terv szerinti fájlok és viselkedés; egy apró plusz a `labelsUnder` javítása (teljes fa indexe,
  összecsukott csomópontok is), mert a gépelt `label:` egy összecsukott konténert is elérhet

## Verification

- 2026-10-17 — `cargo fmt --check` tiszta, `cargo clippy --all-targets -- -D warnings` tiszta, `cargo test`
  96 zöld: 30 app unit (a `QueryInput` shape `subject`-tel) + 1 `ui_query` wrapper + 4 + 5 CLI (`--subject`
  2 eset) + 29 + 8 + 19 core (`subject` szűrő + prefix). `node --test` 16 eset (minden operátor, tokenizer,
  `setOperator`, round-trip, `highlightTerms`, `normalizeDate`) zöld; `node --check` mindkét UI fájlra tiszta.
- 2026-10-17 — Chrome (127.0.0.1:8766, stub backend, valódi `index.html` + `query.js` + `main.js`): 36
  DOM-szinkron ellenőrzés mind zöld (lásd a Log bejegyzést).
- 2026-10-17 — `target\debug\mboxer.exe` a 14 GB-os archívumon (86 085 üzenet, index újrahasznosítva, open
  183 ms), stderr-rel: 47 `query:` sor, medián 1 ms, max 843 ms (12 099 találatos lista, debug build) — a
  "< 1 s billentyűleütésenként" cél tartja. User kézi kör: gépelt operátor → panel és címkefa követi, panel
  → mező helyben átírva, címkekattintás → `label:`, Esc / Clear mindent ürít, betűnkénti prefix-frissítés
  marad. Done-when teljesül: minden operátornak van tesztje, a mező és a panel mindkét irányban szinkronban.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none found
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
