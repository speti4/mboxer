// mboxer shell: open flow, folders, the search field and filter panel, the
// virtualised message list, the reader. Plain JS on the global Tauri API
// (`withGlobalTauri`), no bundler.
(function () {
  "use strict";

  const { invoke, Channel } = window.__TAURI__.core;

  const ROW_H = 28;
  const BLOCK = 200;
  const OVERSCAN = 10;
  const MAX_BLOCKS = 100;
  const READ_DEBOUNCE_MS = 60;
  const SEARCH_DEBOUNCE_MS = 150;
  // Bodies longer than this are shown without highlighting.
  const MAX_HIGHLIGHT_CHARS = 2_000_000;
  const STORAGE_LAST_PATH = "mboxer.lastPath";
  const STORAGE_WIDTHS = "mboxer.paneWidths";
  const STORAGE_FILTERS_OPEN = "mboxer.filtersOpen";

  const el = {};
  for (const id of [
    "toolbar", "open-btn", "file-name", "search", "filters-btn", "filters-count", "filters",
    "f-from", "f-after", "f-before", "f-att", "f-sort", "filters-clear", "panes", "folders",
    "list", "spacer", "rows", "list-empty", "reader", "reader-empty", "reader-content",
    "msg-subject", "msg-headers", "msg-body", "msg-parts-list", "empty", "empty-open-btn",
    "reopen-btn", "error", "error-path", "error-text", "retry-btn", "error-open-btn", "overlay",
    "overlay-title", "overlay-path", "progress", "progress-text", "cancel-btn", "status-left",
    "status-center", "status-right",
  ]) {
    el[id.replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = document.getElementById(id);
  }

  function defaultQuery() {
    return {
      text: "",
      from: "",
      after: "", // YYYY-MM-DD from the date input, or ""
      before: "",
      label: null,
      hasAttachment: null, // true | false | null
      sort: "dateDesc",
    };
  }

  const state = {
    file: null, // OpenSummary of the open file
    generation: 0,
    total: 0,
    blocks: new Map(), // block index -> Row[]
    pending: new Map(), // block index -> Promise
    selected: -1, // row index
    selectedOffset: null,
    view: null, // MessageView shown in the reader
    query: defaultQuery(), // the one source of truth behind the listing
    queryToken: 0, // only the newest set_query response lands
    queryTimer: 0,
    queryDirty: false, // an edit the backend has not seen yet
    highlightTerms: [], // folded words of the query behind the current listing
    lastPath: null,
    opening: null, // { path, startedAt }
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

  function formatBytes(bytes) {
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(2)} GB`;
    if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
    return `${Math.round(bytes / 1e3)} kB`;
  }

  function fmt(n) {
    return numberFormat.format(n);
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
  // Mirrors the index's tokenizer: lower-cased, accents folded (`Számla` and
  // `szamla` are the same word), so a highlighted word is one the index matched.

  const foldCache = new Map();

  function foldChar(ch) {
    let folded = foldCache.get(ch);
    if (folded === undefined) {
      folded = ch.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
      foldCache.set(ch, folded);
    }
    return folded;
  }

  // Folds `text`; `map[k]` is the index in `text` of the character behind folded position `k`.
  function foldWithMap(text) {
    let folded = "";
    const map = [];
    let i = 0;
    for (const ch of text) {
      const f = foldChar(ch);
      for (let k = 0; k < f.length; k++) map.push(i);
      folded += f;
      i += ch.length;
    }
    map.push(text.length);
    return { folded, map };
  }

  function foldText(text) {
    let out = "";
    for (const ch of text) out += foldChar(ch);
    return out;
  }

  // The words of a query worth highlighting: quotes stripped, negated words
  // and `field:value` operators left alone (task 010 owns the syntax).
  function highlightTerms(text) {
    const terms = new Set();
    for (const raw of (text || "").split(/\s+/)) {
      const word = raw.replace(/^["'(]+|["')]+$/g, "");
      if (!word || word.startsWith("-") || word.includes(":")) continue;
      const folded = foldText(word);
      if (folded) terms.add(folded);
    }
    return Array.from(terms).sort((a, b) => b.length - a.length);
  }

  // Fills `node` with `text`, the occurrences of `terms` wrapped in <mark>.
  function renderHighlighted(node, text, terms) {
    if (terms.length === 0 || !text || text.length > MAX_HIGHLIGHT_CHARS) {
      node.textContent = text;
      return;
    }
    const { folded, map } = foldWithMap(text);
    const ranges = [];
    for (const term of terms) {
      let at = folded.indexOf(term);
      while (at >= 0) {
        ranges.push([map[at], map[at + term.length]]);
        at = folded.indexOf(term, at + term.length);
      }
    }
    if (ranges.length === 0) {
      node.textContent = text;
      return;
    }
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
    node.replaceChildren(...parts);
  }

  // ---------- open flow ----------

  async function openMbox(path) {
    if (state.opening) return;
    const channel = new Channel();
    channel.onmessage = handleOpenEvent;
    try {
      await invoke("open_mbox", { path: path || null, onEvent: channel });
    } catch (err) {
      showFailure(path || "", String(err));
    }
  }

  function handleOpenEvent(message) {
    switch (message.event) {
      case "opening":
        state.opening = { path: message.data.path, startedAt: performance.now() };
        el.overlayTitle.textContent = "Opening…";
        el.overlayPath.textContent = message.data.path;
        el.progress.removeAttribute("value");
        el.progressText.textContent = "Checking the index…";
        el.overlay.hidden = false;
        break;
      case "building": {
        const { bytes, totalBytes, messages } = message.data;
        const elapsed = (performance.now() - state.opening.startedAt) / 1000;
        const rate = elapsed > 0 ? bytes / 1e6 / elapsed : 0;
        el.overlayTitle.textContent = "Building the search index…";
        if (totalBytes > 0) el.progress.value = bytes / totalBytes;
        el.progressText.textContent =
          `${formatBytes(bytes)} of ${formatBytes(totalBytes)} · ${fmt(messages)} messages · ${rate.toFixed(0)} MB/s`;
        break;
      }
      case "ready":
        state.opening = null;
        el.overlay.hidden = true;
        showFile(message.data.summary);
        break;
      case "cancelled":
        state.opening = null;
        el.overlay.hidden = true;
        if (!state.file) setView("empty");
        break;
      case "failed": {
        const path = state.opening ? state.opening.path : "";
        state.opening = null;
        el.overlay.hidden = true;
        showFailure(path, message.data.message);
        break;
      }
      default:
        break;
    }
  }

  function showFailure(path, text) {
    if (state.file) {
      el.statusRight.textContent = `Open failed: ${text}`;
      el.statusRight.className = "error";
      el.statusRight.title = path;
      return;
    }
    el.errorPath.textContent = path;
    el.errorText.textContent = text;
    el.retryBtn.hidden = !path;
    state.lastPath = path || state.lastPath;
    setView("error");
  }

  function showFile(summary) {
    state.file = summary;
    state.lastPath = summary.path;
    storageSet(STORAGE_LAST_PATH, summary.path);
    el.fileName.textContent = summary.fileName;
    el.fileName.title = summary.path;
    el.fileName.classList.remove("muted");
    document.title = `${summary.fileName} – mboxer`;
    el.statusLeft.textContent = summary.path;
    el.statusLeft.title = summary.path;
    showWarnings([]);
    resetQueryModel();
    renderFolders();
    resetListing(summary.generation, summary.total);
    setView("ready");
    // The panes had no height while hidden; lay the list out again now that they do.
    scheduleRender();
    el.list.focus();
  }

  function indexStateText(index) {
    if (index.state === "reused") {
      return `index reused (built ${fullDate(index.builtAtUnix)})`;
    }
    const unparsed = index.parseFailures === 1 ? "1 unparsed message" : `${fmt(index.parseFailures)} unparsed messages`;
    return `index built in ${fmt(index.buildSecs)} s, ${unparsed}`;
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
    el.fFrom.value = "";
    el.fAfter.value = "";
    el.fBefore.value = "";
    el.fAtt.value = "";
    el.fSort.value = "dateDesc";
    el.search.disabled = false;
    el.filtersBtn.disabled = false;
    updateFilterBadge();
    updateFolderSelection();
  }

  // Unix seconds of local midnight starting `ymd` (YYYY-MM-DD), or null.
  function localDayStart(ymd, dayOffset) {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(ymd || "");
    if (!m) return null;
    const time = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]) + dayOffset).getTime();
    return Number.isNaN(time) ? null : Math.floor(time / 1000);
  }

  // The QueryInput for the backend: empty fields left out, dates as local
  // days (`to` inclusive: the core's `before` is the next midnight). The
  // text keeps its trailing space: it tells the backend the last word is
  // finished, otherwise that word matches as a prefix while it is typed.
  function queryInput() {
    const q = state.query;
    const input = { labels: q.label ? [q.label] : [], prefixLastWord: true };
    const text = q.text.trim();
    if (text) input.text = q.text.trimStart();
    const from = q.from.trim();
    if (from) input.from = q.from.trimStart();
    const after = localDayStart(q.after, 0);
    if (after != null) input.after = after;
    const before = localDayStart(q.before, 1);
    if (before != null) input.before = before;
    if (q.hasAttachment != null) input.hasAttachment = q.hasAttachment;
    // Relevance means nothing without words to score.
    input.sort = text && q.sort === "relevance" ? "relevance" : "dateDesc";
    return input;
  }

  function activeFilterCount() {
    const q = state.query;
    let n = 0;
    if (q.from.trim()) n++;
    if (q.after) n++;
    if (q.before) n++;
    if (q.hasAttachment != null) n++;
    if (q.sort !== "dateDesc") n++;
    return n;
  }

  function updateFilterBadge() {
    const n = activeFilterCount();
    el.filtersCount.textContent = String(n);
    el.filtersCount.hidden = n === 0;
  }

  function anyNarrowing() {
    const q = state.query;
    return Boolean(q.text.trim() || q.label || activeFilterCount() > 0);
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
    const terms = highlightTerms(input.text);
    try {
      const info = await invoke("set_query", { query: input });
      if (token !== state.queryToken) return;
      state.highlightTerms = terms;
      showWarnings(info.warnings);
      await resetListing(info.generation, info.total, { keepSelection: true });
    } catch (err) {
      if (token !== state.queryToken) return;
      el.statusRight.textContent = `Query failed: ${err}`;
      el.statusRight.className = "error";
    }
  }

  function showWarnings(warnings) {
    el.statusRight.title = "";
    if (warnings && warnings.length > 0) {
      el.statusRight.textContent = `Query: ${warnings.join("; ")}`;
      el.statusRight.className = "warn";
    } else {
      el.statusRight.textContent = state.file ? indexStateText(state.file.index) : "";
      el.statusRight.className = "";
    }
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

  function focusList() {
    el.list.focus();
    if (state.selected < 0 && state.total > 0) select(0);
  }

  el.search.addEventListener("input", () => {
    state.query.text = el.search.value;
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
        applyQuery();
      } else {
        el.list.focus();
      }
    }
  });

  el.fFrom.addEventListener("input", () => {
    state.query.from = el.fFrom.value;
    applyQuery({ debounce: SEARCH_DEBOUNCE_MS });
  });
  el.fAfter.addEventListener("change", () => {
    state.query.after = el.fAfter.value;
    applyQuery();
  });
  el.fBefore.addEventListener("change", () => {
    state.query.before = el.fBefore.value;
    applyQuery();
  });
  el.fAtt.addEventListener("change", () => {
    state.query.hasAttachment = el.fAtt.value === "" ? null : el.fAtt.value === "true";
    applyQuery();
  });
  el.fSort.addEventListener("change", () => {
    state.query.sort = el.fSort.value;
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

  function renderFolders() {
    el.folders.replaceChildren();
    el.folders.appendChild(folderItem("All mail", state.file.messageCount, null));
    if (state.file.labels.length > 0) {
      const separator = document.createElement("div");
      separator.className = "folder-separator";
      el.folders.appendChild(separator);
    }
    for (const label of state.file.labels) {
      el.folders.appendChild(folderItem(label.name, label.count, label.name));
    }
    updateFolderSelection();
  }

  function folderItem(name, count, label) {
    const item = document.createElement("div");
    item.className = "folder";
    item.setAttribute("role", "option");
    item.dataset.label = label == null ? "" : label;
    item.title = name;
    const nameSpan = document.createElement("span");
    nameSpan.className = "folder-name";
    nameSpan.textContent = name;
    const countSpan = document.createElement("span");
    countSpan.className = "folder-count";
    countSpan.textContent = fmt(count);
    item.append(nameSpan, countSpan);
    item.addEventListener("click", () => selectLabel(label));
    return item;
  }

  function updateFolderSelection() {
    for (const item of el.folders.children) {
      if (!item.classList.contains("folder")) continue;
      const label = item.dataset.label === "" ? null : item.dataset.label;
      item.setAttribute("aria-selected", label === state.query.label ? "true" : "false");
    }
  }

  function selectLabel(label) {
    if (label === state.query.label) return;
    state.query.label = label;
    updateFolderSelection();
    applyQuery();
  }

  function folderIndexOf(label) {
    const items = folderItems();
    return items.findIndex((item) => (item.dataset.label === "" ? null : item.dataset.label) === label);
  }

  function folderItems() {
    return Array.from(el.folders.children).filter((item) => item.classList.contains("folder"));
  }

  el.folders.addEventListener("keydown", (event) => {
    const items = folderItems();
    if (items.length === 0) return;
    let index = folderIndexOf(state.query.label);
    if (event.key === "ArrowDown") index = Math.min(items.length - 1, index + 1);
    else if (event.key === "ArrowUp") index = Math.max(0, index - 1);
    else if (event.key === "Home") index = 0;
    else if (event.key === "End") index = items.length - 1;
    else if (event.key === "Enter" || event.key === "ArrowRight") {
      el.list.focus();
      event.preventDefault();
      return;
    } else return;
    event.preventDefault();
    const label = items[index].dataset.label === "" ? null : items[index].dataset.label;
    items[index].scrollIntoView({ block: "nearest" });
    selectLabel(label);
  });

  // ---------- virtualised list ----------

  // Installs a new listing. With `keepSelection` the selected message stays
  // selected (and its reader view stays) when it is still in the listing.
  async function resetListing(generation, total, { keepSelection = false } = {}) {
    const keepOffset = keepSelection ? state.selectedOffset : null;
    const token = state.queryToken;
    state.generation = generation;
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
      try {
        position = await invoke("position_of", { offset: keepOffset });
      } catch (_) {
        position = null;
      }
      if (token !== state.queryToken) return;
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

  function updateStatusCenter() {
    if (!state.file) {
      el.statusCenter.textContent = "";
      return;
    }
    const all = state.file.messageCount;
    el.statusCenter.textContent =
      state.total === all ? `${fmt(all)} messages` : `${fmt(state.total)} of ${fmt(all)} messages`;
  }

  function rowData(index) {
    const block = Math.floor(index / BLOCK);
    const rows = state.blocks.get(block);
    return rows ? rows[index - block * BLOCK] || null : null;
  }

  function requestBlock(block) {
    if (state.blocks.has(block) || state.pending.has(block)) return;
    const generation = state.generation;
    const promise = invoke("list_rows", { start: block * BLOCK, count: BLOCK })
      .then((page) => {
        state.pending.delete(block);
        if (page.generation !== state.generation || generation !== state.generation) return;
        state.blocks.set(block, page.rows);
        evictBlocks();
        scheduleRender();
      })
      .catch((err) => {
        state.pending.delete(block);
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
    for (const cls of ["att", "from", "subject", "date"]) {
      const span = document.createElement("span");
      span.className = cls;
      row.appendChild(span);
    }
    row.addEventListener("click", () => {
      const index = Number(row.dataset.index);
      if (!Number.isNaN(index)) select(index, { scroll: false });
      el.list.focus();
    });
    return row;
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
      const [att, from, subject, date] = row.children;
      if (data) {
        row.classList.remove("placeholder");
        att.textContent = data.hasAttachment ? "📎" : "";
        renderHighlighted(from, senderName(data.from), terms);
        from.title = data.from.join(", ");
        const subjectLine = subjectText(data.subject);
        renderHighlighted(subject, subjectLine, terms);
        subject.title = subjectLine;
        date.textContent = listDate(data.date);
        date.title = fullDate(data.date);
      } else {
        row.classList.add("placeholder");
        att.textContent = "";
        from.textContent = "…";
        from.title = "";
        subject.textContent = "";
        subject.title = "";
        date.textContent = "";
        date.title = "";
        requestBlock(Math.floor(index / BLOCK));
      }
    }
  }

  el.list.addEventListener("scroll", scheduleRender);
  window.addEventListener("resize", scheduleRender);

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
      const view = await invoke("read_message", { offset: data.offset, len: data.len });
      if (token !== state.readToken || state.selected !== index) return;
      state.selectedOffset = data.offset;
      state.view = view;
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
      case "Enter":
      case "ArrowRight":
        if (state.selected >= 0) el.reader.focus();
        event.preventDefault();
        return;
      case "ArrowLeft":
        el.folders.focus();
        event.preventDefault();
        return;
      default:
        return;
    }
    event.preventDefault();
    select(next);
  });

  // ---------- reader ----------

  function clearReader() {
    state.view = null;
    el.readerContent.hidden = true;
    el.readerEmpty.hidden = false;
    el.readerEmpty.textContent = "Select a message";
    el.reader.scrollTop = 0;
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
    if (view.messageId) rows.push(...headerRow("Message-Id", view.messageId));
    el.msgHeaders.replaceChildren(...rows);

    el.msgBody.className = "";
    if (view.body) {
      if (view.body.kind === "raw") {
        el.msgBody.textContent = view.body.text;
        el.msgBody.classList.add("raw");
      } else {
        renderHighlighted(el.msgBody, view.body.text, terms);
        if (view.body.kind === "htmlAsText") el.msgBody.classList.add("html-as-text");
      }
    } else {
      el.msgBody.textContent = "(no text body)";
    }

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
  el.retryBtn.addEventListener("click", () => openMbox(state.lastPath));
  el.reopenBtn.addEventListener("click", () => openMbox(state.lastPath));
  el.cancelBtn.addEventListener("click", () => {
    el.cancelBtn.disabled = true;
    invoke("cancel_open").finally(() => {
      el.cancelBtn.disabled = false;
    });
  });

  document.addEventListener("keydown", (event) => {
    if (!(event.ctrlKey || event.metaKey)) return;
    const key = event.key.toLowerCase();
    if (key === "o") {
      event.preventDefault();
      openMbox(null);
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
  });

  async function start() {
    loadWidths();
    setFiltersOpen(storageGet(STORAGE_FILTERS_OPEN) === "1");
    state.lastPath = storageGet(STORAGE_LAST_PATH);
    if (state.lastPath) {
      const name = state.lastPath.split(/[\\/]/).pop();
      el.reopenBtn.textContent = `Reopen ${name}`;
      el.reopenBtn.title = state.lastPath;
      el.reopenBtn.hidden = false;
    }
    let initial = null;
    try {
      initial = await invoke("initial_path");
    } catch (_) {
      initial = null;
    }
    if (initial) openMbox(initial);
  }

  start();
})();
