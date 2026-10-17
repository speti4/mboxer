use std::fmt;

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
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
