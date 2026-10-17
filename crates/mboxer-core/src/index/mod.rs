//! The sidecar full-text index: one tantivy index per mbox, stored next to it.
//!
//! Layout and freshness live in [`fingerprint`], the field set and the
//! accent-folding tokenizer in [`schema`], the one-pass build in [`build`] and
//! the query side in [`search`]. The index is a cache of the mbox: when the
//! mbox changes (decision D-04) the whole index is rebuilt, never patched.

pub mod build;
pub mod fingerprint;
pub mod listing;
pub mod schema;
pub mod search;

pub use build::{BuildOptions, BuildReport, Progress, build};
pub use fingerprint::{
    FORMAT_VERSION, Fingerprint, IndexMeta, IndexStatus, META_FILE, StaleReason, fingerprint,
    index_dir, status,
};
pub use listing::{ListRow, Listing, RowKind};
pub use schema::TOKENIZER;
pub use search::{Hit, MboxIndex, Opened, SearchQuery, SearchResult, Sort};
