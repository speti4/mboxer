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

Today the reader shows the plain text part (`<pre id="msg-body">`), the html→text conversion for an
HTML-only message. 007 replaces this: the HTML body renders in a sandboxed `srcdoc` iframe, scripts
stripped, remote content blocked by default (a per-message "Show" button), HTML / Text / Source
view switch, Copy button, links clicked in the body open in the system browser; the 006 highlighting
carries over into the sanitized HTML. Why: an HTML mail should look like mail, and WebView2 renders
it natively. Done when: any message renders readably, and without permission not a single network request
leaves the app.

User decisions 2026-10-17 (planning round): "open in browser" = only the body's links open in the
system browser, no separate "open message in browser" button; the sanitizer runs in the webview
(JS, DOMParser — the option that adds no crate), with two independent layers under it:
the sandbox iframe (`allow-same-origin`, without scripts) and the iframe's own meta CSP (`default-src 'none'`);
013 (unescaping `>From ` lines in the plain text body) merges into this task.

Build-time crate (D-002 justification): `tauri-plugin-opener 2` — opening http/https/mailto links
with the system's default application, called from the Rust side (`OpenerExt::open_url`, no capability
needed); the 008 attachment opening (`open_path`) will use the same.

Note from 005: the shell's CSP already allows `frame-src 'self' blob:`; loading `about:srcdoc`
under `frame-src 'self'` has to be verified, the fallback is a `blob:` URL. Note from 006: the plain text
body's highlighting (`renderHighlighted`) must be carried into the srcdoc iframe too.

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
