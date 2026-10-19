//! Streaming mbox reader with a bounded buffer.
//!
//! [`MboxReader`] yields one [`RawMessage`] at a time. Memory use is the read
//! buffer plus the largest single message, never the file. Each message carries
//! the byte offset of its envelope line and its length up to the next envelope
//! line, which is the identity every later layer stores (decision D-003);
//! [`read_at`] turns that pair back into the same bytes. A message over the
//! size limit is an [`Error::MessageTooLarge`] item, and the stream goes on at
//! the next envelope line; an oversized first line, a file that does not start
//! with an envelope and an I/O error end it.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use crate::envelope::{is_envelope, trim_line_break, unquote_from_lines};
use crate::error::{Error, Result};

/// Read buffer size used by [`MboxReader::open`].
pub const DEFAULT_BUFFER_BYTES: usize = 4 << 20;

/// Default upper bound for one message; a larger one is treated as corruption.
pub const DEFAULT_MAX_MESSAGE_BYTES: u64 = 256 << 20;

/// One message as it sits in the mbox: identity plus raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessage {
    /// Byte offset of the first byte of the envelope (`From `) line.
    pub offset: u64,
    /// Bytes from `offset` up to (not including) the next envelope line or the end of the file.
    pub len: u64,
    /// The envelope line without its line break, decoded lossily as UTF-8.
    pub envelope: String,
    /// Everything after the envelope line, line endings untouched.
    pub bytes: Vec<u8>,
}

impl RawMessage {
    /// Offset of the first byte after this message.
    pub fn end(&self) -> u64 {
        self.offset + self.len
    }
}

/// Streams messages out of an mbox.
pub struct MboxReader<R> {
    inner: R,
    /// Absolute position of the next unread byte of `inner`.
    pos: u64,
    /// The line being read. Reused from line to line within a message; an
    /// envelope line is moved into `pending` (`mem::take`), so the next line
    /// starts a new allocation once per message.
    line: Vec<u8>,
    /// The body of the message being read. Moved into the [`RawMessage`] it
    /// becomes (`mem::take`), so every message grows a fresh one: the caller
    /// owns the bytes it is handed.
    body: Vec<u8>,
    /// The envelope line of the next message, already read: (offset, line with break).
    pending: Option<(u64, Vec<u8>)>,
    max_message_bytes: u64,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Start,
    Streaming,
    Done,
}

impl MboxReader<BufReader<File>> {
    /// Opens `path` with a [`DEFAULT_BUFFER_BYTES`] buffer.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self::new(BufReader::with_capacity(
            DEFAULT_BUFFER_BYTES,
            file,
        )))
    }
}

impl<R: BufRead> MboxReader<R> {
    /// Wraps any buffered reader positioned at the start of an mbox.
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            pos: 0,
            line: Vec::new(),
            body: Vec::new(),
            pending: None,
            max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
            state: State::Start,
        }
    }

    /// Sets the per-message size limit (default [`DEFAULT_MAX_MESSAGE_BYTES`]).
    pub fn with_max_message_bytes(mut self, limit: u64) -> Self {
        self.max_message_bytes = limit;
        self
    }

    /// Number of bytes consumed so far; useful for progress reporting.
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Reads one line (including its break) into `self.line`, reading at most
    /// `limit` bytes. Returns the number of bytes read; 0 means end of input.
    fn read_line_bounded(&mut self, limit: u64) -> io::Result<usize> {
        self.line.clear();
        let n = self
            .inner
            .by_ref()
            .take(limit)
            .read_until(b'\n', &mut self.line)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn line_is_complete(&self) -> bool {
        self.line.last() == Some(&b'\n')
    }

    fn read_first(&mut self) -> Result<bool> {
        self.state = State::Streaming;
        let n = self.read_line_bounded(self.max_message_bytes)?;
        if n == 0 {
            self.state = State::Done;
            return Ok(false);
        }
        if !self.line_is_complete() && n as u64 >= self.max_message_bytes {
            self.state = State::Done;
            return Err(Error::MessageTooLarge {
                offset: 0,
                limit: self.max_message_bytes,
            });
        }
        if !is_envelope(&self.line) {
            self.state = State::Done;
            return Err(Error::NotAnMbox { offset: 0 });
        }
        self.pending = Some((0, std::mem::take(&mut self.line)));
        Ok(true)
    }

    fn read_message(&mut self) -> Result<Option<RawMessage>> {
        let Some((offset, envelope_line)) = self.pending.take() else {
            self.state = State::Done;
            return Ok(None);
        };
        self.body.clear();
        loop {
            let line_start = self.pos;
            // Each line is read with the message limit as its own bound, so a file without
            // line breaks cannot grow the buffer past the limit either.
            let n = self.read_line_bounded(self.max_message_bytes)?;
            if n == 0 {
                // End of file: the current message is the last one.
                self.state = State::Done;
                break;
            }
            if is_envelope(&self.line) {
                self.pending = Some((line_start, std::mem::take(&mut self.line)));
                break;
            }
            let used = line_start - offset + n as u64;
            if used > self.max_message_bytes
                || (!self.line_is_complete() && n as u64 == self.max_message_bytes)
            {
                self.skip_to_next_envelope()?;
                return Err(Error::MessageTooLarge {
                    offset,
                    limit: self.max_message_bytes,
                });
            }
            self.body.extend_from_slice(&self.line);
        }
        let end = self.pending.as_ref().map_or(self.pos, |(start, _)| *start);
        Ok(Some(RawMessage {
            offset,
            len: end - offset,
            envelope: String::from_utf8_lossy(trim_line_break(&envelope_line)).into_owned(),
            bytes: std::mem::take(&mut self.body),
        }))
    }

    /// Drops the rest of an oversized message: reads on, one bounded line at a
    /// time, until the next envelope line (kept as the pending one) or the end
    /// of the file, so the stream goes on after the error (task 052). Only a
    /// line that starts at a line break can be an envelope.
    fn skip_to_next_envelope(&mut self) -> Result<()> {
        self.body.clear();
        let mut at_line_start = self.line_is_complete();
        loop {
            let line_start = self.pos;
            let n = match self.read_line_bounded(self.max_message_bytes) {
                Ok(n) => n,
                Err(err) => {
                    self.state = State::Done;
                    return Err(err.into());
                }
            };
            if n == 0 {
                self.state = State::Done;
                return Ok(());
            }
            if at_line_start && is_envelope(&self.line) {
                self.pending = Some((line_start, std::mem::take(&mut self.line)));
                return Ok(());
            }
            at_line_start = self.line_is_complete();
        }
    }
}

impl<R: BufRead> Iterator for MboxReader<R> {
    type Item = Result<RawMessage>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            State::Done => None,
            State::Start => match self.read_first() {
                Ok(true) => self.read_message().transpose(),
                Ok(false) => None,
                Err(err) => Some(Err(err)),
            },
            State::Streaming => self.read_message().transpose(),
        }
    }
}

/// Reads back the message identified by `offset` and `len` from the mbox at `path`.
///
/// Verifies that the range starts with an envelope line; a mismatch means the
/// file changed since the identity was recorded.
/// A `len` over [`DEFAULT_MAX_MESSAGE_BYTES`] is refused; an index built with
/// another limit reads its messages with
/// [`MboxIndex::read_message`](crate::MboxIndex::read_message).
pub fn read_at(path: impl AsRef<Path>, offset: u64, len: u64) -> Result<RawMessage> {
    read_at_with_limit(path, offset, len, DEFAULT_MAX_MESSAGE_BYTES)
}

/// [`read_at`] with the size limit the message was streamed under.
pub fn read_at_with_limit(
    path: impl AsRef<Path>,
    offset: u64,
    len: u64,
    max_message_bytes: u64,
) -> Result<RawMessage> {
    let file = File::open(path)?;
    read_at_from_with_limit(file, offset, len, max_message_bytes)
}

/// [`read_at`] over any seekable source.
pub fn read_at_from<R: Read + Seek>(source: R, offset: u64, len: u64) -> Result<RawMessage> {
    read_at_from_with_limit(source, offset, len, DEFAULT_MAX_MESSAGE_BYTES)
}

/// [`read_at_with_limit`] over any seekable source.
pub fn read_at_from_with_limit<R: Read + Seek>(
    mut source: R,
    offset: u64,
    len: u64,
    max_message_bytes: u64,
) -> Result<RawMessage> {
    if len > max_message_bytes {
        return Err(Error::MessageTooLarge {
            offset,
            limit: max_message_bytes,
        });
    }
    source.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8; len as usize];
    let mut filled = 0;
    while filled < buf.len() {
        let n = source.read(&mut buf[filled..])?;
        if n == 0 {
            return Err(Error::Truncated {
                offset,
                expected: len,
                actual: filled as u64,
            });
        }
        filled += n;
    }
    let line_end = buf
        .iter()
        .position(|&b| b == b'\n')
        .map_or(buf.len(), |i| i + 1);
    if !is_envelope(&buf[..line_end]) {
        return Err(Error::NotAnMbox { offset });
    }
    let bytes = buf.split_off(line_end);
    Ok(RawMessage {
        offset,
        len,
        envelope: String::from_utf8_lossy(trim_line_break(&buf)).into_owned(),
        bytes,
    })
}

/// The message as an `.eml` file: the bytes after the envelope line with the
/// mboxrd quoting undone by [`unquote_from_lines`](crate::unquote_from_lines)
/// (task 011); the viewer's text body goes through the same function.
pub fn eml_bytes(raw: &RawMessage) -> Vec<u8> {
    unquote_from_lines(&raw.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const TWO: &[u8] = b"From a@example.com Mon Jan  5 09:00:00 2026\nSubject: one\n\nbody one\n\nFrom b@example.com Tue Jan  6 10:00:00 +0100 2026\nSubject: two\n\nbody two\n";

    fn collect(data: &[u8], limit: u64) -> Result<Vec<RawMessage>> {
        MboxReader::new(Cursor::new(data))
            .with_max_message_bytes(limit)
            .collect()
    }

    #[test]
    fn splits_two_messages_with_contiguous_ranges() {
        let msgs = collect(TWO, DEFAULT_MAX_MESSAGE_BYTES).unwrap();
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].offset, 0);
        assert_eq!(msgs[0].end(), msgs[1].offset);
        assert_eq!(msgs[1].end(), TWO.len() as u64);
        assert_eq!(
            msgs[0].envelope,
            "From a@example.com Mon Jan  5 09:00:00 2026"
        );
        assert_eq!(msgs[0].bytes, b"Subject: one\n\nbody one\n\n");
        assert_eq!(msgs[1].bytes, b"Subject: two\n\nbody two\n");
    }

    #[test]
    fn empty_input_yields_nothing() {
        assert!(collect(b"", DEFAULT_MAX_MESSAGE_BYTES).unwrap().is_empty());
    }

    #[test]
    fn non_mbox_input_is_an_error_at_offset_zero() {
        let err = collect(b"hello\nFrom Anna wrote:\n", DEFAULT_MAX_MESSAGE_BYTES).unwrap_err();
        assert!(matches!(err, Error::NotAnMbox { offset: 0 }), "{err}");
    }

    #[test]
    fn oversized_message_is_an_error_not_an_allocation() {
        let err = collect(TWO, 40).unwrap_err();
        assert!(
            matches!(
                err,
                Error::MessageTooLarge {
                    offset: 0,
                    limit: 40
                }
            ),
            "{err}"
        );
        // The second message is the large one when the limit fits the first.
        let first_len = collect(TWO, DEFAULT_MAX_MESSAGE_BYTES).unwrap()[0].len;
        let mut reader = MboxReader::new(Cursor::new(TWO)).with_max_message_bytes(first_len);
        assert!(reader.next().unwrap().is_ok());
        let err = reader.next().unwrap().unwrap_err();
        assert!(matches!(err, Error::MessageTooLarge { offset, .. } if offset == first_len));
        // It was the last one: the stream ends after it.
        assert!(reader.next().is_none());
    }

    #[test]
    fn the_stream_goes_on_after_an_oversized_message() {
        let mut data = Vec::new();
        data.extend_from_slice(
            b"From a@example.com Mon Jan  5 09:00:00 2026\nSubject: one\n\nshort\n",
        );
        let big_at = data.len() as u64;
        data.extend_from_slice(b"From b@example.com Tue Jan  6 10:00:00 2026\nSubject: two\n\n");
        // One line longer than the limit: the bounded reads cut it after 300
        // bytes, right before text that would be an envelope at a line start.
        data.extend_from_slice(&[b'x'; 300]);
        data.extend_from_slice(b"From x@example.com Thu Jan  8 12:00:00 2026\n\n");
        let last_at = data.len() as u64;
        data.extend_from_slice(
            b"From c@example.com Wed Jan  7 11:00:00 2026\nSubject: three\n\nend\n",
        );

        let mut reader = MboxReader::new(Cursor::new(&data)).with_max_message_bytes(100);
        assert_eq!(reader.next().unwrap().unwrap().offset, 0);
        let err = reader.next().unwrap().unwrap_err();
        assert!(
            matches!(err, Error::MessageTooLarge { offset, limit: 100 } if offset == big_at),
            "{err}"
        );
        let last = reader.next().unwrap().unwrap();
        assert_eq!(last.offset, last_at);
        assert_eq!(last.end(), data.len() as u64);
        assert!(reader.next().is_none());
    }

    #[test]
    fn single_endless_line_is_bounded() {
        let data = vec![b'x'; 1000];
        let err = collect(&data, 100).unwrap_err();
        assert!(matches!(
            err,
            Error::MessageTooLarge {
                offset: 0,
                limit: 100
            }
        ));
    }

    #[test]
    fn eml_bytes_undo_the_mboxrd_quoting_only() {
        let raw = RawMessage {
            offset: 0,
            len: 0,
            envelope: String::new(),
            bytes: b"Subject: x\r\n\r\n>From one\r\n>>From two\nFrom plain\n>Frome not\n> From spaced\n>From"
                .to_vec(),
        };
        assert_eq!(
            eml_bytes(&raw),
            b"Subject: x\r\n\r\nFrom one\r\n>From two\nFrom plain\n>Frome not\n> From spaced\n>From"
        );
        let empty = RawMessage {
            bytes: Vec::new(),
            ..raw
        };
        assert!(eml_bytes(&empty).is_empty());
    }

    #[test]
    fn read_at_roundtrips_and_validates() {
        let msgs = collect(TWO, DEFAULT_MAX_MESSAGE_BYTES).unwrap();
        for m in &msgs {
            let back = read_at_from(Cursor::new(TWO), m.offset, m.len).unwrap();
            assert_eq!(&back, m);
        }
        let err = read_at_from(Cursor::new(TWO), msgs[1].offset + 1, 10).unwrap_err();
        assert!(matches!(err, Error::NotAnMbox { .. }));
        let err = read_at_from(Cursor::new(TWO), msgs[1].offset, msgs[1].len + 5).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }));
    }
}
