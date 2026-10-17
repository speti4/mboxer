//! The IPC types: what the webview sends and receives, camel-cased on the wire.

use mboxer_core::{Hit, PartInfo, SearchQuery, Sort};
use serde::{Deserialize, Serialize};

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
    pub body: Option<BodyView>,
    pub parts: Vec<PartView>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BodyView {
    pub kind: BodyKind,
    pub text: String,
}

/// How the body text was obtained; task 007 adds a sanitized `html` kind.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BodyKind {
    /// A `text/plain` part.
    Plain,
    /// The text of an HTML part, for a message with no plain part.
    HtmlAsText,
    /// The raw bytes, lossily decoded: the message did not parse at all.
    Raw,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PartView {
    pub depth: usize,
    pub content_type: String,
    pub encoding: Option<String>,
    pub name: Option<String>,
    pub size: usize,
}

impl From<PartInfo> for PartView {
    fn from(part: PartInfo) -> Self {
        PartView {
            depth: part.depth,
            content_type: part.content_type,
            encoding: part.encoding,
            name: part.name,
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
}
