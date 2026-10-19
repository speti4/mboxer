// Tests of the remembered-files module (`ui/files.js`): reading and migrating
// the saved value, remembering, forgetting and activating paths, once per OS
// the viewer runs on (the path spelling rules differ; task 019).
// Run: `node --test crates/mboxer-app/tests/ui/files.test.js` (no package.json,
// no dependency); `cargo test -p mboxer-app` runs it too when `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Base = require(path.join(__dirname, "..", "..", "ui", "files.js"));

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

// Three files per OS, a second spelling of each of the first two that names
// the same file (`A2`, `B2`), and one that only looks like A (`notA`).
const CASES = [
  {
    os: "windows",
    A: "D:\\mail\\All mail-001.mbox",
    A2: "d:/mail/ALL MAIL-001.mbox",
    B: "D:\\mail\\All mail-002.mbox",
    B2: "D:/MAIL/All mail-002.mbox",
    C: "E:\\old\\archive.mbox",
    notA: "D:\\mail\\All mail-001.mbox.bak",
  },
  {
    os: "mac",
    A: "/Users/anna/Mail/All mail-001.mbox",
    A2: "/Users/anna/Mail/All mail-001.mbox/",
    B: "/Users/anna/Mail/All mail-002.mbox",
    B2: "/Users/anna/Mail/All mail-002.mbox//",
    C: "/Volumes/Old/archive.mbox",
    // Case is kept outside Windows: a different spelling is a different file.
    notA: "/Users/anna/mail/all mail-001.mbox",
  },
];

test("detectOs: userAgentData first, then platform; anything unknown is other", () => {
  assert.equal(Base.detectOs({ userAgentData: { platform: "Windows" }, platform: "Win32" }), "windows");
  assert.equal(Base.detectOs({ platform: "Win32" }), "windows");
  assert.equal(Base.detectOs({ platform: "MacIntel" }), "mac");
  assert.equal(Base.detectOs({ userAgentData: { platform: "macOS" } }), "mac");
  assert.equal(Base.detectOs({ platform: "Linux x86_64" }), "other");
  assert.equal(Base.detectOs({}), "other");
  assert.equal(Base.detectOs(undefined), "other");
  // The default instance is bound to the detected OS (Node's own navigator names the host).
  assert.equal(Base.os, Base.detectOs(globalThis.navigator));
  assert.equal(Base.create("mac").os, "mac");
});

for (const { os, A, A2, B, B2, C, notA } of CASES) {
  const Files = Base.create(os);

  test(`${os} read: nothing, garbage and a broken storage all give an empty value`, () => {
    assert.deepEqual(Files.read(storage()), { paths: [], active: null });
    assert.deepEqual(Files.read(storage({ [Files.KEY]: "{not json" })), { paths: [], active: null });
    assert.deepEqual(Files.read(storage({ [Files.KEY]: "[1,2]" })), { paths: [], active: null });
    assert.deepEqual(Files.read(storage({}, { broken: true })), { paths: [], active: null });
  });

  test(`${os} read: a saved value comes back cleaned — strings only, no duplicates, the active one known`, () => {
    const saved = JSON.stringify({ paths: [A, 7, "", B, A2], active: B2 });
    assert.deepEqual(Files.read(storage({ [Files.KEY]: saved })), { paths: [A, B], active: B });
    const unknownActive = JSON.stringify({ paths: [A], active: C });
    assert.deepEqual(Files.read(storage({ [Files.KEY]: unknownActive })), { paths: [A], active: null });
  });

  test(`${os} read: the pre-016 last path is folded in once and removed`, () => {
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

  test(`${os} write: the value is saved cleaned, a broken storage is ignored`, () => {
    const store = storage();
    Files.write({ paths: [A, A, B], active: A2 }, store);
    assert.deepEqual(JSON.parse(store.map.get(Files.KEY)), { paths: [A, B], active: A });
    assert.doesNotThrow(() => Files.write({ paths: [A], active: A }, storage({}, { broken: true })));
  });

  test(`${os} remember: appends once in any spelling, never reorders, never changes the active file`, () => {
    let files = Files.empty();
    files = Files.remember(files, A);
    files = Files.remember(files, B);
    files = Files.remember(files, A2);
    assert.deepEqual(files, { paths: [A, B], active: null });
    const withActive = Files.remember({ paths: [A], active: A }, B);
    assert.deepEqual(withActive, { paths: [A, B], active: A });
    assert.deepEqual(Files.remember(withActive, ""), withActive);
    assert.deepEqual(Files.remember(withActive, notA), { paths: [A, B, notA], active: A });
  });

  test(`${os} forget: removes the path, and the active role when it was that file`, () => {
    const files = { paths: [A, B, C], active: B };
    assert.deepEqual(Files.forget(files, B2), { paths: [A, C], active: null });
    assert.deepEqual(Files.forget(files, A), { paths: [B, C], active: B });
    assert.deepEqual(Files.forget(files, "nowhere.mbox"), files);
    assert.deepEqual(Files.forget(files, notA), files);
  });

  test(`${os} setActive: marks a known path, adds an unknown one, null clears`, () => {
    const files = { paths: [A, B], active: A };
    assert.deepEqual(Files.setActive(files, B2), { paths: [A, B], active: B });
    assert.deepEqual(Files.setActive(files, C), { paths: [A, B, C], active: C });
    assert.deepEqual(Files.setActive(files, null), { paths: [A, B], active: null });
  });

  test(`${os} pathKey, samePath and fileName: the OS's spellings compare equal, the name is the last segment`, () => {
    assert.equal(Files.samePath(A, A2), true);
    assert.equal(Files.pathKey(A), Files.pathKey(A2));
    assert.equal(Files.samePath(A, B), false);
    assert.equal(Files.samePath(A, notA), false);
    assert.equal(Files.pathKey(""), "");
    assert.equal(Files.pathKey(null), "");
    assert.equal(Files.fileName(A), "All mail-001.mbox");
    assert.equal(Files.fileName("archive.mbox"), "archive.mbox");
    assert.equal(Files.fileName("/srv/mail/x.mbox"), "x.mbox");
    assert.equal(Files.fileName(""), "");
  });
}

test("windows: both slashes end a segment; mac: a backslash is part of the name", () => {
  assert.equal(Base.create("windows").fileName("D:/mail\\x.mbox"), "x.mbox");
  assert.equal(Base.create("windows").samePath("D:\\mail\\x.mbox", "D:/mail/x.mbox"), true);
  assert.equal(Base.create("mac").fileName("/Users/anna/odd\\name.mbox"), "odd\\name.mbox");
  assert.equal(Base.create("mac").samePath("/Users/anna/x.mbox", "/Users/Anna/x.mbox"), false);
});
