//! The tantivy schema of the sidecar and its accent-folding tokenizer.

use tantivy::Index;
use tantivy::schema::{
    FAST, Field, INDEXED, IndexRecordOption, STORED, STRING, Schema, TextFieldIndexing, TextOptions,
};
use tantivy::tokenizer::{
    AsciiFoldingFilter, LowerCaser, SimpleTokenizer, TextAnalyzer, Token, TokenFilter, TokenStream,
    Tokenizer,
};

use crate::error::{Error, Result};

/// Name of the registered tokenizer: words, lower-cased, accents folded to ASCII.
pub const TOKENIZER: &str = "fold";

/// The longest token kept, in characters after folding: a longer one is
/// dropped (base64 runs, hashes, long URLs without punctuation). Counted
/// after the folding so an accented word and its ASCII spelling get the same
/// limit (task 060).
const MAX_FOLDED_TOKEN_CHARS: usize = 40;

/// Builds the analyzer behind [`TOKENIZER`]: `számla` and `SZAMLA` both become `szamla`.
pub fn fold_analyzer() -> TextAnalyzer {
    TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .filter(AsciiFoldingFilter)
        .filter(MaxCharsFilter(MAX_FOLDED_TOKEN_CHARS))
        .build()
}

/// Drops the tokens with more than `.0` characters. tantivy's own
/// `RemoveLongFilter` counts UTF-8 bytes and keeps only tokens shorter than
/// its limit, so it would cut a 20-letter accented word before the folding.
#[derive(Clone, Copy)]
struct MaxCharsFilter(usize);

impl TokenFilter for MaxCharsFilter {
    type Tokenizer<T: Tokenizer> = MaxCharsTokenizer<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> MaxCharsTokenizer<T> {
        MaxCharsTokenizer {
            max: self.0,
            inner: tokenizer,
        }
    }
}

#[derive(Clone)]
struct MaxCharsTokenizer<T> {
    max: usize,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for MaxCharsTokenizer<T> {
    type TokenStream<'a> = MaxCharsStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        MaxCharsStream {
            max: self.max,
            tail: self.inner.token_stream(text),
        }
    }
}

struct MaxCharsStream<S> {
    max: usize,
    tail: S,
}

impl<S: TokenStream> TokenStream for MaxCharsStream<S> {
    fn advance(&mut self) -> bool {
        while self.tail.advance() {
            // A token of at most `max` bytes is short enough without counting.
            let text = &self.tail.token().text;
            if text.len() <= self.max || text.chars().count() <= self.max {
                return true;
            }
        }
        false
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}

/// Registers [`TOKENIZER`] on `index`; needed after every create and open.
pub fn register_tokenizer(index: &Index) {
    index.tokenizers().register(TOKENIZER, fold_analyzer());
}

/// Handles of every field, resolved once per index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fields {
    /// Byte offset of the envelope line (decision D-003); the message identity.
    /// A fast field too: it breaks ties in the listing's sort (task 060).
    pub offset: Field,
    pub len: Field,
    /// `Date` header as Unix seconds; absent when the header is missing or unparsable.
    pub date: Field,
    pub thread_id: Field,
    pub message_id: Field,
    /// Display strings, one value per mailbox: `Name <address>`. The address
    /// is folded into words like the name, so `from:` and `to:` match either.
    pub from: Field,
    pub to: Field,
    pub cc: Field,
    pub subject: Field,
    /// Indexed only, never stored: the reader seeks the mbox for the text.
    pub body: Field,
    /// One value per Gmail label, exact (`raw`) terms.
    pub labels: Field,
    pub has_attachment: Field,
    pub attachment_names: Field,
}

const FIELD_NAMES: [&str; 13] = [
    "offset",
    "len",
    "date",
    "thread_id",
    "message_id",
    "from",
    "to",
    "cc",
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
        offset: b.add_u64_field("offset", INDEXED | STORED | FAST),
        len: b.add_u64_field("len", STORED),
        date: b.add_i64_field("date", STORED | FAST),
        thread_id: b.add_u64_field("thread_id", INDEXED | STORED | FAST),
        message_id: b.add_text_field("message_id", STRING | STORED),
        from: b.add_text_field("from", folded_text(true)),
        to: b.add_text_field("to", folded_text(true)),
        cc: b.add_text_field("cc", folded_text(true)),
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
        let mut resolved = [Field::from_field_id(0); FIELD_NAMES.len()];
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
    fn folds_letters_beyond_combining_accents() {
        // The words the highlight's Node tests fold the same way (`ui/query.js`).
        assert_eq!(
            tokens("Straße STRAẞE Łódź Ørsted Œuvre Đakovo đ Æsir Þór ﬁle"),
            [
                "strasse", "strasse", "lodz", "orsted", "oeuvre", "dakovo", "d", "aesir", "thor",
                "file"
            ]
        );
    }

    #[test]
    fn splits_addresses_and_drops_long_runs() {
        assert_eq!(
            tokens("anna.pelda@example.com"),
            ["anna", "pelda", "example", "com"]
        );
        let long = "a".repeat(MAX_FOLDED_TOKEN_CHARS + 1);
        assert!(tokens(&format!("x {long} y")).iter().all(|t| t != &long));
    }

    #[test]
    fn the_length_limit_counts_folded_characters() {
        // 40 ASCII characters is the longest word kept, 41 is a run.
        let forty = "b".repeat(40);
        assert_eq!(tokens(&format!("x {forty} y")), ["x", forty.as_str(), "y"]);
        let forty_one = "b".repeat(41);
        assert_eq!(tokens(&format!("x {forty_one} y")), ["x", "y"]);
        // 25 accented characters are 50 bytes before folding and 25 after.
        let accented = format!("{}á", "áé".repeat(12));
        assert_eq!(accented.chars().count(), 25);
        assert_eq!(tokens(&accented), [format!("{}a", "ae".repeat(12))]);
        // 33 characters, 47 bytes before folding.
        assert_eq!(
            tokens("Árvíztűrőtükörfúrógéptükörfúrógép"),
            ["arvizturotukorfurogeptukorfurogep"]
        );
        // 40 accented characters stay, 41 go.
        assert_eq!(tokens(&"ő".repeat(40)), ["o".repeat(40)]);
        assert!(tokens(&"ő".repeat(41)).is_empty());
    }

    #[test]
    fn schema_roundtrips_through_names() {
        let (schema, fields) = build_schema();
        let back = Fields::from_schema(&schema).unwrap();
        assert_eq!(back.body, fields.body);
        assert_eq!(back.attachment_names, fields.attachment_names);
    }
}
