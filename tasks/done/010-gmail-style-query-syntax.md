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

A keresőmező szövege ma változtatás nélkül a tantivy lenient parserébe megy: a `"frázis"`, az `OR` és a
`-szó` ezért működik, a `from:anna` pedig csak azért, mert a `from` séma-mezőnév. A `label:`,
`has:attachment`, `subject:`, `after:` / `before:` semmit sem csinál, és a mezőbe írt operátor nem
jelenik meg sem a szűrőpanelen, sem a címkefában. A 010 a mezőt teszi valódi lekérdezőnyelvvé a fő
user számára, a panel a többieknek marad. Why: power-user sebesség. Done when: minden operátornak van
tesztje; egy begépelt operátor megjelenik a panelen és fordítva.

User döntések 2026-10-17 (tervezési kör): **a mező szövege az egyetlen igazságforrás** — a panel és a
címkekattintás a szöveget írja át (`from:anna`, `after:2026-01-01`, `label:Inbox`), a gépelés a panelt
és a címkekijelölést frissíti; ebből következik, hogy az Esc és a Clear a teljes lekérdezést üríti, a
címkét is. **`before:` inkluzív** nap, ugyanaz, amit a panel "to" mezője mutat (eltér a Gmailtől,
README-ben dokumentálva). A parser a UI-ban él (`ui/query.js`, a kétirányú szinkronhoz ott kell), a CLI
a flag-jeit tartja; a core egyetlen változása a `subject` szűrő. Nincs új runtime crate (D-02). Méret S
→ M: a mező lett a modell, ami minden panel-handlert érint.

Jegyzet a 006-ból: `state.query` a mező, a panel és a folders pane közös modellje; a core
`typing_word` / `prefix_last_word` és a UI `highlightTerms` már békén hagyja az operátor-kinézetű
szavakat.

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
