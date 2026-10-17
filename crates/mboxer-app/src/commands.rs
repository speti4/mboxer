//! The commands the webview invokes. Errors cross the bridge as strings.
//!
//! The state mutex is never held across an `.await`: each command locks,
//! copies what it needs, unlocks, then does the slow part.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use mboxer_core::{Error, Progress, SearchQuery, read_at};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::state::{AppState, OpenMbox};
use crate::types::{ListInfo, MessageView, OpenEvent, QueryInput, RowsPage};
use crate::view::read_message_view;

/// The largest page `list_rows` serves.
const MAX_ROWS: usize = 500;

/// The path given on the command line, opened at start-up by the webview.
#[tauri::command]
pub fn initial_path() -> Option<String> {
    std::env::args().nth(1)
}

/// Opens `path`, or the file the user picks when `path` is `None`. Progress
/// and the outcome arrive on `on_event`; the `Err` is only for a second open
/// while one is running.
#[tauri::command]
pub async fn open_mbox(
    app: AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
    on_event: Channel<OpenEvent>,
) -> Result<(), String> {
    if state
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err("an open is already in progress".to_owned());
    }
    let result = open_mbox_inner(app, &state, path, on_event).await;
    state.busy.store(false, Ordering::Release);
    result
}

async fn open_mbox_inner(
    app: AppHandle,
    state: &AppState,
    path: Option<String>,
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
        OpenMbox::load(&path, &cancel, generation, |p: Progress| {
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
        Ok(ListInfo {
            generation,
            total: open.listing.len(),
            warnings: open.listing.warnings().to_vec(),
        })
    })
    .await
    .map_err(|err| format!("the query task failed: {err}"))?
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

/// The message at `offset`/`len` of the open mbox, read off the runtime threads.
#[tauri::command]
pub async fn read_message(
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<MessageView, String> {
    let path = {
        let guard = state.open();
        guard.as_ref().ok_or("no mbox is open")?.path.clone()
    };
    tauri::async_runtime::spawn_blocking(move || read_message_view(&path, offset, len))
        .await
        .map_err(|err| format!("the read task failed: {err}"))?
        .map_err(|err| err.to_string())
}

/// The source of the message at `offset`/`len`: the envelope line and the
/// raw bytes, lossily decoded, for the reader's Source view.
#[tauri::command]
pub async fn read_source(
    state: State<'_, AppState>,
    offset: u64,
    len: u64,
) -> Result<String, String> {
    let path = {
        let guard = state.open();
        guard.as_ref().ok_or("no mbox is open")?.path.clone()
    };
    tauri::async_runtime::spawn_blocking(move || {
        read_at(&path, offset, len).map(|raw| {
            let mut source = raw.envelope;
            source.push('\n');
            source.push_str(&String::from_utf8_lossy(&raw.bytes));
            source
        })
    })
    .await
    .map_err(|err| format!("the read task failed: {err}"))?
    .map_err(|err| err.to_string())
}

/// Opens a link clicked in a message body with the system's default browser
/// or mail client. Only web and mail links get out; the sanitizer in the
/// webview has already dropped the rest, this is the second check.
#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    if !allowed_link(&url) {
        return Err("only http, https and mailto links are opened".to_owned());
    }
    tauri::async_runtime::spawn_blocking(move || app.opener().open_url(&url, None::<&str>))
        .await
        .map_err(|err| format!("the open task failed: {err}"))?
        .map_err(|err| err.to_string())
}

/// `http:`, `https:` and `mailto:` links, whatever their case.
pub fn allowed_link(url: &str) -> bool {
    let lower = url.trim_start().to_ascii_lowercase();
    ["http://", "https://", "mailto:"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
}

/// Row index of the message with this offset in the current listing.
#[tauri::command]
pub fn position_of(state: State<'_, AppState>, offset: u64) -> Result<Option<usize>, String> {
    let guard = state.open();
    let open = guard.as_ref().ok_or("no mbox is open")?;
    open.listing
        .position_of(offset)
        .map_err(|err| err.to_string())
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
