---
size: M
origin: session 2026-10-17
created: 2026-10-17
started: 2026-10-17
closed: 2026-10-17
milestone: M2 Viewer MVP
plan: folytassuk-a-007-taskkal-stateful-castle.md
---

# 007 — Message reader with sanitized HTML

## Context

A reader ma a plain text részt mutatja (`<pre id="msg-body">`), HTML-only levélnél a html→text
konverziót. A 007 cseréli ezt: a HTML body sandboxolt `srcdoc` iframe-ben renderelődik, scriptek
kiszedve, távoli tartalom alapból blokkolva (üzenetenkénti "Show" gomb), HTML / Text / Source
nézetváltó, Copy gomb, a body-ban kattintott linkek a rendszer-browserben nyílnak; a 006 kiemelése
átmegy a sanitizált HTML-be. Why: a HTML mail nézzen ki mailnek; az mboxviewer rosszul rendereli, a
WebView2 natívan tudja. Done when: bármely levél olvashatóan renderelődik, és engedélyezés nélkül
egy hálózati kérés sem megy ki az appból.

User döntések 2026-10-17 (tervezési kör): "open in browser" = csak a body linkjei nyílnak a
rendszer-browserben, nincs külön "üzenet megnyitása browserben" gomb; a sanitizer a webview-ban fut
(JS, DOMParser — a user rám hagyta, a nulla-crate megoldás), alatta két független réteg: a sandbox
iframe (`allow-same-origin`, scriptek nélkül) és az iframe saját meta CSP-je (`default-src 'none'`);
a 013 (`>From ` sorok visszaalakítása a plain text body-ban) beleolvad ebbe a taskba.

Build-time crate (D-02 indoklás): `tauri-plugin-opener 2` — a http/https/mailto linkek megnyitása
a rendszer alapértelmezett alkalmazásával, Rust oldalról hívva (`OpenerExt::open_url`, capability
nem kell); a 008 melléklet-megnyitása (`open_path`) ugyanezt használja majd.

Jegyzet 005-ből: a shell CSP-je már enged `frame-src 'self' blob:`-ot; az `about:srcdoc` betöltését
a `frame-src 'self'` alatt ellenőrizni kell, fallback a `blob:` URL. Jegyzet 006-ból: a plain text
body kiemelését (`renderHighlighted`) a srcdoc iframe-nek is vinnie kell.

## Steps

1. [x] Cargo: `tauri-plugin-opener` workspace + app dep indoklással; `lib.rs` plugin + parancsok
2. [x] `types.rs` `html` mező; `view.rs` `html` + `unescape_from_lines` (013) + tesztek; új fixture `html-remote.mbox`
3. [x] `commands.rs` `read_source`, `open_url` + `allowed_link` teszt; `tauri.conf.json` CSP img-src
4. [x] `index.html` + `style.css`: reader bar, remote bar, iframe, segmented gombok
5. [x] `main.js`: `classifyUrl`, `scrubCss`, `sanitizeHtml`, `htmlSrcdoc`, `highlightFragments` refaktor, iframe életciklus, módok, Copy, link-kattintás
6. [x] Node stub-DOM teszt a pure függvényekre; `node --check`; a DOM-bejáró sanitizer Chrome-ban a fixture HTML-jén
7. [x] README; 008 jegyzet (`data-mboxer-cid`); 013 → done
8. [x] Kézi kör (fixture DevTools-szal + 14 GB) → Verification; `cargo fmt`, `clippy -D warnings`, `cargo test`; `tasks_check`; commit

## Log

- 2026-10-17 — plan approved; 007 promoted to wip
- 2026-10-17 — Rust side: `MessageView.html` (the first HTML part as written), `unescape_from_lines` on the
  plain body (013), `read_source` (envelope line + lossy bytes), `open_url` behind `allowed_link`
  (http / https / mailto), `tauri-plugin-opener` registered; the parent CSP's `img-src` gained
  `https: http:` because a srcdoc frame inherits it and the blocking moved into the frame's own meta
  CSP. Fixture `html-remote.mbox`: meta refresh, base, script, onload, javascript: link, remote img,
  remote `url()` in a style attribute and a `<style>`, `@import`, cid image, form, iframe, object.
  16 app tests green on the first run (4 new)
- 2026-10-17 — UI: `sanitizeHtml` (DOMParser, drop / unwrap lists, URL classification, CSS scrub with
  the originals kept in `data-mboxer-*`), `htmlSrcdoc` (fresh document, remote restore on request,
  `<mark>` through a TreeWalker, charset + meta CSP + base style prepended), the iframe sized from
  its document (`allow-same-origin`, no scripts), link clicks routed to `open_url`, Esc / Ctrl+F
  from inside the frame, HTML / Text / Source buttons (HTML or Text sticky in localStorage), the
  remote bar, Copy (clipboard API, `execCommand` fallback), status flash. `renderHighlighted` now
  wraps `highlightFragments`, shared with the frame
- 2026-10-17 — Node stub-DOM check (scratchpad, 35 cases: URL classes incl. `java\tscript:`, CSS scrub,
  fragments, touching matches): green. The DOM walk ran in Chrome on the fixture HTML (served from
  127.0.0.1): remote count 3, 0 scripts / forms / frames / base / meta refresh left, the `javascript:`
  href gone, link titles set, `about:srcdoc` under `sandbox="allow-same-origin"` loads and is readable
  from the parent (height 159 px). Block mode: the frame's resource timing lists no example.com
  entry; after Show: `t.gif`, `bg.png`, `d.png` all fetched (the extension's network panel does not see
  subframe requests, resource timing does). Found: an emptied `url()` still resolved to the document
  itself → remote `url()` becomes `none`. Link click intercepted with the href
- 2026-10-17 — `cargo fmt`, `cargo clippy --all-targets -- -D warnings` clean, `cargo test` 16 + 4 + 5 + 22 +
  7 + 12 green; README reader paragraph; 008 got the `data-mboxer-cid` / `open_path` note; 013 closed
  into `done/` with its own Verification
- 2026-10-17 — user's manual round on the fixture and the 14 GB archive: "minden ok". Follow-ups: none
  filed; two accepted limits noted under Deviations (a match that spans an element boundary is not
  highlighted inside HTML, `srcset` is held back as a whole). Closed

## Deviations from the plan

- A blocked CSS `url()` becomes `none`, not an empty `url()`: Chrome resolved the empty one to the
  document itself (seen in the frame's resource timing) — no network, but a pointless fetch.
- `srcset` is held back whole (and restored whole on Show) instead of per candidate: data URLs hold
  commas, so splitting candidates is not safe; the `src` fallback shows meanwhile.
- No "open the message in the browser" button: user decision in the planning round, links only.
- Accepted limits: the highlighter works per text node, so a word split by an inline tag
  (`sz<b>ámla</b>`) is not marked in the HTML view (it is in the Text view); the `blob:` fallback
  for `about:srcdoc` was not needed — the frame loads under `frame-src 'self'`.

## Verification

- 2026-10-17 — `cargo fmt --check` clean, `cargo clippy --all-targets -- -D warnings` clean, `cargo test`:
  16 app unit (4 new: the `html` field and the unquoted text on `gmail-crlf` and `html-remote`, the
  `<script>` kept in `html` of the HTML-only fixture, `unescape_from_lines` edge cases, `allowed_link`)
  + 4 + 5 CLI + 22 + 7 + 12 core pass.
- 2026-10-17 — `node --check ui/main.js` clean; Node stub-DOM check of the pure helpers (35 cases:
  `classifyUrl` incl. `java\tscript:` and leading control characters, `scrubCss` import / expression /
  behavior / quoted / protocol-relative / data / fragment, `highlightFragments` folding, touching
  matches, operators) green.
- 2026-10-17 — the sanitizer and the frame in Chrome (the same block of `main.js`, the fixture's two
  HTML bodies, page served from 127.0.0.1): 3 remote references held back on message 1, 0 on message 2;
  the frame document has 0 scripts, 0 forms, 0 frames / objects, no `<base>`, no meta refresh, no
  `onload`, no `javascript:` link, link titles set; blocked mode: no example.com entry in the frame's
  resource timing; Show: `t.gif`, `bg.png` (`<style>`), `d.png` (style attribute) fetched and the
  CSP relaxed for images only; `<mark>` appears in the frame for `számla`; a click on `részletek` is
  intercepted with its href; the frame height follows the document (159 px).
- 2026-10-17 — `target\debug\mboxer.exe` on a scratchpad copy of `html-remote.mbox`: index built in 63 ms,
  2 messages. User's manual round on the fixture and on the 14 GB archive (newsletters, Gmail reply
  chains, HTML-only messages): HTML renders, the remote bar counts and Show loads, links open the
  system browser, Text / Source / Copy work, `>From ` lines show unquoted in Text and quoted in Source,
  the search word is highlighted inside the frame, Esc leaves the frame for the list. Reported as
  "minden ok". Done-when met: every message renders readably and nothing leaves the app until Show.

## Closing checklist

- [x] every follow-up is a `backlog/` file (`/tasks add`) or deleted with a reason in the Log — none filed, two accepted limits under Deviations
- [x] Verification written, dated
- [x] `tasks_check` green, README index regenerated
- [x] moved to `done/` with `closed: 2026-10-17`
