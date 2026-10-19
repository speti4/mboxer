// mboxer theme: the saved preference (`auto` follows the OS, `light` and `dark`
// override it) and how it lands on <html data-theme>, which style.css keys its
// dark token set on. Loaded in <head> so the override applies before the first
// paint; `window.MboxerTheme` in the viewer, CommonJS exports under Node for the
// tests. main.js wires the toolbar button and the native title bar.
(function (root, factory) {
  const api = factory(root);
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerTheme = api;
})(typeof window !== "undefined" ? window : globalThis, function (root) {
  "use strict";

  const KEY = "mboxer.theme";
  const MODES = ["auto", "light", "dark"];
  const LABELS = { auto: "Auto", light: "Light", dark: "Dark" };
  const DESCRIPTIONS = {
    auto: "Theme: follows the system.",
    light: "Theme: light.",
    dark: "Theme: dark.",
  };

  function isMode(value) {
    return MODES.includes(value);
  }

  // The saved mode; anything missing, unknown or unreadable is `auto`.
  function read(storage = root.localStorage) {
    try {
      const value = storage.getItem(KEY);
      return isMode(value) ? value : "auto";
    } catch (_) {
      return "auto";
    }
  }

  // Saves `mode`; `auto` removes the key, so a fresh profile and a reset look the same.
  function write(mode, storage = root.localStorage) {
    try {
      if (mode === "auto") storage.removeItem(KEY);
      else storage.setItem(KEY, mode);
    } catch (_) {
      /* storage unavailable: the theme holds for this session only */
    }
  }

  // Puts `mode` on <html>: `data-theme="light|dark"`, no attribute for `auto`
  // (the media query in style.css decides).
  function apply(mode, doc = root.document) {
    const html = doc && doc.documentElement;
    if (!html) return;
    if (mode === "auto") html.removeAttribute("data-theme");
    else html.setAttribute("data-theme", mode);
  }

  // The toolbar button cycles auto → light → dark → auto.
  function next(mode) {
    const at = MODES.indexOf(mode);
    return MODES[(at + 1) % MODES.length];
  }

  function label(mode) {
    return LABELS[mode] || LABELS.auto;
  }

  function title(mode) {
    const following = next(mode);
    return `${DESCRIPTIONS[mode] || DESCRIPTIONS.auto} Click for ${label(following).toLowerCase()}.`;
  }

  // At load in the viewer: the saved override, before anything is painted.
  if (root.document && root.localStorage) apply(read());

  return { KEY, MODES, read, write, apply, next, label, title };
});
