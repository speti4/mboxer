//! The tantivy schema of the sidecar and its accent-folding tokenizer.

use tantivy::Index;
use tantivy::schema::{
    FAST, Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions,
};
use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, RemoveLongFilter, SimpleTokenizer, TextAnalyzer,
};

use crate::error::{Error, Result};

/// Name of the registered tokenizer: words, lower-cased, accents folded to ASCII.
pub const TOKENIZER: &str = "fold";

/// Tokens longer than this are dropped (base64 runs, URLs, hashes).
const MAX_TOKEN_CHARS: usize = 40;

/// Builds the analyzer behind [`TOKENIZER`]: `számla` and `SZAMLA` both become `szamla`.
pub fn fold_analyzer() -> TextAnalyzer {
    TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(RemoveLongFilter::limit(MAX_TOKEN_CHARS))
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .build()
}

/// Registers [`TOKENIZER`] on `index`; needed after every create and open.
pub fn register_tokenizer(index: &Index) {
    index.tokenizers().register(TOKENIZER, fold_analyzer());
}

/// Handles of every field, resolved once per index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fields {
    /// Byte offset of the envelope line (decision D-003); the message identity.
    pub offset: Field,
    pub len: Field,
    /// `Date` header as Unix seconds; absent when the header is missing or unparsable.
    pub date: Field,
    pub thread_id: Field,
    pub message_id: Field,
    /// Display strings, one value per mailbox: `Name <address>`.
    pub from: Field,
    pub to: Field,
    pub cc: Field,
    /// Bare lower-cased addresses, written for exact filters and counts; no query reads them yet.
    pub from_addr: Field,
    pub to_addr: Field,
    pub subject: Field,
    /// Indexed only, never stored: the reader seeks the mbox for the text.
    pub body: Field,
    /// One value per Gmail label, exact (`raw`) terms.
    pub labels: Field,
    pub has_attachment: Field,
    pub attachment_names: Field,
}

const FIELD_NAMES: [&str; 15] = [
    "offset",
    "len",
    "date",
    "thread_id",
    "message_id",
    "from",
    "to",
    "cc",
    "from_addr",
    "to_addr",
    "subject",
    "body",
    "labels",
    "has_attachment",
    "attachment_names",
];

fn folded_text(stored: bool) -> TextOptions {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer(TOKENIZER)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let options = TextOptions::default().set_indexing_options(indexing);
    if stored {
        options.set_stored()
    } else {
        options
    }
}

/// Builds the schema and the field handles of a new index.
pub fn build_schema() -> (Schema, Fields) {
    let mut b = Schema::builder();
    let fields = Fields {
        offset: b.add_u64_field("offset", INDEXED | STORED),
        len: b.add_u64_field("len", STORED),
        date: b.add_i64_field("date", STORED | FAST),
        thread_id: b.add_u64_field("thread_id", INDEXED | STORED | FAST),
        message_id: b.add_text_field("message_id", STRING | STORED),
        from: b.add_text_field("from", folded_text(true)),
        to: b.add_text_field("to", folded_text(true)),
        cc: b.add_text_field("cc", folded_text(true)),
        from_addr: b.add_text_field("from_addr", STRING | FAST),
        to_addr: b.add_text_field("to_addr", STRING | FAST),
        subject: b.add_text_field("subject", folded_text(true)),
        body: b.add_text_field("body", folded_text(false)),
        labels: b.add_text_field("labels", STRING | STORED | FAST),
        has_attachment: b.add_bool_field("has_attachment", INDEXED | STORED | FAST),
        attachment_names: b.add_text_field("attachment_names", folded_text(true)),
    };
    (b.build(), fields)
}

impl Fields {
    /// Resolves the handles from an opened index's schema.
    pub fn from_schema(schema: &Schema) -> Result<Self> {
        let mut resolved = [Field::from_field_id(0); 15];
        for (slot, name) in resolved.iter_mut().zip(FIELD_NAMES) {
            *slot = schema
                .get_field(name)
                .map_err(|_| Error::IndexMeta(format!("index schema has no `{name}` field")))?;
        }
        let [
            offset,
            len,
            date,
            thread_id,
            message_id,
            from,
            to,
            cc,
            from_addr,
            to_addr,
            subject,
            body,
            labels,
            has_attachment,
            attachment_names,
        ] = resolved;
        Ok(Fields {
            offset,
            len,
            date,
            thread_id,
            message_id,
            from,
            to,
            cc,
            from_addr,
            to_addr,
            subject,
            body,
            labels,
            has_attachment,
            attachment_names,
        })
    }

    /// Fields a free-text query searches when it names none.
    pub fn default_search_fields(&self) -> Vec<Field> {
        vec![
            self.subject,
            self.body,
            self.from,
            self.to,
            self.cc,
            self.attachment_names,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> Vec<String> {
        let mut analyzer = fold_analyzer();
        let mut stream = analyzer.token_stream(text);
        let mut out = Vec::new();
        while let Some(token) = stream.next() {
            out.push(token.text.clone());
        }
        out
    }

    #[test]
    fn folds_case_and_hungarian_accents() {
        assert_eq!(
            tokens("Számla ÁRVÍZTŰRŐ tükörfúrógép, Őrült Űrhajó"),
            ["szamla", "arvizturo", "tukorfurogep", "orult", "urhajo"]
        );
    }

    #[test]
    fn splits_addresses_and_drops_long_runs() {
        assert_eq!(
            tokens("anna.pelda@example.com"),
            ["anna", "pelda", "example", "com"]
        );
        let long = "a".repeat(MAX_TOKEN_CHARS + 1);
        assert!(tokens(&format!("x {long} y")).iter().all(|t| t != &long));
    }

    #[test]
    fn schema_roundtrips_through_names() {
        let (schema, fields) = build_schema();
        let back = Fields::from_schema(&schema).unwrap();
        assert_eq!(back.body, fields.body);
        assert_eq!(back.attachment_names, fields.attachment_names);
    }
}
