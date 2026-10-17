---
size: S
origin: wip/009-gmail-labels-and-threads.md
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M3 Comfort
plan: 12-14-15-taskokat-fejlessz-k-le-keen-whale.md
---

# 015 — Decode RFC 2047-encoded Gmail label names

## Context

A `X-Gmail-Labels` fejlécet a mail-parser ismeretlen fejlécként nyersen adja vissza (`parse_raw`), így egy
encoded-word címke (`=?UTF-8?Q?Sz=C3=A1ml=C3=A1k?=`) kódolt formában kerül az indexbe és a folders pane-be,
és a kattintás nem talál rá. Megoldás: a parser `header_text("X-Gmail-Labels")` beállítása (unstructured parse,
RFC 2047 dekódolva, továbbra is egy `as_text()` érték a `split_labels`-nek) és `FORMAT_VERSION` 1 → 2, mert
a tárolt címkék változnak. Why: a szerző archívumában nincs kódolt címke (600 MB fejlécminta, 009), de egy
kolléga nem-ASCII címkenevei torzan jelennének meg. Done when: egy kódolt címkés fixture dekódolva listázza
a címkét a `label_counts`-ban, az exact és az `any_labels` szűrő a dekódolt névre talál, és a meglévő index
egyszer újraépül (formátumverzió). User döntés 2026-10-17 (tervezési kör): sorrend 015 → 014 → 012, egy
commit taskonként; nincs új crate (D-02).

## Steps

1. [x] `message.rs`: egy megosztott `LazyLock<MessageParser>` `header_text("X-Gmail-Labels")`-szel
2. [x] `fingerprint.rs`: `FORMAT_VERSION` 2
3. [x] Fixture `labels-encoded.mbox` (Q-encoded, B-encoded vesszővel és `/`-sal, nyers UTF-8)
4. [x] Tesztek: `tests/index.rs` (`label_counts`, exact, `any_labels`, tárolt címkék), `tests/fixtures.rs` (`summarize`)
5. [x] README "Labels" bekezdés
6. [x] `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; 14 GB: egyszeri rebuild, utána "up to date"
7. [x] `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 015 promoted to wip
- 2026-10-17 — első próba `MessageParser::default().header_text("X-Gmail-Labels")`-szel: 4 fixture-teszt elbukott
  (Subject kódolva maradt, Message-ID `<>`-vel, nincs attachment). Ok: a mail-parser 0.11 a beépített
  fejléctáblát csak üres `header_map` mellett használja (`parsers/header.rs:35`); bármi konfigurálva → csak a
  konfigurált fejlécek. A `PARSER` ezért a teljes beépített táblát sorolja fel (`with_mime_headers` +
  `with_date_headers` + `with_address_headers` + `with_message_ids` + Subject/Comments text, List-* address,
  Return-Path id, Keywords/Content-Language comma, Received, MIME-Version raw) és ehhez adja az `X-Gmail-Labels`
  text parse-t; minden más ismeretlen fejléc nyers marad. Egy `LazyLock` static, nem üzenetenként épített parser
- 2026-10-17 — fixture `labels-encoded.mbox` (3 üzenet: Q-encoded, B-encoded `Munka/Számlák,Inbox` egy encoded
  wordben, nyers UTF-8); `cargo test`: 30 app + 1 + 4 + 5 CLI + 29 + 9 + 20 core zöld; clippy, fmt tiszta
- 2026-10-17 — a leállítva felejtett `target\debug\mboxer.exe` (010 kézi kör) zárolta a bin-t; bezárva
- 2026-10-17 — 14 GB: release CLI `index` → format 1 → 2 rebuild 126,9 s, 86 085 üzenet, 0 parse failure,
  index 134,8 MB; második futás "index is up to date"; closed

## Deviations from the plan

- A parser nem `default() + header_text`, hanem a beépített tábla kézzel felsorolva + `header_text` (lásd Log):
  a mail-parser viselkedése miatt másképp nem megy; a fixture-tesztek (Subject, From, Message-ID, Date,
  attachments, legacy charsets) bizonyítják, hogy a tábla teljes.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`: 30 app
  + 1 ui_query + 4 + 5 CLI + 29 core unit + 9 fixtures (új: `encoded_label_names_are_decoded_in_the_summary`)
  + 20 index (új: `encoded_label_names_are_decoded_before_they_are_stored` — `label_counts` =
  `Inbox 3, Munka/Számlák 1, Számlák 2`; exact `Számlák` 2 találat, a tárolt címkékben nincs `=?`;
  `any_labels [Munka, Munka/Számlák]` 1 találat; a kódolt alak mint címke 0 találat) pass.
- 2026-10-17 — a 14 GB archívum: `mboxer-cli index` egyszer újraépített (format 1 → 2, 126,9 s, 86 085 üzenet,
  0 parse failure, 134 833 535 byte), a második `index` "up to date"; `search --label Inbox --count` 65 646,
  `search számla --count` 1137 (= 003), open 8,7 ms. Done-when met.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
