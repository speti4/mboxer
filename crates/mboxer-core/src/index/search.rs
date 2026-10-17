//! Opening a sidecar index and querying it.
//!
//! Every hit is served from stored fields; the mbox itself is only touched to
//! verify freshness on open. A free-text query goes through tantivy's parser
//! over the folded text fields, so `szamla` and `számla` are the same query;
//! the structured filters are added as `Must` clauses around it.

use std::ops::Bound;
use std::path::{Path, PathBuf};

use tantivy::collector::{Count, TopDocs};
use tantivy::query::{AllQuery, BooleanQuery, Occur, Query, QueryParser, RangeQuery, TermQuery};
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::{
    DocAddress, Index, IndexReader, Order, ReloadPolicy, Searcher, TantivyDocument, Term,
};

use crate::error::Result;
use crate::index::fingerprint::{IndexMeta, IndexStatus, StaleReason, index_dir, status};
use crate::index::schema::{Fields, register_tokenizer};

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
    /// Unix seconds, inclusive.
    pub after: Option<i64>,
    /// Unix seconds, exclusive.
    pub before: Option<i64>,
    /// Exact Gmail labels; every one must be present.
    pub labels: Vec<String>,
    pub has_attachment: Option<bool>,
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
            after: None,
            before: None,
            labels: Vec::new(),
            has_attachment: None,
            limit: 50,
            offset: 0,
            sort: Sort::DateDesc,
        }
    }
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
                    .map(|(_, address)| self.hit(&searcher, address, None))
                    .collect::<Result<Vec<_>>>()?;
                (total, hits)
            }
            Sort::Relevance => {
                let collector = top.order_by_score();
                let (total, docs) = searcher.search(&*tantivy_query, &(Count, collector))?;
                let hits = docs
                    .into_iter()
                    .map(|(score, address)| self.hit(&searcher, address, Some(score)))
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
            Some(&(_, address)) => Ok(Some(self.hit(&searcher, address, None)?)),
            None => Ok(None),
        }
    }

    fn to_tantivy_query(&self, query: &SearchQuery) -> (Box<dyn Query>, Vec<String>) {
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let mut warnings = Vec::new();
        let f = &self.fields;

        let mut parse = |fields: Vec<tantivy::schema::Field>, text: &str, boost_subject: bool| {
            let mut parser = QueryParser::for_index(&self.index, fields);
            parser.set_conjunction_by_default();
            if boost_subject {
                parser.set_field_boost(f.subject, 2.0);
            }
            let (parsed, errors) = parser.parse_query_lenient(text);
            warnings.extend(errors.iter().map(|e| e.to_string()));
            parsed
        };

        if let Some(text) = query
            .text
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            clauses.push((Occur::Must, parse(f.default_search_fields(), text, true)));
        }
        if let Some(from) = query
            .from
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            clauses.push((Occur::Must, parse(vec![f.from], from, false)));
        }
        if let Some(to) = query.to.as_deref().map(str::trim).filter(|t| !t.is_empty()) {
            clauses.push((Occur::Must, parse(vec![f.to, f.cc], to, false)));
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

    fn hit(&self, searcher: &Searcher, address: DocAddress, score: Option<f32>) -> Result<Hit> {
        let doc: TantivyDocument = searcher.doc(address)?;
        let f = &self.fields;
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
            score,
        })
    }
}
