//! Turns one message of the mbox into the reader pane's [`MessageView`].
//!
//! Task 005 shows headers plus a plain-text body; task 007 replaces the body
//! block with a sanitized HTML rendering in a sandboxed iframe.

use std::path::Path;

use mboxer_core::mail_parser::Message;
use mboxer_core::{
    RawMessage, TextKind, html_to_plain_text, parse, part_tree, read_at, summarize, text_parts,
};

use crate::types::{BodyKind, BodyView, MessageView};

/// Reads the message at `offset`/`len` from `mbox` and builds its view.
pub fn read_message_view(mbox: &Path, offset: u64, len: u64) -> mboxer_core::Result<MessageView> {
    let raw = read_at(mbox, offset, len)?;
    Ok(match parse(&raw) {
        Some(message) => message_view(&raw, &message),
        None => raw_view(&raw),
    })
}

/// The view of a parsed message.
pub fn message_view(raw: &RawMessage, message: &Message<'_>) -> MessageView {
    let summary = summarize(raw, message);
    let display = |mailboxes: &[mboxer_core::Mailbox]| {
        mailboxes
            .iter()
            .map(mboxer_core::Mailbox::display)
            .collect::<Vec<_>>()
    };
    MessageView {
        offset: raw.offset,
        len: raw.len,
        date: summary.date_unix,
        from: display(&summary.from),
        to: display(&summary.to),
        cc: display(&summary.cc),
        subject: summary.subject,
        labels: summary.labels,
        message_id: summary.message_id,
        body: body_view(message),
        parts: part_tree(message).into_iter().map(Into::into).collect(),
    }
}

/// The first plain part, else the text of the first HTML part, else nothing.
fn body_view(message: &Message<'_>) -> Option<BodyView> {
    let parts = text_parts(message);
    if let Some(plain) = parts.iter().find(|part| part.kind == TextKind::Plain) {
        return Some(BodyView {
            kind: BodyKind::Plain,
            text: plain.text.clone(),
        });
    }
    parts
        .iter()
        .find(|part| part.kind == TextKind::Html)
        .map(|html| BodyView {
            kind: BodyKind::HtmlAsText,
            text: html_to_plain_text(&html.text),
        })
}

/// A message `mail-parser` could not parse: identity plus its raw bytes.
fn raw_view(raw: &RawMessage) -> MessageView {
    MessageView {
        offset: raw.offset,
        len: raw.len,
        date: None,
        from: Vec::new(),
        to: Vec::new(),
        cc: Vec::new(),
        subject: None,
        labels: Vec::new(),
        message_id: None,
        body: Some(BodyView {
            kind: BodyKind::Raw,
            text: String::from_utf8_lossy(&raw.bytes).into_owned(),
        }),
        parts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mboxer_core::MboxReader;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../mboxer-core/tests/fixtures")
            .join(name)
    }

    fn first_message(name: &str) -> RawMessage {
        MboxReader::open(fixture(name))
            .unwrap()
            .next()
            .expect("fixture has a message")
            .unwrap()
    }

    #[test]
    fn plain_part_wins_and_headers_are_filled() {
        let raw = first_message("gmail-crlf.mbox");
        let view = read_message_view(&fixture("gmail-crlf.mbox"), raw.offset, raw.len).unwrap();
        assert_eq!(view.offset, 0);
        assert_eq!(view.len, raw.len);
        assert!(view.date.is_some());
        assert_eq!(view.from.len(), 1);
        assert!(view.from[0].contains("anna@example.com"), "{:?}", view.from);
        assert!(view.labels.contains(&"Inbox".to_owned()));
        let body = view.body.expect("plain body");
        assert_eq!(body.kind, BodyKind::Plain);
        assert!(body.text.contains("Kedves Béla"), "{}", body.text);
        let message = parse(&raw).unwrap();
        assert_eq!(view.parts.len(), part_tree(&message).len());
        assert!(view.parts.len() > 1, "a nested MIME tree");
    }

    #[test]
    fn html_only_message_falls_back_to_its_text() {
        let raw = first_message("html-only.mbox");
        let message = parse(&raw).unwrap();
        let view = message_view(&raw, &message);
        let body = view.body.expect("html body");
        assert_eq!(body.kind, BodyKind::HtmlAsText);
        assert!(!body.text.contains('<'), "{}", body.text);
        assert!(!body.text.trim().is_empty());
    }

    #[test]
    fn legacy_charsets_are_decoded() {
        let path = fixture("hungarian-charsets.mbox");
        for item in MboxReader::open(&path).unwrap() {
            let raw = item.unwrap();
            let view = read_message_view(&path, raw.offset, raw.len).unwrap();
            let body = view.body.expect("body");
            assert!(
                !body.text.contains('\u{FFFD}'),
                "replacement character in {}",
                body.text
            );
        }
    }

    #[test]
    fn an_unparsable_message_shows_its_raw_bytes() {
        let raw = RawMessage {
            offset: 10,
            len: 20,
            envelope: "From x".to_owned(),
            bytes: b"not a message".to_vec(),
        };
        let view = raw_view(&raw);
        assert_eq!(view.body.unwrap().kind, BodyKind::Raw);
        assert_eq!(view.offset, 10);
    }
}
