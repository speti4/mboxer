// mboxer frame: the parts of the sanitized message frame that need no DOM of
// their own (task 053): the kind of a URL, the CSS scrub, and the meta
// elements in front of the frame's document. The sanitizer itself and the
// srcdoc assembly stay in main.js. `window.MboxerFrame` in the viewer,
// CommonJS exports under Node for the tests.
(function (root, factory) {
  const api = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerFrame = api;
})(typeof window !== "undefined" ? window : globalThis, function () {
  "use strict";

  // The kind of a URL by its scheme: "empty", "fragment", "data", "cid",
  // "remote" (http, https, protocol-relative), "mailto", "relative" or "other".
  // Control characters and spaces are ignored, as browsers ignore them.
  function classifyUrl(value) {
    const cleaned = String(value || "").replace(/[\u0000- ]+/g, "");
    if (!cleaned) return "empty";
    if (cleaned.startsWith("#")) return "fragment";
    if (cleaned.startsWith("//")) return "remote";
    const m = /^([a-z][a-z0-9+.-]*):/i.exec(cleaned);
    if (!m) return "relative";
    const scheme = m[1].toLowerCase();
    if (scheme === "data") return "data";
    if (scheme === "cid") return "cid";
    if (scheme === "http" || scheme === "https") return "remote";
    if (scheme === "mailto") return "mailto";
    return "other";
  }

  // Scrubs a CSS text: `@import`, `expression()` and `behavior` are always
  // dropped; `safe` keeps the remote `url()`s, `blocked` turns them into
  // `none` (an empty `url()` would still resolve to the document itself);
  // `remote` counts them. A dropped `@import` leaves an empty comment, so the
  // text around it never joins into new tokens, and every `<` becomes the CSS
  // escape `\3c `: style text is serialized into the srcdoc as it is, and a
  // `</style` in it would end the element on the re-parse. The comment opener
  // `<!--` of old mail stays; it ends nothing.
  function scrubCss(css) {
    const safe = String(css || "")
      .replace(/@import\b[^;]*;?/gi, "/**/")
      .replace(/expression\s*\(/gi, "expression-blocked(")
      .replace(/-moz-binding|behavior\s*:/gi, "blocked:")
      .replace(/<(?!!--)/g, "\\3c ");
    let remote = 0;
    const blocked = safe.replace(/url\(\s*(['"]?)([^'")]*)\1\s*\)/gi, (whole, _quote, url) => {
      const kind = classifyUrl(url);
      if (kind === "data" || kind === "fragment") return whole;
      if (kind === "remote") remote++;
      return "none";
    });
    return { safe, blocked, remote };
  }

  // The meta elements in front of the frame's document, made with `doc`: the
  // charset, no referrer for the images that *Show* loads, and the CSP, which
  // blocks every remote request but those images (`allowRemote`).
  function headMetas(doc, allowRemote) {
    const charset = doc.createElement("meta");
    charset.setAttribute("charset", "utf-8");
    const referrer = doc.createElement("meta");
    referrer.setAttribute("name", "referrer");
    referrer.setAttribute("content", "no-referrer");
    const csp = doc.createElement("meta");
    csp.setAttribute("http-equiv", "Content-Security-Policy");
    csp.setAttribute(
      "content",
      `default-src 'none'; img-src data: blob:${allowRemote ? " https: http:" : ""}; style-src 'unsafe-inline'`,
    );
    return [charset, referrer, csp];
  }

  return { classifyUrl, scrubCss, headMetas };
});
