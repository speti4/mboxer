//! mboxer-core: a streaming mbox reader and the message model built on it.
//!
//! The reader walks an mbox of any size with a bounded buffer and hands out one
//! [`RawMessage`] at a time, each tagged with the byte offset and length that
//! identify it for the rest of the application (decision D-03). Parsing of
//! headers and MIME parts is delegated to `mail-parser`; [`MessageSummary`] is
//! the owned, index-friendly digest of one message. [`index`] builds and queries
//! the sidecar full-text index that stores those identities next to the mbox.

mod envelope;
mod error;
pub mod index;
pub mod message;
pub mod reader;

/// The MIME parser behind [`message`], re-exported so callers see one version of its types.
pub use mail_parser;

pub use envelope::is_envelope;
pub use error::{Error, Result};
pub use index::{
    BuildOptions, BuildReport, Hit, IndexMeta, IndexStatus, Listing, MboxIndex, Opened, Progress,
    SearchQuery, SearchResult, Sort, StaleReason, build, index_dir, status,
};
pub use message::{
    Attachment, Mailbox, MessageSummary, PartInfo, PartRef, TextKind, TextPart, attachments,
    html_to_plain_text, is_inline, parse, part_at, part_bytes, part_tree, summarize, text_parts,
    walk_parts,
};
pub use reader::{
    DEFAULT_BUFFER_BYTES, DEFAULT_MAX_MESSAGE_BYTES, MboxReader, RawMessage, read_at, read_at_from,
};
