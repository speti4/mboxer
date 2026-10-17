//! End-to-end tests over the synthetic fixtures in `tests/fixtures/`.

use std::path::PathBuf;

use mboxer_core::{
    Error, MboxReader, RawMessage, TextKind, is_envelope, parse, part_tree, read_at, summarize,
    text_parts,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn read_all(name: &str) -> Vec<RawMessage> {
    MboxReader::open(fixture(name))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

/// Offsets and lengths tile the file exactly, and every range starts with an envelope line.
fn assert_contiguous(name: &str, messages: &[RawMessage]) {
    let file = std::fs::read(fixture(name)).unwrap();
    let mut expected_offset = 0;
    for m in messages {
        assert_eq!(
            m.offset, expected_offset,
            "{name}: gap before offset {}",
            m.offset
        );
        let range = &file[m.offset as usize..m.end() as usize];
        let line_end = range
            .iter()
            .position(|&b| b == b'\n')
            .map_or(range.len(), |i| i + 1);
        assert!(
            is_envelope(&range[..line_end]),
            "{name}: range at {} has no envelope",
            m.offset
        );
        assert_eq!(
            &range[line_end..],
            &m.bytes[..],
            "{name}: body mismatch at {}",
            m.offset
        );
        expected_offset = m.end();
    }
    assert_eq!(
        expected_offset,
        file.len() as u64,
        "{name}: ranges do not reach the end"
    );
}

#[test]
fn simple_lf_splits_on_envelopes_only() {
    let messages = read_all("simple-lf.mbox");
    assert_eq!(messages.len(), 3);
    assert_contiguous("simple-lf.mbox", &messages);
    assert_eq!(
        messages[0].envelope,
        "From alice@example.com Mon Jan  5 09:00:00 2026"
    );
    assert_eq!(
        messages[1].envelope,
        "From bob@example.com Tue Jan  6 10:00:00 +0100 2026"
    );
    assert_eq!(
        messages[2].envelope,
        "From MAILER-DAEMON Wed Jan  7 11:00:00 2026"
    );
    let body_two = String::from_utf8_lossy(&messages[1].bytes);
    assert!(body_two.contains("From Anna wrote: this line is not an envelope."));
    assert!(body_two.contains("From the top, neither is this one."));
    let m = parse(&messages[2]).unwrap();
    let s = summarize(&messages[2], &m);
    assert_eq!(s.subject.as_deref(), Some("three"));
    assert_eq!(
        s.from[0].address.as_deref(),
        Some("mailer-daemon@example.com")
    );
    assert!(s.to.is_empty());
    assert!(s.labels.is_empty());
    assert_eq!(s.gmail_thread_id, None);
}

#[test]
fn gmail_crlf_nested_mime_is_fully_decoded() {
    let messages = read_all("gmail-crlf.mbox");
    assert_eq!(messages.len(), 2);
    assert_contiguous("gmail-crlf.mbox", &messages);
    assert_eq!(
        messages[0].envelope,
        "From 1234567890123456789@xxx Tue Jan 13 06:30:04 +0000 2026"
    );

    let m = parse(&messages[0]).unwrap();
    let s = summarize(&messages[0], &m);
    assert_eq!(s.offset, 0);
    assert_eq!(s.len, messages[0].len);
    assert_eq!(s.message_id.as_deref(), Some("msg1@mail.example.com"));
    assert_eq!(s.gmail_thread_id, Some(1234567890123456789));
    assert_eq!(
        s.labels,
        ["Inbox", "Important", "Category personal", "Opened"]
    );
    // Tue, 13 Jan 2026 07:30:04 +0100 == 06:30:04 UTC
    assert_eq!(s.date_unix, Some(1_768_285_804));
    assert_eq!(s.from[0].name.as_deref(), Some("Anna Példa"));
    assert_eq!(s.from[0].address.as_deref(), Some("anna@example.com"));
    assert_eq!(s.to[0].address.as_deref(), Some("bela@example.com"));
    assert_eq!(s.cc[0].address.as_deref(), Some("konyveles@example.com"));
    assert_eq!(s.subject.as_deref(), Some("Számla január"));
    assert!(s.has_attachment);
    assert_eq!(s.attachment_names, ["számla-2026-01.pdf"]);

    let texts = text_parts(&m);
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].kind, TextKind::Plain);
    assert!(texts[0].text.contains("Kedves Béla,"));
    assert!(
        texts[0]
            .text
            .contains(">From a könyvelés felől jött a számla.")
    );
    assert!(texts[0].text.contains("From Anna wrote: ez nem envelope."));
    assert_eq!(texts[1].kind, TextKind::Html);
    assert!(
        texts[1]
            .text
            .contains("<p>Kedves Béla, a számla mellékelve.</p>")
    );

    let parts = part_tree(&m);
    let tree: Vec<(usize, &str)> = parts
        .iter()
        .map(|p| (p.depth, p.content_type.as_str()))
        .collect();
    assert_eq!(
        tree,
        [
            (0, "multipart/mixed"),
            (1, "multipart/alternative"),
            (2, "text/plain"),
            (2, "text/html"),
            (1, "application/pdf"),
        ]
    );
    assert_eq!(parts[2].encoding.as_deref(), Some("quoted-printable"));
    assert_eq!(parts[3].encoding.as_deref(), Some("base64"));
    assert_eq!(parts[4].name.as_deref(), Some("számla-2026-01.pdf"));

    let m2 = parse(&messages[1]).unwrap();
    let s2 = summarize(&messages[1], &m2);
    assert_eq!(s2.gmail_thread_id, Some(9876543210987654321));
    assert_eq!(s2.labels, ["Sent", "Opened"]);
    assert_eq!(s2.from[0].name.as_deref(), Some("Péter"));
    assert!(!s2.has_attachment);
    assert!(s2.attachment_names.is_empty());
    let texts2 = text_parts(&m2);
    assert_eq!(texts2.len(), 1);
    assert_eq!(texts2[0].text.trim(), "Rendben, köszönöm!");
}

#[test]
fn hungarian_legacy_charsets_decode() {
    let messages = read_all("hungarian-charsets.mbox");
    assert_eq!(messages.len(), 1);
    let m = parse(&messages[0]).unwrap();
    let s = summarize(&messages[0], &m);
    assert_eq!(s.subject.as_deref(), Some("Számla árvíztűrő"));
    assert_eq!(s.from[0].name.as_deref(), Some("Péter Szabó"));
    assert_eq!(s.to[0].name.as_deref(), Some("Anna Példa"));
    let texts = text_parts(&m);
    assert_eq!(texts[0].text.trim(), "árvíztűrő tükörfúrógép");
}

#[test]
fn last_message_without_trailing_newline_is_complete() {
    let messages = read_all("no-trailing-newline.mbox");
    assert_eq!(messages.len(), 1);
    assert_contiguous("no-trailing-newline.mbox", &messages);
    assert!(messages[0].bytes.ends_with(b"\ntail"));
}

#[test]
fn empty_file_has_no_messages() {
    assert!(read_all("empty.mbox").is_empty());
}

#[test]
fn text_file_is_rejected_at_offset_zero() {
    let err = MboxReader::open(fixture("not-an-mbox.txt"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap_err();
    assert!(matches!(err, Error::NotAnMbox { offset: 0 }), "{err}");
}

#[test]
fn read_at_reproduces_every_streamed_message() {
    for name in [
        "simple-lf.mbox",
        "gmail-crlf.mbox",
        "hungarian-charsets.mbox",
        "no-trailing-newline.mbox",
    ] {
        let path = fixture(name);
        for streamed in read_all(name) {
            let seeked = read_at(&path, streamed.offset, streamed.len).unwrap();
            assert_eq!(seeked, streamed, "{name} at {}", streamed.offset);
        }
    }
}
