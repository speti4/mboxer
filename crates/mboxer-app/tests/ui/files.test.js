// Tests of the remembered-files module (`ui/files.js`): reading and migrating
// the saved value, remembering, forgetting and activating paths.
// Run: `node --test crates/mboxer-app/tests/ui/files.test.js` (no package.json,
// no dependency); `cargo test -p mboxer-app` runs it too when `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Files = require(path.join(__dirname, "..", "..", "ui", "files.js"));

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

const A = "D:\\mail\\All mail-001.mbox";
const B = "D:\\mail\\All mail-002.mbox";
const C = "E:\\old\\archive.mbox";

test("read: nothing, garbage and a broken storage all give an empty value", () => {
  assert.deepEqual(Files.read(storage()), { paths: [], active: null });
  assert.deepEqual(Files.read(storage({ [Files.KEY]: "{not json" })), { paths: [], active: null });
  assert.deepEqual(Files.read(storage({ [Files.KEY]: "[1,2]" })), { paths: [], active: null });
  assert.deepEqual(Files.read(storage({}, { broken: true })), { paths: [], active: null });
});

test("read: a saved value comes back cleaned — strings only, no duplicates, the active one known", () => {
  const saved = JSON.stringify({ paths: [A, 7, "", B, "d:/mail/all mail-001.mbox"], active: "D:/MAIL/All mail-002.mbox" });
  assert.deepEqual(Files.read(storage({ [Files.KEY]: saved })), { paths: [A, B], active: B });
  const unknownActive = JSON.stringify({ paths: [A], active: C });
  assert.deepEqual(Files.read(storage({ [Files.KEY]: unknownActive })), { paths: [A], active: null });
});

test("read: the pre-016 last path is folded in once and removed", () => {
  const store = storage({ [Files.LEGACY_KEY]: A });
  assert.deepEqual(Files.read(store), { paths: [A], active: A });
  assert.equal(store.map.has(Files.LEGACY_KEY), false);
  assert.deepEqual(JSON.parse(store.map.get(Files.KEY)), { paths: [A], active: A });
  // A second read sees the new key, not the (gone) legacy one.
  assert.deepEqual(Files.read(store), { paths: [A], active: A });
  // The new key wins when both exist.
  const both = storage({ [Files.KEY]: JSON.stringify({ paths: [B], active: null }), [Files.LEGACY_KEY]: A });
  assert.deepEqual(Files.read(both), { paths: [B], active: null });
});

test("write: the value is saved cleaned, a broken storage is ignored", () => {
  const store = storage();
  Files.write({ paths: [A, A, B], active: "d:\\MAIL\\all mail-001.mbox" }, store);
  assert.deepEqual(JSON.parse(store.map.get(Files.KEY)), { paths: [A, B], active: A });
  assert.doesNotThrow(() => Files.write({ paths: [A], active: A }, storage({}, { broken: true })));
});

test("remember: appends once in any spelling, never reorders, never changes the active file", () => {
  let files = Files.empty();
  files = Files.remember(files, A);
  files = Files.remember(files, B);
  files = Files.remember(files, "d:/mail/ALL MAIL-001.mbox");
  assert.deepEqual(files, { paths: [A, B], active: null });
  const withActive = Files.remember({ paths: [A], active: A }, B);
  assert.deepEqual(withActive, { paths: [A, B], active: A });
  assert.deepEqual(Files.remember(withActive, ""), withActive);
});

test("forget: removes the path, and the active role when it was that file", () => {
  const files = { paths: [A, B, C], active: B };
  assert.deepEqual(Files.forget(files, "d:/mail/all mail-002.mbox"), { paths: [A, C], active: null });
  assert.deepEqual(Files.forget(files, A), { paths: [B, C], active: B });
  assert.deepEqual(Files.forget(files, "nowhere.mbox"), files);
});

test("setActive: marks a known path, adds an unknown one, null clears", () => {
  const files = { paths: [A, B], active: A };
  assert.deepEqual(Files.setActive(files, "D:/mail/All mail-002.mbox"), { paths: [A, B], active: B });
  assert.deepEqual(Files.setActive(files, C), { paths: [A, B, C], active: C });
  assert.deepEqual(Files.setActive(files, null), { paths: [A, B], active: null });
});

test("samePath and fileName: Windows spellings compare equal, the name is the last segment", () => {
  assert.equal(Files.samePath(A, "d:/MAIL/all mail-001.mbox"), true);
  assert.equal(Files.samePath(A, `${A}\\`), true);
  assert.equal(Files.samePath(A, B), false);
  assert.equal(Files.fileName(A), "All mail-001.mbox");
  assert.equal(Files.fileName("archive.mbox"), "archive.mbox");
  assert.equal(Files.fileName("/srv/mail/x.mbox"), "x.mbox");
  assert.equal(Files.fileName(""), "");
});
