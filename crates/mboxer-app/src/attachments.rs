//! Attachments on disk: the decoded bytes of a part, a file name Windows
//! accepts, the temporary folder behind "open", Mark-of-the-Web (Windows) or
//! the quarantine attribute (macOS) on every file written. No Tauri here; the
//! dialogs and the opener live in `commands`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mboxer_core::{Attachment, attachments, parse, part_at, part_bytes, read_at};

/// The longest file name written, in characters (Windows allows 255 bytes of
/// path; the folder takes its share).
const MAX_NAME_CHARS: usize = 150;

/// Extensions Windows or macOS runs when "opened": those attachments are saved,
/// never opened. One list for both systems: an attachment is as dangerous on
/// the machine it is forwarded to as on this one.
#[rustfmt::skip]
const BLOCKED_EXTENSIONS: &[&str] = &[
    // Windows
    "exe", "bat", "cmd", "com", "scr", "pif", "msi", "msp", "ps1", "psm1", "vbs", "vbe", "js",
    "jse", "wsf", "wsh", "hta", "lnk", "reg", "jar", "cpl", "inf", "url", "application", "gadget",
    "msc", "appx",
    // macOS: shell and Automator scripts, installers, disk images, bundles, Terminal
    // settings and the internet-location files that open a URL on a double-click.
    "command", "tool", "pkg", "mpkg", "dmg", "app", "sh", "scpt", "applescript", "workflow",
    "terminal", "webloc", "inetloc", "fileloc",
];

/// The macOS half of [`BLOCKED_EXTENSIONS`], for the test that each is blocked.
#[cfg(test)]
#[rustfmt::skip]
const MACOS_BLOCKED_EXTENSIONS: &[&str] = &[
    "command", "tool", "pkg", "mpkg", "dmg", "app", "sh", "scpt", "applescript", "workflow",
    "terminal", "webloc", "inetloc", "fileloc",
];

/// Device names a file may not be called, whatever its extension.
#[rustfmt::skip]
const RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// A decoded attachment ready to be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentBytes {
    /// A name safe for the file system (see [`safe_file_name`]).
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// Reads the message at `offset`/`len` of `mbox` and decodes the attachment at
/// part `index`. A part that is not an attachment (a body) is refused.
pub fn read_attachment(
    mbox: &Path,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<AttachmentBytes, String> {
    let raw = read_at(mbox, offset, len).map_err(|err| err.to_string())?;
    let message = parse(&raw).ok_or("the message does not parse")?;
    let attachment = attachments(&message)
        .into_iter()
        .find(|attachment| attachment.index == index)
        .ok_or_else(|| format!("part {index} is not an attachment"))?;
    let part = part_at(&message, index).ok_or_else(|| format!("part {index} does not exist"))?;
    Ok(AttachmentBytes {
        file_name: safe_file_name(attachment.name.as_deref(), &attachment.content_type, index),
        content_type: attachment.content_type,
        bytes: part_bytes(part).into_owned(),
    })
}

/// Every attachment the reader's strip lists (named, or a nested message), decoded.
pub fn read_listed_attachments(
    mbox: &Path,
    offset: u64,
    len: u64,
) -> Result<Vec<AttachmentBytes>, String> {
    let raw = read_at(mbox, offset, len).map_err(|err| err.to_string())?;
    let message = parse(&raw).ok_or("the message does not parse")?;
    Ok(attachments(&message)
        .into_iter()
        .filter(is_listed)
        .filter_map(|attachment| {
            let part = part_at(&message, attachment.index)?;
            Some(AttachmentBytes {
                file_name: safe_file_name(
                    attachment.name.as_deref(),
                    &attachment.content_type,
                    attachment.index,
                ),
                content_type: attachment.content_type,
                bytes: part_bytes(part).into_owned(),
            })
        })
        .collect())
}

/// Whether the strip shows this attachment: it has a name, or it is a message.
pub fn is_listed(attachment: &Attachment) -> bool {
    attachment.name.is_some() || attachment.is_message
}

/// A file name Windows accepts, from the declared one: the last path component
/// only, forbidden characters replaced, trailing dots and spaces removed,
/// device names prefixed, the length capped; an empty name becomes
/// `attachment-<index>` with an extension from the content type.
pub fn safe_file_name(name: Option<&str>, content_type: &str, index: usize) -> String {
    let last = name
        .unwrap_or("")
        .rsplit(['/', '\\'])
        .find(|component| !component.trim().is_empty())
        .unwrap_or("");
    let mut cleaned: String = last
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '_'
            } else {
                ch
            }
        })
        .collect();
    cleaned = cleaned
        .trim_matches(|ch: char| ch == ' ' || ch == '.')
        .to_owned();
    if cleaned.is_empty() {
        cleaned = format!("attachment-{index}");
        if let Some(ext) = extension_for(content_type) {
            cleaned.push('.');
            cleaned.push_str(ext);
        }
    }
    let (stem, ext) = split_extension(&cleaned);
    let mut stem = stem.to_owned();
    if RESERVED_NAMES.contains(&stem.to_ascii_lowercase().as_str()) {
        stem.insert(0, '_');
    }
    let ext_len = ext.map_or(0, |ext| ext.chars().count() + 1);
    let room = MAX_NAME_CHARS.saturating_sub(ext_len).max(1);
    if stem.chars().count() > room {
        stem = stem.chars().take(room).collect();
    }
    match ext {
        Some(ext) => format!("{stem}.{ext}"),
        None => stem,
    }
}

/// The file name of a message saved as `.eml`: the subject made safe, or
/// `message-<offset>.eml` when there is none.
pub fn eml_file_name(subject: Option<&str>, offset: u64) -> String {
    let subject = subject.map(str::trim).filter(|s| !s.is_empty());
    match subject {
        Some(subject) => safe_file_name(Some(&format!("{subject}.eml")), "message/rfc822", 0),
        None => format!("message-{offset}.eml"),
    }
}

/// `(stem, extension)` of a file name; the extension is after the last dot
/// and never the whole name (`.gitignore` has none).
fn split_extension(name: &str) -> (&str, Option<&str>) {
    match name.rfind('.') {
        Some(dot) if dot > 0 && dot + 1 < name.len() => (&name[..dot], Some(&name[dot + 1..])),
        _ => (name, None),
    }
}

/// The extension of a file name, lower case.
pub fn extension_of(name: &str) -> Option<String> {
    split_extension(name).1.map(str::to_ascii_lowercase)
}

/// The usual extension of a content type, for parts that have no file name.
fn extension_for(content_type: &str) -> Option<&'static str> {
    Some(match content_type.to_ascii_lowercase().as_str() {
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        "application/zip" | "application/x-zip-compressed" => "zip",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/csv" => "csv",
        "text/calendar" => "ics",
        "application/json" => "json",
        "application/xml" | "text/xml" => "xml",
        "message/rfc822" => "eml",
        "application/msword" => "doc",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "docx",
        "application/vnd.ms-excel" => "xls",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => "xlsx",
        "application/vnd.ms-powerpoint" => "ppt",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => "pptx",
        _ => "bin",
    })
}

/// Whether opening this file with its default application would run it.
pub fn open_blocked(file_name: &str) -> bool {
    extension_of(file_name).is_some_and(|ext| BLOCKED_EXTENSIONS.contains(&ext.as_str()))
}

/// `dir/file_name`, or `dir/stem (2).ext`, `(3)`… when that exists already.
pub fn unique_path(dir: &Path, file_name: &str) -> PathBuf {
    let first = dir.join(file_name);
    if !first.exists() {
        return first;
    }
    let (stem, ext) = split_extension(file_name);
    for n in 2.. {
        let candidate = match ext {
            Some(ext) => dir.join(format!("{stem} ({n}).{ext}")),
            None => dir.join(format!("{stem} ({n})")),
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("the counter has no end")
}

/// Writes `bytes` to `path` and marks the file as coming from the internet,
/// as mail clients do: the Mark of the Web (zone 3) on Windows, so Office
/// opens it in Protected View; the `com.apple.quarantine` attribute on macOS,
/// so Gatekeeper and the "downloaded from the internet" prompt see it. The
/// mark is best effort; a file system without alternate streams or extended
/// attributes gets none.
pub fn write_with_motw(path: &Path, bytes: &[u8]) -> io::Result<()> {
    fs::write(path, bytes)?;
    mark_of_the_web(path);
    Ok(())
}

#[cfg(windows)]
fn mark_of_the_web(path: &Path) {
    let stream = format!("{}:Zone.Identifier", path.display());
    let _ = fs::write(stream, "[ZoneTransfer]\r\nZoneId=3\r\n");
}

/// The quarantine attribute a browser download carries: `flags;time;agent;id`,
/// written with the system's `xattr` tool rather than a crate or a libc
/// binding, as the one call is not worth a dependency (D-002). The attribute
/// value on its own is what LaunchServices reads; the id stays empty.
#[cfg(target_os = "macos")]
fn mark_of_the_web(path: &Path) {
    use std::process::{Command, Stdio};
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    let _ = Command::new("xattr")
        .arg("-w")
        .arg(QUARANTINE_ATTR)
        .arg(format!("0081;{seconds:08x};mboxer;"))
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(any(target_os = "macos", test))]
const QUARANTINE_ATTR: &str = "com.apple.quarantine";

#[cfg(not(any(windows, target_os = "macos")))]
fn mark_of_the_web(_path: &Path) {}

/// The folder every temporary attachment copy goes under.
pub fn temp_root() -> PathBuf {
    std::env::temp_dir().join("mboxer")
}

/// Writes the attachment under its own temporary folder (one per open file,
/// message and part, so equal names never clash) and returns the file's path.
pub fn write_temp(
    file: u32,
    offset: u64,
    index: usize,
    attachment: &AttachmentBytes,
) -> io::Result<PathBuf> {
    let dir = temp_root().join(format!("{file}-{offset}-{index}"));
    fs::create_dir_all(&dir)?;
    let path = dir.join(&attachment.file_name);
    write_with_motw(&path, &attachment.bytes)?;
    Ok(path)
}

/// Removes the temporary copies of earlier runs. A file still open in another
/// application stays; everything else goes.
pub fn clean_temp() {
    let Ok(entries) = fs::read_dir(temp_root()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let _ = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../mboxer-core/tests/fixtures")
            .join(name)
    }

    #[test]
    fn file_names_are_made_safe_for_windows() {
        assert_eq!(
            safe_file_name(Some("számla-2026-01.pdf"), "application/pdf", 0),
            "számla-2026-01.pdf"
        );
        assert_eq!(
            safe_file_name(Some(r"..\..\sub/evil.txt"), "text/plain", 0),
            "evil.txt"
        );
        assert_eq!(
            safe_file_name(Some("a<b>:c\"d|e?f*g.txt"), "text/plain", 0),
            "a_b__c_d_e_f_g.txt"
        );
        assert_eq!(
            safe_file_name(Some("trailing. . "), "text/plain", 0),
            "trailing"
        );
        assert_eq!(safe_file_name(Some("CON.txt"), "text/plain", 0), "_CON.txt");
        assert_eq!(safe_file_name(Some("nul"), "text/plain", 0), "_nul");
        assert_eq!(
            safe_file_name(Some("tab\there.txt"), "text/plain", 0),
            "tab_here.txt"
        );
        assert_eq!(
            safe_file_name(Some(".gitignore"), "text/plain", 0),
            "gitignore"
        );
        assert_eq!(safe_file_name(None, "image/png", 3), "attachment-3.png");
        assert_eq!(
            safe_file_name(Some("   "), "application/x-unknown", 7),
            "attachment-7.bin"
        );
        assert_eq!(
            safe_file_name(Some("/"), "message/rfc822", 1),
            "attachment-1.eml"
        );
        let long = format!("{}.docx", "x".repeat(300));
        let safe = safe_file_name(Some(&long), "application/octet-stream", 0);
        assert_eq!(safe.chars().count(), MAX_NAME_CHARS);
        assert!(safe.ends_with(".docx"));
    }

    #[test]
    fn eml_file_names_come_from_the_subject() {
        assert_eq!(eml_file_name(Some("Re: Számla"), 5), "Re_ Számla.eml");
        assert_eq!(eml_file_name(Some("  "), 5), "message-5.eml");
        assert_eq!(eml_file_name(None, 123), "message-123.eml");
        assert_eq!(eml_file_name(Some("a/b\\c"), 0), "c.eml");
        let long = eml_file_name(Some(&"x".repeat(300)), 0);
        assert_eq!(long.chars().count(), MAX_NAME_CHARS);
        assert!(long.ends_with(".eml"));
    }

    #[test]
    fn executables_are_blocked_from_opening() {
        assert!(open_blocked("report.exe"));
        assert!(open_blocked("REPORT.EXE"));
        assert!(open_blocked("run.ps1"));
        assert!(open_blocked("x.lnk"));
        assert!(open_blocked("macro.js"));
        assert!(!open_blocked("számla.pdf"));
        assert!(!open_blocked("photo.jpg"));
        assert!(!open_blocked("README"));
        assert!(!open_blocked("archive.exe.zip"));
        assert_eq!(extension_of("a.b.PDF").as_deref(), Some("pdf"));
        assert_eq!(extension_of("noext"), None);
    }

    #[test]
    fn macos_executable_types_are_blocked_from_opening() {
        for ext in MACOS_BLOCKED_EXTENSIONS {
            assert!(open_blocked(&format!("payload.{ext}")), "{ext}");
            assert!(
                open_blocked(&format!("PAYLOAD.{}", ext.to_uppercase())),
                "{ext}"
            );
            assert!(BLOCKED_EXTENSIONS.contains(ext), "{ext}");
        }
        // The harmless neighbours stay openable.
        assert!(!open_blocked("notes.txt"));
        assert!(!open_blocked("photo.heic"));
        assert!(!open_blocked("pages.zip"));
    }

    #[test]
    fn unique_paths_count_up_and_keep_the_extension() {
        let dir = tempfile::tempdir().unwrap();
        let first = unique_path(dir.path(), "report.pdf");
        assert_eq!(first, dir.path().join("report.pdf"));
        fs::write(&first, b"1").unwrap();
        let second = unique_path(dir.path(), "report.pdf");
        assert_eq!(second, dir.path().join("report (2).pdf"));
        fs::write(&second, b"2").unwrap();
        assert_eq!(
            unique_path(dir.path(), "report.pdf"),
            dir.path().join("report (3).pdf")
        );
        fs::write(dir.path().join("README"), b"").unwrap();
        assert_eq!(
            unique_path(dir.path(), "README"),
            dir.path().join("README (2)")
        );
    }

    #[test]
    fn written_files_carry_the_bytes_and_the_mark_of_the_web() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.bin");
        write_with_motw(&path, b"\x00\x01\x02").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"\x00\x01\x02");
        if cfg!(windows) {
            let zone = fs::read_to_string(format!("{}:Zone.Identifier", path.display())).unwrap();
            assert!(zone.contains("ZoneId=3"), "{zone}");
        }
        if cfg!(target_os = "macos") {
            let output = std::process::Command::new("xattr")
                .arg("-p")
                .arg(QUARANTINE_ATTR)
                .arg(&path)
                .output()
                .unwrap();
            let value = String::from_utf8_lossy(&output.stdout);
            assert!(value.contains(";mboxer;"), "{value}");
        }
    }

    #[test]
    fn attachments_are_read_from_the_fixture() {
        let mbox = fixture("attachments.mbox");
        let raw = mboxer_core::MboxReader::open(&mbox)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let listed = read_listed_attachments(&mbox, raw.offset, raw.len).unwrap();
        let names: Vec<&str> = listed.iter().map(|a| a.file_name.as_str()).collect();
        assert_eq!(
            names,
            [
                "logo.png",
                "osszesito.csv",
                "számla-2026-01.pdf",
                "Forwarded original.eml",
                "report.exe",
                "evil.txt"
            ]
        );
        let message = parse(&raw).unwrap();
        let csv = attachments(&message)
            .into_iter()
            .find(|a| a.name.as_deref() == Some("osszesito.csv"))
            .unwrap();
        let one = read_attachment(&mbox, raw.offset, raw.len, csv.index).unwrap();
        assert_eq!(one.content_type, "text/csv");
        assert!(one.bytes.starts_with(b"N\xe9v;"));
        // The body parts are not attachments.
        let err = read_attachment(&mbox, raw.offset, raw.len, 0).unwrap_err();
        assert!(err.contains("not an attachment"), "{err}");
        let err = read_attachment(&mbox, raw.offset, raw.len, 999).unwrap_err();
        assert!(err.contains("not an attachment"), "{err}");
    }

    #[test]
    fn temp_copies_go_under_their_own_folder_and_are_cleaned() {
        let attachment = AttachmentBytes {
            file_name: "x.txt".to_owned(),
            content_type: "text/plain".to_owned(),
            bytes: b"hello".to_vec(),
        };
        let path = write_temp(1, u64::MAX, 7, &attachment).unwrap();
        assert!(path.starts_with(temp_root()));
        assert_eq!(fs::read(&path).unwrap(), b"hello");
        assert_eq!(write_temp(1, u64::MAX, 7, &attachment).unwrap(), path);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
