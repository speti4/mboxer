//! Turns one message of the mbox into the reader pane's [`MessageView`].
//!
//! The view carries both the text of the body (the plain part, else the HTML
//! reduced to text) and the first HTML part as written; the webview sanitizes
//! the HTML and renders it in a sandboxed iframe (task 007).

use std::path::Path;

use mboxer_core::mail_parser::Message;
use mboxer_core::{
    RawMessage, TextKind, TextPart, attachments, html_to_plain_text, parse, part_tree, read_at,
    summarize, text_parts,
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
    let parts = text_parts(message);
    MessageView {
        file: 0,
        offset: raw.offset,
        len: raw.len,
        date: summary.date_unix,
        from: display(&summary.from),
        to: display(&summary.to),
        cc: display(&summary.cc),
        subject: summary.subject,
        labels: summary.labels,
        message_id: summary.message_id,
        body: body_view(&parts),
        html: html_part(&parts).map(|html| html.text.clone()),
        attachments: attachments(message).into_iter().map(Into::into).collect(),
        parts: part_tree(message).into_iter().map(Into::into).collect(),
    }
}

/// The first plain part, else the text of the first HTML part, else nothing.
fn body_view(parts: &[TextPart]) -> Option<BodyView> {
    if let Some(plain) = parts.iter().find(|part| part.kind == TextKind::Plain) {
        return Some(BodyView {
            kind: BodyKind::Plain,
            text: unescape_from_lines(&plain.text),
        });
    }
    html_part(parts).map(|html| BodyView {
        kind: BodyKind::HtmlAsText,
        text: html_to_plain_text(&html.text),
    })
}

fn html_part(parts: &[TextPart]) -> Option<&TextPart> {
    parts.iter().find(|part| part.kind == TextKind::Html)
}

/// Undoes the mboxrd quoting of a body for display: one `>` comes off every
/// line that is `>`s followed by `From `. The bytes in the file stay as written
/// (D-003): mboxo and mboxrd cannot be told apart, so this is a display choice.
pub fn unescape_from_lines(text: &str) -> String {
    text.split_inclusive('\n')
        .map(|line| {
            let quoted = line.len() - line.trim_start_matches('>').len();
            if quoted > 0 && line[quoted..].starts_with("From ") {
                &line[1..]
            } else {
                line
            }
        })
        .collect()
}

/// A message `mail-parser` could not parse: identity plus its raw bytes.
fn raw_view(raw: &RawMessage) -> MessageView {
    MessageView {
        file: 0,
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
        html: None,
        attachments: Vec::new(),
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
        // The fixture's `>From a könyvelés` line is shown unquoted (task 013).
        assert!(body.text.contains("\nFrom a könyvelés"), "{}", body.text);
        assert!(!body.text.contains(">From"), "{}", body.text);
        let html = view.html.expect("the alternative HTML part travels too");
        assert!(
            html.contains("<p>Kedves Béla, a számla mellékelve.</p>"),
            "{html}"
        );
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
        // The HTML goes to the webview as written; sanitizing is its job.
        let html = view.html.expect("html part");
        assert!(html.contains("<script>"), "{html}");
    }

    #[test]
    fn mboxrd_quoting_comes_off_for_display_only() {
        let text = ">From one\r\n>>From two\nFrom plain\n>Frome not\n> From spaced\n>From";
        assert_eq!(
            unescape_from_lines(text),
            "From one\r\n>From two\nFrom plain\n>Frome not\n> From spaced\n>From"
        );
        assert_eq!(unescape_from_lines(""), "");
        assert_eq!(unescape_from_lines(">From tail"), "From tail");
    }

    #[test]
    fn remote_fixture_carries_its_html_and_unquoted_text() {
        let path = fixture("html-remote.mbox");
        let views: Vec<MessageView> = MboxReader::open(&path)
            .unwrap()
            .map(|item| {
                let raw = item.unwrap();
                read_message_view(&path, raw.offset, raw.len).unwrap()
            })
            .collect();
        assert_eq!(views.len(), 2);
        let first = &views[0];
        let body = first.body.as_ref().expect("plain body");
        assert_eq!(body.kind, BodyKind::Plain);
        assert!(body.text.contains("\nFrom a szerverről"), "{}", body.text);
        assert!(body.text.contains("\n>From kétszer"), "{}", body.text);
        let html = first.html.as_deref().expect("html part");
        assert!(html.contains("https://example.com/t.gif"), "{html}");
        assert!(html.contains("<script>"), "{html}");
        let second = &views[1];
        assert_eq!(second.body.as_ref().unwrap().kind, BodyKind::HtmlAsText);
        assert!(second.html.as_deref().unwrap().contains("<iframe"));
    }

    #[test]
    fn attachments_travel_with_the_view() {
        let path = fixture("attachments.mbox");
        let raw = first_message("attachments.mbox");
        let view = read_message_view(&path, raw.offset, raw.len).unwrap();
        let names: Vec<Option<&str>> = view.attachments.iter().map(|a| a.name.as_deref()).collect();
        assert_eq!(names[0], Some("logo.png"));
        assert_eq!(names[3], Some("Forwarded original.eml"));
        assert_eq!(view.attachments.len(), 6);
        let logo = &view.attachments[0];
        assert!(logo.inline);
        assert_eq!(logo.content_id.as_deref(), Some("logo@example.com"));
        assert_eq!(view.parts[logo.index].content_type, "image/png");
        assert_eq!(view.parts[logo.index].index, logo.index);
        assert!(
            view.html
                .as_deref()
                .unwrap()
                .contains("cid:logo@example.com")
        );
        assert!(view.attachments[3].is_message);
        // The second message: one nameless inline part for the cid, nothing to list.
        let second = MboxReader::open(&path).unwrap().nth(1).unwrap().unwrap();
        let view = read_message_view(&path, second.offset, second.len).unwrap();
        assert_eq!(view.attachments.len(), 1);
        assert_eq!(view.attachments[0].name, None);
        assert!(view.attachments[0].inline);
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
