//! The commands the webview invokes. Errors cross the bridge as strings.
//!
//! The state mutex is never held across an `.await`: each command locks,
//! copies what it needs, unlocks, then does the slow part.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use mboxer_core::{IndexStatus, MboxIndex, SearchQuery, eml_bytes, parse, status, summarize};
use tauri::ipc::{Channel, Response};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::attachments::{
    eml_file_name, extension_of, open_blocked, read_attachment, read_listed_attachments,
    unique_path, write_temp, write_with_motw,
};
use crate::state::{AppState, BatchEnd};
use crate::types::{
    IndexStatusInfo, ListInfo, MessageView, OpenEvent, OpenSummary, PositionInfo, QueryInput,
    RowsPage, SaveAllReport,
};
use crate::view::read_message_view;

/// The largest page `list_rows` serves.
const MAX_ROWS: usize = 500;

/// The paths the webview opens at start-up: the one given on the command
/// line, then the files the system handed the app before the webview was up
/// (macOS: Finder "Open with", a Dock drop). From this call on a system open
/// reaches the webview as an [`OPEN_FILES_EVENT`] instead.
#[tauri::command]
pub fn initial_paths(state: State<'_, AppState>) -> Vec<String> {
    // `args_os`: `args` panics on an argument that is not valid Unicode; a
    // lossy path fails to open instead, with the reason on screen.
    let mut paths: Vec<String> = std::env::args_os()
        .skip(1)
        .take(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let mut pending = state.system_opens();
    pending.webview_ready = true;
    paths.append(&mut pending.paths);
    paths
}

/// The event that carries the paths of a system open to the webview, a list of strings.
pub const OPEN_FILES_EVENT: &str = "open-files";

/// Files the system asks the running app to open (`RunEvent::Opened`, macOS
/// only: Finder "Open with", a Dock drop, `open -a mboxer x.mbox`). A path
/// that is not a local file is ignored. Before the webview has asked for its
/// start-up paths the files wait with them; after that they go to the webview
/// as an [`OPEN_FILES_EVENT`], which opens them like a dialog pick.
#[cfg(target_os = "macos")]
pub fn open_from_system(app: &AppHandle, urls: &[tauri::Url]) {
    use tauri::Emitter;

    let paths: Vec<String> = urls
        .iter()
        .filter_map(|url| url.to_file_path().ok())
        .map(|path| path.display().to_string())
        .collect();
    if paths.is_empty() {
        return;
    }
    let state = app.state::<AppState>();
    let mut pending = state.system_opens();
    if pending.webview_ready {
        let _ = app.emit(OPEN_FILES_EVENT, paths);
    } else {
        pending.paths.extend(paths);
    }
}

/// The app's version (the workspace `Cargo.toml`, stamped by `tauri-build`).
#[tauri::command]
pub fn app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Adds `paths` to the open files, or the files the user picks when `paths`
/// is `None`, one after the other in file-name order. With `activate` the
/// first file of the batch becomes the active one; a file that is already
/// open is not loaded again, only re-listed (and activated). Progress and the
/// outcome of every file arrive on `on_event`; the `Err` is only for a second
/// open while one is running.
#[tauri::command]
pub async fn open_mbox(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Option<Vec<String>>,
    group_threads: bool,
    activate: bool,
    on_event: Channel<OpenEvent>,
) -> Result<(), String> {
    if state
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("an open is already in progress".to_owned());
    }
    let result = open_mbox_inner(app, paths, group_threads, activate, on_event).await;
    state.busy.store(false, Ordering::Release);
    result
}

/// The Tauri side of an open: the paths (given, or picked in the dialog), the
/// batch off the async runtime's threads, the closing events on the channel.
async fn open_mbox_inner(
    app: AppHandle,
    paths: Option<Vec<String>>,
    group_threads: bool,
    activate: bool,
    on_event: Channel<OpenEvent>,
) -> Result<(), String> {
    // Given paths keep their order (the webview puts the file to activate
    // first); a dialog pick comes in file-name order, as Takeout numbers its parts.
    let mut paths: Vec<PathBuf> = match paths {
        Some(paths) => paths.into_iter().map(PathBuf::from).collect(),
        None => match pick_files(app.clone()).await? {
            Some(mut paths) => {
                paths.sort_by_key(|path| {
                    (
                        path.file_name().map(|name| name.to_os_string()),
                        path.clone(),
                    )
                });
                paths
            }
            None => return Ok(()),
        },
    };
    paths.dedup();

    let events = on_event.clone();
    let end = tauri::async_runtime::spawn_blocking(move || {
        app.state::<AppState>()
            .open_batch(paths, group_threads, activate, |event| {
                let _ = events.send(event);
            })
    })
    .await
    .map_err(|err| format!("the open task failed: {err}"))?;
    if end == BatchEnd::Cancelled {
        let _ = on_event.send(OpenEvent::Cancelled);
    }
    let _ = on_event.send(OpenEvent::Done);
    Ok(())
}

/// The native open dialog (several files allowed), off the async runtime's threads.
async fn pick_files(app: AppHandle) -> Result<Option<Vec<PathBuf>>, String> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .set_title("Open mbox files")
            .add_filter("mbox archives", &["mbox"])
            .add_filter("All files", &["*"])
            .blocking_pick_files()
    })
    .await
    .map_err(|err| format!("the file dialog failed: {err}"))?;
    match picked {
        Some(files) => files
            .into_iter()
            .map(|file| {
                file.into_path()
                    .map_err(|err| format!("unusable path from the dialog: {err}"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        None => Ok(None),
    }
}

/// Stops the index build of the open in progress, if any.
#[tauri::command]
pub fn cancel_open(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Release);
}

/// Whether the index of the mbox at `path` can be reused as it is: the
/// webview reopens remembered files at start-up only when it can, and shows
/// the others as not loaded (D-006). Cheap: a fingerprint sample, no build.
#[tauri::command]
pub async fn index_status(path: String) -> Result<IndexStatusInfo, String> {
    blocking(move || {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Ok(IndexStatusInfo::NoFile);
        }
        match status(&path).map_err(|err| err.to_string())? {
            IndexStatus::Fresh(_) => Ok(IndexStatusInfo::Fresh),
            IndexStatus::Stale(_) => Ok(IndexStatusInfo::Stale),
            IndexStatus::Missing => Ok(IndexStatusInfo::Missing),
        }
    })
    .await
}

/// Makes `file` the one the list, the search and the reader show; its
/// listing gets a new generation. The summary carries the whole set.
#[tauri::command]
pub fn activate_file(state: State<'_, AppState>, file: u32) -> Result<OpenSummary, String> {
    let mut files = state.files();
    let generation = state.generation.fetch_add(1, Ordering::AcqRel) + 1;
    files.activate(file, generation)?;
    files.summary(file)
}

/// Removes `file` from the set. The summary of the file that is active
/// afterwards comes back; `None` when no file is left.
#[tauri::command]
pub fn close_file(state: State<'_, AppState>, file: u32) -> Result<Option<OpenSummary>, String> {
    let mut files = state.files();
    let generation = state.generation.fetch_add(1, Ordering::AcqRel) + 1;
    let was_active = files.active_id() == Some(file);
    let Some(active) = files.close(file)? else {
        return Ok(None);
    };
    if was_active {
        files.activate(active, generation)?;
    }
    files.summary(active).map(Some)
}

/// Replaces the listing behind the message list. The search runs off the
/// main thread and outside the files lock ([`AppState::set_query`]): the
/// sync commands that read the listing (`list_rows`, `position_of`,
/// `toggle_thread`, `activate_file`, `close_file`) keep answering from the
/// listing in place while a relevance query scores every message of a large
/// archive. Only the newest query of a file swaps its listing in; an
/// overtaken one answers `superseded`, and stops its search where it is when
/// the newer query comes in during it.
#[tauri::command]
pub async fn set_query(app: AppHandle, file: u32, query: QueryInput) -> Result<ListInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let query: SearchQuery = query.into();
        state.set_query(file, query)
    })
    .await
    .map_err(|err| format!("the query task failed: {err}"))?
}

/// Expands or collapses the conversation whose thread row is `row`. The
/// listing must still be the one the webview sees (`generation`); any other
/// row is a no-op that returns the unchanged counts.
#[tauri::command]
pub fn toggle_thread(
    state: State<'_, AppState>,
    file: u32,
    generation: u64,
    row: usize,
) -> Result<ListInfo, String> {
    let mut files = state.files();
    let open = files.by_id_mut(file)?;
    if open.generation != generation {
        return Err("the listing has changed".to_owned());
    }
    open.listing.toggle(row);
    Ok(open.list_info())
}

/// One page of the current listing.
#[tauri::command]
pub fn list_rows(
    state: State<'_, AppState>,
    file: u32,
    start: usize,
    count: usize,
) -> Result<RowsPage, String> {
    let files = state.files();
    let open = files.by_id(file)?;
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

/// The index of open file `file`, shared out from under the lock. Every read
/// of a message goes through [`MboxIndex::read_message`], so the size limit of
/// the build that listed the message holds (task 069).
fn file_index(state: &AppState, file: u32) -> Result<Arc<MboxIndex>, String> {
    Ok(Arc::clone(&state.files().by_id(file)?.index))
}

/// Runs `work` off the runtime threads and flattens the join error.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|err| format!("the task failed: {err}"))?
}

/// The message at `offset`/`len` of open file `file`, read off the runtime threads.
#[tauri::command]
pub async fn read_message(
    state: State<'_, AppState>,
    file: u32,
    offset: u64,
    len: u64,
) -> Result<MessageView, String> {
    let index = file_index(&state, file)?;
    blocking(move || {
        let mut view = read_message_view(&index, offset, len).map_err(|err| err.to_string())?;
        view.file = file;
        Ok(view)
    })
    .await
}

/// The source of the message at `offset`/`len`: the envelope line and the
/// raw bytes, lossily decoded, for the reader's Source view.
#[tauri::command]
pub async fn read_source(
    state: State<'_, AppState>,
    file: u32,
    offset: u64,
    len: u64,
) -> Result<String, String> {
    let index = file_index(&state, file)?;
    blocking(move || {
        index
            .read_message(offset, len)
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
pub fn position_of(
    state: State<'_, AppState>,
    file: u32,
    offset: u64,
) -> Result<PositionInfo, String> {
    let mut files = state.files();
    let open = files.by_id_mut(file)?;
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
    file: u32,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<Response, String> {
    let mbox_index = file_index(&state, file)?;
    let attachment = blocking(move || read_attachment(&mbox_index, offset, len, index)).await?;
    Ok(Response::new(attachment.bytes))
}

/// Writes the attachment to a temporary file and opens it with the default
/// application. Executable files are refused: they can only be saved.
#[tauri::command]
pub async fn open_attachment(
    app: AppHandle,
    state: State<'_, AppState>,
    file: u32,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<(), String> {
    let mbox_index = file_index(&state, file)?;
    blocking(move || {
        let attachment = read_attachment(&mbox_index, offset, len, index)?;
        if open_blocked(&attachment.file_name) {
            return Err(format!(
                "{} is an executable file: save it instead of opening it",
                attachment.file_name
            ));
        }
        let copy = write_temp(file, offset, index, &attachment)
            .map_err(|err| format!("could not write the temporary file: {err}"))?;
        app.opener()
            .open_path(copy.to_string_lossy(), None::<&str>)
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
    file: u32,
    offset: u64,
    len: u64,
    index: usize,
) -> Result<Option<String>, String> {
    let mbox_index = file_index(&state, file)?;
    blocking(move || {
        let attachment = read_attachment(&mbox_index, offset, len, index)?;
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
    file: u32,
    offset: u64,
    len: u64,
) -> Result<Option<SaveAllReport>, String> {
    let index = file_index(&state, file)?;
    blocking(move || {
        let attachments = read_listed_attachments(&index, offset, len)?;
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
    file: u32,
    offset: u64,
    len: u64,
) -> Result<Option<String>, String> {
    let index = file_index(&state, file)?;
    blocking(move || {
        let raw = index
            .read_message(offset, len)
            .map_err(|err| err.to_string())?;
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

/// Prints `html`, the finished print document of the reader's message, in a
/// print window of its own (macOS, task 020): WKWebView ignores `print()` on
/// the iframe the other platforms print from. `subject` titles the window;
/// `allow_remote` lets the remote images the user allowed in the reader load.
/// Async, so the window is not built inside a synchronous command.
#[tauri::command]
pub async fn print_message(
    app: AppHandle,
    html: String,
    subject: String,
    allow_remote: bool,
) -> Result<(), String> {
    crate::print::check_html(&html)?;
    #[cfg(target_os = "macos")]
    return crate::print::open_window(&app, html, &subject, allow_remote);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, subject, allow_remote);
        Err("printing through a print window is only used on macOS".to_owned())
    }
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
