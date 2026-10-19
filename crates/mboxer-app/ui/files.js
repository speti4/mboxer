// mboxer files: the remembered mbox files — the paths in the order of the
// folder pane and which of them was active — saved in localStorage under one
// key (task 016, D-006). Pure functions over a `{ paths, active }` value; main.js
// holds the value, calls `write` after a change, and draws the pane from it.
// The module knows the OS it runs on (task 019): a path is compared the way
// that file system spells it. `window.MboxerFiles` in the viewer, CommonJS
// exports under Node for the tests, which build one instance per OS with `create`.
(function (root, factory) {
  const api = factory(root);
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerFiles = api;
})(typeof window !== "undefined" ? window : globalThis, function (root) {
  "use strict";

  const KEY = "mboxer.files";
  // The single last path of the versions before 016; read once and folded in.
  const LEGACY_KEY = "mboxer.lastPath";

  // The OS the viewer runs on, from the navigator alone: "windows", "mac" or
  // "other" (Linux). Synchronous and local, so the path keys and the shortcut
  // labels are right before the first IPC call and the module needs no Tauri.
  // `userAgentData.platform` where the engine has it (WebView2 says "Windows"),
  // the older `platform` elsewhere (WebKit says "MacIntel" on every Mac).
  function detectOs(navigator) {
    const data = navigator && navigator.userAgentData;
    const hint = String((data && data.platform) || (navigator && navigator.platform) || "");
    if (/^win/i.test(hint)) return "windows";
    if (/^mac/i.test(hint)) return "mac";
    return "other";
  }

  // The API bound to one OS.
  function create(os) {
    const windows = os === "windows";

    function empty() {
      return { paths: [], active: null };
    }

    // Two spellings of one path share a key. On Windows case and the slash
    // direction do not matter. Elsewhere only a trailing slash is dropped: a
    // backslash is a legal name character, and although a Mac volume is
    // case-insensitive by default, it need not be, and the paths come from the
    // native dialog, Finder and the command line in one spelling anyway — two
    // list entries for one file would be a nuisance, one entry for two files a bug.
    function pathKey(path) {
      const text = String(path || "");
      if (windows) return text.replace(/\//g, "\\").replace(/\\+$/, "").toLowerCase();
      return text.replace(/\/+$/, "");
    }

    function samePath(a, b) {
      return pathKey(a) === pathKey(b);
    }

    function fileName(path) {
      const parts = String(path || "").split(windows ? /[\\/]/ : /\//);
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

    return {
      KEY,
      LEGACY_KEY,
      os,
      detectOs,
      create,
      empty,
      read,
      write,
      remember,
      forget,
      setActive,
      pathKey,
      samePath,
      fileName,
    };
  }

  return create(detectOs(root.navigator));
});
