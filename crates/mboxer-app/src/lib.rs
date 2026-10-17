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

/// Builds the window and runs the event loop until it closes.
pub fn run() {
    // The temporary attachment copies of earlier runs go, off the main thread.
    std::thread::spawn(attachments::clean_temp);
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::initial_path,
            commands::open_mbox,
            commands::cancel_open,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running the mboxer window");
}
