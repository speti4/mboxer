//! Opening a sidecar index and querying it.
//!
//! Every hit is served from stored fields; the mbox itself is only touched to
//! verify freshness on open. The free text is turned into term and phrase
//! queries over the folded text fields by [`free_text`](crate::index::free_text),
//! so `szamla` and `számla` are the same query; the structured filters are
//! added as `Must` clauses around it.

use std::collections::BTreeMap;
use std::ops::Bound;
use std::path::{Path, PathBuf};

use tantivy::collector::{Count, TopDocs};
use tantivy::query::{AllQuery, BooleanQuery, Occur, Query, RangeQuery, TermQuery};
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::{
    DocAddress, Index, IndexReader, Order, ReloadPolicy, Searcher, TantivyDocument, Term,
};

use crate::error::Result;
use crate::index::fingerprint::{IndexMeta, IndexStatus, StaleReason, default_index_home, resolve};
use crate::index::free_text::free_text_query;
use crate::index::schema::{Fields, register_tokenizer};

/// How much more a free-text word in the subject weighs than one elsewhere.
const SUBJECT_BOOST: f32 = 2.0;

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
    /// Problems with the query text for the viewer's status bar. The free-text
    /// grammar takes any text as words, so none are reported at present.
    pub warnings: Vec<String>,
}

impl MboxIndex {
    /// Opens the index of `mbox` — beside the mbox or under the default index
    /// home — when it exists and matches the file.
    pub fn open(mbox: &Path) -> Result<Opened> {
        Self::open_in(mbox, &default_index_home())
    }

    /// [`MboxIndex::open`] with an explicit index home.
    pub fn open_in(mbox: &Path, home: &Path) -> Result<Opened> {
        let (dir, status) = resolve(mbox, home)?;
        let meta = match status {
            IndexStatus::Missing => return Ok(Opened::Missing),
            IndexStatus::Stale(reason) => return Ok(Opened::Stale(reason)),
            IndexStatus::Fresh(meta) => meta,
        };
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
    /// index is built in one pass and never holds deleted documents (D-004), so
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
        let tantivy_query = self.to_tantivy_query(query);
        let warnings = Vec::new();
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

    /// Fetches one message by its D-003 identity.
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

    pub(crate) fn to_tantivy_query(&self, query: &SearchQuery) -> Box<dyn Query> {
        let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        let f = &self.fields;
        let prefix = query.prefix_last_word;

        // Trailing whitespace closes the word being typed: only the start is trimmed.
        if let Some(text) = query
            .text
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            let fields = f.default_search_fields();
            clauses.push((
                Occur::Must,
                free_text_query(&fields, Some((f.subject, SUBJECT_BOOST)), text, prefix),
            ));
        }
        if let Some(from) = query
            .from
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            clauses.push((Occur::Must, free_text_query(&[f.from], None, from, prefix)));
        }
        if let Some(to) = query
            .to
            .as_deref()
            .map(str::trim_start)
            .filter(|t| !t.trim().is_empty())
        {
            clauses.push((
                Occur::Must,
                free_text_query(&[f.to, f.cc], None, to, prefix),
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
                free_text_query(&[f.subject], None, subject, prefix),
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
        combined
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
