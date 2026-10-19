// Tests of the search field's query syntax (`ui/query.js`): one case per
// operator, the tokenizer, the rewriter the filter panel uses and the
// highlight terms. Run: `node --test crates/mboxer-app/tests/ui/query.test.js`
// (no package.json, no dependency); `cargo test -p mboxer-app` runs it too
// when `node` is on PATH.
"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const path = require("node:path");

const Query = require(path.join(__dirname, "..", "..", "ui", "query.js"));

// A label tree with the label `Work/Invoices` under the container `Work`,
// and `Category personal` under the `Categories` container.
const KEYS = new Map([
  ["Inbox", "Inbox"],
  ["Sent", "Sent"],
  ["Work/Invoices", "Work/Invoices"],
  ["Work", "Work#"],
  ["Category personal", "Category personal"],
  ["Categories", "Categories#"],
]);
const resolve = (name) => KEYS.get(name) ?? name;
const parse = (text) => Query.parse(text, resolve);

function raws(text) {
  return Query.tokenize(text).map((t) => t.raw);
}

test("tokenize splits on whitespace and keeps quoted words together", () => {
  assert.deepEqual(raws('a  "b c" from:"d e" f'), ["a", '"b c"', 'from:"d e"', "f"]);
  assert.deepEqual(raws('x "open quote runs on'), ["x", '"open quote runs on']);
  assert.deepEqual(raws("   "), []);
  const [first] = Query.tokenize("  from:anna ");
  assert.equal(first.start, 2);
  assert.equal(first.end, 11);
});

test("from: collects the sender words, several from: operators concatenate", () => {
  assert.equal(parse("számla from:anna").from, "anna");
  assert.equal(parse("from:anna from:példa").from, "anna példa");
  assert.equal(parse('from:"anna példa"').from, "anna példa");
  assert.equal(parse("FROM:anna").from, "anna", "operator names are case-insensitive");
  assert.equal(parse("from:").from, "", "an empty value is ignored");
});

test("to: and subject: fill their own fields", () => {
  const parsed = parse("to:konyveles subject:számla");
  assert.equal(parsed.to, "konyveles");
  assert.equal(parsed.subject, "számla");
  assert.equal(parsed.free, "");
});

test("after: takes a day, in either notation, padded", () => {
  assert.equal(parse("after:2026-01-05 ").after, "2026-01-05");
  assert.equal(parse("after:2026/1/5 ").after, "2026-01-05");
  const bad = parse("after:2026-13-01 ");
  assert.equal(bad.after, "");
  assert.deepEqual(bad.warnings, ["after: expects a day as YYYY-MM-DD"]);
  const typing = parse("after:2026-1");
  assert.equal(typing.after, "");
  assert.deepEqual(typing.warnings, [], "an unfinished last token is still being typed");
});

test("before: is the same day notation", () => {
  assert.equal(parse("before:2026-01-31").before, "2026-01-31");
  assert.equal(parse("x before:2026-02-30 y").warnings.length, 1);
});

test("has:attachment and -has:attachment set the attachment filter", () => {
  assert.equal(parse("has:attachment").hasAttachment, true);
  assert.equal(parse("-has:attachment").hasAttachment, false);
  assert.equal(parse("HAS:Attachment").hasAttachment, true);
  assert.equal(parse("x").hasAttachment, null);
  const bad = parse("has:photo ");
  assert.equal(bad.hasAttachment, null);
  assert.deepEqual(bad.warnings, ["has: expects `attachment`"]);
});

test("label: the first one is the tree node, further ones exact labels", () => {
  assert.equal(parse("label:Inbox").label, "Inbox");
  assert.equal(parse("label:Work").label, "Work#", "a container resolves to its node key");
  assert.equal(parse('label:"Category personal"').label, "Category personal");
  assert.equal(parse("label:Nope").label, "Nope", "an unknown label stays an exact term");
  const two = parse("label:Inbox label:Sent label:Opened");
  assert.equal(two.label, "Inbox");
  assert.deepEqual(two.labels, ["Sent", "Opened"]);
  assert.equal(parse("x").label, null);
});

test("unknown operators, negated fields and phrases stay free text", () => {
  assert.equal(parse("foo:bar").free, "foo:bar");
  assert.equal(parse("-from:anna").free, "-from:anna");
  assert.equal(parse('"big bad" wolf OR -cat').free, '"big bad" wolf OR -cat');
});

test("a colon in a subject word is not an operator: Re: and Fwd: reach the backend as free text", () => {
  const parsed = parse("Re: számla from:anna");
  assert.equal(parsed.free, "Re: számla ");
  assert.equal(parsed.from, "anna");
  assert.equal(parsed.subject, "");
  assert.deepEqual(parsed.warnings, []);
  assert.equal(parse("re:számla").free, "re:számla");
  assert.equal(parse("Fwd: Re: 10:30").free, "Fwd: Re: 10:30");
  assert.equal(parse('"Re: számla"').free, '"Re: számla"');
  assert.equal(parse("https://example.com/a").free, "https://example.com/a");
});

test("the free text ends with a space when the raw text does or an operator came last", () => {
  assert.equal(parse("szaml").free, "szaml");
  assert.equal(parse("szamla ").free, "szamla ");
  assert.equal(parse("szamla from:anna").free, "szamla ");
  assert.equal(parse("from:anna szaml").free, "szaml");
  assert.equal(parse("a from:anna b ").free, "a b ");
  assert.equal(parse("from:anna").free, "");
});

test("setOperator replaces the first token in place and drops the rest", () => {
  assert.equal(Query.setOperator("számla from:anna után", "from", "bela"), "számla from:bela után");
  assert.equal(Query.setOperator("from:a x from:b y", "from", "c"), "from:c x y");
  assert.equal(Query.setOperator('x from:"anna példa"', "from", "anna"), "x from:anna");
});

test("setOperator removes the operator for an empty value, with one run of whitespace", () => {
  assert.equal(Query.setOperator("számla from:anna után", "from", ""), "számla után");
  assert.equal(Query.setOperator("from:anna számla", "from", null), "számla");
  assert.equal(Query.setOperator("számla  from:anna", "from", ""), "számla");
  assert.equal(Query.setOperator("from:anna", "from", ""), "");
  assert.equal(Query.setOperator("számla", "from", ""), "számla", "nothing to remove");
});

test("setOperator appends a missing operator at the end", () => {
  assert.equal(Query.setOperator("számla ", "from", "anna"), "számla from:anna");
  assert.equal(Query.setOperator("", "from", "anna"), "from:anna");
  assert.equal(Query.setOperator("x", "label", "Category personal"), 'x label:"Category personal"');
  assert.equal(Query.setOperator("x", "after", "2026-01-05"), "x after:2026-01-05");
});

test("setOperator writes has:attachment, -has:attachment or nothing", () => {
  assert.equal(Query.setOperator("x", "has", true), "x has:attachment");
  assert.equal(Query.setOperator("x has:attachment", "has", false), "x -has:attachment");
  assert.equal(Query.setOperator("x -has:attachment y", "has", null), "x y");
  assert.equal(Query.setOperator("x -has:attachment", "has", true), "x has:attachment");
});

test("setOperator and parse round-trip the panel's values", () => {
  let text = "számla";
  text = Query.setOperator(text, "from", "anna");
  text = Query.setOperator(text, "after", "2026-01-01");
  text = Query.setOperator(text, "has", false);
  text = Query.setOperator(text, "label", "Work");
  assert.equal(text, "számla from:anna after:2026-01-01 -has:attachment label:Work");
  const parsed = parse(text);
  assert.equal(parsed.free, "számla ");
  assert.equal(parsed.from, "anna");
  assert.equal(parsed.after, "2026-01-01");
  assert.equal(parsed.hasAttachment, false);
  assert.equal(parsed.label, "Work#");
});

test("highlightTerms folds words, keeps phrases whole and skips the rest of the syntax", () => {
  const terms = Query.highlightTerms('"Big  Bad" Wolf from:Anna -cat OR label:Inbox after:2026-01-01 +dog');
  assert.deepEqual([...terms].sort(), ["anna", "big bad", "dog", "wolf"]);
  assert.equal(terms[0], "big bad", "longest first");
  assert.deepEqual(Query.highlightTerms("Számla"), ["szamla"]);
  assert.deepEqual(Query.highlightTerms(""), []);
  assert.deepEqual(Query.highlightTerms('subject:"számla január"'), ["szamla", "januar"]);
});

test("normalizeDate rejects anything but a real day", () => {
  assert.equal(Query.normalizeDate("2026-02-29"), null);
  assert.equal(Query.normalizeDate("2024-02-29"), "2024-02-29");
  assert.equal(Query.normalizeDate("yesterday"), null);
  assert.equal(Query.normalizeDate("2026-1"), null);
});
