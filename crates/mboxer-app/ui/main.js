// mboxer shell: open flow, folders, the search field and filter panel, the
// virtualized message list, the reader. Plain JS on the global Tauri API
// (`withGlobalTauri`), no bundler.
(function () {
  "use strict";

  const { invoke, Channel } = window.__TAURI__.core;
  // The query syntax of the search field (query.js, loaded first).
  const Query = window.MboxerQuery;
  // The theme preference (theme.js, loaded in <head> so the override is on
  // <html> before the first paint); the button and the title bar are wired here.
  const Theme = window.MboxerTheme;
  // The remembered files (files.js): the pane order and the active one.
  const Files = window.MboxerFiles;
  // Printing on a Mac (print.js): the print window and the inlined cid images.
  const Print = window.MboxerPrint;
  // The sanitized frame's URL kinds, CSS scrub and head metas (frame.js).
  const Frame = window.MboxerFrame;

  const ROW_H = 28;
  const BLOCK = 200;
  const OVERSCAN = 10;
  const MAX_BLOCKS = 100;
  const READ_DEBOUNCE_MS = 60;
  const SEARCH_DEBOUNCE_MS = 150;
  // Bodies longer than this are shown without highlighting.
  const MAX_HIGHLIGHT_CHARS = 2_000_000;
  // A fresh index opens in well under this; the progress overlay waits for a build.
  const OVERLAY_DELAY_MS = 400;
  const STORAGE_WIDTHS = "mboxer.paneWidths";
  const STORAGE_FILTERS_OPEN = "mboxer.filtersOpen";
  const STORAGE_BODY_MODE = "mboxer.bodyMode";
  const STORAGE_GROUP_THREADS = "mboxer.groupThreads";
  const STATUS_FLASH_MS = 2000;
  // A saved path deserves a longer look.
  const SAVED_FLASH_MS = 6000;

  const el = {};
  for (const id of [
    "toolbar", "open-btn", "file-name", "search", "filters-btn", "filters-count", "theme-btn", "filters",
    "f-from", "f-after", "f-before", "f-att", "f-threads", "f-sort", "filters-clear", "panes", "folders",
    "list", "spacer", "rows", "list-empty", "reader", "reader-empty", "reader-content",
    "msg-subject", "msg-headers", "attachments", "attachment-list", "save-all-btn", "msg-body",
    "msg-html", "msg-parts-list", "view-html", "view-text",
    "view-source", "copy-btn", "save-eml-btn", "print-btn", "print-frame", "remote-bar", "remote-text",
    "remote-allow", "empty", "empty-open-btn",
    "reopen-list", "error", "error-path", "error-text", "retry-btn", "error-open-btn", "overlay",
    "overlay-title", "overlay-path", "progress", "progress-text", "cancel-btn", "status-left",
    "status-center", "status-right", "status-version",
  ]) {
    el[id.replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = document.getElementById(id);
  }

  // The sticky "Conversations" view mode; a setting, not a filter, so Clear
  // leaves it alone.
  function groupThreadsPref() {
    return storageGet(STORAGE_GROUP_THREADS) !== "0";
  }

  // The query model: the text of the search field is the one source of
  // truth (task 010); the filter panel and the folder pane edit it through
  // its operators and render from `state.parsed`. Only the view mode and
  // the sort order live beside it.
  function defaultQuery() {
    return {
      text: "",
      groupThreads: groupThreadsPref(),
      sort: "dateDesc",
    };
  }

  const state = {
    file: null, // OpenSummary of the active file: the one the list, the search and the reader show
    files: [], // FileInfo[] of every open file, from the last summary (task 016)
    remembered: Files.empty(), // { paths, active }: the pane order, loaded or not; saved by files.js
    fileQueries: new Map(), // path key -> { text, sort } of a file that is open but not active
    generation: 0,
    total: 0, // visible rows
    messages: 0, // matching messages behind the rows
    conversations: 0, // groups behind the rows (= messages when not grouped)
    layout: 0, // bumped when rows shift without a new generation (a thread opened or closed)
    blocks: new Map(), // block index -> Row[]
    pending: new Map(), // block index -> Promise
    folderNodes: new Map(), // node key -> LabelNode of the rendered label tree
    foldersCollapsed: new Set(), // node keys folded in the folder pane
    selected: -1, // row index
    selectedOffset: null,
    view: null, // MessageView shown in the reader
    bodyMode: "html", // the sticky body view: "html" (when the message has one) or "text"
    sourceMode: false, // the Source view, for this message only
    sanitized: null, // { offset, doc, remote, length } of the HTML in the reader
    allowRemote: false, // the remote content of this message was allowed
    source: null, // { offset, text } the raw source, read on demand
    cidUrls: new Map(), // attachment index -> Promise<blob URL> of this message's cid images
    blobUrls: [], // every blob URL made for this message, revoked when it changes
    cidBlobs: new Map(), // blob URL -> Blob of the cid images, inlined for the print window (macOS)
    printing: false, // a print document is loading into the print frame or going to the print window
    warnings: [], // the query warnings behind the status bar
    statusTimer: 0,
    query: defaultQuery(), // the one source of truth behind the listing
    queryToken: 0, // only the newest set_query response lands
    queryTimer: 0,
    queryDirty: false, // an edit the backend has not seen yet
    highlightTerms: [], // folded words of the query behind the current listing
    parsed: Query.parse(""), // the operators and the free text of query.text (syncFromText)
    labelIndex: new Map(), // every node of the label tree by key, collapsed ones included
    labelKeys: new Map(), // label name or container path → node key
    errorPath: null, // the path behind the error screen's Retry
    batch: false, // an open_mbox call is in flight (until its `done`)
    systemOpens: [], // paths the system handed the app during a batch; opened after it
    opening: null, // { path, startedAt, position, count } of the file being opened
    overlayTimer: 0,
    readTimer: 0,
    readToken: 0,
    renderQueued: false,
  };

  // ---------- formatting ----------

  const numberFormat = new Intl.NumberFormat();
  const timeFormat = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
  const dayFormat = new Intl.DateTimeFormat(undefined, { day: "numeric", month: "short" });
  const fullFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" });

  function listDate(seconds) {
    if (seconds == null) return "–";
    const date = new Date(seconds * 1000);
    const now = new Date();
    if (date.toDateString() === now.toDateString()) return timeFormat.format(date);
    if (date.getFullYear() === now.getFullYear()) return dayFormat.format(date);
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${date.getFullYear()}-${month}-${day}`;
  }

  function fullDate(seconds) {
    return seconds == null ? "–" : fullFormat.format(new Date(seconds * 1000));
  }

  function senderName(from) {
    if (!from || from.length === 0) return "(unknown sender)";
    const first = from[0];
    const angle = first.indexOf(" <");
    return angle > 0 ? first.slice(0, angle) : first;
  }

  function subjectText(subject) {
    return subject && subject.trim() ? subject : "(no subject)";
  }

  // "Re: Fwd: Vá: Offer" -> "Offer", for a conversation row; the member rows
  // keep their own subjects.
  function stripReplyPrefix(subject) {
    const stripped = subject.replace(/^(\s*(re|fw|fwd|vá|v|aw|wg|továbbítás|tov)\s*(\[\d+\])?\s*:\s*)+/i, "");
    return stripped.trim() ? stripped : subject;
  }

  function formatBytes(bytes) {
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`;
    if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
    if (bytes >= 1e3) return `${Math.round(bytes / 1e3)} kB`;
    return `${bytes} B`;
  }

  function fmt(n) {
    return numberFormat.format(n);
  }

  // The backend's error messages start in lower case; one shown on its own starts a sentence.
  function sentence(text) {
    return text.charAt(0).toUpperCase() + text.slice(1);
  }

  function setView(view) {
    document.body.dataset.view = view;
  }

  function storageGet(key) {
    try {
      return localStorage.getItem(key);
    } catch (_) {
      return null;
    }
  }

  function storageSet(key, value) {
    try {
      localStorage.setItem(key, value);
    } catch (_) {
      /* storage unavailable: nothing to remember */
    }
  }

  // ---------- highlighting ----------
  //
  // Folding and the words worth highlighting live in query.js
  // (`Query.foldWithMap`, `Query.highlightTerms`); the marks are made here.

  // The nodes that show `text` with the occurrences of `terms` wrapped in
  // <mark>, or null when no term occurs.
  function highlightFragments(text, terms) {
    if (terms.length === 0 || !text) return null;
    const { folded, map } = Query.foldWithMap(text);
    const ranges = [];
    for (const term of terms) {
      let at = folded.indexOf(term);
      while (at >= 0) {
        ranges.push([map[at], map[at + term.length]]);
        at = folded.indexOf(term, at + term.length);
      }
    }
    if (ranges.length === 0) return null;
    ranges.sort((a, b) => a[0] - b[0]);
    // Overlapping and touching matches become one mark.
    const merged = [];
    for (const [start, end] of ranges) {
      const last = merged[merged.length - 1];
      if (last && start <= last[1]) last[1] = Math.max(last[1], end);
      else merged.push([start, end]);
    }
    const parts = [];
    let cursor = 0;
    for (const [start, end] of merged) {
      if (start > cursor) parts.push(document.createTextNode(text.slice(cursor, start)));
      const mark = document.createElement("mark");
      mark.textContent = text.slice(start, end);
      parts.push(mark);
      cursor = end;
    }
    if (cursor < text.length) parts.push(document.createTextNode(text.slice(cursor)));
    return parts;
  }

  // Fills `node` with `text`, the occurrences of `terms` wrapped in <mark>.
  function renderHighlighted(node, text, terms) {
    const parts = text && text.length <= MAX_HIGHLIGHT_CHARS ? highlightFragments(text, terms) : null;
    if (parts) node.replaceChildren(...parts);
    else node.textContent = text;
  }

  // ---------- HTML sanitizer ----------
  //
  // The HTML of a message is parsed into an inert document (DOMParser loads
  // nothing and runs nothing), stripped to a safe subset and serialized into a
  // sandboxed iframe whose own CSP blocks every remote request. Remote
  // references move into `data-mboxer-*` attributes so that the per-message
  // "Show" button can put them back; the CSP is relaxed for images only then.

  const DROP_ELEMENTS = new Set([
    "script", "noscript", "iframe", "frame", "frameset", "object", "embed", "applet", "base",
    "link", "meta", "template",
  ]);
  const UNWRAP_ELEMENTS = new Set(["form"]);
  // Attributes that hold a URL; `href` is a link on a / area, a resource elsewhere.
  const RESOURCE_ATTRIBUTES = new Set(["src", "poster", "background", "xlink:href", "href"]);
  const DROP_ATTRIBUTES = new Set([
    "srcdoc", "target", "action", "formaction", "ping", "longdesc", "usemap", "cite", "data",
  ]);
  const RESTORE_ATTRIBUTES = ["src", "srcset", "poster", "background", "xlink:href", "href"];
  // Text under these elements is not highlighted.
  const NO_HIGHLIGHT = new Set(["style", "title", "textarea", "script"]);
  // The frame is a light page in both themes: HTML mail is written for one,
  // and a dark canvas would hide mails with explicit dark text and no background.
  const IFRAME_STYLE = [
    ":root{color-scheme:light}",
    "html,body{height:auto!important;min-height:0!important;overflow:visible!important}",
    'body{font-family:"Segoe UI",system-ui,sans-serif;font-size:13px;overflow-wrap:anywhere;background:#fff;color:#1f2328}',
    "img{max-width:100%;height:auto}",
    "pre{white-space:pre-wrap}",
    "mark{background:#ffe58f;color:inherit}",
    "img[data-mboxer-cid]{min-width:16px;min-height:16px;border:1px dashed #d9dee3}",
  ].join("");
  // The print document (task 011): a header block over the body, on a light page.
  const PRINT_STYLE = [
    "@page{margin:15mm}",
    ":root{color-scheme:light}",
    'body{font-family:"Segoe UI",system-ui,sans-serif;font-size:13px;overflow-wrap:anywhere;background:#fff;color:#1f2328}',
    ".mboxer-print-header{margin:0 0 14px;padding:0 0 10px;border-bottom:1px solid #999}",
    ".mboxer-print-header h1{margin:0 0 8px;font-size:17px;font-weight:600}",
    ".mboxer-print-header dl{display:grid;grid-template-columns:max-content minmax(0,1fr);gap:2px 12px;margin:0}",
    ".mboxer-print-header dt{color:#666}",
    ".mboxer-print-header dd{margin:0}",
    ".mboxer-print-note{margin:0 0 8px;color:#666;font-style:italic}",
    "pre.mboxer-print-body{margin:0;font:inherit;white-space:pre-wrap;overflow-wrap:anywhere}",
    "pre.mboxer-print-body.source{font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12px}",
    "img{max-width:100%;height:auto}",
  ].join("");

  // The kind of a URL and the CSS scrub live in frame.js.
  const { classifyUrl, scrubCss } = Frame;

  // Parses `html` into an inert document and strips it: active content,
  // unsafe URLs and remote references. Returns the document and the number
  // of remote references held back.
  function sanitizeHtml(html) {
    const doc = new DOMParser().parseFromString(html, "text/html");
    const counter = { remote: 0 };
    for (const element of Array.from(doc.querySelectorAll("*"))) {
      if (!element.isConnected) continue;
      const tag = element.localName;
      if (DROP_ELEMENTS.has(tag)) {
        element.remove();
        continue;
      }
      if (UNWRAP_ELEMENTS.has(tag)) {
        element.replaceWith(...element.childNodes);
        continue;
      }
      sanitizeAttributes(element, counter);
      if (tag === "style") {
        const { safe, blocked, remote } = scrubCss(element.textContent);
        element.textContent = blocked;
        if (remote > 0) {
          element.setAttribute("data-mboxer-css", safe);
          counter.remote += remote;
        }
      } else if ((tag === "a" || tag === "area") && !element.hasAttribute("title")) {
        const href = element.getAttribute("href");
        if (href) element.setAttribute("title", href);
      }
    }
    return { doc, remote: counter.remote };
  }

  function sanitizeAttributes(element, counter) {
    const isLink = element.localName === "a" || element.localName === "area";
    for (const attr of Array.from(element.attributes)) {
      const name = attr.name.toLowerCase();
      const value = attr.value;
      if (name.startsWith("on") || DROP_ATTRIBUTES.has(name)) {
        element.removeAttribute(attr.name);
      } else if (name === "style") {
        const { safe, blocked, remote } = scrubCss(value);
        element.setAttribute("style", blocked);
        if (remote > 0) {
          element.setAttribute("data-mboxer-style", safe);
          counter.remote += remote;
        }
      } else if (name === "srcset") {
        // Candidates are hard to split (data URLs hold commas); the whole set
        // is held back and the `src` fallback shows instead.
        element.removeAttribute(attr.name);
        if (/https?:\/\/|(^|[\s,])\/\//i.test(value)) {
          element.setAttribute("data-mboxer-srcset", value);
          counter.remote++;
        }
      } else if (name === "href" && isLink) {
        const kind = classifyUrl(value);
        if (kind !== "remote" && kind !== "mailto" && kind !== "fragment") {
          element.removeAttribute(attr.name);
        }
      } else if (RESOURCE_ATTRIBUTES.has(name)) {
        const kind = classifyUrl(value);
        if (kind === "data") continue;
        element.removeAttribute(attr.name);
        if (kind === "cid") {
          element.setAttribute("data-mboxer-cid", value.trim().slice(4));
        } else if (kind === "remote") {
          element.setAttribute(`data-mboxer-${name}`, value);
          counter.remote++;
        }
      }
    }
  }

  // Puts the held-back remote references of `doc` back.
  function restoreRemote(doc) {
    for (const element of doc.querySelectorAll("*")) {
      for (const name of RESTORE_ATTRIBUTES) {
        const held = element.getAttribute(`data-mboxer-${name}`);
        if (held != null) element.setAttribute(name, held);
      }
      const style = element.getAttribute("data-mboxer-style");
      if (style != null) element.setAttribute("style", style);
      const css = element.getAttribute("data-mboxer-css");
      if (css != null) element.textContent = css;
    }
  }

  // Wraps the occurrences of `terms` in the text of `doc` in <mark>.
  function highlightDocument(doc, terms) {
    if (!doc.body || terms.length === 0) return;
    const walker = doc.createTreeWalker(doc.body, NodeFilter.SHOW_TEXT);
    const nodes = [];
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (!NO_HIGHLIGHT.has(node.parentNode.localName)) nodes.push(node);
    }
    for (const node of nodes) {
      const parts = highlightFragments(node.data, terms);
      if (parts) node.replaceWith(...parts);
    }
  }

  // The srcdoc of the iframe: a copy of the sanitized document with our
  // charset, referrer policy, CSP and base style in front, remote references
  // restored on request, the search words marked.
  function htmlSrcdoc(sanitized, { allowRemote, terms }) {
    const doc = document.implementation.createHTMLDocument("");
    doc.replaceChild(doc.importNode(sanitized.doc.documentElement, true), doc.documentElement);
    if (allowRemote) restoreRemote(doc);
    if (sanitized.length <= MAX_HIGHLIGHT_CHARS) highlightDocument(doc, terms);
    const style = doc.createElement("style");
    style.textContent = IFRAME_STYLE;
    doc.head.prepend(...Frame.headMetas(doc, allowRemote), style);
    return `<!doctype html>${doc.documentElement.outerHTML}`;
  }

  // ---------- open flow ----------
  //
  // Several files can be open at once; one is active (task 016, D-006). `paths`
  // is a list, or null for the native dialog (several files allowed). The
  // backend adds the files to its set, one after the other, and reports each
  // on the channel; with `activate` the first of them becomes the active file.

  async function openMbox(paths, { activate = true } = {}) {
    if (state.batch) return;
    state.batch = true;
    const channel = new Channel();
    channel.onmessage = handleOpenEvent;
    try {
      await invoke("open_mbox", {
        paths: paths || null,
        groupThreads: groupThreadsPref(),
        activate,
        onEvent: channel,
      });
    } catch (err) {
      showFailure(paths && paths[0] ? paths[0] : "", String(err));
      finishBatch();
    } finally {
      state.batch = false;
    }
    if (state.systemOpens.length > 0) openMbox(state.systemOpens.splice(0));
  }

  // Files the system asks the running app to open (macOS: Finder "Open with",
  // a Dock drop), sent by the backend as an event with the paths. They open
  // like a dialog pick; during a batch they wait for its end.
  function openFromSystem(paths) {
    if (!Array.isArray(paths) || paths.length === 0) return;
    if (state.batch) state.systemOpens.push(...paths);
    else openMbox(paths);
  }

  function handleOpenEvent(message) {
    switch (message.event) {
      case "opening": {
        const { path, position, count } = message.data;
        state.opening = { path, position, count, startedAt: performance.now() };
        el.overlayTitle.textContent = count > 1 ? `Opening file ${position} of ${count}…` : "Opening…";
        el.overlayPath.textContent = path;
        el.progress.removeAttribute("value");
        el.progressText.textContent = "Checking the index…";
        showOverlaySoon();
        break;
      }
      case "building": {
        const { bytes, totalBytes, messages } = message.data;
        const startedAt = state.opening ? state.opening.startedAt : performance.now();
        const elapsed = (performance.now() - startedAt) / 1000;
        const rate = elapsed > 0 ? bytes / 1e6 / elapsed : 0;
        const { position = 1, count = 1 } = state.opening || {};
        el.overlayTitle.textContent =
          count > 1 ? `Building the search index (file ${position} of ${count})…` : "Building the search index…";
        if (totalBytes > 0) el.progress.value = bytes / totalBytes;
        el.progressText.textContent =
          `${formatBytes(bytes)} of ${formatBytes(totalBytes)} · ${fmt(messages)} messages · ${rate.toFixed(0)} MB/s`;
        showOverlayNow();
        break;
      }
      case "ready":
        addFile(message.data.summary);
        break;
      case "cancelled":
        if (state.file) flashStatus("Open cancelled");
        break;
      case "failed":
        showFailure(message.data.path, message.data.message);
        break;
      case "done":
        finishBatch();
        break;
      default:
        break;
    }
  }

  // The overlay appears for a build at once, for a mere index check only when
  // it drags on: reopening the remembered files at start-up must not flicker.
  function showOverlaySoon() {
    if (!el.overlay.hidden || state.overlayTimer) return;
    state.overlayTimer = setTimeout(() => {
      state.overlayTimer = 0;
      if (state.opening) el.overlay.hidden = false;
    }, OVERLAY_DELAY_MS);
  }

  function showOverlayNow() {
    clearTimeout(state.overlayTimer);
    state.overlayTimer = 0;
    el.overlay.hidden = false;
  }

  function hideOverlay() {
    clearTimeout(state.overlayTimer);
    state.overlayTimer = 0;
    el.overlay.hidden = true;
  }

  function finishBatch() {
    state.opening = null;
    hideOverlay();
    renderReopenList();
    if (!state.file && document.body.dataset.view !== "error") setView("empty");
  }

  function showFailure(path, text) {
    if (state.file) {
      flashStatus(`Open failed: ${text}`, "error", SAVED_FLASH_MS);
      el.statusRight.title = path;
      return;
    }
    el.errorPath.textContent = path;
    el.errorText.textContent = sentence(text);
    el.retryBtn.hidden = !path;
    state.errorPath = path || state.errorPath;
    setView("error");
  }

  // A file the backend reports open: into the pane, and into the list, the
  // search and the reader when the backend made it the active one.
  function addFile(summary) {
    const known = state.files.some((info) => info.id === summary.id);
    state.files = summary.files;
    state.remembered = Files.remember(state.remembered, summary.path);
    const me = summary.files.find((info) => info.id === summary.id);
    if (me && me.active && state.file && state.file.id === summary.id) {
      // The active file opened again (picked once more in the dialog): the
      // pane and the listing's generation are new, the query stays as typed.
      persistFiles();
      renderFolders();
      applyQuery();
    } else if (me && me.active) {
      // A file just loaded lists all its mail already; a re-listed one may
      // hold an older query, so it is queried again like a switch.
      activateFile(summary, { fresh: !known });
    } else {
      persistFiles();
      renderFolders();
    }
  }

  function persistFiles() {
    Files.write(state.remembered);
  }

  // The key `fileQueries` uses for a file; two spellings of one path share it
  // (the OS's rules, as in files.js).
  function queryKey(path) {
    return Files.pathKey(path);
  }

  // Makes `summary` the file the list, the search and the reader show. The
  // previously active file keeps its query for its next turn. A `fresh`
  // summary (a file just loaded) carries its listing; any other file is
  // queried again, so the list follows the restored query and today's view mode.
  function activateFile(summary, { fresh = false } = {}) {
    if (state.file && state.file.id !== summary.id) {
      state.fileQueries.set(queryKey(state.file.path), { text: state.query.text, sort: state.query.sort });
    }
    const switching = Boolean(state.file) && state.file.id !== summary.id;
    state.file = summary;
    state.files = summary.files;
    state.remembered = Files.setActive(state.remembered, summary.path);
    persistFiles();
    el.fileName.textContent = summary.fileName;
    el.fileName.title = summary.path;
    el.fileName.classList.remove("muted");
    document.title = `${summary.fileName} – mboxer`;
    el.statusLeft.textContent = summary.path;
    el.statusLeft.title = summary.path;
    showWarnings([]);
    indexLabelTree();
    resetQueryModel();
    const saved = state.fileQueries.get(queryKey(summary.path));
    if (saved) {
      state.fileQueries.delete(queryKey(summary.path));
      el.search.value = saved.text;
      state.query.text = saved.text;
      state.query.sort = saved.sort;
      el.fSort.value = saved.sort;
      syncFromText();
      updateFilterBadge();
    }
    state.foldersCollapsed.clear();
    renderFolders();
    // The backend's current listing of the file first (it stamped a new
    // generation, so the pages of the previous file are stale) …
    state.messages = summary.messages;
    state.conversations = summary.conversations;
    resetListing(summary.generation, summary.total);
    // … then, unless the file was just loaded and lists all its mail, the
    // restored query, which also brings the listing to today's view mode.
    if (switching || !fresh || saved) applyQuery();
    setView("ready");
    // The panes had no height while hidden; lay the list out again now that they do.
    scheduleRender();
    el.list.focus();
  }

  // Switches to open file `id` (a click on its row in the pane).
  async function switchFile(id) {
    if (state.file && state.file.id === id) return;
    try {
      const summary = await invoke("activate_file", { file: id });
      activateFile(summary);
    } catch (err) {
      flashStatus(`Could not switch: ${err}`, "error");
    }
  }

  // Closes open file `id`; the backend names the file that is active afterwards.
  async function closeFile(id) {
    const info = state.files.find((candidate) => candidate.id === id);
    let summary;
    try {
      summary = await invoke("close_file", { file: id });
    } catch (err) {
      flashStatus(`Could not close: ${err}`, "error");
      return;
    }
    if (info) {
      state.remembered = Files.forget(state.remembered, info.path);
      state.fileQueries.delete(queryKey(info.path));
    }
    if (!summary) {
      showNoFile();
      return;
    }
    if (state.file && state.file.id === summary.id) {
      state.files = summary.files;
      state.remembered = Files.setActive(state.remembered, summary.path);
      persistFiles();
      renderFolders();
    } else {
      activateFile(summary);
    }
  }

  // Forgets a remembered file that is not loaded (its row's × in the pane).
  function forgetFile(path) {
    state.remembered = Files.forget(state.remembered, path);
    state.fileQueries.delete(queryKey(path));
    persistFiles();
    renderFolders();
    renderReopenList();
  }

  // The last file was closed: back to the empty screen.
  function showNoFile() {
    state.file = null;
    state.files = [];
    state.remembered = Files.setActive(state.remembered, null);
    persistFiles();
    clearReader();
    state.blocks.clear();
    state.pending.clear();
    state.selected = -1;
    state.selectedOffset = null;
    state.total = 0;
    el.spacer.style.height = "0px";
    render();
    el.fileName.textContent = "No file open";
    el.fileName.title = "";
    el.fileName.classList.add("muted");
    document.title = "mboxer";
    el.statusLeft.textContent = "";
    el.statusLeft.title = "";
    el.statusCenter.textContent = "";
    el.search.value = "";
    el.search.disabled = true;
    el.filtersBtn.disabled = true;
    showWarnings([]);
    renderFolders();
    renderReopenList();
    setView("empty");
  }

  // The empty screen's buttons: one per remembered file.
  function renderReopenList() {
    const paths = state.remembered.paths;
    el.reopenList.replaceChildren(
      ...paths.map((path) => {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = `Reopen ${Files.fileName(path)}`;
        button.title = path;
        button.addEventListener("click", () => openMbox([path]));
        return button;
      }),
    );
    el.reopenList.hidden = paths.length === 0;
  }

  function indexStateText(index) {
    if (index.state === "reused") {
      return `Index reused (built ${fullDate(index.builtAtUnix)})`;
    }
    const unparsed = index.parseFailures === 1 ? "1 unparsed message" : `${fmt(index.parseFailures)} unparsed messages`;
    return `Index built in ${fmt(index.buildSecs)} s, ${unparsed}`;
  }

  // ---------- query model ----------

  // Resets the model and the inputs to "all mail" without running a query.
  function resetQueryModel() {
    clearTimeout(state.queryTimer);
    state.queryTimer = 0;
    state.queryDirty = false;
    state.query = defaultQuery();
    state.highlightTerms = [];
    el.search.value = "";
    el.fThreads.checked = state.query.groupThreads;
    el.fSort.value = "dateDesc";
    el.search.disabled = false;
    el.filtersBtn.disabled = false;
    syncFromText();
    updateFilterBadge();
  }

  // Re-parses the field text into `state.parsed` and renders the panel and
  // the folder selection from it. An input keeps its value (and its caret)
  // when it already shows the parsed one.
  function syncFromText() {
    state.parsed = Query.parse(state.query.text, findLabelKey);
    const p = state.parsed;
    setInputValue(el.fFrom, p.from);
    setInputValue(el.fAfter, p.after);
    setInputValue(el.fBefore, p.before);
    setInputValue(el.fAtt, p.hasAttachment == null ? "" : String(p.hasAttachment));
    updateFolderSelection();
  }

  function setInputValue(input, value) {
    if (input.value !== value) input.value = value;
  }

  // Sets the operator `name` of the field text to `value` (the panel inputs
  // and the folder pane edit the text this way) and runs the query.
  function rewrite(name, value, { debounce = 0 } = {}) {
    const text = Query.setOperator(state.query.text, name, value);
    if (text === state.query.text) return;
    el.search.value = text;
    state.query.text = text;
    syncFromText();
    applyQuery({ debounce });
  }

  // Unix seconds of local midnight starting `ymd` (YYYY-MM-DD), or null.
  function localDayStart(ymd, dayOffset) {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd || "");
    if (!m) return null;
    const time = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]) + dayOffset).getTime();
    return Number.isNaN(time) ? null : Math.floor(time / 1000);
  }

  // The QueryInput for the backend from the parsed text: empty fields left
  // out, dates as local days (`before:` inclusive like the panel's "to": the
  // core's `before` is the next midnight). The free text keeps its trailing
  // space: it tells the backend the last word is finished, otherwise that
  // word matches as a prefix while it is typed.
  function queryInput() {
    const q = state.query;
    const p = state.parsed;
    const input = {
      labels: p.labels,
      anyLabels: labelsUnder(p.label),
      groupThreads: q.groupThreads,
      prefixLastWord: true,
    };
    const text = p.free.trim();
    if (text) input.text = p.free;
    if (p.from) input.from = p.from;
    if (p.to) input.to = p.to;
    if (p.subject) input.subject = p.subject;
    const after = localDayStart(p.after, 0);
    if (after != null) input.after = after;
    const before = localDayStart(p.before, 1);
    if (before != null) input.before = before;
    if (p.hasAttachment != null) input.hasAttachment = p.hasAttachment;
    // Relevance means nothing without words to score.
    input.sort = text && q.sort === "relevance" ? "relevance" : "dateDesc";
    return input;
  }

  // The filters beyond the free text and the selected label: the badge count.
  function activeFilterCount() {
    const p = state.parsed;
    let n = 0;
    if (p.from) n++;
    if (p.to) n++;
    if (p.subject) n++;
    if (p.after) n++;
    if (p.before) n++;
    if (p.hasAttachment != null) n++;
    n += p.labels.length;
    if (state.query.sort !== "dateDesc") n++;
    return n;
  }

  function updateFilterBadge() {
    const n = activeFilterCount();
    el.filtersCount.textContent = String(n);
    el.filtersCount.hidden = n === 0;
  }

  function anyNarrowing() {
    const p = state.parsed;
    return Boolean(p.free.trim() || p.label || activeFilterCount() > 0);
  }

  // Sends the model to the backend; `debounce` delays it for a typist.
  function applyQuery({ debounce = 0 } = {}) {
    state.queryDirty = true;
    updateFilterBadge();
    clearTimeout(state.queryTimer);
    state.queryTimer = 0;
    if (debounce > 0) {
      state.queryTimer = setTimeout(runQuery, debounce);
      return;
    }
    runQuery();
  }

  async function runQuery() {
    if (!state.file) return;
    clearTimeout(state.queryTimer);
    state.queryTimer = 0;
    state.queryDirty = false;
    const token = ++state.queryToken;
    const input = queryInput();
    const terms = Query.highlightTerms(state.query.text);
    try {
      const info = await invoke("set_query", { file: state.file.id, query: input });
      if (token !== state.queryToken) return;
      state.highlightTerms = terms;
      state.messages = info.messages;
      state.conversations = info.conversations;
      showWarnings(info.warnings);
      await resetListing(info.generation, info.total, { keepSelection: true });
    } catch (err) {
      if (token !== state.queryToken) return;
      el.statusRight.textContent = `Query failed: ${err}`;
      el.statusRight.className = "error";
    }
  }

  // Shows the backend's query warnings after the syntax ones of the field.
  function showWarnings(warnings) {
    state.warnings = warnings || [];
    clearTimeout(state.statusTimer);
    el.statusRight.title = "";
    const all = state.parsed.warnings.concat(state.warnings);
    if (all.length > 0) {
      el.statusRight.textContent = `Query: ${all.join("; ")}`;
      el.statusRight.className = "warn";
    } else {
      el.statusRight.textContent = state.file ? indexStateText(state.file.index) : "";
      el.statusRight.title = state.file ? `index: ${state.file.indexDir}` : "";
      el.statusRight.className = "";
    }
  }

  // Shows `text` in the status bar for a moment, then the usual text again.
  function flashStatus(text, className = "", duration = STATUS_FLASH_MS) {
    el.statusRight.textContent = text;
    el.statusRight.title = text;
    el.statusRight.className = className;
    clearTimeout(state.statusTimer);
    state.statusTimer = setTimeout(() => showWarnings(state.warnings), duration);
  }

  function clearQuery() {
    resetQueryModel();
    applyQuery();
  }

  function setFiltersOpen(open, { focus = false } = {}) {
    el.filters.hidden = !open;
    el.filtersBtn.setAttribute("aria-expanded", open ? "true" : "false");
    storageSet(STORAGE_FILTERS_OPEN, open ? "1" : "0");
    if (open && focus) el.fFrom.focus();
  }

  // ---------- theme ----------

  // The page follows `mode` at once (theme.js), the button names it, and the
  // native title bar is asked to match; `auto` hands the title bar back to the OS.
  // The title bar is best effort: without the permission or the API the page is still right.
  function setTheme(mode) {
    Theme.write(mode);
    Theme.apply(mode);
    el.themeBtn.textContent = Theme.label(mode);
    el.themeBtn.title = Theme.title(mode);
    try {
      const current = window.__TAURI__.window.getCurrentWindow();
      current.setTheme(mode === "auto" ? null : mode).catch(() => {});
    } catch (_) {
      /* no window API in this host: the title bar keeps the OS theme */
    }
  }

  el.themeBtn.addEventListener("click", () => setTheme(Theme.next(Theme.read())));

  function focusList() {
    el.list.focus();
    if (state.selected < 0 && state.total > 0) select(0);
  }

  el.search.addEventListener("input", () => {
    state.query.text = el.search.value;
    syncFromText();
    applyQuery({ debounce: SEARCH_DEBOUNCE_MS });
  });

  el.search.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === "ArrowDown") {
      event.preventDefault();
      if (state.queryDirty) runQuery();
      focusList();
    } else if (event.key === "Escape") {
      event.preventDefault();
      if (el.search.value) {
        el.search.value = "";
        state.query.text = "";
        syncFromText();
        applyQuery();
      } else {
        el.list.focus();
      }
    }
  });

  // The panel inputs edit the operators of the field text.
  el.fFrom.addEventListener("input", () => {
    rewrite("from", el.fFrom.value, { debounce: SEARCH_DEBOUNCE_MS });
  });
  el.fAfter.addEventListener("change", () => {
    rewrite("after", el.fAfter.value);
  });
  el.fBefore.addEventListener("change", () => {
    rewrite("before", el.fBefore.value);
  });
  el.fAtt.addEventListener("change", () => {
    rewrite("has", el.fAtt.value === "" ? null : el.fAtt.value === "true");
  });
  el.fSort.addEventListener("change", () => {
    state.query.sort = el.fSort.value;
    applyQuery();
  });
  el.fThreads.addEventListener("change", () => {
    state.query.groupThreads = el.fThreads.checked;
    storageSet(STORAGE_GROUP_THREADS, el.fThreads.checked ? "1" : "0");
    applyQuery();
  });
  el.filters.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      el.list.focus();
    } else if (event.key === "Enter" && event.target === el.fFrom) {
      event.preventDefault();
      if (state.queryDirty) runQuery();
      focusList();
    }
  });
  el.filtersBtn.addEventListener("click", () => {
    setFiltersOpen(el.filters.hidden, { focus: true });
  });
  el.filtersClear.addEventListener("click", () => {
    clearQuery();
    el.search.focus();
  });

  // ---------- folders ----------

  // The key of a node of the label tree: the label itself, or the path of a
  // container (`Work` with only `Work/Invoices` present, the Categories node).
  function nodeKey(node, parentKey) {
    if (node.label != null) return node.label;
    return `${parentKey == null ? "" : `${parentKey}/`}${node.name}#`;
  }

  // The exact labels a click on the node with `key` sends (`anyLabels`).
  function labelsUnder(key) {
    if (key == null) return [];
    const node = state.labelIndex.get(key);
    return node ? node.labels : [key];
  }

  // Every node of the label tree by key, and the keys by the name the
  // field text uses: a label's own name, or the `/`-joined path of a
  // container (`Work`, `Categories`); a label wins over a container.
  function indexLabelTree() {
    state.labelIndex.clear();
    state.labelKeys.clear();
    const containers = new Map();
    const walk = (nodes, parentKey, parentPath) => {
      for (const node of nodes) {
        const key = nodeKey(node, parentKey);
        const path = parentPath == null ? node.name : `${parentPath}/${node.name}`;
        state.labelIndex.set(key, node);
        if (node.label != null) state.labelKeys.set(node.label, key);
        else if (!containers.has(path)) containers.set(path, key);
        walk(node.children, key, path);
      }
    };
    walk(state.file ? state.file.labels : [], null, null);
    for (const [path, key] of containers) {
      if (!state.labelKeys.has(path)) state.labelKeys.set(path, key);
    }
  }

  // The node key behind a `label:` value; an unknown name is its own key
  // (an exact label the index may not have).
  function findLabelKey(name) {
    return state.labelKeys.get(name) ?? name;
  }

  // The name the field text carries for the node with `key`: the label
  // itself, or the container path without the `#` marks.
  function labelName(key) {
    return key
      .split("/")
      .map((part) => part.replace(/#$/, ""))
      .join("/");
  }

  // The pane: one root row per remembered file, in the remembered order (an
  // open file the storage forgot is appended); under the active one its
  // "All mail" and its label tree, one level in. The other files stay folded.
  function renderFolders() {
    el.folders.replaceChildren();
    state.folderNodes.clear();
    const paths = state.remembered.paths.slice();
    for (const info of state.files) {
      if (!paths.some((path) => Files.samePath(path, info.path))) paths.push(info.path);
    }
    for (const path of paths) {
      const info = state.files.find((candidate) => Files.samePath(candidate.path, path)) || null;
      const active = Boolean(info && state.file && info.id === state.file.id);
      el.folders.appendChild(fileItem(path, info, active));
      if (active) renderLabels(1);
    }
    updateFolderSelection();
  }

  function renderLabels(depth) {
    const allMail = { name: "All mail", label: null, count: state.file.messageCount, total: 0, labels: [], children: [] };
    el.folders.appendChild(folderItem(allMail, null, depth));
    if (state.file.labels.length > 0) {
      const separator = document.createElement("div");
      separator.className = "folder-separator";
      el.folders.appendChild(separator);
    }
    const walk = (nodes, parentKey, level) => {
      for (const node of nodes) {
        const key = nodeKey(node, parentKey);
        state.folderNodes.set(key, node);
        el.folders.appendChild(folderItem(node, key, level));
        if (node.children.length > 0 && !state.foldersCollapsed.has(key)) {
          walk(node.children, key, level + 1);
        }
      }
    };
    walk(state.file.labels, null, depth);
  }

  // A file's root row. Loaded: a click makes it active (or, when it is,
  // selects its All mail); its × closes it. Not loaded (the index needs a
  // build, or the file was remembered and not reopened yet): a click opens it,
  // its × only forgets it.
  function fileItem(path, info, active) {
    const item = document.createElement("div");
    item.className = "folder root";
    if (active) item.classList.add("active");
    if (!info) item.classList.add("unloaded");
    item.setAttribute("role", "option");
    item.setAttribute("aria-selected", "false");
    item.dataset.path = path;
    if (info) item.dataset.file = String(info.id);
    item.style.setProperty("--depth", "0");
    item.title = info ? path : `${path}\nNot loaded: click to open (the search index may need a build)`;
    const twisty = document.createElement("span");
    twisty.className = "twisty";
    twisty.textContent = active ? "▾" : "▸";
    const nameSpan = document.createElement("span");
    nameSpan.className = "folder-name";
    nameSpan.textContent = Files.fileName(path);
    const countSpan = document.createElement("span");
    countSpan.className = "folder-count";
    countSpan.textContent = info ? fmt(info.messageCount) : "";
    const close = document.createElement("button");
    close.type = "button";
    close.className = "close-file";
    close.textContent = "×";
    close.title = info ? "Close this file" : "Remove this file from the list";
    close.setAttribute("aria-label", close.title);
    close.addEventListener("click", (event) => {
      event.stopPropagation();
      if (info) closeFile(info.id);
      else forgetFile(path);
    });
    item.append(twisty, nameSpan, countSpan, close);
    item.addEventListener("click", () => rootAction(item));
    return item;
  }

  // What a click (or the keyboard) does on a file's root row.
  function rootAction(item) {
    if (item.classList.contains("unloaded")) {
      openMbox([item.dataset.path]);
    } else if (item.classList.contains("active")) {
      selectLabel(null);
    } else {
      switchFile(Number(item.dataset.file));
    }
  }

  function folderItem(node, key, depth) {
    const item = document.createElement("div");
    item.className = "folder";
    if (node.label == null && key != null) item.classList.add("container");
    item.setAttribute("role", "option");
    item.dataset.label = key == null ? "" : key;
    item.style.setProperty("--depth", String(depth));
    item.title = node.label == null ? node.name : node.label;
    const twisty = document.createElement("span");
    twisty.className = "twisty";
    if (node.children.length > 0) {
      twisty.textContent = state.foldersCollapsed.has(key) ? "▸" : "▾";
      twisty.title = state.foldersCollapsed.has(key) ? "Show the sub-labels" : "Hide the sub-labels";
      twisty.addEventListener("click", (event) => {
        event.stopPropagation();
        toggleFolder(key);
      });
    }
    const nameSpan = document.createElement("span");
    nameSpan.className = "folder-name";
    nameSpan.textContent = node.name;
    const countSpan = document.createElement("span");
    countSpan.className = "folder-count";
    // A label shows its own messages; a container the messages under it.
    countSpan.textContent = fmt(node.label == null && key != null ? node.total : node.count);
    item.append(twisty, nameSpan, countSpan);
    item.addEventListener("click", () => selectLabel(key));
    return item;
  }

  function toggleFolder(key) {
    if (state.foldersCollapsed.has(key)) state.foldersCollapsed.delete(key);
    else state.foldersCollapsed.add(key);
    renderFolders();
    const item = folderItems().find((candidate) => candidate.dataset.label === key);
    if (item) item.scrollIntoView({ block: "nearest" });
  }

  function updateFolderSelection() {
    for (const item of el.folders.children) {
      if (!item.classList.contains("folder") || item.classList.contains("root")) continue;
      const label = item.dataset.label === "" ? null : item.dataset.label;
      item.setAttribute("aria-selected", label === state.parsed.label ? "true" : "false");
    }
  }

  // A click on a node writes `label:` into the field; All mail drops every label.
  function selectLabel(label) {
    if (label === state.parsed.label) return;
    rewrite("label", label == null ? null : labelName(label));
  }

  function folderIndexOf(label) {
    const items = folderItems();
    return items.findIndex(
      (item) => !item.classList.contains("root") && (item.dataset.label === "" ? null : item.dataset.label) === label,
    );
  }

  // The rows the keyboard walks: the label rows and the other loaded files'
  // roots (an arrow onto one switches). The active file's own root is passed
  // over, its All mail row stands for it; a file that is not loaded is skipped
  // too, an arrow key must not start a build.
  function folderItems() {
    return Array.from(el.folders.children).filter(
      (item) =>
        item.classList.contains("folder") &&
        !item.classList.contains("unloaded") &&
        !item.classList.contains("active"),
    );
  }

  el.folders.addEventListener("keydown", (event) => {
    const items = folderItems();
    if (items.length === 0) return;
    let index = folderIndexOf(state.parsed.label);
    const node = state.parsed.label == null ? null : state.folderNodes.get(state.parsed.label);
    const hasChildren = Boolean(node && node.children.length > 0);
    const collapsed = hasChildren && state.foldersCollapsed.has(state.parsed.label);
    if (event.key === "ArrowDown") index = Math.min(items.length - 1, index + 1);
    else if (event.key === "ArrowUp") index = Math.max(0, index - 1);
    else if (event.key === "Home") index = 0;
    else if (event.key === "End") index = items.length - 1;
    else if (event.key === "ArrowRight" && collapsed) {
      // Open the node first; a second press moves on to the list.
      event.preventDefault();
      toggleFolder(state.parsed.label);
      return;
    } else if (event.key === "ArrowLeft") {
      event.preventDefault();
      if (hasChildren && !collapsed) toggleFolder(state.parsed.label);
      return;
    } else if (event.key === "Enter" || event.key === "ArrowRight") {
      el.list.focus();
      event.preventDefault();
      return;
    } else return;
    event.preventDefault();
    items[index].scrollIntoView({ block: "nearest" });
    if (items[index].classList.contains("root")) {
      rootAction(items[index]);
      return;
    }
    const label = items[index].dataset.label === "" ? null : items[index].dataset.label;
    selectLabel(label);
  });

  // ---------- virtualized list ----------

  // Installs a new listing. With `keepSelection` the selected message stays
  // selected (and its reader view stays) when it is still in the listing.
  async function resetListing(generation, total, { keepSelection = false } = {}) {
    const keepOffset = keepSelection ? state.selectedOffset : null;
    const token = state.queryToken;
    state.generation = generation;
    state.layout++;
    state.total = total;
    state.blocks.clear();
    state.pending.clear();
    state.selected = -1;
    el.spacer.style.height = `${total * ROW_H}px`;
    el.listEmpty.hidden = total > 0;
    el.listEmpty.textContent = anyNarrowing() ? "No messages match" : "No messages";
    updateStatusCenter();

    let position = null;
    if (keepOffset != null && total > 0) {
      let info = null;
      try {
        info = await invoke("position_of", { file: state.file.id, offset: keepOffset });
      } catch (_) {
        info = null;
      }
      if (token !== state.queryToken) return;
      if (info && info.generation === state.generation) {
        position = info.position;
        // The lookup may have opened the conversation around the message.
        if (info.total !== state.total) setTotal(info.total);
      }
    }
    if (position == null) {
      state.selectedOffset = null;
      clearReader();
      el.list.scrollTop = 0;
      render();
    } else {
      // Same message, maybe new words to highlight.
      if (state.view) renderMessage(state.view, { keepScroll: true });
      select(position);
    }
  }

  // Rows shifted without a new listing: drop the cached pages and re-render.
  function setTotal(total) {
    state.layout++;
    state.total = total;
    state.blocks.clear();
    state.pending.clear();
    el.spacer.style.height = `${total * ROW_H}px`;
    el.listEmpty.hidden = total > 0;
  }

  function updateStatusCenter() {
    if (!state.file) {
      el.statusCenter.textContent = "";
      return;
    }
    const all = state.file.messageCount;
    const messages =
      state.messages === all ? `${fmt(all)} messages` : `${fmt(state.messages)} of ${fmt(all)} messages`;
    el.statusCenter.textContent = state.query.groupThreads
      ? `${fmt(state.conversations)} conversations · ${messages}`
      : messages;
  }

  function rowData(index) {
    const block = Math.floor(index / BLOCK);
    const rows = state.blocks.get(block);
    return rows ? rows[index - block * BLOCK] || null : null;
  }

  function requestBlock(block) {
    if (!state.file || state.blocks.has(block) || state.pending.has(block)) return;
    const generation = state.generation;
    const layout = state.layout;
    const promise = invoke("list_rows", { file: state.file.id, start: block * BLOCK, count: BLOCK })
      .then((page) => {
        if (state.pending.get(block) === promise) state.pending.delete(block);
        if (page.generation !== state.generation || generation !== state.generation) return;
        if (layout !== state.layout) return; // the rows moved under this page
        state.blocks.set(block, page.rows);
        evictBlocks();
        scheduleRender();
      })
      .catch((err) => {
        if (state.pending.get(block) === promise) state.pending.delete(block);
        console.error("list_rows failed", err);
      });
    state.pending.set(block, promise);
  }

  function evictBlocks() {
    if (state.blocks.size <= MAX_BLOCKS) return;
    const centre = Math.floor((el.list.scrollTop + el.list.clientHeight / 2) / ROW_H / BLOCK);
    const farthest = Array.from(state.blocks.keys()).sort(
      (a, b) => Math.abs(b - centre) - Math.abs(a - centre),
    );
    for (const block of farthest.slice(0, state.blocks.size - MAX_BLOCKS)) {
      state.blocks.delete(block);
    }
  }

  function scheduleRender() {
    if (state.renderQueued) return;
    state.renderQueued = true;
    requestAnimationFrame(() => {
      state.renderQueued = false;
      render();
    });
  }

  function makeRow() {
    const row = document.createElement("div");
    row.className = "row";
    row.setAttribute("role", "option");
    for (const cls of ["twisty", "att", "from", "subject", "count", "date"]) {
      const span = document.createElement("span");
      span.className = cls;
      row.appendChild(span);
    }
    row.firstChild.addEventListener("click", (event) => {
      const index = Number(row.dataset.index);
      const data = Number.isNaN(index) ? null : rowData(index);
      if (data && data.kind === "thread") {
        event.stopPropagation();
        toggleThread(index);
        el.list.focus();
      }
    });
    row.addEventListener("click", () => {
      const index = Number(row.dataset.index);
      if (!Number.isNaN(index)) select(index, { scroll: false });
      el.list.focus();
    });
    return row;
  }

  // Opens or closes the conversation whose thread row is `row`; the rows
  // below it shift, the selection follows.
  async function toggleThread(row) {
    const data = rowData(row);
    if (!data || data.kind !== "thread") return;
    const generation = state.generation;
    let info;
    try {
      info = await invoke("toggle_thread", { file: state.file.id, generation, row });
    } catch (err) {
      console.error("toggle_thread failed", err);
      return;
    }
    if (info.generation !== state.generation || generation !== state.generation) return;
    const delta = info.total - state.total;
    if (delta === 0) return;
    setTotal(info.total);
    if (state.selected > row) {
      // Rows below move with the delta; a member of a closed conversation
      // lands on its thread row, whose message the reader then shows.
      state.selected = Math.max(row, state.selected + delta);
    }
    updateStatusCenter();
    render();
    scheduleRead();
  }

  function render() {
    const total = state.total;
    const scrollTop = el.list.scrollTop;
    const height = el.list.clientHeight || 1;
    const first = Math.max(0, Math.floor(scrollTop / ROW_H) - OVERSCAN);
    const visible = Math.ceil(height / ROW_H) + 2 * OVERSCAN;
    const last = Math.min(total, first + visible);
    const needed = Math.max(0, last - first);
    const terms = state.highlightTerms;

    while (el.rows.children.length < needed) el.rows.appendChild(makeRow());
    while (el.rows.children.length > needed) el.rows.lastChild.remove();
    el.rows.style.transform = `translateY(${first * ROW_H}px)`;

    for (let i = 0; i < needed; i++) {
      const index = first + i;
      const row = el.rows.children[i];
      row.dataset.index = String(index);
      row.setAttribute("aria-selected", index === state.selected ? "true" : "false");
      const data = rowData(index);
      const [twisty, att, from, subject, count, date] = row.children;
      if (data) {
        row.classList.remove("placeholder");
        const isThread = data.kind === "thread";
        row.classList.toggle("thread", isThread);
        row.classList.toggle("member", data.kind === "member");
        twisty.textContent = isThread ? (data.expanded ? "▾" : "▸") : "";
        twisty.title = isThread ? (data.expanded ? "Close the conversation" : "Open the conversation") : "";
        att.textContent = data.hasAttachment ? "📎" : "";
        renderHighlighted(from, senderName(data.from), terms);
        from.title = data.from.join(", ");
        const subjectLine = isThread ? stripReplyPrefix(subjectText(data.subject)) : subjectText(data.subject);
        renderHighlighted(subject, subjectLine, terms);
        subject.title = subjectText(data.subject);
        count.textContent = isThread ? fmt(data.threadCount) : "";
        count.title = isThread ? `${fmt(data.threadCount)} messages in this conversation` : "";
        date.textContent = listDate(data.date);
        date.title = fullDate(data.date);
      } else {
        row.classList.add("placeholder");
        row.classList.remove("thread", "member");
        twisty.textContent = "";
        twisty.title = "";
        att.textContent = "";
        from.textContent = "…";
        from.title = "";
        subject.textContent = "";
        subject.title = "";
        count.textContent = "";
        count.title = "";
        date.textContent = "";
        date.title = "";
        requestBlock(Math.floor(index / BLOCK));
      }
    }
  }

  el.list.addEventListener("scroll", scheduleRender);
  window.addEventListener("resize", () => {
    scheduleRender();
    syncHtmlHeight();
  });

  function visibleRows() {
    return Math.max(1, Math.floor(el.list.clientHeight / ROW_H));
  }

  function select(index, { scroll = true } = {}) {
    if (state.total === 0) return;
    index = Math.max(0, Math.min(state.total - 1, index));
    state.selected = index;
    if (scroll) {
      const top = index * ROW_H;
      const bottom = top + ROW_H;
      if (top < el.list.scrollTop) el.list.scrollTop = top;
      else if (bottom > el.list.scrollTop + el.list.clientHeight) {
        el.list.scrollTop = bottom - el.list.clientHeight;
      }
    }
    render();
    scheduleRead();
  }

  function scheduleRead() {
    clearTimeout(state.readTimer);
    state.readTimer = setTimeout(readSelected, READ_DEBOUNCE_MS);
  }

  async function readSelected() {
    const index = state.selected;
    if (index < 0) return;
    const data = rowData(index);
    if (!data) {
      // The block is still on its way; try again once it should be here.
      requestBlock(Math.floor(index / BLOCK));
      scheduleRead();
      return;
    }
    if (data.offset === state.selectedOffset) return;
    const token = ++state.readToken;
    try {
      const view = await invoke("read_message", { file: state.file.id, offset: data.offset, len: data.len });
      if (token !== state.readToken || state.selected !== index) return;
      state.selectedOffset = data.offset;
      state.view = view;
      resetBodyState();
      renderMessage(view);
    } catch (err) {
      if (token !== state.readToken) return;
      clearReader();
      el.readerEmpty.textContent = `Could not read the message: ${err}`;
    }
  }

  el.list.addEventListener("keydown", (event) => {
    if (state.total === 0) return;
    const current = state.selected < 0 ? -1 : state.selected;
    let next = null;
    switch (event.key) {
      case "ArrowDown":
        next = current + 1;
        break;
      case "ArrowUp":
        next = Math.max(0, current - 1);
        break;
      case "PageDown":
        next = current + visibleRows() - 1;
        break;
      case "PageUp":
        next = current - visibleRows() + 1;
        break;
      case "Home":
        next = 0;
        break;
      case "End":
        next = state.total - 1;
        break;
      case "ArrowRight": {
        // A closed conversation opens first; anything else moves to the reader.
        event.preventDefault();
        const data = current >= 0 ? rowData(current) : null;
        if (data && data.kind === "thread" && !data.expanded) toggleThread(current);
        else if (state.selected >= 0) el.reader.focus();
        return;
      }
      case "Enter":
        if (state.selected >= 0) el.reader.focus();
        event.preventDefault();
        return;
      case "ArrowLeft": {
        // An open conversation closes (from its thread row or a member);
        // anything else moves to the folders.
        event.preventDefault();
        const data = current >= 0 ? rowData(current) : null;
        if (data && data.kind === "member") {
          const threadRow = current - data.threadIndex - 1;
          select(threadRow);
          toggleThread(threadRow);
        } else if (data && data.kind === "thread" && data.expanded) {
          toggleThread(current);
        } else {
          el.folders.focus();
        }
        return;
      }
      default:
        return;
    }
    event.preventDefault();
    select(next);
  });

  // ---------- reader ----------

  function clearReader() {
    state.view = null;
    resetBodyState();
    if (el.msgHtml.hasAttribute("srcdoc")) el.msgHtml.removeAttribute("srcdoc");
    if (el.printFrame.hasAttribute("srcdoc")) el.printFrame.removeAttribute("srcdoc");
    el.readerContent.hidden = true;
    el.readerEmpty.hidden = false;
    el.readerEmpty.textContent = "Select a message";
    el.reader.scrollTop = 0;
  }

  // Forgets what belonged to the previous message's body.
  function resetBodyState() {
    state.sourceMode = false;
    state.sanitized = null;
    state.allowRemote = false;
    state.source = null;
    state.cidUrls = new Map();
    for (const url of state.blobUrls) URL.revokeObjectURL(url);
    state.blobUrls = [];
    state.cidBlobs = new Map();
    state.printing = false;
    if (el.printFrame.hasAttribute("srcdoc")) el.printFrame.removeAttribute("srcdoc");
  }

  function headerRow(term, value) {
    const dt = document.createElement("dt");
    dt.textContent = term;
    const dd = document.createElement("dd");
    dd.textContent = value;
    return [dt, dd];
  }

  function renderMessage(view, { keepScroll = false } = {}) {
    const terms = state.highlightTerms;
    renderHighlighted(el.msgSubject, subjectText(view.subject), terms);
    const rows = [];
    rows.push(...headerRow("From", view.from.length ? view.from.join(", ") : "(unknown sender)"));
    if (view.to.length) rows.push(...headerRow("To", view.to.join(", ")));
    if (view.cc.length) rows.push(...headerRow("Cc", view.cc.join(", ")));
    rows.push(...headerRow("Date", fullDate(view.date)));
    if (view.labels.length) rows.push(...headerRow("Labels", view.labels.join(", ")));
    if (view.messageId) rows.push(...headerRow("Message-ID", view.messageId));
    el.msgHeaders.replaceChildren(...rows);

    renderAttachments(view);
    renderBody(view, { keepScroll: true });

    const parts = view.parts.map((part) => {
      const li = document.createElement("li");
      let text = `${"  ".repeat(part.depth)}${part.contentType}`;
      if (part.encoding) text += ` [${part.encoding}]`;
      if (part.name) text += ` "${part.name}"`;
      text += ` ${fmt(part.size)} B`;
      li.textContent = text;
      li.style.whiteSpace = "pre";
      return li;
    });
    el.msgPartsList.replaceChildren(...parts);

    el.readerEmpty.hidden = true;
    el.readerContent.hidden = false;
    if (!keepScroll) el.reader.scrollTop = 0;
  }

  // ---------- attachments ----------
  //
  // One chip per listed attachment (named, or a nested message): the chip
  // opens it with the default application through a temporary file, the Save
  // button next to it asks where to put it. Executables cannot be opened.

  // The attachments the strip shows; nameless inline parts only serve the cid images.
  function listedAttachments(view) {
    return view.attachments.filter((a) => a.name || a.isMessage);
  }

  function attachmentName(attachment) {
    return attachment.name || "message.eml";
  }

  function attachmentIcon(attachment) {
    if (attachment.isMessage) return "✉";
    if (attachment.contentType.startsWith("image/")) return "🖼";
    return "📎";
  }

  function renderAttachments(view) {
    const listed = listedAttachments(view);
    el.attachmentList.replaceChildren(...listed.map((a) => attachmentItem(view, a)));
    el.saveAllBtn.hidden = listed.length < 2;
    el.attachments.hidden = listed.length === 0;
  }

  function attachmentItem(view, attachment) {
    const name = attachmentName(attachment);
    const args = { file: view.file, offset: view.offset, len: view.len, index: attachment.index };
    const item = document.createElement("li");
    item.className = "attachment-item";

    const open = document.createElement("button");
    open.type = "button";
    open.className = "attachment";
    const icon = document.createElement("span");
    icon.className = "att-icon";
    icon.textContent = attachmentIcon(attachment);
    const label = document.createElement("span");
    label.className = "att-name";
    label.textContent = name;
    const size = document.createElement("span");
    size.className = "att-size";
    size.textContent = formatBytes(attachment.size);
    open.append(icon, label, size);
    if (attachment.inline) {
      const tag = document.createElement("span");
      tag.className = "att-inline";
      tag.textContent = "inline";
      open.appendChild(tag);
    }
    if (attachment.canOpen) {
      open.title = `Open ${name} (${attachment.contentType})`;
      open.addEventListener("click", () => runAttachmentAction(open, "open_attachment", args));
    } else {
      open.disabled = true;
      open.title = `${name}: executable attachments can only be saved`;
    }

    const save = document.createElement("button");
    save.type = "button";
    save.className = "attachment-save";
    save.textContent = "Save…";
    save.title = `Save ${name}…`;
    save.addEventListener("click", () =>
      runAttachmentAction(save, "save_attachment", args, (path) => {
        if (path) flashStatus(`Saved ${path}`, "", SAVED_FLASH_MS);
      }),
    );

    item.append(open, save);
    return item;
  }

  // Runs one attachment command behind `button`, once at a time; a cancelled
  // dialog gives `null` and says nothing.
  async function runAttachmentAction(button, command, args, onDone = () => {}) {
    if (button.classList.contains("busy")) return;
    button.classList.add("busy");
    try {
      const result = await invoke(command, args);
      onDone(result);
    } catch (err) {
      flashStatus(sentence(String(err)), "error", SAVED_FLASH_MS);
    } finally {
      button.classList.remove("busy");
    }
  }

  el.saveAllBtn.addEventListener("click", () => {
    const view = state.view;
    if (!view) return;
    runAttachmentAction(
      el.saveAllBtn,
      "save_all_attachments",
      { file: view.file, offset: view.offset, len: view.len },
      (report) => {
        if (report) {
          const files = report.saved === 1 ? "1 file" : `${report.saved} files`;
          flashStatus(`${files} saved to ${report.dir}`, "", SAVED_FLASH_MS);
        }
      },
    );
  });

  // The Content-ID a `cid:` reference names: percent-decoding undone, angle
  // brackets and surrounding space removed.
  function normalizeCid(value) {
    let cid = String(value || "").trim();
    try {
      cid = decodeURIComponent(cid);
    } catch (_) {
      /* not percent-encoded */
    }
    return cid.replace(/^<|>$/g, "").trim();
  }

  // The attachment a `cid:` reference points at: an exact Content-ID match,
  // else a case-insensitive one.
  function findCidAttachment(attachments, reference) {
    const wanted = normalizeCid(reference);
    if (!wanted) return null;
    const exact = attachments.find((a) => a.contentId === wanted);
    if (exact) return exact;
    const lower = wanted.toLowerCase();
    return attachments.find((a) => a.contentId && a.contentId.toLowerCase() === lower) || null;
  }

  // Gives the cid images of the body frame their bytes: each referenced part
  // is read once per message and served as a blob URL (the frame's CSP allows
  // `blob:` images). An image whose cid matches nothing keeps its placeholder.
  function fillCidImages(doc) {
    const view = state.view;
    if (!view || view.attachments.length === 0) return;
    const offset = view.offset;
    for (const img of doc.querySelectorAll("img[data-mboxer-cid]")) {
      const attachment = findCidAttachment(view.attachments, img.getAttribute("data-mboxer-cid"));
      if (!attachment) continue;
      let pending = state.cidUrls.get(attachment.index);
      if (!pending) {
        pending = invoke("read_part", { file: view.file, offset, len: view.len, index: attachment.index }).then((bytes) => {
          const blob = new Blob([bytes], { type: attachment.contentType });
          const url = URL.createObjectURL(blob);
          state.blobUrls.push(url);
          state.cidBlobs.set(url, blob);
          return url;
        });
        state.cidUrls.set(attachment.index, pending);
      }
      pending
        .then((url) => {
          if (!state.view || state.view.offset !== offset || !img.isConnected) return;
          img.addEventListener("load", syncHtmlHeight);
          img.setAttribute("src", url);
          img.removeAttribute("data-mboxer-cid");
        })
        .catch((err) => console.error("read_part failed", err));
    }
  }

  // ---------- body views ----------

  // Which view the body gets: Source when asked for this message, else the
  // sticky choice, else Text when the message has no HTML.
  function bodyMode(view) {
    if (state.sourceMode) return "source";
    return state.bodyMode === "html" && view.html ? "html" : "text";
  }

  function renderBody(view, { keepScroll = false } = {}) {
    const mode = bodyMode(view);
    el.viewHtml.disabled = !view.html;
    for (const button of [el.viewHtml, el.viewText, el.viewSource]) {
      button.setAttribute("aria-pressed", button.dataset.mode === mode ? "true" : "false");
    }
    el.remoteBar.hidden = true;
    el.msgHtml.hidden = mode !== "html";
    el.msgBody.hidden = mode === "html";
    el.msgBody.className = "";
    if (mode === "html") showHtml(view);
    else if (mode === "source") showSource(view);
    else showText(view);
    if (mode !== "html" && el.msgHtml.hasAttribute("srcdoc")) el.msgHtml.removeAttribute("srcdoc");
    if (!keepScroll) el.reader.scrollTop = 0;
  }

  function showText(view) {
    if (!view.body) {
      el.msgBody.textContent = "(no text body)";
      return;
    }
    if (view.body.kind === "raw") {
      el.msgBody.textContent = view.body.text;
      el.msgBody.classList.add("raw");
      return;
    }
    renderHighlighted(el.msgBody, view.body.text, state.highlightTerms);
    if (view.body.kind === "htmlAsText") el.msgBody.classList.add("html-as-text");
  }

  function showHtml(view) {
    if (!state.sanitized || state.sanitized.offset !== view.offset) {
      const { doc, remote } = sanitizeHtml(view.html);
      state.sanitized = { offset: view.offset, doc, remote, length: view.html.length };
    }
    const sanitized = state.sanitized;
    el.msgHtml.srcdoc = htmlSrcdoc(sanitized, {
      allowRemote: state.allowRemote,
      terms: state.highlightTerms,
    });
    if (sanitized.remote > 0 && !state.allowRemote) {
      el.remoteText.textContent =
        sanitized.remote === 1
          ? "1 remote image or style blocked"
          : `${fmt(sanitized.remote)} remote images or styles blocked`;
      el.remoteBar.hidden = false;
    }
  }

  function showSource(view) {
    el.msgBody.classList.add("source");
    if (state.source && state.source.offset === view.offset) {
      el.msgBody.textContent = state.source.text;
      return;
    }
    el.msgBody.textContent = "Reading the source…";
    const offset = view.offset;
    invoke("read_source", { file: view.file, offset, len: view.len })
      .then((text) => {
        if (!state.view || state.view.offset !== offset) return;
        state.source = { offset, text };
        if (bodyMode(state.view) === "source") el.msgBody.textContent = text;
      })
      .catch((err) => {
        if (!state.view || state.view.offset !== offset) return;
        el.msgBody.textContent = `Could not read the source: ${err}`;
      });
  }

  function setBodyMode(mode) {
    if (mode === "source") {
      state.sourceMode = true;
    } else {
      state.sourceMode = false;
      state.bodyMode = mode;
      storageSet(STORAGE_BODY_MODE, mode);
    }
    if (state.view) renderBody(state.view);
  }

  for (const button of [el.viewHtml, el.viewText, el.viewSource]) {
    button.addEventListener("click", () => setBodyMode(button.dataset.mode));
  }

  el.remoteAllow.addEventListener("click", () => {
    state.allowRemote = true;
    if (state.view) renderBody(state.view, { keepScroll: true });
  });

  // The iframe is sized to its content; the sandbox keeps `allow-same-origin`
  // (and nothing else, so no script runs inside) for exactly this access.
  function syncHtmlHeight() {
    const doc = el.msgHtml.contentDocument;
    if (el.msgHtml.hidden || !doc || !doc.documentElement) return;
    let height = Math.ceil(doc.documentElement.getBoundingClientRect().height);
    if (doc.body) height = Math.max(height, doc.body.scrollHeight);
    if (doc.documentElement.scrollWidth > el.msgHtml.clientWidth) height += 20; // the horizontal scrollbar
    el.msgHtml.style.height = `${Math.max(40, height)}px`;
  }

  el.msgHtml.addEventListener("load", () => {
    const doc = el.msgHtml.contentDocument;
    if (!doc) return;
    syncHtmlHeight();
    for (const img of doc.images) img.addEventListener("load", syncHtmlHeight);
    fillCidImages(doc);
    doc.addEventListener("click", onFrameClick);
    doc.addEventListener("auxclick", onFrameClick);
    doc.addEventListener("keydown", onFrameKeydown);
  });

  // Links leave through the backend to the system browser; the sandbox would
  // otherwise navigate the frame itself (and the shell's CSP would stop it).
  function onFrameClick(event) {
    const anchor = event.target && event.target.closest ? event.target.closest("a[href]") : null;
    if (!anchor) return;
    const href = anchor.getAttribute("href") || "";
    if (href.trim().startsWith("#")) return;
    event.preventDefault();
    if (event.type === "auxclick" && event.button !== 1) return;
    invoke("open_url", { url: href }).catch((err) => {
      flashStatus(`Could not open the link: ${err}`, "error");
    });
  }

  function onFrameKeydown(event) {
    if (event.key === "Escape" || event.key === "ArrowLeft") {
      el.list.focus();
      event.preventDefault();
      return;
    }
    globalShortcut(event);
  }

  async function copyBody() {
    const view = state.view;
    if (!view) return;
    const mode = bodyMode(view);
    let text = "";
    if (mode === "html") {
      const doc = el.msgHtml.contentDocument;
      text = doc && doc.body ? doc.body.innerText : "";
    } else if (mode === "source") {
      text = state.source && state.source.offset === view.offset ? state.source.text : "";
    } else {
      text = view.body ? view.body.text : "";
    }
    try {
      await writeClipboard(text);
      flashStatus("Copied");
    } catch (err) {
      flashStatus(`Copy failed: ${err}`, "error");
    }
  }

  async function writeClipboard(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      try {
        await navigator.clipboard.writeText(text);
        return;
      } catch (_) {
        /* fall back to the selection command */
      }
    }
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    const ok = document.execCommand("copy");
    area.remove();
    if (!ok) throw new Error("the clipboard is not available");
  }

  el.copyBtn.addEventListener("click", copyBody);

  // ---------- export: save as .eml, print / PDF (task 011) ----------
  //
  // Print lays the message out as its own document in the hidden print frame
  // (the headers, then the body as the reader shows it) and prints that frame
  // alone: WebView2's print preview lists the printers and "Save as PDF". The
  // frame's sandbox keeps `allow-same-origin` (the document is ours) and adds
  // `allow-modals`, without which Chromium ignores print(); no script runs in it.
  // On a Mac WKWebView ignores print() on a frame (task 020): the same document,
  // its cid images inlined as `data:` URLs, goes to a print window of its own
  // that opens the native print panel (`print_message`).

  function printHeader(doc, view) {
    const header = doc.createElement("header");
    header.className = "mboxer-print-header";
    const h1 = doc.createElement("h1");
    h1.textContent = subjectText(view.subject);
    const dl = doc.createElement("dl");
    const row = (term, value) => {
      const dt = doc.createElement("dt");
      dt.textContent = term;
      const dd = doc.createElement("dd");
      dd.textContent = value;
      dl.append(dt, dd);
    };
    row("From", view.from.length ? view.from.join(", ") : "(unknown sender)");
    if (view.to.length) row("To", view.to.join(", "));
    if (view.cc.length) row("Cc", view.cc.join(", "));
    row("Date", fullDate(view.date));
    if (view.labels.length) row("Labels", view.labels.join(", "));
    const listed = listedAttachments(view);
    if (listed.length) {
      row("Attachments", listed.map((a) => `${attachmentName(a)} (${formatBytes(a.size)})`).join(", "));
    }
    header.append(h1, dl);
    return header;
  }

  // The print document of the current view: the HTML view is a copy of the
  // body frame's live document (cid and allowed remote images included, the
  // search marks unwrapped), the Text and Source views a <pre>. Null when the
  // HTML frame has nothing yet.
  function printDocument(view) {
    const mode = bodyMode(view);
    let doc;
    if (mode === "html") {
      const live = el.msgHtml.contentDocument;
      if (!live || !live.documentElement || !live.body) return null;
      doc = document.implementation.createHTMLDocument("");
      doc.replaceChild(doc.importNode(live.documentElement, true), doc.documentElement);
      for (const mark of doc.querySelectorAll("mark")) mark.replaceWith(...mark.childNodes);
    } else {
      doc = document.implementation.createHTMLDocument("");
      const charset = doc.createElement("meta");
      charset.setAttribute("charset", "utf-8");
      const csp = doc.createElement("meta");
      csp.setAttribute("http-equiv", "Content-Security-Policy");
      csp.setAttribute("content", "default-src 'none'; style-src 'unsafe-inline'");
      doc.head.append(charset, csp);
      const pre = doc.createElement("pre");
      pre.className = "mboxer-print-body";
      let note = "";
      if (mode === "source") {
        pre.classList.add("source");
        pre.textContent = state.source && state.source.offset === view.offset ? state.source.text : "";
      } else if (!view.body) {
        pre.textContent = "(no text body)";
      } else {
        pre.textContent = view.body.text;
        if (view.body.kind === "htmlAsText") note = "HTML message shown as text";
        else if (view.body.kind === "raw") note = "This message could not be parsed; raw source shown";
      }
      if (note) {
        const p = doc.createElement("p");
        p.className = "mboxer-print-note";
        p.textContent = note;
        doc.body.appendChild(p);
      }
      doc.body.appendChild(pre);
    }
    doc.title = subjectText(view.subject);
    const style = doc.createElement("style");
    style.textContent = PRINT_STYLE;
    doc.head.appendChild(style);
    doc.body.prepend(printHeader(doc, view));
    return doc;
  }

  function serializeDocument(doc) {
    return `<!doctype html>${doc.documentElement.outerHTML}`;
  }

  function printMessage() {
    const view = state.view;
    if (!view || state.printing) return;
    if (bodyMode(view) === "source" && !(state.source && state.source.offset === view.offset)) {
      flashStatus("The source is still loading, try again in a moment", "error");
      return;
    }
    const doc = printDocument(view);
    if (!doc) {
      flashStatus("The message is still loading, try again in a moment", "error");
      return;
    }
    state.printing = true;
    if (Print.usesPrintWindow(Files.os)) {
      printInWindow(view, doc);
      return;
    }
    el.printFrame.srcdoc = serializeDocument(doc);
  }

  // macOS: the print document, its cid images inlined, goes to a print
  // window, which shows it and opens the print panel once it has loaded.
  async function printInWindow(view, doc) {
    const { file, offset } = view;
    try {
      await Print.inlineBlobImages(doc, (url) => state.cidBlobs.get(url));
      await invoke("print_message", {
        html: serializeDocument(doc),
        subject: subjectText(view.subject),
        allowRemote: bodyMode(view) === "html" && state.allowRemote,
      });
    } catch (err) {
      flashStatus(`Print failed: ${err}`, "error");
    } finally {
      if (state.view && state.view.file === file && state.view.offset === offset) state.printing = false;
    }
  }

  // The frame has loaded the print document and its images: print it. The
  // document stays in the frame until the message changes, so the preview
  // keeps its content whether or not print() returns before it closes.
  el.printFrame.addEventListener("load", () => {
    if (!state.printing) return;
    state.printing = false;
    const win = el.printFrame.contentWindow;
    if (!win) return;
    try {
      win.print();
    } catch (err) {
      flashStatus(`Print failed: ${err}`, "error");
    }
  });

  function saveEml() {
    const view = state.view;
    if (!view) return;
    runAttachmentAction(
      el.saveEmlBtn,
      "save_message_eml",
      { file: view.file, offset: view.offset, len: view.len },
      (path) => {
        if (path) flashStatus(`Saved ${path}`, "", SAVED_FLASH_MS);
      },
    );
  }

  el.saveEmlBtn.addEventListener("click", saveEml);
  el.printBtn.addEventListener("click", printMessage);

  el.reader.addEventListener("keydown", (event) => {
    if (event.key === "Escape" || event.key === "ArrowLeft") {
      el.list.focus();
      event.preventDefault();
    }
  });

  // ---------- splitters ----------

  function loadWidths() {
    try {
      const saved = JSON.parse(storageGet(STORAGE_WIDTHS) || "{}");
      if (saved.folders) el.panes.style.setProperty("--w-folders", `${saved.folders}px`);
      if (saved.list) el.panes.style.setProperty("--w-list", `${saved.list}px`);
    } catch (_) {
      /* ignore a corrupt value */
    }
  }

  function currentWidth(target) {
    const value = getComputedStyle(el.panes).getPropertyValue(`--w-${target}`);
    return parseFloat(value) || (target === "folders" ? 220 : 480);
  }

  for (const splitter of document.querySelectorAll(".splitter")) {
    const target = splitter.dataset.target;
    let startX = 0;
    let startWidth = 0;
    splitter.addEventListener("pointerdown", (event) => {
      startX = event.clientX;
      startWidth = currentWidth(target);
      splitter.setPointerCapture(event.pointerId);
      splitter.classList.add("dragging");
      event.preventDefault();
    });
    splitter.addEventListener("pointermove", (event) => {
      if (!splitter.hasPointerCapture(event.pointerId)) return;
      const width = Math.max(120, Math.min(600, startWidth + event.clientX - startX));
      el.panes.style.setProperty(`--w-${target}`, `${width}px`);
      scheduleRender();
      syncHtmlHeight();
    });
    splitter.addEventListener("pointerup", (event) => {
      if (!splitter.hasPointerCapture(event.pointerId)) return;
      splitter.releasePointerCapture(event.pointerId);
      splitter.classList.remove("dragging");
      storageSet(
        STORAGE_WIDTHS,
        JSON.stringify({ folders: currentWidth("folders"), list: currentWidth("list") }),
      );
    });
  }

  // ---------- wiring ----------

  el.openBtn.addEventListener("click", () => openMbox(null));
  el.emptyOpenBtn.addEventListener("click", () => openMbox(null));
  el.errorOpenBtn.addEventListener("click", () => openMbox(null));
  el.retryBtn.addEventListener("click", () => {
    if (state.errorPath) openMbox([state.errorPath]);
  });
  el.cancelBtn.addEventListener("click", () => {
    el.cancelBtn.disabled = true;
    invoke("cancel_open").finally(() => {
      el.cancelBtn.disabled = false;
    });
  });

  // Ctrl+O / Ctrl+F / Ctrl+K / Ctrl+Shift+F / Ctrl+P / Ctrl+S (⌘ on a Mac), from the
  // document or the body iframe. Ctrl+P is taken over even without a message: the
  // webview's own default would print the whole window.
  function globalShortcut(event) {
    if (!(event.ctrlKey || event.metaKey)) return;
    const key = event.key.toLowerCase();
    if (key === "o") {
      event.preventDefault();
      openMbox(null);
    } else if (key === "p") {
      event.preventDefault();
      printMessage();
    } else if (key === "s") {
      event.preventDefault();
      saveEml();
    } else if (key === "f" && event.shiftKey) {
      event.preventDefault();
      if (state.file) setFiltersOpen(el.filters.hidden, { focus: true });
    } else if (key === "f" || key === "k") {
      event.preventDefault();
      if (state.file) {
        el.search.focus();
        el.search.select();
      }
    }
  }

  document.addEventListener("keydown", globalShortcut);

  // The markup names the shortcuts the Windows way; on a Mac the tooltips, the
  // search placeholder and the empty screen's button say ⌘ (and ⇧⌘ for Shift).
  function nameShortcutsForMac() {
    if (Files.os !== "mac") return;
    const mac = (text) => text.replace(/Ctrl\+Shift\+/g, "⇧⌘").replace(/Ctrl\+/g, "⌘");
    for (const node of document.querySelectorAll("[title*='Ctrl+']")) node.title = mac(node.title);
    for (const node of document.querySelectorAll("[placeholder*='Ctrl+']")) node.placeholder = mac(node.placeholder);
    el.emptyOpenBtn.textContent = mac(el.emptyOpenBtn.textContent);
  }

  // The app's version at the right end of the status bar (task 012).
  async function showVersion() {
    try {
      const version = await invoke("app_version");
      el.statusVersion.textContent = `v${version}`;
      el.statusVersion.title = `mboxer ${version}`;
    } catch (_) {
      el.statusVersion.textContent = "";
    }
  }

  // The remembered files come back at start-up: those whose index is fresh are
  // reopened (the one that was active first, so it is active again), a file
  // whose index needs a build waits in the pane until clicked, a file that is
  // gone leaves the list. The start-up paths (the command line, a Finder open)
  // come last and take over.
  async function restoreFiles(initial) {
    const remembered = state.remembered;
    const statuses = await Promise.all(
      remembered.paths.map((path) => invoke("index_status", { path }).catch(() => ({ state: "stale" }))),
    );
    const fresh = [];
    let gone = 0;
    remembered.paths.forEach((path, at) => {
      if (statuses[at].state === "noFile") {
        state.remembered = Files.forget(state.remembered, path);
        gone++;
      } else if (statuses[at].state === "fresh") {
        if (remembered.active && Files.samePath(path, remembered.active)) fresh.unshift(path);
        else fresh.push(path);
      }
    });
    if (gone > 0) persistFiles();
    if (fresh.length > 0) await openMbox(fresh);
    if (initial.length > 0) await openMbox(initial);
    renderReopenList();
    if (state.file) {
      renderFolders();
      if (gone > 0) {
        // After the first listing has settled its own status text.
        const text = gone === 1 ? "1 remembered file is gone" : `${gone} remembered files are gone`;
        setTimeout(() => flashStatus(text, "warn", SAVED_FLASH_MS), OVERLAY_DELAY_MS);
      }
    } else if (document.body.dataset.view !== "error") {
      setView("empty");
    }
  }

  async function start() {
    loadWidths();
    setTheme(Theme.read());
    showVersion();
    setFiltersOpen(storageGet(STORAGE_FILTERS_OPEN) === "1");
    if (storageGet(STORAGE_BODY_MODE) === "text") state.bodyMode = "text";
    state.remembered = Files.read();
    renderReopenList();
    nameShortcutsForMac();
    // Files the system hands the running app from here on (macOS); the ones it
    // handed before this point come back with the start-up paths.
    try {
      await window.__TAURI__.event.listen("open-files", (event) => openFromSystem(event.payload));
    } catch (_) {
      /* no event API: the start-up paths still arrive */
    }
    let initial = [];
    try {
      initial = await invoke("initial_paths");
    } catch (_) {
      initial = [];
    }
    await restoreFiles(initial);
  }

  start();
})();
