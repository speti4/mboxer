//! The sidecar full-text index: one tantivy index per mbox, stored next to it
//! (or under the index home when the mbox folder is not writable).
//!
//! Layout and freshness live in [`fingerprint`], the field set and the
//! accent-folding tokenizer in [`schema`], the one-pass build in [`build`] and
//! the query side in [`search`], with the free-text grammar in `free_text` and the completion of the word
//! being typed in `prefix`. The index is a cache of the mbox: when the
//! mbox changes (decision D-004) the whole index is rebuilt, never patched.

pub mod build;
pub mod fingerprint;
mod free_text;
pub mod listing;
mod prefix;
pub mod schema;
pub mod search;

pub use build::{BuildOptions, BuildReport, Progress, build};
pub use fingerprint::{
    FORMAT_VERSION, Fingerprint, INDEX_HOME_ENV, IndexMeta, IndexStatus, META_FILE, StaleReason,
    default_index_home, fallback_dir, fingerprint, index_dir, status, status_in,
};
pub use listing::{ListRow, Listing, RowKind};
pub use schema::TOKENIZER;
pub use search::{Hit, MboxIndex, Opened, SearchQuery, SearchResult, Sort};
