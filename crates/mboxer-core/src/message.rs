//! Parsed view of a message: a thin layer over `mail-parser` plus the owned
//! [`MessageSummary`] that the index stores, the MIME tree for display and the
//! attachments with their decoded bytes (task 008).

use std::borrow::Cow;

use mail_parser::decoders::base64::base64_decode;
use mail_parser::decoders::html::html_to_text;
use mail_parser::decoders::quoted_printable::quoted_printable_decode;
use mail_parser::{Address, Encoding, Message, MessageParser, MessagePart, MimeHeaders, PartType};

use crate::reader::RawMessage;

/// HTML reduced to its text, for indexing and for a plain-text preview.
/// `mail-parser` drops the whitespace that follows a tag, so `<b>Ár</b> a`
/// would come out as `Ára`; a space in front of every tag keeps the words
/// apart (the extra spaces cost the tokenizer nothing and read fine).
pub fn html_to_plain_text(html: &str) -> String {
    html_to_text(&html.replace('<', " <"))
}

/// Parses the raw bytes of a message. `None` means `mail-parser` found no message at all.
pub fn parse(raw: &RawMessage) -> Option<Message<'_>> {
    MessageParser::default().parse(&raw.bytes)
}

/// One mailbox from an address header.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Mailbox {
    pub name: Option<String>,
    pub address: Option<String>,
}

impl Mailbox {
    /// `Name <address>`, or whichever half exists; `?` when neither does.
    pub fn display(&self) -> String {
        match (&self.name, &self.address) {
            (Some(name), Some(address)) => format!("{name} <{address}>"),
            (None, Some(address)) => address.clone(),
            (Some(name), None) => name.clone(),
            (None, None) => "?".to_owned(),
        }
    }
}

/// The index-friendly digest of one message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageSummary {
    pub offset: u64,
    pub len: u64,
    pub message_id: Option<String>,
    /// Gmail's `X-GM-THRID` header.
    pub gmail_thread_id: Option<u64>,
    /// Gmail's `X-Gmail-Labels` header, split on commas and trimmed.
    pub labels: Vec<String>,
    /// The `Date` header as Unix seconds.
    pub date_unix: Option<i64>,
    pub from: Vec<Mailbox>,
    pub to: Vec<Mailbox>,
    pub cc: Vec<Mailbox>,
    pub subject: Option<String>,
    /// At least one named part that is not an inline (cid-referenced) resource.
    pub has_attachment: bool,
    pub attachment_names: Vec<String>,
}

/// Builds the summary of `message`, which must have been parsed from `raw`.
pub fn summarize(raw: &RawMessage, message: &Message<'_>) -> MessageSummary {
    let labels = message
        .header("X-Gmail-Labels")
        .and_then(|value| value.as_text())
        .map(split_labels)
        .unwrap_or_default();
    let gmail_thread_id = message
        .header("X-GM-THRID")
        .and_then(|value| value.as_text())
        .and_then(|text| text.trim().parse().ok());
    let mut attachment_names = Vec::new();
    let mut has_attachment = false;
    for part in message.attachments() {
        if let Some(name) = part.attachment_name() {
            attachment_names.push(name.to_owned());
            if !is_inline(part) {
                has_attachment = true;
            }
        } else if matches!(part.body, PartType::Message(_)) {
            has_attachment = true;
        }
    }
    MessageSummary {
        offset: raw.offset,
        len: raw.len,
        message_id: message.message_id().map(str::to_owned),
        gmail_thread_id,
        labels,
        date_unix: message.date().map(|d| d.to_timestamp()),
        from: mailboxes(message.from()),
        to: mailboxes(message.to()),
        cc: mailboxes(message.cc()),
        subject: message.subject().map(str::to_owned),
        has_attachment,
        attachment_names,
    }
}

fn split_labels(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
        .collect()
}

fn mailboxes(address: Option<&Address<'_>>) -> Vec<Mailbox> {
    address
        .map(|address| {
            address
                .iter()
                .map(|addr| Mailbox {
                    name: addr.name().map(str::to_owned),
                    address: addr.address().map(str::to_owned),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Which kind of text a [`TextPart`] holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    Plain,
    Html,
}

/// One decoded text or HTML body, including those of nested `message/rfc822` parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextPart {
    pub kind: TextKind,
    pub text: String,
}

/// All decoded text and HTML bodies of `message`, outer message first, nested messages after.
pub fn text_parts(message: &Message<'_>) -> Vec<TextPart> {
    let mut out = Vec::new();
    collect_text_parts(message, &mut out);
    out
}

fn collect_text_parts(message: &Message<'_>, out: &mut Vec<TextPart>) {
    // mail-parser lists a lone text part under both `text_bodies` and `html_bodies` (and
    // vice versa) so that either representation can be produced on demand; filter on the
    // actual part type to report each body once.
    for part in message.text_bodies() {
        if let PartType::Text(text) = &part.body {
            out.push(TextPart {
                kind: TextKind::Plain,
                text: text.to_string(),
            });
        }
    }
    for part in message.html_bodies() {
        if let PartType::Html(text) = &part.body {
            out.push(TextPart {
                kind: TextKind::Html,
                text: text.to_string(),
            });
        }
    }
    for part in message.attachments() {
        if let PartType::Message(nested) = &part.body {
            collect_text_parts(nested, out);
        }
    }
}

/// One part of the MIME tree together with the message that owns it (the
/// outer message, or the nested `message/rfc822` the part sits in).
#[derive(Debug, Clone, Copy)]
pub struct PartRef<'a, 'x> {
    /// 0 for the root part.
    pub depth: usize,
    pub owner: &'a Message<'x>,
    pub part: &'a MessagePart<'x>,
}

/// The MIME tree of `message` in document order, nested messages expanded.
/// The position in the result is the part's address for [`part_at`].
pub fn walk_parts<'a, 'x>(message: &'a Message<'x>) -> Vec<PartRef<'a, 'x>> {
    let mut out = Vec::new();
    walk_part(message, message.root_part(), 0, &mut out);
    out
}

fn walk_part<'a, 'x>(
    owner: &'a Message<'x>,
    part: &'a MessagePart<'x>,
    depth: usize,
    out: &mut Vec<PartRef<'a, 'x>>,
) {
    out.push(PartRef { depth, owner, part });
    match &part.body {
        PartType::Multipart(children) => {
            for &child in children {
                if let Some(child) = owner.part(child) {
                    walk_part(owner, child, depth + 1, out);
                }
            }
        }
        PartType::Message(nested) => {
            walk_part(nested, nested.root_part(), depth + 1, out);
        }
        _ => {}
    }
}

/// The part at `index` of [`walk_parts`].
pub fn part_at<'a, 'x>(message: &'a Message<'x>, index: usize) -> Option<PartRef<'a, 'x>> {
    walk_parts(message).into_iter().nth(index)
}

/// One node of the MIME tree, for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    /// The part's address: its position in [`walk_parts`].
    pub index: usize,
    /// 0 for the root part.
    pub depth: usize,
    /// `type/subtype`, lower case; `text/plain` when the part has no Content-Type.
    pub content_type: String,
    pub encoding: Option<String>,
    /// `attachment` or `inline` (lower case), when the part has a Content-Disposition.
    pub disposition: Option<String>,
    pub name: Option<String>,
    /// The Content-ID without its angle brackets.
    pub content_id: Option<String>,
    pub size: usize,
}

/// The MIME tree of `message` in document order, nested messages expanded.
pub fn part_tree(message: &Message<'_>) -> Vec<PartInfo> {
    walk_parts(message)
        .into_iter()
        .enumerate()
        .map(|(index, part_ref)| PartInfo {
            index,
            depth: part_ref.depth,
            content_type: content_type_of(part_ref.part),
            encoding: part_ref
                .part
                .content_transfer_encoding()
                .map(|e| e.to_ascii_lowercase()),
            disposition: part_ref
                .part
                .content_disposition()
                .map(|d| d.ctype().to_ascii_lowercase()),
            name: part_ref.part.attachment_name().map(str::to_owned),
            content_id: content_id_of(part_ref.part),
            size: part_bytes(part_ref).len(),
        })
        .collect()
}

fn content_type_of(part: &MessagePart<'_>) -> String {
    part.content_type()
        .map(|ct| match ct.subtype() {
            Some(sub) => format!("{}/{}", ct.ctype(), sub).to_ascii_lowercase(),
            None => ct.ctype().to_ascii_lowercase(),
        })
        .unwrap_or_else(|| "text/plain".to_owned())
}

/// The Content-ID as a `cid:` URL would name it: no angle brackets, no surrounding space.
fn content_id_of(part: &MessagePart<'_>) -> Option<String> {
    part.content_id()
        .map(|id| {
            id.trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .trim()
        })
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

/// A cid-referenced resource: it has a Content-ID and is not declared an attachment.
pub fn is_inline(part: &MessagePart<'_>) -> bool {
    part.content_id().is_some()
        && part
            .content_disposition()
            .is_none_or(|disposition| disposition.is_inline())
}

/// One attachment of a message, for the reader's attachment strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// The part's address: its position in [`walk_parts`].
    pub index: usize,
    /// The declared file name; a nested message without one is named after its subject.
    pub name: Option<String>,
    pub content_type: String,
    pub content_id: Option<String>,
    /// Decoded size in bytes.
    pub size: usize,
    /// A cid-referenced resource (shown inside the HTML body).
    pub inline: bool,
    /// A nested `message/rfc822`.
    pub is_message: bool,
}

/// The attachments of the outer `message` (what `mail-parser` lists besides the
/// text bodies), in document order; a nested message counts as one attachment.
pub fn attachments(message: &Message<'_>) -> Vec<Attachment> {
    walk_parts(message)
        .into_iter()
        .enumerate()
        .filter(|(_, part_ref)| {
            std::ptr::eq(part_ref.owner, message)
                && message
                    .attachments()
                    .any(|attachment| std::ptr::eq(attachment, part_ref.part))
        })
        .map(|(index, part_ref)| {
            let part = part_ref.part;
            let nested = part.message();
            let name = part.attachment_name().map(str::to_owned).or_else(|| {
                nested.map(|nested| format!("{}.eml", nested.subject().unwrap_or("message")))
            });
            Attachment {
                index,
                name,
                content_type: content_type_of(part),
                content_id: content_id_of(part),
                size: part_bytes(part_ref).len(),
                inline: is_inline(part),
                is_message: nested.is_some(),
            }
        })
        .collect()
}

/// The decoded bytes of a part. Binary parts come decoded from `mail-parser`;
/// a text part is decoded here from its raw bytes, so that a `windows-1250` csv
/// keeps its bytes instead of becoming UTF-8; a nested message is the raw bytes
/// of its own range (`mail-parser` hands the whole outer message otherwise).
pub fn part_bytes<'a>(part_ref: PartRef<'a, '_>) -> Cow<'a, [u8]> {
    let part = part_ref.part;
    match &part.body {
        PartType::Binary(_) | PartType::InlineBinary(_) | PartType::Multipart(_) => {
            Cow::Borrowed(part.contents())
        }
        // An encoded nested message was decoded into its own buffer.
        PartType::Message(nested) if part.encoding != Encoding::None => {
            Cow::Borrowed(nested.raw_message())
        }
        PartType::Text(_) | PartType::Html(_) | PartType::Message(_) => {
            raw_decoded(part_ref).unwrap_or_else(|| Cow::Borrowed(part.contents()))
        }
    }
}

/// The part's raw range of its owner, transfer decoding undone.
fn raw_decoded<'a>(part_ref: PartRef<'a, '_>) -> Option<Cow<'a, [u8]>> {
    let part = part_ref.part;
    let raw = part_ref.owner.raw_message();
    let start = part.raw_body_offset() as usize;
    let end = part.raw_end_offset() as usize;
    if start > end || end > raw.len() {
        return None;
    }
    let slice = &raw[start..end];
    match part.encoding {
        Encoding::Base64 => base64_decode(slice).map(Cow::Owned),
        Encoding::QuotedPrintable => quoted_printable_decode(slice).map(Cow::Owned),
        Encoding::None => Some(Cow::Borrowed(trim_boundary_break(slice))),
    }
}

/// The line break that precedes a MIME boundary belongs to the boundary, not
/// to the part (RFC 2046 §5.1.1).
fn trim_boundary_break(bytes: &[u8]) -> &[u8] {
    bytes
        .strip_suffix(b"\r\n")
        .or_else(|| bytes.strip_suffix(b"\n"))
        .unwrap_or(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_to_plain_text_keeps_words_apart_around_inline_tags() {
        let text = html_to_plain_text(
            "<p>Kedves P&eacute;ter,</p><p><b>Árajánlat</b> a gépre.</p><style>.x{}</style>",
        );
        let words: Vec<&str> = text.split_whitespace().collect();
        assert_eq!(words, ["Kedves", "Péter,", "Árajánlat", "a", "gépre."]);
    }

    #[test]
    fn the_boundary_line_break_is_not_part_of_the_text() {
        assert_eq!(trim_boundary_break(b"a\r\n"), b"a");
        assert_eq!(trim_boundary_break(b"a\n"), b"a");
        assert_eq!(trim_boundary_break(b"a\n\n"), b"a\n");
        assert_eq!(trim_boundary_break(b"a"), b"a");
        assert_eq!(trim_boundary_break(b""), b"");
    }
}
