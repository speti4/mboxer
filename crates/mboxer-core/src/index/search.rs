//! Opening a sidecar index and querying it.
//!
//! Every hit is served from stored fields; the mbox itself is only touched to
//! verify freshness on open. A free-text query goes through tantivy's parser
//! over the folded text fields, so `szamla` and `számla` are the same query;
//! the structured filters are added as `Must` clauses around it.

use std::collections::BTreeMap;
use std::ops::Bound;
use std::path::{Path, PathBuf};

use tantivy::collector::{Count, TopDocs};
use tantivy::query::{
    AllQuery, BooleanQuery, Occur, PhrasePrefixQuery, Query, QueryParser, RangeQuery, TermQuery,
};
use tantivy::schema::{Field, IndexRecordOption, Value};
use tantivy::{
    DocAddress, Index, IndexReader, Order, ReloadPolicy, Searcher, TantivyDocument, Term,
};

use crate::error::Result;
use crate::index::fingerprint::{IndexMeta, IndexStatus, StaleReason, index_dir, status};
use crate::index::schema::{Fields, fold_analyzer, register_tokenizer};

/// How many dictionary terms the word being typed may expand to; the terms
/// come in dictionary order, so the shortest completions come first.
const PREFIX_EXPANSIONS: u32 = 200;

/// Result of [`MboxIndex::open`].
#[derive(Debug)]
pub enum Opened {
    Ready(Box<MboxIndex>),
    Missing,
    Stale(StaleReason),
}

/// An opened, fresh sidecar index.
pub struct MboxIndex {
    mbox: PathBuf,
    dir: PathBuf,
    meta: IndexMeta,
    index: Index,
    reader: IndexReader,
    fields: Fields,
}

impl std::fmt::Debug for MboxIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MboxIndex")
            .field("mbox", &self.mbox)
            .field("dir", &self.dir)
            .field("meta", &self.meta)
            .finish_non_exhaustive()
    }
}

/// Ordering of hits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Newest first; messages without a date come last.
    #[default]
    DateDesc,
    /// Best match first (BM25).
    Relevance,
}

/// One search: free text plus structured filters, all AND-ed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    /// Free text over subject, body, from, to, cc and attachment names.
    pub text: Option<String>,
    /// Words that must appear in the `From` display string.
    pub from: Option<String>,
    /// Words that must appear in `To` or `Cc`.
    pub to: Option<String>,
    /// Words that must appear in the subject.
    pub subject: Option<String>,
    /// Unix seconds, inclusive.
    pub after: Option<i64>,
    /// Unix seconds, exclusive.
    pub before: Option<i64>,
    /// Exact Gmail labels; every one must be present.
    pub labels: Vec<String>,
    /// Exact Gmail labels; at least one must be present. The viewer sends a
    /// label and its sub-labels (`Work`, `Work/Clients`, …) for a click on a
    /// node of the label tree.
    pub any_labels: Vec<String>,
    pub has_attachment: Option<bool>,
    /// [`MboxIndex::list`] groups the matches by Gmail thread (`X-GM-THRID`);
    /// `search` ignores it. The CLI leaves it off.
    pub group_threads: bool,
    /// The last word of `text`, `from`, `to` and `subject` is still being typed: it
    /// matches as a prefix (`szaml` finds `számla`) unless the text ends in
    /// whitespace, the word carries syntax, or a phrase is open before it.
    /// For a search that refreshes on every keystroke; the CLI leaves it off.
    pub prefix_last_word: bool,
    pub limit: usize,
    pub offset: usize,
    pub sort: Sort,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            from: None,
            to: None,
            subject: None,
            after: None,
            before: None,
            labels: Vec::new(),
            any_labels: Vec::new(),
            has_attachment: None,
            group_threads: false,
            prefix_last_word: false,
            limit: 50,
            offset: 0,
            sort: Sort::DateDesc,
        }
    }
}

/// Splits off the word still being typed at the end of `text`: the last
/// whitespace-delimited word when the text does not end in whitespace, the
/// word is plain (no quotes, operators or wildcards) and no phrase is open
/// before it. Returns `(rest, word)`.
pub(crate) fn typing_word(text: &str) -> Option<(&str, &str)> {
    if text.is_empty() || text.ends_with(char::is_whitespace) {
        return None;
    }
    let start = text
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map_or(0, |(i, c)| i + c.len_utf8());
    let (rest, word) = text.split_at(start);
    let syntax = |c: char| {
        matches!(
            c,
            '"' | '\'' | ':' | '*' | '~' | '^' | '(' | ')' | '[' | ']' | '{' | '}'
        )
    };
    if word.starts_with('-') || word.starts_with('+') || word.contains(syntax) {
        return None;
    }
    if rest.matches('"').count() % 2 == 1 {
        return None;
    }
    Some((rest, word))
}

/// `word` as a prefix on every one of `fields`: its folded tokens in order,
/// the last one completed from the dictionary. `None` when the word folds
/// to nothing or its last token is a single character.
fn prefix_query(fields: &[Field], word: &str) -> Option<Box<dyn Query>> {
    let mut analyzer = fold_analyzer();
    let mut stream = analyzer.token_stream(word);
    let mut tokens = Vec::new();
    while let Some(token) = stream.next() {
        tokens.push(token.text.clone());
    }
    if tokens.last()?.chars().count() < 2 {
        return None;
    }
    let clauses = fields
        .iter()
        .map(|&field| {
            let terms = tokens
                .iter()
                .map(|t| Term::from_field_text(field, t))
                .collect();
            let mut query = PhrasePrefixQuery::new(terms);
            query.set_max_expansions(PREFIX_EXPANSIONS);
            (Occur::Should, Box::new(query) as Box<dyn Query>)
        })
        .collect();
    Some(Box::new(BooleanQuery::new(clauses)))
}

impl SearchQuery {
    /// A free-text query with the default limit and sort.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            ..Self::default()
        }
    }
}

/// One matching message, assembled from stored fields only.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub offset: u64,
    pub len: u64,
    pub date: Option<i64>,
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub subject: Option<String>,
    pub labels: Vec<String>,
    pub has_attachment: bool,
    pub attachment_names: Vec<String>,
    /// Gmail's `X-GM-THRID`; absent for mail that did not come from Gmail.
    pub thread_id: Option<u64>,
    /// Present when sorted by relevance.
    pub score: Option<f32>,
}

/// What a search returned.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// Number of matching messages, independent of `limit`/`offset`.
    pub total: usize,
    pub hits: Vec<Hit>,
    /// Query-syntax problems the lenient parser skipped over.
    pub warnings: Vec<String>,
}

impl MboxIndex {
    /// Opens the sidecar of `mbox` when it exists and matches the file.
    pub fn open(mbox: &Path) -> Result<Opened> {
        let meta = match status(mbox)? {
            IndexStatus::Missing => return Ok(Opened::Missing),
            IndexStatus::Stale(reason) => return Ok(Opened::Stale(reason)),
            IndexStatus::Fresh(meta) => meta,
        };
        let dir = index_dir(mbox);
        let index = Index::open_in_dir(&dir)?;
        register_tokenizer(&index);
        let fields = Fields::from_schema(&index.schema())?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        Ok(Opened::Ready(Box::new(Self {
            mbox: mbox.to_path_buf(),
            dir,
            meta,
            index,
            reader,
            fields,
        })))
    }

    pub fn mbox(&self) -> &Path {
        &self.mbox
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn meta(&self) -> &IndexMeta {
        &self.meta
    }

    pub fn message_count(&self) -> u64 {
        self.meta.message_count
    }

    pub(crate) fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    pub(crate) fn fields(&self) -> &Fields {
        &self.fields
    }

    /// How many messages carry each Gmail label, sorted by label. Exact: the
    /// index is built in one pass and never holds deleted documents (D-04), so
    /// a term's document frequency is its message count. Labels are
    /// multi-valued, so the counts do not add up to the message count.
    pub fn label_counts(&self) -> Result<Vec<(String, u64)>> {
        let searcher = self.reader.searcher();
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for segment in searcher.segment_readers() {
            let inverted = segment.inverted_index(self.fields.labels)?;
            let mut terms = inverted.terms().stream()?;
            while let Some((key, info)) = terms.next() {
                let label = String::from_utf8_lossy(key).into_owned();
                *counts.entry(label).or_insert(0) += u64::from(info.doc_freq);
            }
        }
        Ok(counts.into_iter().collect())
    }

    /// Runs `query` and returns one page of hits plus the total count.
    pub fn search(&self, query: &SearchQuery) -> Result<SearchResult> {
        let (tantivy_query, warnings) = self.to_tantivy_query(query);
        let searcher = self.reader.searcher();
        if query.limit == 0 {
            let total = searcher.search(&*tantivy_query, &Count)?;
            return Ok(SearchResult {
                total,
                hits: Vec::new(),
                warnings,
            });
        }
        let top = TopDocs::with_limit(query.limit).and_offset(query.offset);
        let (total, hits) = match query.sort {
            Sort::DateDesc => {
                let collector = top.order_by_fast_field::<i64>("date", Order::Desc);
                let (total, docs) = searcher.search(&*tantivy_query, &(Count, collector))?;
                let hits = docs
                    .into_iter()
                    .map(|(_, address)| hit_from_address(&self.fields, &searcher, address, None))
                    .collect::<Result<Vec<_>>>()?;
                (total, hits)
            }
            Sort::Relevance => {
                let collector = top.order_by_score();
                let (total, docs) = searcher.search(&*tantivy_query, &(Count, collector))?;
                let hits = docs
                    .into_iter()
                    .map(|(score, address)| {
                        hit_from_address(&self.fields, &searcher, address, Some(score))
                    })
                    .collect::<Result<Vec<_>>>()?;
                (total, hits)
            }
        };
        Ok(SearchResult {
            total,
            hits,
            warnings,
        })
    }

    /// Fetches one message by its D-03 identity.
    pub fn lookup(&self, offset: u64) -> Result<Option<Hit>> {
        let query = TermQuery::new(
            Term::from_field_u64(self.fields.offset, offset),
            IndexRecordOption::Basic,
        );
        let searcher = self.reader.searcher();
        let docs = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
        match docs.first() {
            Some(&(_, address)) => Ok(Some(hit_from_address(
                &self.fields,
                &searcher,
                address,
                None,
            )?)),
            None => Ok(None),
        }
    }

    /// `text` through the lenient query parser over `fields`.
    fn parse_text(
        &self,
        fields: Vec<Field>,
        text: &str,
        boost_subject: bool,
        warnings: &mut Vec<String>,
    ) -> Box<dyn Query> {
        let mut parser = QueryParser::for_index(&self.index, fields);
        parser.set_conjunction_by_default();
        if boost_subject {
            parser.set_field_boost(self.fields.subject, 2.0);
        }
        let (parsed, errors) = parser.parse_query_lenient(text);
        warnings.extend(errors.iter().map(|e| e.to_string()));
        parsed
    }

    /// One free-text clause; with `prefix_last_word` the word being typed
    /// becomes a prefix query AND-ed to the parse of the rest.
    fn text_clause(
        &self,
        fields: Vec<Field>,
        text: &str,
        boost_subject: bool,
        prefix_last_word: bool,
        warnings: &mut Vec<String>,
    ) -> Box<dyn Query> {
        if prefix_last_word
            && let Some((rest, word)) = typing_word(text)
            && let Some(prefix) = prefix_query(&fields, word)
        {
            let rest = rest.trim();
            if rest.is_empty() {
                return prefix;
            }
            let parsed = self.parse_text(fields, rest, boost_subject, warnings);
            return Box::new(BooleanQuery::new(vec![
                (Occur::Must, parsed),
                (Occur::Must, prefix),
            ]));
        }
        self.parse_text(fields, text, boost_subject, warnings)
    }

    pub(crate) fn to_tantivy_query(&self, query: &SearchQuery) -> (Box<dyn Query>, Vec<String>) {
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let mut warnings = Vec::new();
        let f = &self.fields;
        let prefix = query.prefix_last_word;

        // Trailing whitespace is meaningful to `typing_word`: only the start is trimmed.
        if let Some(text) = query
            .text
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            let fields = f.default_search_fields();
            clauses.push((
                Occur::Must,
                self.text_clause(fields, text, true, prefix, &mut warnings),
            ));
        }
        if let Some(from) = query
            .from
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            clauses.push((
                Occur::Must,
                self.text_clause(vec![f.from], from, false, prefix, &mut warnings),
            ));
        }
        if let Some(to) = query
            .to
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            clauses.push((
                Occur::Must,
                self.text_clause(vec![f.to, f.cc], to, false, prefix, &mut warnings),
            ));
        }
        if let Some(subject) = query
            .subject
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            clauses.push((
                Occur::Must,
                self.text_clause(vec![f.subject], subject, false, prefix, &mut warnings),
            ));
        }
        if query.after.is_some() || query.before.is_some() {
            let lower = query.after.map_or(Bound::Unbounded, |t| {
                Bound::Included(Term::from_field_i64(f.date, t))
            });
            let upper = query.before.map_or(Bound::Unbounded, |t| {
                Bound::Excluded(Term::from_field_i64(f.date, t))
            });
            clauses.push((Occur::Must, Box::new(RangeQuery::new(lower, upper))));
        }
        for label in &query.labels {
            let term = Term::from_field_text(f.labels, label);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
            ));
        }
        if !query.any_labels.is_empty() {
            let any: Vec<(Occur, Box<dyn Query>)> = query
                .any_labels
                .iter()
                .map(|label| {
                    let term = Term::from_field_text(f.labels, label);
                    let query: Box<dyn Query> =
                        Box::new(TermQuery::new(term, IndexRecordOption::Basic));
                    (Occur::Should, query)
                })
                .collect();
            clauses.push((Occur::Must, Box::new(BooleanQuery::new(any))));
        }
        if let Some(flag) = query.has_attachment {
            let term = Term::from_field_bool(f.has_attachment, flag);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
            ));
        }

        let combined: Box<dyn Query> = match clauses.len() {
            0 => Box::new(AllQuery),
            1 => clauses.pop().map(|(_, q)| q).expect("one clause"),
            _ => Box::new(BooleanQuery::new(clauses)),
        };
        (combined, warnings)
    }
}

/// Assembles a [`Hit`] from the stored fields of one document.
pub(crate) fn hit_from_address(
    f: &Fields,
    searcher: &Searcher,
    address: DocAddress,
    score: Option<f32>,
) -> Result<Hit> {
    let doc: TantivyDocument = searcher.doc(address)?;
    let strings = |field| {
        doc.get_all(field)
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    Ok(Hit {
        offset: doc
            .get_first(f.offset)
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        len: doc.get_first(f.len).and_then(|v| v.as_u64()).unwrap_or(0),
        date: doc.get_first(f.date).and_then(|v| v.as_i64()),
        from: strings(f.from),
        to: strings(f.to),
        subject: doc
            .get_first(f.subject)
            .and_then(|v| v.as_str().map(str::to_owned)),
        labels: strings(f.labels),
        has_attachment: doc
            .get_first(f.has_attachment)
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        attachment_names: strings(f.attachment_names),
        thread_id: doc.get_first(f.thread_id).and_then(|v| v.as_u64()),
        score,
    })
}

#[cfg(test)]
mod tests {
    use super::typing_word;

    #[test]
    fn the_word_being_typed_is_the_plain_last_word() {
        assert_eq!(typing_word("szaml"), Some(("", "szaml")));
        assert_eq!(
            typing_word("konyveles szaml"),
            Some(("konyveles ", "szaml"))
        );
        assert_eq!(typing_word("  Száml"), Some(("  ", "Száml")));
        assert_eq!(typing_word("\"big bad\" wo"), Some(("\"big bad\" ", "wo")));
    }

    #[test]
    fn a_finished_word_or_syntax_is_left_to_the_parser() {
        assert_eq!(typing_word(""), None);
        assert_eq!(typing_word("szaml "), None, "a space closes the word");
        assert_eq!(typing_word("from:ann"), None);
        assert_eq!(typing_word("a -b"), None);
        assert_eq!(typing_word("foo*"), None);
        assert_eq!(typing_word("\"szamla mell"), None, "inside an open phrase");
        assert_eq!(typing_word("x \"y\""), None);
    }
}
