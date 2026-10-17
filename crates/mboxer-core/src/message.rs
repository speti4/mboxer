//! Parsed view of a message: a thin layer over `mail-parser` plus the owned
//! [`MessageSummary`] that the index stores.

use mail_parser::decoders::html::html_to_text;
use mail_parser::{Address, Message, MessageParser, MessagePart, MimeHeaders, PartType};

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
        let inline = part.content_id().is_some()
            && part
                .content_disposition()
                .is_none_or(|disposition| disposition.is_inline());
        if let Some(name) = part.attachment_name() {
            attachment_names.push(name.to_owned());
            if !inline {
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

/// One node of the MIME tree, for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    /// 0 for the root part.
    pub depth: usize,
    /// `type/subtype`, lower case; `text/plain` when the part has no Content-Type.
    pub content_type: String,
    pub encoding: Option<String>,
    pub name: Option<String>,
    pub size: usize,
}

/// The MIME tree of `message` in document order, nested messages expanded.
pub fn part_tree(message: &Message<'_>) -> Vec<PartInfo> {
    let mut out = Vec::new();
    walk_part(message, message.root_part(), 0, &mut out);
    out
}

fn walk_part(message: &Message<'_>, part: &MessagePart<'_>, depth: usize, out: &mut Vec<PartInfo>) {
    let content_type = part
        .content_type()
        .map(|ct| match ct.subtype() {
            Some(sub) => format!("{}/{}", ct.ctype(), sub).to_ascii_lowercase(),
            None => ct.ctype().to_ascii_lowercase(),
        })
        .unwrap_or_else(|| "text/plain".to_owned());
    out.push(PartInfo {
        depth,
        content_type,
        encoding: part
            .content_transfer_encoding()
            .map(|e| e.to_ascii_lowercase()),
        name: part.attachment_name().map(str::to_owned),
        size: part.len(),
    });
    match &part.body {
        PartType::Multipart(children) => {
            for &child in children {
                if let Some(child) = message.part(child) {
                    walk_part(message, child, depth + 1, out);
                }
            }
        }
        PartType::Message(nested) => {
            walk_part(nested, nested.root_part(), depth + 1, out);
        }
        _ => {}
    }
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
}
