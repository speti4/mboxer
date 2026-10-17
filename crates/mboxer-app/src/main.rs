// A release build is a windowed app; the debug build keeps its console for `eprintln!`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mboxer_app_lib::run()
}
