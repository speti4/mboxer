// Tests of the theme preference module (`ui/theme.js`): the saved mode, the
// cycle, how it lands on <html>. Run: `node --test crates/mboxer-app/tests/ui/theme.test.js`
// (no package.json, no dependency); `cargo test -p mboxer-app` runs it too
// when `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Theme = require(path.join(__dirname, "..", "..", "ui", "theme.js"));

// A localStorage stand-in; `broken` makes every access throw, as a blocked profile does.
function storage(initial = {}, { broken = false } = {}) {
  const map = new Map(Object.entries(initial));
  const fail = () => {
    throw new Error("storage unavailable");
  };
  return {
    map,
    getItem: broken ? fail : (key) => (map.has(key) ? map.get(key) : null),
    setItem: broken ? fail : (key, value) => map.set(key, String(value)),
    removeItem: broken ? fail : (key) => map.delete(key),
  };
}

// A document stand-in with only what `apply` touches.
function document() {
  const attrs = new Map();
  return {
    documentElement: {
      attrs,
      setAttribute: (name, value) => attrs.set(name, value),
      removeAttribute: (name) => attrs.delete(name),
      getAttribute: (name) => (attrs.has(name) ? attrs.get(name) : null),
    },
  };
}

test("read: no value, an unknown value and a broken storage are all auto", () => {
  assert.equal(Theme.read(storage()), "auto");
  assert.equal(Theme.read(storage({ [Theme.KEY]: "sepia" })), "auto");
  assert.equal(Theme.read(storage({}, { broken: true })), "auto");
});

test("read: a saved light or dark comes back as saved", () => {
  assert.equal(Theme.read(storage({ [Theme.KEY]: "light" })), "light");
  assert.equal(Theme.read(storage({ [Theme.KEY]: "dark" })), "dark");
});

test("write: light and dark are stored, auto removes the key, a broken storage is ignored", () => {
  const store = storage();
  Theme.write("dark", store);
  assert.equal(store.map.get(Theme.KEY), "dark");
  assert.equal(Theme.read(store), "dark");
  Theme.write("light", store);
  assert.equal(Theme.read(store), "light");
  Theme.write("auto", store);
  assert.equal(store.map.has(Theme.KEY), false);
  assert.doesNotThrow(() => Theme.write("dark", storage({}, { broken: true })));
});

test("next: the cycle is auto → light → dark → auto, and an unknown mode restarts it", () => {
  assert.equal(Theme.next("auto"), "light");
  assert.equal(Theme.next("light"), "dark");
  assert.equal(Theme.next("dark"), "auto");
  assert.equal(Theme.next("sepia"), "auto");
});

test("apply: light and dark set data-theme on <html>, auto removes it", () => {
  const doc = document();
  Theme.apply("dark", doc);
  assert.equal(doc.documentElement.getAttribute("data-theme"), "dark");
  Theme.apply("light", doc);
  assert.equal(doc.documentElement.getAttribute("data-theme"), "light");
  Theme.apply("auto", doc);
  assert.equal(doc.documentElement.getAttribute("data-theme"), null);
  assert.doesNotThrow(() => Theme.apply("dark", null));
});

test("label and title: the button text names the mode, the tooltip names the next one", () => {
  assert.equal(Theme.label("auto"), "Auto");
  assert.equal(Theme.label("dark"), "Dark");
  assert.equal(Theme.label("sepia"), "Auto");
  assert.equal(Theme.title("auto"), "Theme: follows the system. Click for light.");
  assert.equal(Theme.title("light"), "Theme: light. Click for dark.");
  assert.equal(Theme.title("dark"), "Theme: dark. Click for auto.");
});
