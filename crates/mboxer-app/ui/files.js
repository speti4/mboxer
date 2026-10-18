// mboxer files: the remembered mbox files — the paths in the order of the
// folder pane and which of them was active — saved in localStorage under one
// key (task 016, D-06). Pure functions over a `{ paths, active }` value; main.js
// holds the value, calls `write` after a change, and draws the pane from it.
// `window.MboxerFiles` in the viewer, CommonJS exports under Node for the tests.
(function (root, factory) {
  const api = factory(root);
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerFiles = api;
})(typeof window !== "undefined" ? window : globalThis, function (root) {
  "use strict";

  const KEY = "mboxer.files";
  // The single last path of the versions before 016; read once and folded in.
  const LEGACY_KEY = "mboxer.lastPath";

  function empty() {
    return { paths: [], active: null };
  }

  // Two spellings of one Windows path: case and the slash direction do not matter.
  function pathKey(path) {
    return String(path || "")
      .replace(/\//g, "\\")
      .replace(/\\+$/, "")
      .toLowerCase();
  }

  function samePath(a, b) {
    return pathKey(a) === pathKey(b);
  }

  function fileName(path) {
    const parts = String(path || "").split(/[\\/]/);
    return parts[parts.length - 1] || String(path || "");
  }

  // A saved value made safe: strings only, no duplicates, the active one present.
  function normalize(value) {
    const files = empty();
    if (!value || typeof value !== "object" || !Array.isArray(value.paths)) return files;
    for (const path of value.paths) {
      if (typeof path === "string" && path && !files.paths.some((known) => samePath(known, path))) {
        files.paths.push(path);
      }
    }
    if (typeof value.active === "string") {
      files.active = files.paths.find((path) => samePath(path, value.active)) || null;
    }
    return files;
  }

  // The remembered files; nothing, garbage or a blocked storage gives an empty
  // value. A pre-016 `lastPath` becomes the one remembered (and active) file
  // and is removed, so it is folded in exactly once.
  function read(storage = root.localStorage) {
    try {
      const raw = storage.getItem(KEY);
      if (raw != null) return normalize(JSON.parse(raw));
      const legacy = storage.getItem(LEGACY_KEY);
      if (legacy) {
        const files = { paths: [legacy], active: legacy };
        write(files, storage);
        storage.removeItem(LEGACY_KEY);
        return files;
      }
    } catch (_) {
      /* unreadable: start empty */
    }
    return empty();
  }

  function write(files, storage = root.localStorage) {
    try {
      storage.setItem(KEY, JSON.stringify(normalize(files)));
    } catch (_) {
      /* storage unavailable: the list holds for this session only */
    }
  }

  // `files` with `path` at the end (unless it is already there, in any spelling).
  function remember(files, path) {
    const next = normalize(files);
    if (path && !next.paths.some((known) => samePath(known, path))) next.paths.push(path);
    return next;
  }

  // `files` without `path`; an active file that goes leaves no active one.
  function forget(files, path) {
    const next = normalize(files);
    next.paths = next.paths.filter((known) => !samePath(known, path));
    if (next.active != null && samePath(next.active, path)) next.active = null;
    return next;
  }

  // `files` with `path` active (it is added when missing); null clears it.
  function setActive(files, path) {
    const next = path ? remember(files, path) : normalize(files);
    next.active = path ? next.paths.find((known) => samePath(known, path)) : null;
    return next;
  }

  return { KEY, LEGACY_KEY, empty, read, write, remember, forget, setActive, samePath, fileName };
});
