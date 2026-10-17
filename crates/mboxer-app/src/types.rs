//! The IPC types: what the webview sends and receives, camel-cased on the wire.

use mboxer_core::{Attachment, Hit, PartInfo, SearchQuery, Sort};
use serde::{Deserialize, Serialize};

use crate::attachments::{open_blocked, safe_file_name};

/// Progress of `open_mbox`, streamed over a channel; the last event is
/// `Ready`, `Cancelled` or `Failed`.
#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum OpenEvent {
    Opening {
        path: String,
    },
    Building {
        bytes: u64,
        total_bytes: u64,
        messages: u64,
    },
    Ready {
        summary: OpenSummary,
    },
    Cancelled,
    Failed {
        message: String,
    },
}

/// What the UI needs once a file is open.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSummary {
    pub path: String,
    pub file_name: String,
    pub message_count: u64,
    pub labels: Vec<LabelCount>,
    pub index: IndexState,
    /// Identifies the current listing; rows of an older generation are stale.
    pub generation: u64,
    /// Rows in the current listing (every message right after an open).
    pub total: usize,
}

/// Whether the open reused the sidecar index or built it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "state"
)]
pub enum IndexState {
    Reused {
        built_at_unix: i64,
    },
    Built {
        build_secs: u64,
        parse_failures: u64,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LabelCount {
    pub name: String,
    pub count: u64,
}

/// The query behind the list; every field optional, so `{}` is "all mail".
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueryInput {
    pub text: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub after: Option<i64>,
    pub before: Option<i64>,
    pub labels: Vec<String>,
    pub has_attachment: Option<bool>,
    /// The last word is still being typed and matches as a prefix.
    pub prefix_last_word: bool,
    pub sort: SortInput,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SortInput {
    #[default]
    DateDesc,
    Relevance,
}

impl From<QueryInput> for SearchQuery {
    fn from(input: QueryInput) -> Self {
        SearchQuery {
            text: input.text,
            from: input.from,
            to: input.to,
            after: input.after,
            before: input.before,
            labels: input.labels,
            has_attachment: input.has_attachment,
            prefix_last_word: input.prefix_last_word,
            sort: match input.sort {
                SortInput::DateDesc => Sort::DateDesc,
                SortInput::Relevance => Sort::Relevance,
            },
            ..SearchQuery::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListInfo {
    pub generation: u64,
    pub total: usize,
    pub warnings: Vec<String>,
}

/// One line of the message list.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub offset: u64,
    pub len: u64,
    pub date: Option<i64>,
    pub from: Vec<String>,
    pub subject: Option<String>,
    pub has_attachment: bool,
    pub labels: Vec<String>,
}

impl From<Hit> for Row {
    fn from(hit: Hit) -> Self {
        Row {
            offset: hit.offset,
            len: hit.len,
            date: hit.date,
            from: hit.from,
            subject: hit.subject,
            has_attachment: hit.has_attachment,
            labels: hit.labels,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RowsPage {
    pub generation: u64,
    pub start: usize,
    pub total: usize,
    pub rows: Vec<Row>,
}

/// One message for the reader pane.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MessageView {
    pub offset: u64,
    pub len: u64,
    pub date: Option<i64>,
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: Option<String>,
    pub labels: Vec<String>,
    pub message_id: Option<String>,
    /// The text view of the body: the plain part, else the HTML reduced to text.
    pub body: Option<BodyView>,
    /// The first HTML part as written; the webview sanitizes it before rendering.
    pub html: Option<String>,
    /// The attachments, for the strip and for the cid images of the HTML.
    pub attachments: Vec<AttachmentView>,
    pub parts: Vec<PartView>,
}

/// One attachment of the message; `index` addresses the part in the
/// attachment commands.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentView {
    pub index: usize,
    pub name: Option<String>,
    pub content_type: String,
    pub content_id: Option<String>,
    pub size: usize,
    pub inline: bool,
    pub is_message: bool,
    /// False for an executable: it can be saved, not opened.
    pub can_open: bool,
}

impl From<Attachment> for AttachmentView {
    fn from(attachment: Attachment) -> Self {
        let file_name = safe_file_name(
            attachment.name.as_deref(),
            &attachment.content_type,
            attachment.index,
        );
        AttachmentView {
            can_open: !open_blocked(&file_name),
            index: attachment.index,
            name: attachment.name,
            content_type: attachment.content_type,
            content_id: attachment.content_id,
            size: attachment.size,
            inline: attachment.inline,
            is_message: attachment.is_message,
        }
    }
}

/// The outcome of "Save all": where and how many.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveAllReport {
    pub dir: String,
    pub saved: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BodyView {
    pub kind: BodyKind,
    pub text: String,
}

/// How the text of the body was obtained.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BodyKind {
    /// A `text/plain` part, mboxrd `>From ` quoting undone.
    Plain,
    /// The text of an HTML part, for a message with no plain part.
    HtmlAsText,
    /// The raw bytes, lossily decoded: the message did not parse at all.
    Raw,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PartView {
    pub index: usize,
    pub depth: usize,
    pub content_type: String,
    pub encoding: Option<String>,
    pub disposition: Option<String>,
    pub name: Option<String>,
    pub content_id: Option<String>,
    pub size: usize,
}

impl From<PartInfo> for PartView {
    fn from(part: PartInfo) -> Self {
        PartView {
            index: part.index,
            depth: part.depth,
            content_type: part.content_type,
            encoding: part.encoding,
            disposition: part.disposition,
            name: part.name,
            content_id: part.content_id,
            size: part.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_query_input_is_the_default_search() {
        let query: SearchQuery = QueryInput::default().into();
        assert_eq!(query, SearchQuery::default());
        let json: QueryInput = serde_json::from_str("{}").unwrap();
        assert_eq!(json, QueryInput::default());
        let json: QueryInput =
            serde_json::from_str(r#"{"labels":["Inbox"],"hasAttachment":true,"sort":"relevance"}"#)
                .unwrap();
        let query: SearchQuery = json.into();
        assert_eq!(query.labels, vec!["Inbox".to_owned()]);
        assert_eq!(query.has_attachment, Some(true));
        assert_eq!(query.sort, Sort::Relevance);
    }

    #[test]
    fn the_full_query_input_shape_maps_onto_a_search_query() {
        let json: QueryInput = serde_json::from_str(
            r#"{"text":"számla","from":"anna","after":1700000000,"before":1700086400,
                "labels":["Inbox"],"hasAttachment":false,"prefixLastWord":true,"sort":"dateDesc"}"#,
        )
        .unwrap();
        let query: SearchQuery = json.into();
        assert!(query.prefix_last_word);
        assert!(!SearchQuery::from(QueryInput::default()).prefix_last_word);
        assert_eq!(query.text.as_deref(), Some("számla"));
        assert_eq!(query.from.as_deref(), Some("anna"));
        assert_eq!(query.to, None);
        assert_eq!(query.after, Some(1_700_000_000));
        assert_eq!(query.before, Some(1_700_086_400));
        assert_eq!(query.labels, vec!["Inbox".to_owned()]);
        assert_eq!(query.has_attachment, Some(false));
        assert_eq!(query.sort, Sort::DateDesc);
        // The CLI-only paging fields keep their defaults.
        assert_eq!(query.limit, SearchQuery::default().limit);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn open_events_are_tagged_for_the_webview() {
        let building = serde_json::to_value(OpenEvent::Building {
            bytes: 1,
            total_bytes: 2,
            messages: 3,
        })
        .unwrap();
        assert_eq!(
            building,
            serde_json::json!({"event": "building", "data": {"bytes": 1, "totalBytes": 2, "messages": 3}})
        );
        let cancelled = serde_json::to_value(OpenEvent::Cancelled).unwrap();
        assert_eq!(cancelled, serde_json::json!({"event": "cancelled"}));
        let reused = serde_json::to_value(IndexState::Reused { built_at_unix: 7 }).unwrap();
        assert_eq!(
            reused,
            serde_json::json!({"state": "reused", "builtAtUnix": 7})
        );
    }

    #[test]
    fn attachments_cross_the_bridge_camel_cased() {
        let view = AttachmentView::from(Attachment {
            index: 4,
            name: Some("a.pdf".to_owned()),
            content_type: "application/pdf".to_owned(),
            content_id: None,
            size: 12,
            inline: false,
            is_message: false,
        });
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            serde_json::json!({
                "index": 4, "name": "a.pdf", "contentType": "application/pdf",
                "contentId": null, "size": 12, "inline": false, "isMessage": false,
                "canOpen": true
            })
        );
        let exe = AttachmentView::from(Attachment {
            index: 5,
            name: Some("Report.EXE".to_owned()),
            content_type: "application/octet-stream".to_owned(),
            content_id: None,
            size: 1,
            inline: false,
            is_message: false,
        });
        assert!(!exe.can_open);
        let report = SaveAllReport {
            dir: "C:\\x".to_owned(),
            saved: 2,
        };
        assert_eq!(
            serde_json::to_value(&report).unwrap(),
            serde_json::json!({"dir": "C:\\x", "saved": 2})
        );
    }
}
