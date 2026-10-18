---
size: S
origin: session 2026-10-18
created: 2026-10-18
started: 2026-10-18
closed: 2026-10-18
milestone: M5 Public release
plan: peppy-imagining-pony.md
---

# 026 — README as a front page, the reference moved into docs

## Context

The README rewritten as a front page: a one-line pitch, badges (CI, latest release, licence), a screenshot of
the viewer over a synthetic mbox (never the author's archive), a feature list, download and quick start, and
links to usage, build and decisions. The current long reference moves to `docs/reference.md`, or into
`docs/usage.md` where it addresses the user. Why: today the README is a dense reference with no picture, and a
visitor decides within one screen. Done when: the README fits on two screens and every moved paragraph has a home
under `docs/`. Planning decisions 2026-10-18 (approved plan): the viewer and CLI reference goes to
`docs/reference.md`; layout, CI, release and scripts go to `docs/development.md`; build prerequisites and checks go
to `CONTRIBUTING.md` (023). The screenshot is taken over a demo mbox that `scripts/make-demo-mbox.py` generates
(stdlib only, deterministic, made-up mail under `example.*` domains), which can also serve the social preview in 028.

## Steps

1. [x] `scripts/make-demo-mbox.py` → `target/demo/demo.mbox`; `mboxer-cli scan` / `index` / `search` rajta
2. [x] Képernyőkép a demo mboxon → `docs/images/viewer.png`
3. [x] `docs/reference.md` (viewer + CLI referencia) és `docs/development.md` (layout, CI, release, scriptek)
4. [x] Új `README.md` front page (pitch, badge-ek, kép, features, download, build, docs, status, licenc)
5. [x] `docs/usage.md`: link a referenciára; `CLAUDE.md` Scope: az új docs fájlok
6. [x] Ellenőrzés: sorszám, minden régi bekezdésnek megvan az új helye, relatív linkek; `tasks_check`; commit

## Log

- 2026-10-18 — plan approved; 026 promoted to wip (023-mal együtt)
- 2026-10-18 — `make-demo-mbox.py`: 50 üzenet, 42 862 bájt, két futás SHA-256-ja azonos (`069f772c…`);
  `mboxer-cli scan`: 50 üzenet, 11 csatolmányos, 10 label, 0 parse failure; `index` 0.1 s; `search kave` → 1 találat
  (a „kávé” a v2.0.0 release levélben), `--label Work/Invoices --count` → 4
- 2026-10-18 — képernyőkép: scratchpad `capture-viewer.ps1` (debug exe, `cargo build` előtte; a crate-ek
  utolsó commitja óta friss). Védelem a valódi adatok ellen: eldobható WebView2 profil
  (`WEBVIEW2_USER_DATA_FOLDER`, minden futás előtt törölve), így a megjegyzett fájllista (a 14 GB archívum) nem
  nyílik meg; a valódi profil legfrissebb fájlja előtte-utána ugyanaz (04:20:55). `--lang=en-US`
  (`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`) az angol dátumokhoz; az mbox `subst M:` alatt Takeout-szerű úton
  (`M:\Takeout\Mail\All mail Including Spam and Trash.mbox`), hogy a status bar ne mutasson lokális utat; a subst
  utána törölve. 1440×860, a list/reader splitter húzva, *Light* téma, a Go-live conversation nyitva, a
  csatolmányos üzenet kijelölve, a címsor levágva (31 px) → 1426×822, 105 KB
- 2026-10-18 — a képkészítés közben látott két UI apróság backlogra került: 031 (a natív címsor sötét marad a
  Light override alatt dark Windows-on), 032 (a dátum oszlop elcsúszik a conversation és a sima sorok között)
- 2026-10-18 — `README.md` 146 → 71 sor; scratchpad `check-docs.py`: 0 törött relatív link 8 fájlban, a régi README
  50 jellegzetes kifejezéséből 49 szó szerint megvan az új helyén, az 50. („one-way into”) átfogalmazva
  („mirrored one way into”)

## Deviations from the plan

- A képernyőkép a debug exe-ből készült, nem a release-ből: a `target\release` exe a 016/017 előtti; a UI ugyanaz,
  a status bar verziója is (v0.1.0)
- A címsort levágtam a képről (a 031-es hiba miatt, és a front page-en a natív keret amúgy is zaj)
- `docs/development.md` egy „The demo mbox” szakaszt is kapott (a script és a képkészítés szabálya)

## Verification

- 2026-10-18 — `README.md` 71 sor (a „két képernyő” határon belül); 0 törött relatív link a 8 érintett fájlban;
  a régi README minden bekezdésének van otthona (`docs/reference.md`, `docs/development.md`, `CONTRIBUTING.md`),
  50 kifejezéses greppel ellenőrizve
- 2026-10-18 — a user megnézte a README-t és a képernyőképet, rendben („szuper, mehet a push”); commit `f640447`,
  pusholva, a `sync-issues` futás zöld
- A badge-ek a privát repón nem renderelnek; ez a 028 checklistjére került (README badges a switch után)

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-18`
