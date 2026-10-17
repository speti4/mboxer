// mboxer shell: open flow, folders, the virtualised message list, the reader.
// Plain JS on the global Tauri API (`withGlobalTauri`), no bundler.
(function () {
  "use strict";

  const { invoke, Channel } = window.__TAURI__.core;

  const ROW_H = 28;
  const BLOCK = 200;
  const OVERSCAN = 10;
  const MAX_BLOCKS = 100;
  const READ_DEBOUNCE_MS = 60;
  const STORAGE_LAST_PATH = "mboxer.lastPath";
  const STORAGE_WIDTHS = "mboxer.paneWidths";

  const el = {};
  for (const id of [
    "toolbar", "open-btn", "file-name", "panes", "folders", "list", "spacer", "rows", "reader",
    "reader-empty", "reader-content", "msg-subject", "msg-headers", "msg-body", "msg-parts-list",
    "empty", "empty-open-btn", "reopen-btn", "error", "error-path", "error-text", "retry-btn",
    "error-open-btn", "overlay", "overlay-title", "overlay-path", "progress", "progress-text",
    "cancel-btn", "status-left", "status-center", "status-right",
  ]) {
    el[id.replace(/-([a-z])/g, (_, c) => c.toUpperCase())] = document.getElementById(id);
  }

  const state = {
    file: null, // OpenSummary of the open file
    generation: 0,
    total: 0,
    blocks: new Map(), // block index -> Row[]
    pending: new Map(), // block index -> Promise
    selected: -1, // row index
    selectedOffset: null,
    activeLabel: null,
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
      el.statusRight.classList.add("error");
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
    state.activeLabel = null;
    el.fileName.textContent = summary.fileName;
    el.fileName.title = summary.path;
    el.fileName.classList.remove("muted");
    document.title = `${summary.fileName} – mboxer`;
    el.statusLeft.textContent = summary.path;
    el.statusLeft.title = summary.path;
    el.statusRight.classList.remove("error");
    el.statusRight.title = "";
    el.statusRight.textContent = indexStateText(summary.index);
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
      item.setAttribute("aria-selected", label === state.activeLabel ? "true" : "false");
    }
  }

  async function selectLabel(label) {
    if (label === state.activeLabel) return;
    state.activeLabel = label;
    updateFolderSelection();
    try {
      const info = await invoke("set_query", { query: { labels: label ? [label] : [] } });
      resetListing(info.generation, info.total);
    } catch (err) {
      el.statusRight.textContent = `Query failed: ${err}`;
      el.statusRight.classList.add("error");
    }
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
    let index = folderIndexOf(state.activeLabel);
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

  function resetListing(generation, total) {
    state.generation = generation;
    state.total = total;
    state.blocks.clear();
    state.pending.clear();
    state.selected = -1;
    state.selectedOffset = null;
    clearReader();
    el.list.scrollTop = 0;
    el.spacer.style.height = `${total * ROW_H}px`;
    updateStatusCenter();
    render();
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
        from.textContent = senderName(data.from);
        from.title = data.from.join(", ");
        subject.textContent = subjectText(data.subject);
        subject.title = subject.textContent;
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

  function renderMessage(view) {
    el.msgSubject.textContent = subjectText(view.subject);
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
      el.msgBody.textContent = view.body.text;
      if (view.body.kind === "htmlAsText") el.msgBody.classList.add("html-as-text");
      if (view.body.kind === "raw") el.msgBody.classList.add("raw");
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
    el.reader.scrollTop = 0;
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
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "o") {
      event.preventDefault();
      openMbox(null);
    }
  });

  async function start() {
    loadWidths();
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
