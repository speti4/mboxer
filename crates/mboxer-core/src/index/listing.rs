//! A full, sorted result set for a virtualized list, optionally grouped by
//! Gmail thread.
//!
//! [`MboxIndex::search`] serves one page per call, which is right for a CLI but
//! wrong for a list the user drags through: tantivy's `TopDocs` with an offset
//! of 80 000 re-collects a heap of that size on every page. [`Listing`] runs
//! the query once, keeps every matching document's address in sort order
//! (8 bytes each: under a megabyte for a 100 000-message archive) and fetches
//! the stored fields of any slice on demand.
//!
//! With [`SearchQuery::group_threads`] the matches are arranged into groups
//! (one per `X-GM-THRID`, a message without one is its own group) and the
//! list shows one row per group — the newest member stands for it — until the
//! user expands a group, which inserts its members, oldest first, under that
//! row. Groups are stored in a CSR layout (`members` in row order, `starts`
//! per group) and the expanded set is a small sorted vector with prefix sums,
//! so a row index maps to its group in `O(log expanded)`.

use std::cmp::Ordering;
use std::num::NonZeroU64;
use std::ops::Range;

use tantivy::collector::{Collector, SegmentCollector};
use tantivy::columnar::Column;
use tantivy::query::TermQuery;
use tantivy::schema::IndexRecordOption;
use tantivy::{DocAddress, DocId, Score, Searcher, SegmentOrdinal, SegmentReader, Term};

use crate::error::Result;
use crate::index::schema::Fields;
use crate::index::search::{Hit, MboxIndex, SearchQuery, Sort, hit_from_address};

/// One row of the list: the message's stored fields and what the row stands for.
#[derive(Debug, Clone, PartialEq)]
pub struct ListRow {
    pub hit: Hit,
    pub kind: RowKind,
}

/// What a row of a listing stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A group of one message (with or without a thread id).
    Message,
    /// A collapsed or expanded conversation; the hit is its newest member and
    /// `count` is the number of matching members.
    Thread { count: u32, expanded: bool },
    /// A member of an expanded conversation, `index` 0 being the oldest; the
    /// thread row sits at `row - index - 1`.
    Member { index: u32, count: u32 },
}

/// Every match of one query in sort order; built once, paged many times.
pub struct Listing {
    searcher: Searcher,
    fields: Fields,
    warnings: Vec<String>,
    /// Document addresses in row order: group by group, oldest member first.
    members: Vec<DocAddress>,
    layout: Layout,
}

impl std::fmt::Debug for Listing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listing")
            .field("len", &self.len())
            .field("messages", &self.members.len())
            .field("expanded", &self.layout.expanded.len())
            .field("warnings", &self.warnings)
            .finish_non_exhaustive()
    }
}

impl Listing {
    /// Number of visible rows: every group plus the members of the expanded ones.
    pub fn len(&self) -> usize {
        self.layout.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Number of matching messages, whatever is expanded.
    pub fn message_count(&self) -> usize {
        self.members.len()
    }

    /// Number of groups: conversations plus single messages. Equals
    /// [`message_count`](Self::message_count) when threads are not grouped.
    pub fn group_count(&self) -> usize {
        self.layout.group_count()
    }

    /// Problems with the query text for the viewer's status bar; none are
    /// reported at present (see [`SearchResult::warnings`](crate::SearchResult::warnings)).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// The rows `start..start + count`, clamped to the end; empty when `start`
    /// is past the end.
    pub fn rows(&self, start: usize, count: usize) -> Result<Vec<ListRow>> {
        let end = start.saturating_add(count).min(self.len());
        if start >= end {
            return Ok(Vec::new());
        }
        (start..end)
            .map(|row| {
                let slot = self.layout.locate(row).expect("row within len");
                let (address, kind) = self.describe(slot);
                let hit = hit_from_address(&self.fields, &self.searcher, address, None)?;
                Ok(ListRow { hit, kind })
            })
            .collect()
    }

    /// Expands or collapses the conversation whose thread row is `row`; the
    /// new [`len`](Self::len), or `None` when the row is not a thread row.
    pub fn toggle(&mut self, row: usize) -> Option<usize> {
        match self.layout.locate(row)? {
            Slot::Group(group) if self.layout.group_range(group).len() > 1 => {
                self.layout.toggle(group);
                Some(self.len())
            }
            _ => None,
        }
    }

    /// Row index of the message with this D-003 `offset`, when it is in the
    /// listing. A member hidden inside a collapsed conversation gets the
    /// conversation expanded, so the row exists afterwards; the newest member
    /// of a collapsed conversation is the thread row itself.
    pub fn position_of(&mut self, offset: u64) -> Result<Option<usize>> {
        let query = TermQuery::new(
            Term::from_field_u64(self.fields.offset, offset),
            IndexRecordOption::Basic,
        );
        let found = self.searcher.search(&query, &AllMatches::unsorted())?;
        let Some(target) = found.first().map(|entry| entry.address) else {
            return Ok(None);
        };
        let Some(member) = self.members.iter().position(|a| *a == target) else {
            return Ok(None);
        };
        Ok(Some(self.layout.row_of_member(member)))
    }

    fn describe(&self, slot: Slot) -> (DocAddress, RowKind) {
        match slot {
            Slot::Group(group) => {
                let range = self.layout.group_range(group);
                let count = range.len() as u32;
                let address = self.members[range.end - 1];
                let kind = if count == 1 {
                    RowKind::Message
                } else {
                    RowKind::Thread {
                        count,
                        expanded: self.layout.is_expanded(group),
                    }
                };
                (address, kind)
            }
            Slot::Member { group, index } => {
                let range = self.layout.group_range(group);
                let kind = RowKind::Member {
                    index: index as u32,
                    count: range.len() as u32,
                };
                (self.members[range.start + index], kind)
            }
        }
    }
}

impl MboxIndex {
    /// Runs `query` and keeps every match in `query.sort` order, grouped by
    /// thread when `query.group_threads` asks for it; `limit` and `offset` are
    /// ignored, the caller pages with [`Listing::rows`].
    pub fn list(&self, query: &SearchQuery) -> Result<Listing> {
        let tantivy_query = self.to_tantivy_query(query);
        let searcher = self.searcher();
        let collector = AllMatches {
            sort: query.sort,
            group: query.group_threads,
        };
        let entries = searcher.search(&*tantivy_query, &collector)?;
        let (members, starts) = arrange(entries, query.group_threads);
        Ok(Listing {
            searcher,
            fields: *self.fields(),
            warnings: Vec::new(),
            layout: Layout::new(members.len(), starts),
            members,
        })
    }
}

/// The group structure and the expanded set; the row arithmetic lives here
/// so it can be tested without an index.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Layout {
    /// Matching messages (the members).
    messages: usize,
    /// `starts[g]..starts[g + 1]` are group `g`'s members; `None` when every
    /// message is its own group.
    starts: Option<Vec<u32>>,
    /// The expanded groups, by group index.
    expanded: Vec<Expanded>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Expanded {
    group: u32,
    /// Member count of the group.
    len: u32,
    /// Members of the expanded groups before this one (prefix sum).
    rows_before: u32,
}

/// What a row index points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    /// The row of group `g`: a message, or a thread row.
    Group(usize),
    /// Member `index` (oldest first) of an expanded group.
    Member { group: usize, index: usize },
}

impl Layout {
    fn new(messages: usize, starts: Option<Vec<u32>>) -> Self {
        Self {
            messages,
            starts,
            expanded: Vec::new(),
        }
    }

    fn group_count(&self) -> usize {
        self.starts
            .as_ref()
            .map_or(self.messages, |starts| starts.len() - 1)
    }

    fn group_range(&self, group: usize) -> Range<usize> {
        match &self.starts {
            Some(starts) => starts[group] as usize..starts[group + 1] as usize,
            None => group..group + 1,
        }
    }

    /// Rows that the first `count` expanded groups add.
    fn extra_before(&self, count: usize) -> usize {
        if count == 0 {
            0
        } else {
            let last = self.expanded[count - 1];
            (last.rows_before + last.len) as usize
        }
    }

    fn len(&self) -> usize {
        self.group_count() + self.extra_before(self.expanded.len())
    }

    fn is_expanded(&self, group: usize) -> bool {
        self.expanded
            .binary_search_by_key(&(group as u32), |e| e.group)
            .is_ok()
    }

    /// The thread row of an expanded group.
    fn first_row(entry: Expanded) -> usize {
        (entry.group + entry.rows_before) as usize
    }

    fn row_of_group(&self, group: usize) -> usize {
        let before = self
            .expanded
            .partition_point(|e| (e.group as usize) < group);
        group + self.extra_before(before)
    }

    fn locate(&self, row: usize) -> Option<Slot> {
        let i = self
            .expanded
            .partition_point(|e| Self::first_row(*e) <= row);
        if i > 0 {
            let entry = self.expanded[i - 1];
            let first = Self::first_row(entry);
            if row < first + 1 + entry.len as usize {
                return Some(if row == first {
                    Slot::Group(entry.group as usize)
                } else {
                    Slot::Member {
                        group: entry.group as usize,
                        index: row - first - 1,
                    }
                });
            }
        }
        let group = row - self.extra_before(i);
        (group < self.group_count()).then_some(Slot::Group(group))
    }

    /// The group of member `member` (an index into the members).
    fn group_of_member(&self, member: usize) -> usize {
        match &self.starts {
            Some(starts) => starts.partition_point(|&start| start as usize <= member) - 1,
            None => member,
        }
    }

    /// The row that shows member `member`, expanding its group when the
    /// member is hidden inside a collapsed conversation.
    fn row_of_member(&mut self, member: usize) -> usize {
        let group = self.group_of_member(member);
        let range = self.group_range(group);
        if range.len() == 1 {
            return self.row_of_group(group);
        }
        let newest = member == range.end - 1;
        if !self.is_expanded(group) {
            if newest {
                return self.row_of_group(group);
            }
            self.toggle(group);
        }
        self.row_of_group(group) + 1 + (member - range.start)
    }

    fn toggle(&mut self, group: usize) {
        let len = self.group_range(group).len() as u32;
        match self
            .expanded
            .binary_search_by_key(&(group as u32), |e| e.group)
        {
            Ok(i) => {
                self.expanded.remove(i);
            }
            Err(i) => self.expanded.insert(
                i,
                Expanded {
                    group: group as u32,
                    len,
                    rows_before: 0,
                },
            ),
        }
        let mut sum = 0;
        for entry in &mut self.expanded {
            entry.rows_before = sum;
            sum += entry.len;
        }
    }
}

/// One matching document with its sort key, as the collector sees it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Entry {
    /// The date as f64, or the score (today's sort key).
    key: f64,
    /// `i64::MIN` when the message has no date; orders members in a group.
    date: i64,
    thread: Option<NonZeroU64>,
    address: DocAddress,
}

/// The span of one group inside the thread-sorted entries.
#[derive(Debug, Clone, Copy)]
struct GroupKey {
    start: u32,
    len: u32,
    /// The newest member's key.
    key: f64,
    /// The newest member's address breaks key ties.
    tie: DocAddress,
}

/// Highest key first.
fn key_desc(a: f64, b: f64) -> Ordering {
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

/// Sorts the entries into row order: ungrouped, every message in key order
/// (ties in document order, which is the order in the mbox); grouped, the
/// groups in the order of their newest member and the members of each group
/// oldest first. Returns the member addresses and, when grouped, the group
/// starts (`groups + 1` entries).
fn arrange(mut entries: Vec<Entry>, group: bool) -> (Vec<DocAddress>, Option<Vec<u32>>) {
    if !group {
        entries.sort_unstable_by(|a, b| {
            key_desc(a.key, b.key).then_with(|| a.address.cmp(&b.address))
        });
        return (entries.into_iter().map(|e| e.address).collect(), None);
    }
    entries.sort_unstable_by(|a, b| {
        a.thread
            .cmp(&b.thread)
            .then_with(|| a.date.cmp(&b.date))
            .then_with(|| a.address.cmp(&b.address))
    });
    let mut groups: Vec<GroupKey> = Vec::new();
    let mut i = 0;
    while i < entries.len() {
        let start = i;
        let thread = entries[i].thread;
        let mut key = entries[i].key;
        i += 1;
        if thread.is_some() {
            while i < entries.len() && entries[i].thread == thread {
                key = key.max(entries[i].key);
                i += 1;
            }
        }
        groups.push(GroupKey {
            start: start as u32,
            len: (i - start) as u32,
            key,
            tie: entries[i - 1].address,
        });
    }
    groups.sort_unstable_by(|a, b| key_desc(a.key, b.key).then_with(|| a.tie.cmp(&b.tie)));
    let mut members = Vec::with_capacity(entries.len());
    let mut starts = Vec::with_capacity(groups.len() + 1);
    for group in &groups {
        starts.push(members.len() as u32);
        let span = group.start as usize..(group.start + group.len) as usize;
        members.extend(entries[span].iter().map(|e| e.address));
    }
    starts.push(members.len() as u32);
    (members, Some(starts))
}

/// Collects every matching document with the columns the arrangement needs.
struct AllMatches {
    sort: Sort,
    group: bool,
}

impl AllMatches {
    /// Key-less collection, for a lookup that expects at most one document.
    fn unsorted() -> Self {
        Self {
            sort: Sort::DateDesc,
            group: false,
        }
    }
}

struct SegmentMatches {
    ordinal: SegmentOrdinal,
    /// Absent when no document of the segment has a date, or when not needed.
    date: Option<Column<i64>>,
    /// Absent when not grouping, or when no document of the segment has a thread id.
    thread: Option<Column<u64>>,
    by_score: bool,
    entries: Vec<Entry>,
}

impl Collector for AllMatches {
    type Fruit = Vec<Entry>;
    type Child = SegmentMatches;

    fn for_segment(
        &self,
        ordinal: SegmentOrdinal,
        segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        let by_score = self.sort == Sort::Relevance;
        let date = if by_score && !self.group {
            None
        } else {
            segment.fast_fields().column_opt::<i64>("date")?
        };
        let thread = if self.group {
            segment.fast_fields().column_opt::<u64>("thread_id")?
        } else {
            None
        };
        Ok(SegmentMatches {
            ordinal,
            date,
            thread,
            by_score,
            entries: Vec::new(),
        })
    }

    fn requires_scoring(&self) -> bool {
        self.sort == Sort::Relevance
    }

    fn merge_fruits(&self, segment_fruits: Vec<Vec<Entry>>) -> tantivy::Result<Self::Fruit> {
        Ok(segment_fruits.into_iter().flatten().collect())
    }
}

impl SegmentCollector for SegmentMatches {
    type Fruit = Vec<Entry>;

    fn collect(&mut self, doc: DocId, score: Score) {
        let date = self.date.as_ref().and_then(|column| column.first(doc));
        let key = if self.by_score {
            f64::from(score)
        } else {
            date.map_or(f64::NEG_INFINITY, |date| date as f64)
        };
        let thread = self
            .thread
            .as_ref()
            .and_then(|column| column.first(doc))
            .and_then(NonZeroU64::new);
        self.entries.push(Entry {
            key,
            date: date.unwrap_or(i64::MIN),
            thread,
            address: DocAddress::new(self.ordinal, doc),
        });
    }

    fn harvest(self) -> Self::Fruit {
        self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(key: f64, thread: u64, doc: u32) -> Entry {
        Entry {
            key,
            date: key as i64,
            thread: NonZeroU64::new(thread),
            address: DocAddress::new(0, doc),
        }
    }

    fn docs(addresses: &[DocAddress]) -> Vec<u32> {
        addresses.iter().map(|a| a.doc_id).collect()
    }

    #[test]
    fn ungrouped_order_is_key_desc_then_document_order() {
        let entries = vec![
            entry(10.0, 0, 0),
            entry(30.0, 0, 1),
            entry(f64::NEG_INFINITY, 0, 2),
            entry(30.0, 0, 3),
        ];
        let (members, starts) = arrange(entries, false);
        assert_eq!(docs(&members), [1, 3, 0, 2]);
        assert_eq!(starts, None);
    }

    #[test]
    fn singletons_grouped_keep_the_ungrouped_order() {
        let entries = vec![entry(10.0, 7, 0), entry(30.0, 0, 1), entry(20.0, 9, 2)];
        let (members, starts) = arrange(entries.clone(), true);
        let (plain, _) = arrange(entries, false);
        assert_eq!(members, plain);
        assert_eq!(starts, Some(vec![0, 1, 2, 3]));
    }

    #[test]
    fn groups_follow_their_newest_member_and_hold_members_oldest_first() {
        // Thread 1: docs 0 (10), 2 (12), 5 (20). Thread 2: docs 1 (11), 4 (14),
        // 6 (9). Doc 3 has no thread (13). Groups by newest: 20, 14, 13.
        let entries = vec![
            entry(10.0, 1, 0),
            entry(11.0, 2, 1),
            entry(12.0, 1, 2),
            entry(13.0, 0, 3),
            entry(14.0, 2, 4),
            entry(20.0, 1, 5),
            entry(9.0, 2, 6),
        ];
        let (members, starts) = arrange(entries, true);
        assert_eq!(docs(&members), [0, 2, 5, 6, 1, 4, 3]);
        assert_eq!(starts, Some(vec![0, 3, 6, 7]));
    }

    fn layout_of_sizes(sizes: &[u32]) -> Layout {
        let mut starts = vec![0];
        for size in sizes {
            starts.push(starts.last().unwrap() + size);
        }
        let messages = *starts.last().unwrap() as usize;
        Layout::new(messages, Some(starts))
    }

    #[test]
    fn rows_map_onto_groups_and_members_through_the_expanded_set() {
        let sizes: Vec<u32> = (0..1000u32).map(|g| g % 4 + 1).collect();
        let mut layout = layout_of_sizes(&sizes);
        assert_eq!(layout.len(), 1000);
        assert_eq!(layout.locate(999), Some(Slot::Group(999)));
        assert_eq!(layout.locate(1000), None);

        for group in [3usize, 501, 999] {
            assert!(sizes[group] > 1);
            layout.toggle(group);
        }
        let extra: u32 = sizes[3] + sizes[501] + sizes[999];
        assert_eq!(layout.len(), 1000 + extra as usize);

        // Every group's row round-trips, and so does every member row.
        let mut row = 0;
        for (group, &size) in sizes.iter().enumerate() {
            assert_eq!(layout.row_of_group(group), row);
            assert_eq!(layout.locate(row), Some(Slot::Group(group)));
            row += 1;
            if layout.is_expanded(group) {
                for index in 0..size as usize {
                    assert_eq!(layout.locate(row), Some(Slot::Member { group, index }));
                    row += 1;
                }
            }
        }
        assert_eq!(row, layout.len());
        assert_eq!(layout.locate(row), None);

        // Collapsing the middle one fixes the prefix sums.
        layout.toggle(501);
        assert_eq!(layout.len(), 1000 + (sizes[3] + sizes[999]) as usize);
        assert_eq!(layout.row_of_group(999), 999 + sizes[3] as usize);
        assert_eq!(
            layout.locate(layout.len() - 1),
            Some(Slot::Member {
                group: 999,
                index: sizes[999] as usize - 1
            })
        );
    }

    #[test]
    fn a_member_row_expands_its_group_unless_it_is_the_newest() {
        let mut layout = layout_of_sizes(&[3, 1, 2]);
        // Group 0 = members 0..3, group 1 = member 3, group 2 = members 4..6.
        assert_eq!(
            layout.row_of_member(2),
            0,
            "the newest stays the thread row"
        );
        assert_eq!(layout.len(), 3);
        assert_eq!(layout.row_of_member(3), 1);
        assert_eq!(layout.row_of_member(0), 1, "the oldest expands group 0");
        assert_eq!(layout.len(), 6);
        assert_eq!(
            layout.row_of_member(2),
            3,
            "expanded: the newest is a member row"
        );
        assert_eq!(layout.row_of_member(3), 4);
        assert_eq!(layout.row_of_member(4), 6);
        assert_eq!(layout.len(), 8);
    }

    #[test]
    fn an_ungrouped_layout_has_one_row_per_message() {
        let mut layout = Layout::new(4, None);
        assert_eq!(layout.len(), 4);
        assert_eq!(layout.locate(2), Some(Slot::Group(2)));
        assert_eq!(layout.group_range(2), 2..3);
        assert_eq!(layout.row_of_member(3), 3);
        assert_eq!(layout.len(), 4);
    }
}
