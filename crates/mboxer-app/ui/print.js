// mboxer print: the macOS side of printing (task 020). WKWebView ignores
// print() on an iframe, so on a Mac the print document goes to a print window
// of its own (the `print_message` command) instead of the hidden print frame
// of task 011. A blob URL does not resolve in another window, so the cid
// images, which the reader serves as blob URLs, are inlined as `data:` URLs
// first. `window.MboxerPrint` in the viewer, CommonJS exports under Node for
// the tests.
(function (root, factory) {
  const api = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerPrint = api;
})(typeof window !== "undefined" ? window : globalThis, function () {
  "use strict";

  // Bytes per String.fromCharCode call: well under the engines' argument limits.
  const CHUNK = 0x8000;

  // Whether the OS prints through a print window ("mac", from MboxerFiles.os)
  // rather than the print frame.
  function usesPrintWindow(os) {
    return os === "mac";
  }

  // A `data:` URL of `bytes` (a Uint8Array) with the media type `type`, its
  // parameters dropped; an empty or odd type becomes application/octet-stream.
  function dataUrl(type, bytes) {
    const essence = String(type || "").split(";")[0].trim().toLowerCase();
    const mediaType = /^[\w.+-]+\/[\w.+-]+$/.test(essence) ? essence : "application/octet-stream";
    let binary = "";
    for (let at = 0; at < bytes.length; at += CHUNK) {
      binary += String.fromCharCode.apply(null, bytes.subarray(at, at + CHUNK));
    }
    return `data:${mediaType};base64,${btoa(binary)}`;
  }

  // Replaces every `blob:` image source in `doc` with a `data:` URL of the
  // same bytes. `lookup(url)` gives the Blob behind a blob URL; an image
  // whose blob is unknown loses its source (it could not load in the print
  // window anyway). Resolves to the number of images inlined.
  async function inlineBlobImages(doc, lookup) {
    const images = Array.from(doc.querySelectorAll("img[src^='blob:']"));
    const cache = new Map();
    let inlined = 0;
    for (const img of images) {
      const url = img.getAttribute("src");
      let data = cache.get(url);
      if (data === undefined) {
        const blob = lookup(url);
        data = blob ? dataUrl(blob.type, new Uint8Array(await blob.arrayBuffer())) : null;
        cache.set(url, data);
      }
      if (data) {
        img.setAttribute("src", data);
        inlined += 1;
      } else {
        img.removeAttribute("src");
      }
    }
    return inlined;
  }

  return { usesPrintWindow, dataUrl, inlineBlobImages };
});
