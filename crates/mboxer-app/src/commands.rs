//! The commands the webview invokes. Errors cross the bridge as strings.
//!
//! The state mutex is never held across an `.await`: each command locks,
//! copies what it needs, unlocks, then does the slow part.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use mboxer_core::{Error, Progress, SearchQuery, eml_bytes, parse, read_at, summarize};
use tauri::ipc::{Channel, Response};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::attachments::{
    eml_file_name, extension_of, open_blocked, read_attachment, read_listed_attachments,
    unique_path, write_temp, write_with_motw,
};
use crate::state::{AppState, OpenMbox};
use crate::types::{
    ListInfo, MessageView, OpenEvent, PositionInfo, QueryInput, RowsPage, SaveAllReport,
};
use crate::view::read_message_view;

/// The largest page `list_rows` serves.
const MAX_ROWS: usize = 500;

/// The path given on the command line, opened at start-up by the webview.
#[tauri::command]
pub fn initial_path() -> Option<String> {
    std::env::args().nth(1)
}

/// The version of the exe (the workspace `Cargo.toml`, stamped by `tauri-build`).
#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Opens `path`, or the file the user picks when `path` is `None`. Progress
/// and the outcome arrive on `on_event`; the `Err` is only for a second open
/// while one is running.
#[tauri::command]
pub async fn open_mbox(
    app: AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
    group_threads: bool,
    on_event: Channel<OpenEvent>,
) -> Result<(), String> {
    if state
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("an open is already in progress".to_owned());
    }
    let result = open_mbox_inner(app, &state, path, group_threads, on_event).await;
    state.busy.store(false, Ordering::Release);
    result
}

async fn open_mbox_inner(
    app: AppHandle,
    state: &AppState,
    path: Option<String>,
    group_threads: bool,
    on_event: Channel<OpenEvent>,
) -> Result<(), String> {
    let path = match path {
        Some(path) => PathBuf::from(path),
        None => match pick_file(app).await? {
            Some(path) => path,
            None => return Ok(()),
        },
    };
    let _ = on_event.send(OpenEvent::Opening {
        path: path.display().to_string(),
    });

    let cancel = Arc::clone(&state.cancel);
    cancel.store(false, Ordering::Release);
    let started = std::time::Instant::now();
    let generation = state.generation.fetch_add(1, Ordering::AcqRel) + 1;
    let progress = on_event.clone();
    let loaded = tauri::async_runtime::spawn_blocking(move || {
        OpenMbox::load(&path, &cancel, generation, group_threads, |p: Progress| {
            let _ = progress.send(OpenEvent::Building {
                bytes: p.bytes,
                total_bytes: p.total_bytes,
                messages: p.messages,
            });
        })
    })
    .await;

    let event = match loaded {
        Ok(Ok(open)) => {
            let summary = open.summary();
            eprintln!(
                "opened {}: {} messages, {:?}, {} ms",
                summary.file_name,
                summary.message_count,
                summary.index,
                started.elapsed().as_millis()
            );
            *state.open() = Some(open);
            OpenEvent::Ready { summary }
        }
        Ok(Err(Error::Cancelled)) => {
            eprintln!("open cancelled");
            OpenEvent::Cancelled
        }
        Ok(Err(err)) => {
            eprintln!("open failed: {err}");
            OpenEvent::Failed {
                message: err.to_string(),
            }
        }
        Err(err) => OpenEvent::Failed {
            message: format!("the open task failed: {err}"),
        },
    };
    let _ = on_event.send(event);
    Ok(())
}

/// The native open dialog, off the async runtime's threads.
async fn pick_file(app: AppHandle) -> Result<Option<PathBuf>, String> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Open mbox")
            .add_filter("mbox archives", &["mbox"])
            .add_filter("All files", &["*"])
            .blocking_pick_file()
    })
    .await
    .map_err(|err| format!("the file dialog failed: {err}"))?;
    match picked {
        Some(file) => file
            .into_path()
            .map(Some)
            .map_err(|err| format!("unusable path from the dialog: {err}")),
        None => Ok(None),
    }
}

/// Stops the index build of the open in progress, if any.
#[tauri::command]
pub fn cancel_open(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Release);
}

/// Replaces the listing behind the message list. The search runs off the
/// main thread: a sync command would block the webview while a relevance
/// query scores every message of a large archive.
#[tauri::command]
pub async fn set_query(app: AppHandle, query: QueryInput) -> Result<ListInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let started = std::time::Instant::now();
        let generation = state.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let mut guard = state.open();
        let open = guard.as_mut().ok_or("no mbox is open")?;
        let query: SearchQuery = query.into();
        open.set_query(query, generation)
            .map_err(|err| err.to_string())?;
        eprintln!(
            "query: {} hits, {} ms",
            open.listing.len(),
            started.elapsed().as_millis()
        );
        Ok(list_info(open))
    })
    .await
    .map_err(|err| format!("the query task failed: {err}"))?
}

fn list_info(open: &OpenMbox) -> ListInfo {
    ListInfo {
        generation: open.generation,
        total: open.listing.len(),
        messages: open.listing.message_count(),
        conversations: open.listing.group_count(),
        warnings: open.listing.warnings().to_vec(),
    }
}

/// Expands or collapses the conversation whose thread row is `row`. The
/// listing must still be the one the webview sees (`generation`); any other
/// row is a no-op that returns the unchanged counts.
#[tauri::command]
pub fn toggle_thread(
    state: State<'_, AppState>,
    generation: u64,
    row: usize,
) -> Result<ListInfo, String> {
    let mut guard = state.open();
    let open = guard.as_mut().ok_or("no mbox is open")?;
    if open.generation != generation {
        return Err("the listing has changed".to_owned());
    }
    open.listing.toggle(row);
    Ok(list_info(open))
}

/// One page of the current listing.
#[tauri::command]
pub fn list_rows(
    state: State<'_, AppState>,
    start: usize,
    count: usize,
) -> Result<RowsPage, String> {
    let guard = state.open();
    let open = guard.as_ref().ok_or("no mbox is open")?;
    let rows = open
        .listing
        .rows(start, count.min(MAX_ROWS))
        .map_err(|err| err.to_string())?;
    Ok(RowsPage {
        generation: open.generation,
        start,
        total: open.listing.len(),
        rows: rows.into_iter().map(Into::into).collect(),
    })
}

/// The path of the open mbox, copied out from under the lock.
fn open_path(state: &AppState) -> Result<PathBuf, String> {
    let guard = state.open();
    Ok(guard.as_ref().ok_or("no mbox is open")?.path.clone())
}

/// Runs `work` off the runtime threads and flattens the join error.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|err| format!("the task failed: {err}"))?
}

/// The message at `offset`/`len` of the open mbox, read off the runtime threads.
#[tauri::command]
pub async fn read_message(
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<MessageView, String> {
    let path = open_path(&state)?;
    blocking(move || read_message_view(&path, offset, len).map_err(|err| err.to_string())).await
}

/// The source of the message at `offset`/`len`: the envelope line and the
/// raw bytes, lossily decoded, for the reader's Source view.
#[tauri::command]
pub async fn read_source(
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<String, String> {
    let path = open_path(&state)?;
    blocking(move || {
        read_at(&path, offset, len)
            .map(|raw| {
                let mut source = raw.envelope;
                source.push('\n');
                source.push_str(&String::from_utf8_lossy(&raw.bytes));
                source
            })
            .map_err(|err| err.to_string())
    })
    .await
}

/// Opens a link clicked in a message body with the system's default browser
/// or mail client. Only web and mail links get out; the sanitizer in the
/// webview has already dropped the rest, this is the second check.
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    if !allowed_link(&url) {
        return Err("only http, https and mailto links are opened".to_owned());
    }
    blocking(move || {
        app.opener()
            .open_url(&url, None::<&str>)
            .map_err(|err| err.to_string())
    })
    .await
}

/// `http:`, `https:` and `mailto:` links, whatever their case.
pub fn allowed_link(url: &str) -> bool {
    let lower = url.trim_start().to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
}

/// Row index of the message with this offset in the current listing; a
/// member hidden in a collapsed conversation gets it expanded, so `total`
/// comes back with the position.
#[tauri::command]
pub fn position_of(state: State<'_, AppState>, offset: u64) -> Result<PositionInfo, String> {
    let mut guard = state.open();
    let open = guard.as_mut().ok_or("no mbox is open")?;
    let position = open
        .listing
        .position_of(offset)
        .map_err(|err| err.to_string())?;
    Ok(PositionInfo {
        generation: open.generation,
        position,
        total: open.listing.len(),
    })
}

/// The decoded bytes of attachment part `index` of the message at
/// `offset`/`len`, raw over the bridge: the webview turns a cid-referenced
/// image into a blob URL for the sandboxed body frame.
#[tauri::command]
pub async fn read_part(
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<Response, String> {
    let path = open_path(&state)?;
    let attachment = blocking(move || read_attachment(&path, offset, len, index)).await?;
    Ok(Response::new(attachment.bytes))
}

/// Writes the attachment to a temporary file and opens it with the default
/// application. Executable files are refused: they can only be saved.
#[tauri::command]
pub async fn open_attachment(
    app: AppHandle,
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<(), String> {
    let path = open_path(&state)?;
    blocking(move || {
        let attachment = read_attachment(&path, offset, len, index)?;
        if open_blocked(&attachment.file_name) {
            return Err(format!(
                "{} is an executable file: save it instead of opening it",
                attachment.file_name
            ));
        }
        let file = write_temp(offset, index, &attachment)
            .map_err(|err| format!("could not write the temporary file: {err}"))?;
        app.opener()
            .open_path(file.to_string_lossy(), None::<&str>)
            .map_err(|err| err.to_string())
    })
    .await
}

/// Saves one attachment where the user picks. `None` when the dialog is
/// cancelled; the saved path otherwise.
#[tauri::command]
pub async fn save_attachment(
    app: AppHandle,
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<Option<String>, String> {
    let path = open_path(&state)?;
    blocking(move || {
        let attachment = read_attachment(&path, offset, len, index)?;
        let mut dialog = save_dialog(&app, "Save attachment").set_file_name(&attachment.file_name);
        if let Some(ext) = extension_of(&attachment.file_name) {
            dialog = dialog.add_filter(format!("{} files", ext.to_ascii_uppercase()), &[&ext]);
        }
        let Some(picked) = dialog.add_filter("All files", &["*"]).blocking_save_file() else {
            return Ok(None);
        };
        let target = picked
            .into_path()
            .map_err(|err| format!("unusable path from the dialog: {err}"))?;
        write_with_motw(&target, &attachment.bytes)
            .map_err(|err| format!("could not write {}: {err}", target.display()))?;
        remember_dir(&app, target.parent());
        Ok(Some(target.display().to_string()))
    })
    .await
}

/// Saves every listed attachment of the message into a folder the user
/// picks, never overwriting (` (2)` and so on). `None` when cancelled.
#[tauri::command]
pub async fn save_all_attachments(
    app: AppHandle,
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<Option<SaveAllReport>, String> {
    let path = open_path(&state)?;
    blocking(move || {
        let attachments = read_listed_attachments(&path, offset, len)?;
        if attachments.is_empty() {
            return Err("the message has no attachments".to_owned());
        }
        let Some(picked) = save_dialog(&app, "Save all attachments").blocking_pick_folder() else {
            return Ok(None);
        };
        let dir = picked
            .into_path()
            .map_err(|err| format!("unusable path from the dialog: {err}"))?;
        let mut saved = 0;
        for attachment in &attachments {
            let target = unique_path(&dir, &attachment.file_name);
            write_with_motw(&target, &attachment.bytes)
                .map_err(|err| format!("could not write {}: {err}", target.display()))?;
            saved += 1;
        }
        remember_dir(&app, Some(&dir));
        Ok(Some(SaveAllReport {
            dir: dir.display().to_string(),
            saved,
        }))
    })
    .await
}

/// Saves the message at `offset`/`len` as an `.eml` file where the user picks:
/// the raw bytes with the mboxrd quoting undone (task 011). `None` when the
/// dialog is cancelled; the saved path otherwise.
#[tauri::command]
pub async fn save_message_eml(
    app: AppHandle,
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<Option<String>, String> {
    let path = open_path(&state)?;
    blocking(move || {
        let raw = read_at(&path, offset, len).map_err(|err| err.to_string())?;
        let subject = parse(&raw).and_then(|message| summarize(&raw, &message).subject);
        let file_name = eml_file_name(subject.as_deref(), offset);
        let Some(picked) = save_dialog(&app, "Save message as .eml")
            .set_file_name(&file_name)
            .add_filter("EML files", &["eml"])
            .add_filter("All files", &["*"])
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let target = picked
            .into_path()
            .map_err(|err| format!("unusable path from the dialog: {err}"))?;
        write_with_motw(&target, &eml_bytes(&raw))
            .map_err(|err| format!("could not write {}: {err}", target.display()))?;
        remember_dir(&app, target.parent());
        Ok(Some(target.display().to_string()))
    })
    .await
}

/// A file dialog that starts in the folder of the last save.
fn save_dialog(app: &AppHandle, title: &str) -> FileDialogBuilder<tauri::Wry> {
    let mut dialog = app.dialog().file().set_title(title);
    if let Some(dir) = app.state::<AppState>().last_save_dir() {
        dialog = dialog.set_directory(dir);
    }
    dialog
}

fn remember_dir(app: &AppHandle, dir: Option<&std::path::Path>) {
    if let Some(dir) = dir {
        app.state::<AppState>().set_last_save_dir(dir.to_path_buf());
    }
}

#[cfg(test)]
mod tests {
    use super::allowed_link;

    #[test]
    fn only_web_and_mail_links_are_opened() {
        assert!(allowed_link("https://example.com/x?y=1"));
        assert!(allowed_link("http://example.com"));
        assert!(allowed_link("HTTPS://EXAMPLE.COM"));
        assert!(allowed_link("mailto:anna@example.com?subject=hi"));
        assert!(allowed_link("  https://example.com"));
        assert!(!allowed_link("javascript:alert(1)"));
        assert!(!allowed_link("file:///C:/Windows"));
        assert!(!allowed_link("ftp://example.com"));
        assert!(!allowed_link("httpss://example.com"));
        assert!(!allowed_link("//example.com"));
        assert!(!allowed_link(""));
    }
}
