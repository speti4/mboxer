// mboxer query syntax: the Gmail-style operators of the search field
// (`from:`, `to:`, `subject:`, `after:`, `before:`, `has:attachment`,
// `label:`), the tokenizer behind them, the rewriter the filter panel uses to
// edit the field, and the words worth highlighting. Pure functions, no DOM:
// `window.MboxerQuery` in the viewer, CommonJS exports under Node for the tests.
(function (root, factory) {
  const api = factory();
  if (typeof module !== "undefined" && module.exports) module.exports = api;
  else root.MboxerQuery = api;
})(typeof window !== "undefined" ? window : globalThis, function () {
  "use strict";

  const OPERATORS = new Set(["from", "to", "subject", "label", "after", "before", "has"]);
  // The boolean keywords of the free text (`free_text.rs` in the core); not words to highlight.
  const KEYWORDS = new Set(["OR", "AND", "NOT"]);

  // ---------- folding ----------
  //
  // Mirrors the index's tokenizer: lower-cased, accents folded (`Számla` and
  // `szamla` are the same word), so a highlighted word is one the index matched.

  const foldCache = new Map();

  function foldChar(ch) {
    let folded = foldCache.get(ch);
    if (folded === undefined) {
      folded = ch.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
      foldCache.set(ch, folded);
    }
    return folded;
  }

  // Folds `text`; `map[k]` is the index in `text` of the character behind folded position `k`.
  function foldWithMap(text) {
    let folded = "";
    const map = [];
    let i = 0;
    for (const ch of text) {
      const f = foldChar(ch);
      for (let k = 0; k < f.length; k++) map.push(i);
      folded += f;
      i += ch.length;
    }
    map.push(text.length);
    return { folded, map };
  }

  function foldText(text) {
    let out = "";
    for (const ch of text) out += foldChar(ch);
    return out;
  }

  // ---------- tokens ----------

  // Whitespace-separated tokens; a double quote keeps the words together
  // until the next one (`"a b"`, `from:"a b"`), an unterminated quote runs to
  // the end. Each token is `{ raw, start, end }` with offsets into `text`.
  function tokenize(text) {
    const tokens = [];
    let i = 0;
    const n = text.length;
    while (i < n) {
      if (/\s/.test(text[i])) {
        i++;
        continue;
      }
      const start = i;
      let quoted = false;
      while (i < n && (quoted || !/\s/.test(text[i]))) {
        if (text[i] === '"') quoted = !quoted;
        i++;
      }
      tokens.push({ raw: text.slice(start, i), start, end: i });
    }
    return tokens;
  }

  // The value of an operator token with its quotes removed.
  function unquote(value) {
    return value.replace(/"/g, "");
  }

  // `{ name, value, negated }` when `raw` is an operator token, else null.
  // Names are case-insensitive; `-` counts only in front of `has:`.
  function classify(raw) {
    const m = /^(-?)([A-Za-z]+):([\s\S]*)$/.exec(raw);
    if (!m) return null;
    const name = m[2].toLowerCase();
    if (!OPERATORS.has(name)) return null;
    const negated = m[1] === "-";
    if (negated && name !== "has") return null;
    return { name, value: unquote(m[3]), negated };
  }

  // `YYYY-MM-DD` for a day written as `2026-01-05`, `2026/1/5` or alike; null otherwise.
  function normalizeDate(value) {
    const m = /^(\d{4})[-/](\d{1,2})[-/](\d{1,2})$/.exec(value.trim());
    if (!m) return null;
    const year = Number(m[1]);
    const month = Number(m[2]);
    const day = Number(m[3]);
    const date = new Date(year, month - 1, day);
    if (date.getFullYear() !== year || date.getMonth() !== month - 1 || date.getDate() !== day) return null;
    const pad = (x) => String(x).padStart(2, "0");
    return `${year}-${pad(month)}-${pad(day)}`;
  }

  // ---------- parse ----------

  // The structured query behind `text`. `resolveLabel(name)` maps a label
  // name to the key of a node of the label tree (the caller's tree; the name
  // itself when unknown). The first `label:` is that node (it and its
  // sub-labels), further ones are exact labels that must all be present.
  // `free` is the rest of the text in its original order; it ends with a
  // space when the raw text does or when an operator came last, so the
  // backend treats the last free word as finished rather than as a prefix.
  // An incomplete last token (no whitespace after it) is still being typed:
  // its value is ignored without a warning.
  function parse(text, resolveLabel = (name) => name) {
    const result = {
      free: "",
      from: "",
      to: "",
      subject: "",
      after: "",
      before: "",
      hasAttachment: null,
      label: null,
      labels: [],
      warnings: [],
    };
    const tokens = tokenize(text);
    const free = [];
    let lastWasOperator = false;
    const endsInSpace = /\s$/.test(text);
    tokens.forEach((token, index) => {
      const op = classify(token.raw);
      if (!op) {
        free.push(token.raw);
        lastWasOperator = false;
        return;
      }
      lastWasOperator = true;
      const typing = index === tokens.length - 1 && !endsInSpace;
      const value = op.value.trim();
      if (!value) return;
      switch (op.name) {
        case "from":
        case "to":
        case "subject":
          result[op.name] = result[op.name] ? `${result[op.name]} ${value}` : value;
          break;
        case "after":
        case "before": {
          const day = normalizeDate(value);
          if (day) result[op.name] = day;
          else if (!typing) result.warnings.push(`${op.name}: expects a day as YYYY-MM-DD`);
          break;
        }
        case "has":
          if (value.toLowerCase() === "attachment") result.hasAttachment = !op.negated;
          else if (!typing) result.warnings.push(`has: expects \`attachment\``);
          break;
        case "label":
          if (result.label == null) result.label = resolveLabel(value);
          else result.labels.push(value);
          break;
        default:
          break;
      }
    });
    result.free = free.join(" ");
    if (result.free && (endsInSpace || lastWasOperator)) result.free += " ";
    return result;
  }

  // ---------- rewrite ----------

  function quoteValue(value) {
    const clean = String(value).replace(/"/g, "").trim();
    return /\s/.test(clean) ? `"${clean}"` : clean;
  }

  // The token that carries `value` for the operator `name`, or null to drop
  // the operator: `has` takes true / false / null, the others a string.
  function serialize(name, value) {
    if (name === "has") {
      if (value === true) return "has:attachment";
      if (value === false) return "-has:attachment";
      return null;
    }
    if (value == null) return null;
    const clean = quoteValue(value);
    return clean ? `${name}:${clean}` : null;
  }

  // `text` with the operator `name` set to `value`: the first `name:` token is
  // replaced in place and the rest of that name removed, a missing operator
  // is appended at the end, an empty value removes the operator. Removing a
  // token takes one run of whitespace with it.
  function setOperator(text, name, value) {
    const token = serialize(name, value);
    const matches = tokenize(text).filter((t) => {
      const op = classify(t.raw);
      return op && op.name === name;
    });
    if (matches.length === 0) {
      if (!token) return text;
      const base = text.trimEnd();
      return base ? `${base} ${token}` : token;
    }
    let out = text;
    // Back to front, so the earlier offsets stay valid.
    for (let i = matches.length - 1; i >= 0; i--) {
      const { start, end } = matches[i];
      if (i === 0 && token) {
        out = out.slice(0, start) + token + out.slice(end);
        continue;
      }
      let from = start;
      let to = end;
      if (from > 0) {
        while (from > 0 && /\s/.test(out[from - 1])) from--;
      } else {
        while (to < out.length && /\s/.test(out[to])) to++;
      }
      out = out.slice(0, from) + out.slice(to);
    }
    return out;
  }

  // ---------- highlighting ----------

  // The folded words of `text` worth highlighting: free words, phrases as one
  // term, the values of `from:` / `to:` / `subject:` (they show in the list);
  // other operators, boolean keywords and negated words are left alone.
  function highlightTerms(text) {
    const terms = new Set();
    const add = (word) => {
      const folded = foldText(word).trim();
      if (folded) terms.add(folded);
    };
    const addWords = (value) => {
      for (const word of value.split(/\s+/)) {
        if (!word || word.startsWith("-") || KEYWORDS.has(word)) continue;
        add(word.replace(/^[+(]+|[)*]+$/g, ""));
      }
    };
    for (const { raw } of tokenize(text || "")) {
      const op = classify(raw);
      if (op) {
        if (op.name === "from" || op.name === "to" || op.name === "subject") addWords(op.value);
        continue;
      }
      if (raw.startsWith("-") || KEYWORDS.has(raw)) continue;
      const bare = raw.replace(/^[+(]+|[)*]+$/g, "");
      if (bare.startsWith('"')) add(bare.replace(/"/g, "").replace(/\s+/g, " "));
      else addWords(bare);
    }
    return Array.from(terms).sort((a, b) => b.length - a.length);
  }

  return {
    OPERATORS,
    foldChar,
    foldWithMap,
    foldText,
    tokenize,
    classify,
    normalizeDate,
    parse,
    setOperator,
    highlightTerms,
  };
});
