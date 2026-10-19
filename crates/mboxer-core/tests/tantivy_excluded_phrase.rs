//! The tantivy behaviour behind the `Excluded` wrapper in
//! `src/index/free_text.rs`, against tantivy alone (no mboxer code), so it can
//! go upstream as it is: a `BooleanQuery` with a `MustNot` phrase panics in a
//! debug build when the phrase's first match lies after the first candidate of
//! the `Must` side. `PhraseScorer` moves to its first match when it is
//! created, `Exclude` then asks it with `seek_danger` about the earlier
//! candidate, and `PhraseScorer::seek_danger` debug-asserts that the target is
//! not behind it. Reported upstream as quickwit-oss/tantivy#3193 (task 063).
//!
//! The test expects the panic. Once a tantivy release fixes the assertion it
//! fails: that is the signal to drop the `Excluded` wrapper and this file.
//! Release builds skip the assertion and return the right count.

use tantivy::collector::Count;
use tantivy::query::{BooleanQuery, Occur, PhraseQuery, Query, TermQuery};
use tantivy::schema::{IndexRecordOption, Schema, TEXT};
use tantivy::{Index, IndexWriter, Term, doc};

fn count_without_the_phrase() -> tantivy::Result<usize> {
    let mut schema = Schema::builder();
    let text = schema.add_text_field("text", TEXT);
    let index = Index::create_in_ram(schema.build());
    let mut writer: IndexWriter = index.writer_with_num_threads(1, 15_000_000)?;
    // Doc 0 is a candidate of the `Must` side; doc 1 is the phrase's first match.
    writer.add_document(doc!(text => "invoice for march"))?;
    writer.add_document(doc!(text => "weekly spam report invoice"))?;
    writer.commit()?;

    let term = |word: &str| Term::from_field_text(text, word);
    let query = BooleanQuery::new(vec![
        (
            Occur::Must,
            Box::new(TermQuery::new(term("invoice"), IndexRecordOption::Basic)) as Box<dyn Query>,
        ),
        (
            Occur::MustNot,
            Box::new(PhraseQuery::new(vec![term("spam"), term("report")])),
        ),
    ]);
    let searcher = index.reader()?.searcher();
    searcher.search(&query, &Count)
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "target (0) should be greater than or equal to doc (1)")]
fn an_excluded_phrase_trips_the_seek_danger_assertion_in_a_debug_build() {
    let _ = count_without_the_phrase();
}

#[cfg(not(debug_assertions))]
#[test]
fn an_excluded_phrase_counts_right_in_a_release_build() {
    assert_eq!(count_without_the_phrase().unwrap(), 1);
}
