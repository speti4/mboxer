// Tests of the macOS print helpers (`ui/print.js`, task 020): which OS prints
// through a print window, the `data:` URLs, the cid images inlined into the
// print document. Run: `node --test crates/mboxer-app/tests/ui/print.test.js`
// (no package.json, no dependency); `cargo test -p mboxer-app` runs it too
// when `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Print = require(path.join(__dirname, "..", "..", "ui", "print.js"));

// A document stand-in: the images with their attributes, and the one query
// print.js makes (`img[src^='blob:']`).
function documentWith(sources) {
  const images = sources.map((src) => {
    const attributes = new Map(src === null ? [] : [["src", src]]);
    return {
      attributes,
      getAttribute: (name) => (attributes.has(name) ? attributes.get(name) : null),
      setAttribute: (name, value) => attributes.set(name, String(value)),
      removeAttribute: (name) => attributes.delete(name),
    };
  });
  return {
    images,
    querySelectorAll(selector) {
      assert.equal(selector, "img[src^='blob:']");
      return images.filter((img) => String(img.getAttribute("src") || "").startsWith("blob:"));
    },
  };
}

test("only a Mac prints through the print window", () => {
  assert.equal(Print.usesPrintWindow("mac"), true);
  assert.equal(Print.usesPrintWindow("windows"), false);
  assert.equal(Print.usesPrintWindow("other"), false);
});

test("data URLs carry the bytes in base64 and the bare media type", () => {
  const png = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  assert.equal(Print.dataUrl("image/png", png), "data:image/png;base64,iVBORw0KGgo=");
  assert.equal(Print.dataUrl("Image/GIF; name=\"a.gif\"", new Uint8Array([71, 73, 70])), "data:image/gif;base64,R0lG");
  assert.equal(Print.dataUrl("", new Uint8Array([0])), "data:application/octet-stream;base64,AA==");
  assert.equal(Print.dataUrl("not a type", new Uint8Array([])), "data:application/octet-stream;base64,");
});

test("data URLs of large parts survive the chunking", () => {
  const bytes = new Uint8Array(100_000).map((_, i) => (i * 7) % 256);
  const url = Print.dataUrl("image/jpeg", bytes);
  const decoded = Buffer.from(url.slice("data:image/jpeg;base64,".length), "base64");
  assert.deepEqual(new Uint8Array(decoded), bytes);
});

test("blob images are inlined, unknown ones lose their source, others stay", async () => {
  const known = new Blob([new Uint8Array([71, 73, 70])], { type: "image/gif" });
  const doc = documentWith([
    "blob:tauri://localhost/aaa",
    "data:image/png;base64,AA==",
    "blob:tauri://localhost/gone",
    "blob:tauri://localhost/aaa",
    "https://example.com/remote.png",
    null,
  ]);
  const looked = [];
  const inlined = await Print.inlineBlobImages(doc, (url) => {
    looked.push(url);
    return url.endsWith("/aaa") ? known : undefined;
  });
  assert.equal(inlined, 2);
  const sources = doc.images.map((img) => img.getAttribute("src"));
  assert.deepEqual(sources, [
    "data:image/gif;base64,R0lG",
    "data:image/png;base64,AA==",
    null,
    "data:image/gif;base64,R0lG",
    "https://example.com/remote.png",
    null,
  ]);
  // Each blob is read once however many images show it.
  assert.deepEqual(looked, ["blob:tauri://localhost/aaa", "blob:tauri://localhost/gone"]);
});
