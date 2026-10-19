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
  // The index folds with tantivy's `AsciiFoldingFilter` (`fold_analyzer` in
  // `crates/mboxer-core/src/index/schema.rs`); here NFD and the combining
  // accents dropped cover most of it, and `LETTER_FOLDS` holds the rest of
  // what the filter does in Latin-1, Latin Extended-A, Latin Extended
  // Additional and the Latin ligatures. `tests/ui_query.rs` checks every
  // letter and digit of those blocks against the core's analyzer. Not
  // mirrored: the filter's other blocks (Latin Extended-B, IPA, circled and
  // full-width letters), and the accents it keeps outside the Latin letters
  // (`й`, `ά`), which NFD drops here.

  // A lower-cased letter the filter folds and NFD leaves whole, and its fold.
  // `ẞ` lower-cases to `ß`; `İ` lower-cases to `i` and a combining dot, which
  // the filter keeps; `ẛ` folds to `f`, the filter's choice.
  const LETTER_FOLDS = new Map(
    Object.entries({
      "¹": "1", "²": "2", "³": "3",
      æ: "ae", ð: "d", ø: "o", þ: "th", ß: "ss",
      đ: "d", ħ: "h", ı: "i", "i\u0307": "i\u0307", ĳ: "ij", ĸ: "q", ŀ: "l", ł: "l",
      ŉ: "n", ŋ: "n", œ: "oe", ŧ: "t", ſ: "s",
      ẚ: "a", ẛ: "f", ẜ: "s", ẝ: "s", ỻ: "ll", ỿ: "y",
      ﬀ: "ff", ﬁ: "fi", ﬂ: "fl", ﬃ: "ffi", ﬄ: "ffl", ﬆ: "st",
    }),
  );

  const foldCache = new Map();

  // `ch` as the index folds it: one character may fold to several (`ß` → `ss`).
  function foldChar(ch) {
    let folded = foldCache.get(ch);
    if (folded === undefined) {
      const lower = ch.toLowerCase();
      folded = LETTER_FOLDS.get(lower) ?? lower.normalize("NFD").replace(/[̀-ͯ]/g, "");
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

  // The `[start, end)` range of the folded text's `[start, end)` in the text
  // behind `map` (from `foldWithMap`): a range that ends inside a character
  // folded to several (`stras` in `strasse` from `Straße`) takes all of it.
  function textRange(map, start, end) {
    let to = end;
    while (to < map.length - 1 && map[to] === map[to - 1]) to++;
    return [map[start], map[to]];
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
  //
  // The split rule mirrors the index's analyzer (`fold_analyzer` in
  // `crates/mboxer-core/src/index/schema.rs`): tantivy's `SimpleTokenizer`
  // cuts the text at every character that is not `char::is_alphanumeric`
  // (Unicode alphabetic or numeric, `PIECE_BREAK` below), then the pieces are
  // lower-cased and accent-folded. The core searches a word or a quoted phrase
  // as its pieces in order, one position apart (`free_text.rs`), so a term
  // here is the folded pieces joined by one space, and it matches them in
  // order with any run of other characters between: `re:számla` marks
  // `Re: számla`, a lone `számla` stays unmarked. Not mirrored: the analyzer
  // drops pieces of 40 bytes or more, and the edges of a match may fall
  // inside a word (`re` marks the end of `there`), as a typed prefix does.

  // A run of characters the tokenizer does not keep in a piece.
  const PIECE_BREAK = /[^\p{Alphabetic}\p{N}]+/u;
  // The same run inside a term's pattern; its source must stay in sync.
  const PIECE_BREAK_SOURCE = "[^\\p{Alphabetic}\\p{N}]+";

  // `text` as a term: its folded pieces joined by one space, "" when none.
  function pieceTerm(text) {
    return text
      .split(PIECE_BREAK)
      .filter(Boolean)
      .map((piece) => foldText(piece))
      .join(" ");
  }

  // The folded terms of `text` worth highlighting: free words and phrases
  // (each one term, see the rule above), the words of the `from:` / `to:` /
  // `subject:` values (they show in the list); other operators, boolean
  // keywords and negated words are left alone. Longest first.
  function highlightTerms(text) {
    const terms = new Set();
    const add = (value) => {
      const term = pieceTerm(value);
      if (term) terms.add(term);
    };
    const addWords = (value) => {
      for (const word of value.split(/\s+/)) {
        if (!word || word.startsWith("-") || KEYWORDS.has(word)) continue;
        add(word);
      }
    };
    for (const { raw } of tokenize(text || "")) {
      const op = classify(raw);
      if (op) {
        if (op.name === "from" || op.name === "to" || op.name === "subject") addWords(op.value);
        continue;
      }
      if (raw.startsWith("-") || KEYWORDS.has(raw)) continue;
      const bare = raw.replace(/^[+(]+/, "");
      if (bare.startsWith('"')) add(bare);
      else addWords(bare);
    }
    return Array.from(terms).sort((a, b) => b.length - a.length);
  }

  const termPatterns = new Map();

  // The global pattern of a term of several pieces, cached.
  function termPattern(term) {
    let pattern = termPatterns.get(term);
    if (!pattern) {
      const pieces = term.split(" ").map((piece) => piece.replace(/[\\^$.*+?()[\]{}|/]/g, "\\$&"));
      pattern = new RegExp(pieces.join(PIECE_BREAK_SOURCE), "gu");
      termPatterns.set(term, pattern);
    }
    return pattern;
  }

  // The `[start, end)` ranges of `folded` (text folded by `foldWithMap`) where
  // one of `terms` (from `highlightTerms`) occurs, unsorted and maybe overlapping.
  function termRanges(folded, terms) {
    const ranges = [];
    for (const term of terms) {
      if (!term.includes(" ")) {
        let at = folded.indexOf(term);
        while (at >= 0) {
          ranges.push([at, at + term.length]);
          at = folded.indexOf(term, at + term.length);
        }
        continue;
      }
      const pattern = termPattern(term);
      pattern.lastIndex = 0;
      for (let m = pattern.exec(folded); m; m = pattern.exec(folded)) {
        ranges.push([m.index, m.index + m[0].length]);
      }
    }
    return ranges;
  }

  return {
    OPERATORS,
    foldChar,
    foldWithMap,
    textRange,
    foldText,
    tokenize,
    classify,
    normalizeDate,
    parse,
    setOperator,
    highlightTerms,
    termRanges,
  };
});
