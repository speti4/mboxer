---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: folytassuk-a-009-taskkal-peaceful-sunbeam.md
---

# 009 — Gmail labels and conversation threads

## Context

Since 005/006 the folders pane shows a flat label list with counts and a click is an exact `labels` filter;
`thread_id` (X-GM-THRID) is indexed + stored + FAST, but nobody reads it back. 009 adds two things: a
conversation view in the list (one row = one thread with the newest message and the count, expandable, the
grouping in the core `Listing`, because the members are scattered in the date-sorted list) and a label tree
in the folders pane (`/` hierarchy + the Takeout `Category …` labels under one *Categories* node).
Why: Takeout gives the Gmail structure for free, this is what makes the flat archive navigable.
Done when: a click on a label narrows the list; a message with replies shows as one expandable
conversation.

User decisions 2026-10-17 (planning round): grouping on by default, a *Conversations* switch on the filter strip;
a click on a parent label = parent + every sub-label (OR, `any_labels`); `Category *` → *Categories*
node; oldest first in an expanded thread. The index format does not change (no rebuild); no new
crate (D-02). The real archive's header sample (600 MB) has no label with `/` and none RFC 2047-encoded.

## Steps

1. [x] Core `search.rs`: `SearchQuery.any_labels` (Should-klauzula), `group_threads`, `Hit.thread_id`
2. [x] Core `listing.rs`: `Entry` collector (`date` + `thread_id` oszlop), `arrange` CSR, `Expanded` prefix-összeg,
   `locate` / `row_of_group`, `ListRow` / `RowKind`, `rows` / `toggle` / `position_of` / `len` / `message_count` /
   `group_count`; unit tesztek; exportok
3. [x] Fixture `threads.mbox`; `tests/index.rs` esetek; `cargo test -p mboxer-core`
4. [x] App `types.rs` (`QueryInput`, `Row` kind, `ListInfo`, `PositionInfo`, `LabelNode`), `folders.rs` (`label_tree`,
   `Bin`), `state.rs` (`load` grouping-gal), `commands.rs` (`open_mbox`, `toggle_thread`, `position_of`), `lib.rs`; tesztek
5. [x] UI `index.html` + `style.css`; `main.js` címkefa, thread sorok, toggle + layout token, billentyűk, státusz, kapcsoló
6. [x] `node --check` + stub-DOM check (`stripReplyPrefix`, `labelsUnder`, fa-sorrend, kijelölés-eltolás)
7. [x] README; Log / Deviations / Verification; backlog item az RFC 2047 címkenevekre
8. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; debug exe a fixture-másolaton és a 14 GB-on
   (stderr `query:` idők); user kézi kör; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 009 promoted to wip
- 2026-10-17 — core: `SearchQuery.any_labels` (Should-TermQuery-k egy Must-klauzulában) és `group_threads`,
  `Hit.thread_id`; `listing.rs` újraírva: az `AllMatches` collector `Entry`-ket gyűjt (`date` + `thread_id`
  FAST oszlop, `NonZeroU64` niche), `arrange` CSR-be rendez (ungrouped ág bitre a mai sorrend), a sor-aritmetika
  a `Layout` structban (`Expanded` prefix-összeg, `partition_point`), `ListRow`/`RowKind`, `toggle`,
  `position_of` auto-kibontással. Fixture `threads.mbox` (6 üzenet, B3 a fájl végén de a legrégebbi,
  `Workshop`/`Work-archive` negatív esetek). Első futás: egy unit-teszt várakozása volt rossz (a 2-es thread
  legújabb tagja 14 > a thread nélküli 13 → a thread előz), a kód jó. 29 unit + 8 + 19 core zöld
- 2026-10-17 — app: `LabelNode` fa (`folders.rs` `label_tree`: `/` szegmensek, `Category <x>` → `Categories`
  csomópont, `Bin`/`Chat` a well-known listában), `QueryInput.anyLabels/groupThreads`, `Row.kind/threadCount/
  expanded/threadIndex` (thread id nem megy ki: u64 > 2^53), `ListInfo.messages/conversations`, `PositionInfo`,
  `open_mbox(groupThreads)` → `OpenMbox::load` grouping-gal, `toggle_thread(generation, row)`; a CLI literál
  a két új mezővel. Egy tesztvárakozás javítva (a `Categories` csomópont a well-known blokk után, a user-címkék
  előtt áll). `cargo test` 30 app + 4 + 5 CLI + 29 + 8 + 19 core zöld; fmt + clippy tiszta
- 2026-10-17 — UI: `Conversations` checkbox (`#f-threads`, localStorage `mboxer.groupThreads`, a Clear nem bántja),
  címkefa chevronnal és `--depth` behúzással (konténer: `total` dőlten; kulcs = exact címke vagy `<út>#`),
  `labelsUnder(key)` → `anyLabels`; a lista 6 oszlopos sor (`twisty, att, from, subject, count, date`),
  thread sor `stripReplyPrefix`-szel és darabszám-jelvénnyel, member sor behúzva; `toggleThread` a
  `state.layout` tokennel (a `requestBlock` eldobja a régi layout lapjait, a `pending` csak a saját promise-át
  törli), a kijelölés `max(row, selected + delta)`; ArrowRight nyit / ArrowLeft zár a listában és a fában;
  `position_of` → `PositionInfo.total` átvétele. `node --check` tiszta; stub-DOM check 35 eset, 0 hiba
- 2026-10-17 — `target\debug\mboxer.exe` a `threads.mbox` másolatán: `opened threads.mbox: 6 messages, Built
  { build_secs: 0, parse_failures: 0 }, 92 ms`; a 14 GB-on hideg cache-sel 1352 ms, melegen 88 ms (a 005/006
  ungrouped listingje ~60 ms volt ugyanebben a debug buildben → a 86 085 üzenet csoportosítása ~30 ms). Mindkét
  ablak nyitva maradt a user kézi köréhez. Follow-up 015 (RFC 2047-kódolt címkenevek) felvéve a backlogba
- 2026-10-17 — implementation committed as `636afe7`; waiting for the user's manual round
- 2026-10-17 — user's manual round on the fixture copy and the 14 GB archive: "minden ok". Follow-ups: 015 filed
  earlier (RFC 2047 label names); the "only matching members in a conversation" limit stays under Deviations as
  accepted. Closed

## Deviations from the plan

- `setTotal` (toggle vagy `position_of` utáni sorszám-változás) minden cache-elt lapot eldob, nem csak a thread
  sor blokkjától lefelé: egyszerűbb, és a látható ablak 1–3 lapja újratöltődik, ami 200 soros lapoknál nem látszik.
- A `Layout` struct (sor-aritmetika index nélkül tesztelve) a terv "a listing mezői" helyett külön típus; a
  `Listing` delegál rá. Az API a tervezett.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  (új: `label_tree` 5 eset — rendezés `Bin`-nel és a `Categories` csomóponttal, `/` hierarchia + `labels` +
  `total`, konténer-szülő, Takeout-kategóriák, furcsa címkék; `QueryInput` `anyLabels`/`groupThreads`; `Row`
  kind serde, `ListInfo`/`PositionInfo`/`LabelNode` alak; `conversations_are_grouped_from_the_first_listing`) +
  4 + 5 CLI + 29 core unit (új: `arrange` 3 eset, `Layout` sor-aritmetika 1 000 csoporttal, member-sor kibontás,
  ungrouped layout) + 8 fixtures + 19 index (új 7: ungrouped 6 sor, grouped 3 sor, toggle, `position_of`
  auto-kibontás, csak találó tagok relevance alatt is, `any_labels`, dátum nélküli tag) pass.
- 2026-10-17 — `node --check ui/main.js` clean; stub-DOM check (scratchpad `ui-check-009.js`, a `main.js`-ből
  kiszelt függvényeken): `stripReplyPrefix` 8 eset, a címkefa renderelése (látható nevek, mélység, kulcsok,
  konténer `total` vs saját count, chevronok, szeparátor), `labelsUnder` 5 eset, kijelölés a kulcsok szerint,
  összezárás/kibontás kattintással és a twisty `stopPropagation`-je, a toggle kijelölés-szabálya 6 eset:
  35 eset, 0 hiba.
- 2026-10-17 — `target\debug\mboxer.exe` a fixture-másolaton: 6 üzenet, index 92 ms alatt épült; a 14 GB-on
  újrahasznált index, hideg cache 1352 ms, meleg 88 ms (grouping ≈ 30 ms debug buildben, cél < 100 ms teljesül).
- 2026-10-17 — user's manual round on the fixture copy and on the 14 GB archive: conversations show as one
  row with the count, the chevron / ArrowRight opens them oldest first and ArrowLeft closes them, a label click
  narrows to the label and its sub-labels (Categories node included), the selection survives a narrowing with
  the conversation opened around it, the Conversations switch returns to one row per message, the status bar
  counts conversations and messages. Reported as "minden ok". Done-when met: a label click narrows the list;
  a message with replies shows as one expandable conversation.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — 015 filed; one accepted limit under Deviations
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
