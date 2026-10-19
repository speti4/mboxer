// Tests of the sanitized frame's DOM-free parts (`ui/frame.js`, task 053): the
// URL classes, the CSS scrub and the meta elements in front of the frame's
// document. Run: `node --test crates/mboxer-app/tests/ui/frame.test.js` (no
// package.json, no dependency); `cargo test -p mboxer-app` runs it too when
// `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Frame = require(path.join(__dirname, "..", "..", "ui", "frame.js"));

// A document stand-in: elements that keep their attributes and serialize as
// HTML, enough for the metas the frame's head gets.
const stubDocument = {
  createElement(tag) {
    const attributes = new Map();
    return {
      localName: tag,
      setAttribute: (name, value) => attributes.set(name, String(value)),
      getAttribute: (name) => (attributes.has(name) ? attributes.get(name) : null),
      get outerHTML() {
        const attrs = [...attributes].map(([name, value]) => ` ${name}="${value}"`).join("");
        return `<${tag}${attrs}>`;
      },
    };
  },
};

test("URLs are classified by their scheme", () => {
  assert.equal(Frame.classifyUrl(""), "empty");
  assert.equal(Frame.classifyUrl("#top"), "fragment");
  assert.equal(Frame.classifyUrl("//example.com/x.png"), "remote");
  assert.equal(Frame.classifyUrl(" HTTPS://example.com/x.png"), "remote");
  assert.equal(Frame.classifyUrl("java\tscript:alert(1)"), "other");
  assert.equal(Frame.classifyUrl("data:image/png;base64,AA=="), "data");
  assert.equal(Frame.classifyUrl("cid:logo@example.com"), "cid");
  assert.equal(Frame.classifyUrl("mailto:someone@example.com"), "mailto");
  assert.equal(Frame.classifyUrl("images/x.png"), "relative");
});

test("a plain @import is replaced by an empty comment", () => {
  const { safe, blocked, remote } = Frame.scrubCss("@import url(x);p{color:red}");
  assert.equal(safe, "/**/p{color:red}");
  assert.equal(blocked, "/**/p{color:red}");
  assert.equal(remote, 0);
});

test("the removal of an @import never joins the style text into a closing tag", () => {
  const { safe, blocked } = Frame.scrubCss("</@import;style><img src=https://example.com/x>");
  for (const text of [safe, blocked]) {
    assert.doesNotMatch(text, /<\/?style/i, text);
    assert.doesNotMatch(text, /</, text);
  }
  assert.equal(safe, "\\3c //**/style>\\3c img src=https://example.com/x>");
});

test("a < in style text becomes a CSS escape, the comment opener of old mail stays", () => {
  assert.equal(Frame.scrubCss('p::after{content:"</b>"}').safe, 'p::after{content:"\\3c /b>"}');
  assert.equal(Frame.scrubCss("<!-- p{color:red} -->").safe, "<!-- p{color:red} -->");
  assert.equal(Frame.scrubCss("a>b{color:red}").safe, "a>b{color:red}");
});

test("expression(), behavior and remote url()s are still scrubbed", () => {
  const css = "a{width:expression(alert(1));behavior:url(x.htc);background:url('https://example.com/b.png')}";
  const { safe, blocked, remote } = Frame.scrubCss(css);
  assert.match(safe, /expression-blocked\(/);
  assert.match(safe, /blocked:url\(x\.htc\)/);
  assert.match(safe, /url\('https:\/\/example\.com\/b\.png'\)/);
  assert.doesNotMatch(blocked, /https:/);
  assert.equal(remote, 1);
  assert.equal(Frame.scrubCss(null).safe, "");
});

test("the frame's head declares the charset, no referrer and the CSP", () => {
  const held = Frame.headMetas(stubDocument, false).map((meta) => meta.outerHTML);
  assert.deepEqual(held, [
    '<meta charset="utf-8">',
    '<meta name="referrer" content="no-referrer">',
    `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data: blob:; style-src 'unsafe-inline'">`,
  ]);
  const shown = Frame.headMetas(stubDocument, true).map((meta) => meta.outerHTML);
  assert.equal(shown[1], '<meta name="referrer" content="no-referrer">');
  assert.equal(
    shown[2],
    `<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data: blob: https: http:; style-src 'unsafe-inline'">`,
  );
});
