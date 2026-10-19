//! mboxer-app: the Tauri 2 shell over `mboxer-core` (task 005).
//!
//! The webview loads the static files under `ui/` (embedded at compile time)
//! and talks to Rust through the commands in [`commands`]. Everything that
//! does not need a webview — the state, the open-or-build flow, the message
//! view, the label order — lives in plain modules with unit tests.

pub mod attachments;
pub mod commands;
pub mod folders;
pub mod state;
pub mod types;
pub mod view;

/// Builds the window and runs the event loop until it closes. On macOS the
/// files the system asks the app to open (Finder "Open with", a Dock drop)
/// arrive as `RunEvent::Opened` and go the way of a command-line path; the
/// other platforms pass such a file as the first argument and have no event.
pub fn run() {
    // The temporary attachment copies of earlier runs go, off the main thread.
    std::thread::spawn(attachments::clean_temp);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::initial_paths,
            commands::app_version,
            commands::open_mbox,
            commands::cancel_open,
            commands::index_status,
            commands::activate_file,
            commands::close_file,
            commands::set_query,
            commands::list_rows,
            commands::read_message,
            commands::read_source,
            commands::position_of,
            commands::toggle_thread,
            commands::open_url,
            commands::read_part,
            commands::open_attachment,
            commands::save_attachment,
            commands::save_all_attachments,
            commands::save_message_eml,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the mboxer window")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Opened { urls } = &event {
                commands::open_from_system(app, urls);
            }
            #[cfg(not(target_os = "macos"))]
            let _ = (app, event);
        });
}
