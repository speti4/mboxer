//! A full, sorted result set for a virtualised list.
//!
//! [`MboxIndex::search`] serves one page per call, which is right for a CLI but
//! wrong for a list the user drags through: tantivy's `TopDocs` with an offset
//! of 80 000 re-collects a heap of that size on every page. [`Listing`] runs
//! the query once, keeps every matching document's address in sort order
//! (8 bytes each: under a megabyte for a 100 000-message archive) and fetches
//! the stored fields of any slice on demand.

use std::cmp::Ordering;

use tantivy::collector::{Collector, SegmentCollector};
use tantivy::columnar::Column;
use tantivy::query::TermQuery;
use tantivy::schema::IndexRecordOption;
use tantivy::{DocAddress, DocId, Score, Searcher, SegmentOrdinal, SegmentReader, Term};

use crate::error::Result;
use crate::index::schema::Fields;
use crate::index::search::{Hit, MboxIndex, SearchQuery, Sort, hit_from_address};

/// Every match of one query in sort order; built once, paged many times.
pub struct Listing {
    searcher: Searcher,
    fields: Fields,
    entries: Vec<DocAddress>,
    warnings: Vec<String>,
}

impl std::fmt::Debug for Listing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listing")
            .field("len", &self.entries.len())
            .field("warnings", &self.warnings)
            .finish_non_exhaustive()
    }
}

impl Listing {
    /// Number of matching messages.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Query-syntax problems the lenient parser skipped over.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The stored fields of rows `start..start + count`, clamped to the end;
    /// empty when `start` is past the end.
    pub fn rows(&self, start: usize, count: usize) -> Result<Vec<Hit>> {
        let end = start.saturating_add(count).min(self.entries.len());
        if start >= end {
            return Ok(Vec::new());
        }
        self.entries[start..end]
            .iter()
            .map(|&address| hit_from_address(&self.fields, &self.searcher, address, None))
            .collect()
    }

    /// Row index of the message with this D-03 `offset`, when it is in the listing.
    pub fn position_of(&self, offset: u64) -> Result<Option<usize>> {
        let query = TermQuery::new(
            Term::from_field_u64(self.fields.offset, offset),
            IndexRecordOption::Basic,
        );
        let addresses = self.searcher.search(&query, &AllMatches::unsorted())?;
        Ok(addresses
            .first()
            .and_then(|target| self.entries.iter().position(|a| a == target)))
    }
}

impl MboxIndex {
    /// Runs `query` and keeps every match in `query.sort` order; `limit` and
    /// `offset` are ignored, the caller pages with [`Listing::rows`].
    pub fn list(&self, query: &SearchQuery) -> Result<Listing> {
        let (tantivy_query, warnings) = self.to_tantivy_query(query);
        let searcher = self.searcher();
        let entries = searcher.search(&*tantivy_query, &AllMatches { sort: query.sort })?;
        Ok(Listing {
            searcher,
            fields: *self.fields(),
            entries,
            warnings,
        })
    }
}

/// Collects every matching document with its sort key.
struct AllMatches {
    sort: Sort,
}

impl AllMatches {
    /// Key-less collection, for a lookup that expects at most one document.
    fn unsorted() -> Self {
        Self {
            sort: Sort::DateDesc,
        }
    }
}

struct SegmentMatches {
    ordinal: SegmentOrdinal,
    /// Absent when no document of the segment has a date.
    date: Option<Column<i64>>,
    by_score: bool,
    entries: Vec<(f64, DocId)>,
}

impl Collector for AllMatches {
    type Fruit = Vec<DocAddress>;
    type Child = SegmentMatches;

    fn for_segment(
        &self,
        ordinal: SegmentOrdinal,
        segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        let by_score = self.sort == Sort::Relevance;
        let date = if by_score {
            None
        } else {
            segment.fast_fields().column_opt::<i64>("date")?
        };
        Ok(SegmentMatches {
            ordinal,
            date,
            by_score,
            entries: Vec::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        self.sort == Sort::Relevance
    }

    fn merge_fruits(
        &self,
        segment_fruits: Vec<Vec<(f64, DocAddress)>>,
    ) -> tantivy::Result<Self::Fruit> {
        let mut all: Vec<(f64, DocAddress)> = segment_fruits.into_iter().flatten().collect();
        // Highest key first; ties (same second, equal scores) fall back to the
        // document order, which is the order of the messages in the mbox.
        all.sort_unstable_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.1.cmp(&b.1))
        });
        Ok(all.into_iter().map(|(_, address)| address).collect())
    }
}

impl SegmentCollector for SegmentMatches {
    type Fruit = Vec<(f64, DocAddress)>;

    fn collect(&mut self, doc: DocId, score: Score) {
        let key = if self.by_score {
            f64::from(score)
        } else {
            self.date
                .as_ref()
                .and_then(|column| column.first(doc))
                .map_or(f64::NEG_INFINITY, |date| date as f64)
        };
        self.entries.push((key, doc));
    }

    fn harvest(self) -> Self::Fruit {
        let ordinal = self.ordinal;
        self.entries
            .into_iter()
            .map(|(key, doc)| (key, DocAddress::new(ordinal, doc)))
            .collect()
    }
}
