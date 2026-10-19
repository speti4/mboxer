//! The IPC types: what the webview sends and receives, camel-cased on the wire.

use mboxer_core::{Attachment, ListRow, PartInfo, RowKind, SearchQuery, Sort};
use serde::{Deserialize, Serialize};

use crate::attachments::{open_blocked, safe_file_name};

/// Progress of `open_mbox`, streamed over a channel. Every file of the batch
/// gets `Opening`, then `Building` while its index is built, then `Ready` or
/// `Failed`; `Cancelled` ends the batch early; `Done` is always the last event.
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
        /// 1-based place of this file in the batch, and the batch size.
        position: usize,
        count: usize,
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
        path: String,
        message: String,
    },
    Done,
}

/// One row of the file list in the folder pane.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    /// The id the commands take; valid for this process only (D-006).
    pub id: u32,
    pub path: String,
    pub file_name: String,
    pub message_count: u64,
    pub index: IndexState,
    /// The file whose list, search and reader the webview shows.
    pub active: bool,
}

/// Whether the sidecar index of a file can be reused without a build.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum IndexStatusInfo {
    Fresh,
    Stale,
    Missing,
    /// The mbox itself is not there (moved, unplugged drive).
    NoFile,
}

/// What the UI needs once a file is open: the file itself and, from
/// `OpenFiles::summary`, the whole set for the pane.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSummary {
    pub id: u32,
    pub files: Vec<FileInfo>,
    pub path: String,
    pub file_name: String,
    pub message_count: u64,
    /// The label tree of the folder pane, in display order.
    pub labels: Vec<LabelNode>,
    pub index: IndexState,
    /// Where the index lives: beside the mbox, or under the index home when
    /// the mbox folder is not writable (task 014).
    pub index_dir: String,
    /// Identifies the current listing; rows of an older generation are stale.
    pub generation: u64,
    /// Rows in the current listing (every conversation right after an open).
    pub total: usize,
    /// Matching messages and conversations behind those rows.
    pub messages: usize,
    pub conversations: usize,
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

/// One node of the label tree: a Gmail label (`label` set) or a container
/// that only groups sub-labels (`label` absent: `Work` when only
/// `Work/Invoices` exists, or the synthetic `Categories` node).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LabelNode {
    /// The last path segment, for display.
    pub name: String,
    /// The exact label, when messages carry this node itself.
    pub label: Option<String>,
    /// Messages carrying this label itself.
    pub count: u64,
    /// Messages carrying this label or any label under it (not de-duplicated).
    pub total: u64,
    /// This label and every label under it: what a click on the node sends
    /// as `anyLabels`.
    pub labels: Vec<String>,
    pub children: Vec<LabelNode>,
}

/// The query behind the list; every field optional, so `{}` is "all mail".
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct QueryInput {
    pub text: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
    pub after: Option<i64>,
    pub before: Option<i64>,
    pub labels: Vec<String>,
    /// At least one of these labels: a node of the label tree and its sub-labels.
    pub any_labels: Vec<String>,
    pub has_attachment: Option<bool>,
    /// The last word is still being typed and matches as a prefix.
    pub prefix_last_word: bool,
    /// One row per conversation, expandable.
    pub group_threads: bool,
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
            subject: input.subject,
            after: input.after,
            before: input.before,
            labels: input.labels,
            any_labels: input.any_labels,
            has_attachment: input.has_attachment,
            prefix_last_word: input.prefix_last_word,
            group_threads: input.group_threads,
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
    /// Visible rows: conversations plus the members of the expanded ones.
    pub total: usize,
    /// Matching messages and conversations behind those rows.
    pub messages: usize,
    pub conversations: usize,
    pub warnings: Vec<String>,
}

/// Where a message sits in the current listing; `total` may have grown when
/// the lookup expanded the conversation around it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    pub generation: u64,
    pub position: Option<usize>,
    pub total: usize,
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
    pub kind: RowKindView,
    /// Matching messages of the conversation; 1 for a single message.
    pub thread_count: u32,
    /// A thread row that shows its members.
    pub expanded: bool,
    /// A member's position in its conversation, oldest first; the thread row
    /// is `threadIndex + 1` rows up.
    pub thread_index: u32,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RowKindView {
    Message,
    Thread,
    Member,
}

impl From<ListRow> for Row {
    fn from(row: ListRow) -> Self {
        let (kind, thread_count, expanded, thread_index) = match row.kind {
            RowKind::Message => (RowKindView::Message, 1, false, 0),
            RowKind::Thread { count, expanded } => (RowKindView::Thread, count, expanded, 0),
            RowKind::Member { index, count } => (RowKindView::Member, count, false, index),
        };
        let hit = row.hit;
        Row {
            offset: hit.offset,
            len: hit.len,
            date: hit.date,
            from: hit.from,
            subject: hit.subject,
            has_attachment: hit.has_attachment,
            labels: hit.labels,
            kind,
            thread_count,
            expanded,
            thread_index,
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
    /// The open file the message was read from; the attachment, source and
    /// save commands take it back.
    pub file: u32,
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
            r#"{"text":"számla","from":"anna","subject":"január","after":1700000000,"before":1700086400,
                "labels":["Inbox"],"hasAttachment":false,"prefixLastWord":true,"sort":"dateDesc"}"#,
        )
        .unwrap();
        let query: SearchQuery = json.into();
        assert!(query.prefix_last_word);
        assert!(!SearchQuery::from(QueryInput::default()).prefix_last_word);
        assert_eq!(query.text.as_deref(), Some("számla"));
        assert_eq!(query.from.as_deref(), Some("anna"));
        assert_eq!(query.to, None);
        assert_eq!(query.subject.as_deref(), Some("január"));
        assert_eq!(query.after, Some(1_700_000_000));
        assert_eq!(query.before, Some(1_700_086_400));
        assert_eq!(query.labels, vec!["Inbox".to_owned()]);
        assert_eq!(query.has_attachment, Some(false));
        assert_eq!(query.sort, Sort::DateDesc);
        assert!(!query.group_threads);
        assert!(query.any_labels.is_empty());
        // The CLI-only paging fields keep their defaults.
        assert_eq!(query.limit, SearchQuery::default().limit);
        assert_eq!(query.offset, 0);

        let json: QueryInput =
            serde_json::from_str(r#"{"anyLabels":["Work","Work/Invoices"],"groupThreads":true}"#)
                .unwrap();
        let query: SearchQuery = json.into();
        assert_eq!(query.any_labels, ["Work", "Work/Invoices"]);
        assert!(query.group_threads);
        assert!(query.labels.is_empty());
    }

    #[test]
    fn list_rows_carry_their_kind_camel_cased() {
        let hit = mboxer_core::Hit {
            offset: 10,
            len: 20,
            date: Some(5),
            from: vec!["A <a@x>".to_owned()],
            to: vec![],
            subject: Some("Re: x".to_owned()),
            labels: vec!["Inbox".to_owned()],
            has_attachment: false,
            attachment_names: vec![],
            thread_id: Some(9_000_000_000_000_000_000),
            score: None,
        };
        let thread = Row::from(ListRow {
            hit: hit.clone(),
            kind: RowKind::Thread {
                count: 3,
                expanded: true,
            },
        });
        let json = serde_json::to_value(&thread).unwrap();
        assert_eq!(json["kind"], "thread");
        assert_eq!(json["threadCount"], 3);
        assert_eq!(json["expanded"], true);
        assert_eq!(json["threadIndex"], 0);
        assert_eq!(json["offset"], 10);
        assert!(
            json.get("threadId").is_none(),
            "u64 ids do not fit a JS number"
        );

        let member = Row::from(ListRow {
            hit: hit.clone(),
            kind: RowKind::Member { index: 2, count: 3 },
        });
        assert_eq!(
            (member.kind, member.thread_index, member.thread_count),
            (RowKindView::Member, 2, 3)
        );
        assert!(!member.expanded);
        let message = Row::from(ListRow {
            hit,
            kind: RowKind::Message,
        });
        assert_eq!(
            (message.kind, message.thread_count),
            (RowKindView::Message, 1)
        );

        let info = ListInfo {
            generation: 3,
            total: 4,
            messages: 9,
            conversations: 4,
            warnings: vec![],
        };
        assert_eq!(
            serde_json::to_value(&info).unwrap(),
            serde_json::json!({"generation": 3, "total": 4, "messages": 9, "conversations": 4, "warnings": []})
        );
        let position = PositionInfo {
            generation: 3,
            position: Some(2),
            total: 7,
        };
        assert_eq!(
            serde_json::to_value(&position).unwrap(),
            serde_json::json!({"generation": 3, "position": 2, "total": 7})
        );
        let node = LabelNode {
            name: "Work".to_owned(),
            label: Some("Work".to_owned()),
            count: 2,
            total: 3,
            labels: vec!["Work".to_owned(), "Work/Invoices".to_owned()],
            children: vec![],
        };
        assert_eq!(
            serde_json::to_value(&node).unwrap(),
            serde_json::json!({"name": "Work", "label": "Work", "count": 2, "total": 3,
                "labels": ["Work", "Work/Invoices"], "children": []})
        );
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
        let done = serde_json::to_value(OpenEvent::Done).unwrap();
        assert_eq!(done, serde_json::json!({"event": "done"}));
        let opening = serde_json::to_value(OpenEvent::Opening {
            path: "a.mbox".to_owned(),
            position: 2,
            count: 3,
        })
        .unwrap();
        assert_eq!(
            opening,
            serde_json::json!({"event": "opening", "data": {"path": "a.mbox", "position": 2, "count": 3}})
        );
        let failed = serde_json::to_value(OpenEvent::Failed {
            path: "a.mbox".to_owned(),
            message: "no".to_owned(),
        })
        .unwrap();
        assert_eq!(
            failed,
            serde_json::json!({"event": "failed", "data": {"path": "a.mbox", "message": "no"}})
        );
        let info = FileInfo {
            id: 3,
            path: "D:\\m\\a.mbox".to_owned(),
            file_name: "a.mbox".to_owned(),
            message_count: 5,
            index: IndexState::Reused { built_at_unix: 7 },
            active: true,
        };
        assert_eq!(
            serde_json::to_value(&info).unwrap(),
            serde_json::json!({"id": 3, "path": "D:\\m\\a.mbox", "fileName": "a.mbox", "messageCount": 5,
                "index": {"state": "reused", "builtAtUnix": 7}, "active": true})
        );
        assert_eq!(
            serde_json::to_value(IndexStatusInfo::NoFile).unwrap(),
            serde_json::json!({"state": "noFile"})
        );
        assert_eq!(
            serde_json::to_value(IndexStatusInfo::Fresh).unwrap(),
            serde_json::json!({"state": "fresh"})
        );
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
