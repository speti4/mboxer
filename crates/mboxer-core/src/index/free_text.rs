//! The free text of a search, turned into tantivy queries by hand.
//!
//! The grammar is small: words are AND-ed, `"double quotes"` make a phrase
//! (an unterminated quote runs to the end), `OR` joins the words on either
//! side, `-word`, `-"a phrase"` and `NOT word` exclude, `AND` is the default
//! and may be written, and a trailing `*` makes a word a prefix. Everything
//! else is plain text: a colon, a bracket, a caret or a tilde is punctuation
//! that the index's own analyzer drops, so `Re: számla` searches the two
//! words `re` and `szamla`. A word with punctuation inside (`re:számla`,
//! `10:30`, a URL) folds to several tokens and matches them in order, as a
//! phrase.
//!
//! tantivy's query parser is not used here: its grammar reads `Re:` as a
//! field name, `[a TO b]` as a range, `^2` as a boost and `/x/` as a regex,
//! and its escapes differ between those contexts. Building the term and
//! phrase queries through [`fold_analyzer`] keeps the accent folding of the
//! index and leaves nothing for the parser to misread.

use tantivy::query::{
    AllQuery, BooleanQuery, BoostQuery, EmptyScorer, EnableScoring, Explanation, Occur,
    PhrasePrefixQuery, PhraseQuery, Query, Scorer, TermQuery, Weight,
};
use tantivy::schema::{Field, IndexRecordOption};
use tantivy::{DocId, DocSet, Score, SegmentReader, Term};

use crate::index::schema::fold_analyzer;

/// How many dictionary terms a prefix may expand to; the terms come in
/// dictionary order, so the shortest completions come first.
const PREFIX_EXPANSIONS: u32 = 200;

/// One searchable piece of the free text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Item<'a> {
    /// The word or the phrase between the quotes, as typed.
    pub text: &'a str,
    /// Its last token matches as a prefix: a trailing `*`, or the word
    /// still being typed.
    pub prefix: bool,
}

/// The free text as clauses: every group of `must` has to match (a group is
/// one item or several joined by `OR`), no item of `must_not` may.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FreeText<'a> {
    pub must: Vec<Vec<Item<'a>>>,
    pub must_not: Vec<Item<'a>>,
}

/// A whitespace-delimited piece of the text before grouping.
#[derive(Debug)]
struct Lexeme<'a> {
    text: &'a str,
    negated: bool,
    quoted: bool,
}

fn lex(text: &str) -> Vec<Lexeme<'_>> {
    let mut lexemes = Vec::new();
    let mut rest = text;
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return lexemes;
        }
        // A `-` negates only when something follows it directly.
        let (negated, body) = match rest.strip_prefix('-') {
            Some(body) if body.starts_with(|c: char| !c.is_whitespace()) => (true, body),
            _ => (false, rest),
        };
        if let Some(after) = body.strip_prefix('"') {
            let end = after.find('"');
            lexemes.push(Lexeme {
                text: &after[..end.unwrap_or(after.len())],
                negated,
                quoted: true,
            });
            rest = end.map_or("", |end| &after[end + 1..]);
        } else {
            let end = body
                .find(|c: char| c.is_whitespace() || c == '"')
                .unwrap_or(body.len());
            lexemes.push(Lexeme {
                text: &body[..end],
                negated,
                quoted: false,
            });
            rest = &body[end..];
        }
    }
}

/// Splits `text` into clauses. With `prefix_last_word` the last word is still
/// being typed and matches as a prefix (`szaml` finds `számla`), unless the
/// text ends in whitespace, the word is negated or quoted, or it ends in
/// punctuation (`Re:` is a finished word).
pub(crate) fn parse(text: &str, prefix_last_word: bool) -> FreeText<'_> {
    let lexemes = lex(text);
    let typing = prefix_last_word && !text.ends_with(char::is_whitespace);
    let mut out = FreeText::default();
    // An `OR` waits for its right side; a `NOT` for the item it excludes.
    let mut join = false;
    let mut negate = false;
    let mut last_positive = false;
    for (i, lexeme) in lexemes.iter().enumerate() {
        let typed_last = typing && i + 1 == lexemes.len() && !lexeme.quoted && !lexeme.negated;
        if !lexeme.quoted && !lexeme.negated && !typed_last {
            match lexeme.text {
                "OR" => {
                    join = last_positive;
                    continue;
                }
                "AND" => {
                    join = false;
                    continue;
                }
                "NOT" => {
                    negate = true;
                    join = false;
                    continue;
                }
                _ => {}
            }
        }
        let item = if lexeme.quoted {
            Item {
                text: lexeme.text,
                prefix: false,
            }
        } else if let Some(stem) = lexeme.text.strip_suffix('*') {
            Item {
                text: stem.trim_end_matches('*'),
                prefix: true,
            }
        } else {
            Item {
                text: lexeme.text,
                prefix: typed_last && lexeme.text.ends_with(char::is_alphanumeric),
            }
        };
        if lexeme.negated || std::mem::take(&mut negate) {
            out.must_not.push(item);
            last_positive = false;
        } else if join && let Some(group) = out.must.last_mut() {
            group.push(item);
        } else {
            out.must.push(vec![item]);
            last_positive = true;
        }
        join = false;
    }
    out
}

/// `item` on every one of `fields`, its folded tokens in order; `boost`
/// weighs one field up. `None` when the item folds to nothing. A prefix
/// needs at least two letters in its last token, else the item is a whole
/// word.
fn item_query(fields: &[Field], boost: Option<(Field, f32)>, item: Item) -> Option<Box<dyn Query>> {
    let mut analyzer = fold_analyzer();
    let mut stream = analyzer.token_stream(item.text);
    let mut tokens: Vec<(usize, String)> = Vec::new();
    while let Some(token) = stream.next() {
        tokens.push((token.position, token.text.clone()));
    }
    let last = tokens.last()?;
    let prefix = item.prefix && last.1.chars().count() >= 2;
    let per_field = fields
        .iter()
        .map(|&field| {
            let mut terms: Vec<(usize, Term)> = tokens
                .iter()
                .map(|(position, text)| (*position, Term::from_field_text(field, text)))
                .collect();
            let query: Box<dyn Query> = if prefix {
                let mut query = PhrasePrefixQuery::new_with_offset(terms);
                query.set_max_expansions(PREFIX_EXPANSIONS);
                Box::new(query)
            } else if terms.len() == 1 {
                let (_, term) = terms.remove(0);
                Box::new(TermQuery::new(term, IndexRecordOption::WithFreqs))
            } else {
                Box::new(PhraseQuery::new_with_offset(terms))
            };
            let query: Box<dyn Query> = match boost {
                Some((boosted, factor)) if boosted == field => {
                    Box::new(BoostQuery::new(query, factor))
                }
                _ => query,
            };
            (Occur::Should, query)
        })
        .collect();
    Some(Box::new(BooleanQuery::new(per_field)))
}

/// The query behind the free `text` over `fields`. Text without a word to
/// search matches everything, and so does an exclusion alone (`-spam` is every
/// message without `spam`).
pub(crate) fn free_text_query(
    fields: &[Field],
    boost: Option<(Field, f32)>,
    text: &str,
    prefix_last_word: bool,
) -> Box<dyn Query> {
    let parsed = parse(text, prefix_last_word);
    let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    for group in parsed.must {
        let mut any: Vec<(Occur, Box<dyn Query>)> = group
            .into_iter()
            .filter_map(|item| item_query(fields, boost, item))
            .map(|query| (Occur::Should, query))
            .collect();
        match any.len() {
            0 => {}
            1 => clauses.push((Occur::Must, any.remove(0).1)),
            _ => clauses.push((Occur::Must, Box::new(BooleanQuery::new(any)))),
        }
    }
    let positive = !clauses.is_empty();
    for item in parsed.must_not {
        if let Some(query) = item_query(fields, boost, item) {
            clauses.push((Occur::MustNot, Box::new(Excluded(query))));
        }
    }
    if !positive {
        clauses.push((Occur::Must, Box::new(AllQuery)));
    }
    match clauses.len() {
        1 => clauses.remove(0).1,
        _ => Box::new(BooleanQuery::new(clauses)),
    }
}

/// An excluded clause. tantivy's `Exclude` probes its exclusion set with
/// `seek_danger` from the first document of the include side, which can lie
/// before the first match a phrase scorer already stands on, and
/// `PhraseScorer::seek_danger` debug-asserts against a target behind it
/// (tantivy 0.27: `számla -"spam jelentés"` panics in a debug build). The
/// scorer of this wrapper answers `seek_danger` with the default `DocSet`
/// implementation, which handles such a target.
#[derive(Debug)]
struct Excluded(Box<dyn Query>);

impl Clone for Excluded {
    fn clone(&self) -> Self {
        Self(self.0.box_clone())
    }
}

impl Query for Excluded {
    fn weight(&self, enable_scoring: EnableScoring<'_>) -> tantivy::Result<Box<dyn Weight>> {
        Ok(Box::new(ExcludedWeight(self.0.weight(enable_scoring)?)))
    }

    fn query_terms<'a>(&'a self, visitor: &mut dyn FnMut(&'a Term, bool)) {
        self.0.query_terms(visitor);
    }
}

struct ExcludedWeight(Box<dyn Weight>);

impl Weight for ExcludedWeight {
    fn scorer(&self, reader: &SegmentReader, boost: Score) -> tantivy::Result<Box<dyn Scorer>> {
        let scorer = self.0.scorer(reader, boost)?;
        // An empty scorer stays recognisable, so the boolean query drops it.
        if scorer.is::<EmptyScorer>() {
            return Ok(scorer);
        }
        Ok(Box::new(ExcludedScorer(scorer)))
    }

    fn explain(&self, reader: &SegmentReader, doc: DocId) -> tantivy::Result<Explanation> {
        self.0.explain(reader, doc)
    }
}

struct ExcludedScorer(Box<dyn Scorer>);

impl DocSet for ExcludedScorer {
    fn advance(&mut self) -> DocId {
        self.0.advance()
    }

    fn seek(&mut self, target: DocId) -> DocId {
        self.0.seek(target)
    }

    fn doc(&self) -> DocId {
        self.0.doc()
    }

    fn size_hint(&self) -> u32 {
        self.0.size_hint()
    }

    fn cost(&self) -> u64 {
        self.0.cost()
    }
}

impl Scorer for ExcludedScorer {
    fn score(&mut self) -> Score {
        self.0.score()
    }
}

#[cfg(test)]
mod tests {
    use super::{FreeText, Item, parse};

    fn word(text: &str) -> Item<'_> {
        Item {
            text,
            prefix: false,
        }
    }

    fn prefix(text: &str) -> Item<'_> {
        Item { text, prefix: true }
    }

    fn must<'a>(groups: &[&[Item<'a>]]) -> FreeText<'a> {
        FreeText {
            must: groups.iter().map(|group| group.to_vec()).collect(),
            must_not: Vec::new(),
        }
    }

    #[test]
    fn grammar_characters_are_part_of_the_word() {
        assert_eq!(
            parse("Re: számla", false),
            must(&[&[word("Re:")], &[word("számla")]])
        );
        assert_eq!(parse("re:számla", false), must(&[&[word("re:számla")]]));
        assert_eq!(
            parse("a^2 [b TO c]", false),
            must(&[&[word("a^2")], &[word("[b")], &[word("TO")], &[word("c]")]])
        );
    }

    #[test]
    fn quotes_or_and_not_and_the_minus() {
        assert_eq!(
            parse("\"Re: számla\" x\"y z", false),
            must(&[&[word("Re: számla")], &[word("x")], &[word("y z")]])
        );
        assert_eq!(
            parse("a b OR c AND d", false),
            must(&[&[word("a")], &[word("b"), word("c")], &[word("d")]])
        );
        assert_eq!(
            parse("OR a OR", false),
            must(&[&[word("a")]]),
            "an OR without both sides is dropped"
        );
        assert_eq!(parse("or", false), must(&[&[word("or")]]));
        let negated = parse("-spam NOT \"big bad\" a OR -b - \"-c\"", false);
        assert_eq!(
            negated.must,
            [vec![word("a")], vec![word("-")], vec![word("-c")]]
        );
        assert_eq!(negated.must_not, [word("spam"), word("big bad"), word("b")]);
        assert_eq!(
            parse("-", false),
            must(&[&[word("-")]]),
            "a lone minus is no exclusion"
        );
    }

    #[test]
    fn the_word_being_typed_and_a_trailing_star_are_prefixes() {
        assert_eq!(parse("szaml", true), must(&[&[prefix("szaml")]]));
        assert_eq!(
            parse("konyveles szaml", true),
            must(&[&[word("konyveles")], &[prefix("szaml")]])
        );
        assert_eq!(parse("re:száml", true), must(&[&[prefix("re:száml")]]));
        assert_eq!(parse("OR", true), must(&[&[prefix("OR")]]), "maybe ORSZÁG");
        assert_eq!(parse("szaml ", true), must(&[&[word("szaml")]]));
        assert_eq!(parse("Re:", true), must(&[&[word("Re:")]]));
        assert_eq!(parse("szaml", false), must(&[&[word("szaml")]]));
        assert_eq!(
            parse("\"szamla mell", true),
            must(&[&[word("szamla mell")]])
        );
        assert_eq!(parse("a -b", true).must_not, [word("b")]);
        assert_eq!(parse("számla**", false), must(&[&[prefix("számla")]]));
        assert_eq!(parse("x*", true), must(&[&[prefix("x")]]));
    }
}
