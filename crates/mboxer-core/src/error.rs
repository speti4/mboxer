use std::fmt;
use std::path::PathBuf;

/// Errors produced by the mbox reader.
#[derive(Debug)]
pub enum Error {
    /// An I/O error from the underlying file.
    Io(std::io::Error),
    /// The data at `offset` does not start with an mbox envelope (`From `) line.
    NotAnMbox { offset: u64 },
    /// A single message exceeded the configured size limit; the file is probably corrupt.
    MessageTooLarge { offset: u64, limit: u64 },
    /// A seek-and-read asked for more bytes than the file holds.
    Truncated {
        offset: u64,
        expected: u64,
        actual: u64,
    },
    /// The sidecar index engine failed.
    Index(tantivy::TantivyError),
    /// The sidecar's own metadata or schema is not what this build expects.
    IndexMeta(String),
    /// `mboxer.json` could not be encoded or decoded.
    Json(serde_json::Error),
    /// An index build was stopped through its cancel flag.
    Cancelled,
    /// Another build of the same mbox holds the build lock at `lock`.
    BuildLocked { lock: PathBuf },
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {err}"),
            Error::NotAnMbox { offset } => {
                write!(f, "no mbox envelope (`From ` line) at byte offset {offset}")
            }
            Error::MessageTooLarge { offset, limit } => write!(
                f,
                "message at byte offset {offset} exceeds the {limit}-byte limit; the file is probably not a valid mbox"
            ),
            Error::Truncated {
                offset,
                expected,
                actual,
            } => write!(
                f,
                "message at byte offset {offset}: expected {expected} bytes, file holds {actual}"
            ),
            Error::Index(err) => write!(f, "index error: {err}"),
            Error::IndexMeta(msg) => write!(f, "index metadata: {msg}"),
            Error::Json(err) => write!(f, "index metadata: {err}"),
            Error::Cancelled => write!(f, "index build cancelled"),
            Error::BuildLocked { lock } => write!(
                f,
                "another build holds `{}`; wait for it to finish",
                lock.display()
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Index(err) => Some(err),
            Error::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<tantivy::TantivyError> for Error {
    fn from(err: tantivy::TantivyError) -> Self {
        Error::Index(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}
