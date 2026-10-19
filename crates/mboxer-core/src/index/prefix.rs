//! The word still being typed, completed to the terms it most likely means.
//!
//! tantivy's `PhrasePrefixQuery` expands a prefix to the first N terms of the
//! dictionary in byte order. Over a large archive a short prefix spends those
//! N on one-off tokens (hex ids, base64 runs, tracking codes) that sort ahead
//! of the word the user means, and the word itself is never searched.
//! [`RankedPrefixQuery`] reads the whole prefix range of the dictionary once
//! per search instead, sums each term's document frequency over the
//! segments, and keeps the N most frequent, the shorter term first on a tie
//! (task 060).
//!
//! The scan touches every term of the range, and only the terms that enter a
//! segment's best N are copied: a million distinct terms under one prefix
//! cost about 28 ms per search in a release build (task 060), against 0.1 ms
//! for the byte-order expansion that missed the word.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use tantivy::query::{
    BooleanQuery, EnableScoring, Occur, PhrasePrefixQuery, PhraseQuery, Query, TermSetQuery, Weight,
};
use tantivy::schema::Field;
use tantivy::{Searcher, Term};

/// A prefix query, optionally after a phrase, that expands its last term to
/// the `max_expansions` most frequent completions.
#[derive(Debug, Clone)]
pub(crate) struct RankedPrefixQuery {
    field: Field,
    /// The words before the prefix, with their positions.
    phrase: Vec<(usize, Term)>,
    prefix: (usize, Term),
    max_expansions: usize,
}

impl RankedPrefixQuery {
    /// `terms` in position order, all of one field, the last one the prefix.
    pub(crate) fn new(mut terms: Vec<(usize, Term)>, max_expansions: usize) -> Self {
        let prefix = terms.pop().expect("a prefix query has a prefix term");
        Self {
            field: prefix.1.field(),
            phrase: terms,
            prefix,
            max_expansions,
        }
    }

    /// The completions of the prefix, most frequent first. Each segment keeps
    /// its own best `max_expansions` in a bounded heap, so only those terms
    /// are copied; the segments' picks are then summed and ranked again. A
    /// term that is frequent only across segments can lose to one that is
    /// frequent in one: the index is built in one pass (D-004), so the
    /// segments are large and that stays rare.
    fn completions(&self, searcher: &Searcher) -> tantivy::Result<Vec<Term>> {
        let start = self.prefix.1.serialized_value_bytes();
        let end = prefix_end(start);
        let mut doc_freqs: HashMap<Vec<u8>, u64> = HashMap::new();
        for segment in searcher.segment_readers() {
            let inverted = segment.inverted_index(self.field)?;
            let mut range = inverted.terms().range().ge(start);
            if let Some(end) = &end {
                range = range.lt(end);
            }
            let mut stream = range.into_stream()?;
            let mut best: BinaryHeap<Candidate> = BinaryHeap::new();
            while let Some((key, info)) = stream.next() {
                let freq = u64::from(info.doc_freq);
                if best.len() < self.max_expansions {
                    best.push(Candidate::new(freq, key));
                } else if best
                    .peek()
                    .is_some_and(|worst| rank(freq, key, worst.freq, &worst.key).is_lt())
                {
                    best.pop();
                    best.push(Candidate::new(freq, key));
                }
            }
            for candidate in best {
                *doc_freqs.entry(candidate.key).or_insert(0) += candidate.freq;
            }
        }
        let mut ranked: Vec<(Vec<u8>, u64)> = doc_freqs.into_iter().collect();
        ranked.sort_unstable_by(|(a, a_freq), (b, b_freq)| rank(*a_freq, a, *b_freq, b));
        ranked.truncate(self.max_expansions);
        Ok(ranked
            .into_iter()
            .filter_map(|(key, _)| {
                std::str::from_utf8(&key)
                    .ok()
                    .map(|text| Term::from_field_text(self.field, text))
            })
            .collect())
    }

    /// The query of the chosen completions: a term set for a lone prefix, one
    /// phrase per completion after words.
    fn expanded(&self, completions: Vec<Term>) -> Box<dyn Query> {
        if self.phrase.is_empty() {
            return Box::new(TermSetQuery::new(completions));
        }
        let position = self.prefix.0;
        let phrases = completions
            .into_iter()
            .map(|completion| {
                let mut terms = self.phrase.clone();
                terms.push((position, completion));
                let query: Box<dyn Query> = Box::new(PhraseQuery::new_with_offset(terms));
                (Occur::Should, query)
            })
            .collect();
        Box::new(BooleanQuery::new(phrases))
    }
}

impl Query for RankedPrefixQuery {
    fn weight(&self, enable_scoring: EnableScoring<'_>) -> tantivy::Result<Box<dyn Weight>> {
        let Some(searcher) = enable_scoring.searcher() else {
            // No dictionary to rank: tantivy's own expansion, in byte order.
            let mut terms = self.phrase.clone();
            terms.push(self.prefix.clone());
            let mut query = PhrasePrefixQuery::new_with_offset(terms);
            query.set_max_expansions(self.max_expansions as u32);
            return query.weight(enable_scoring);
        };
        let completions = self.completions(searcher)?;
        self.expanded(completions).weight(enable_scoring)
    }

    fn query_terms<'a>(&'a self, visitor: &mut dyn FnMut(&'a Term, bool)) {
        for (_, term) in &self.phrase {
            visitor(term, true);
        }
    }
}

/// The completion order: the more documents the better, then the shorter
/// term, then byte order. `Less` means `a` comes first.
fn rank(a_freq: u64, a: &[u8], b_freq: u64, b: &[u8]) -> Ordering {
    b_freq
        .cmp(&a_freq)
        .then_with(|| a.len().cmp(&b.len()))
        .then_with(|| a.cmp(b))
}

/// A term kept in a segment's heap; the heap's top is the worst one kept.
#[derive(PartialEq, Eq)]
struct Candidate {
    freq: u64,
    key: Vec<u8>,
}

impl Candidate {
    fn new(freq: u64, key: &[u8]) -> Self {
        Self {
            freq,
            key: key.to_vec(),
        }
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        rank(self.freq, &self.key, other.freq, &other.key)
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// The first byte string after every string that starts with `prefix`;
/// `None` when there is none (the prefix is all `0xff`).
fn prefix_end(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut end = prefix.to_vec();
    while let Some(last) = end.pop() {
        if last < u8::MAX {
            end.push(last + 1);
            return Some(end);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_prefix_range_ends_after_the_last_completion() {
        assert_eq!(prefix_end(b"ab"), Some(b"ac".to_vec()));
        assert_eq!(prefix_end(b"a\xff"), Some(b"b".to_vec()));
        assert_eq!(prefix_end(b"\xff\xff"), None);
        assert_eq!(prefix_end(b""), None);
    }
}
