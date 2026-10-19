//! Runs the Node tests of the UI modules — the search field's query syntax
//! (`tests/ui/query.test.js`, against `ui/query.js`), the theme preference
//! (`tests/ui/theme.test.js`, against `ui/theme.js`), the remembered files
//! (`tests/ui/files.test.js`, against `ui/files.js`) and the macOS print helpers
//! (`tests/ui/print.test.js`, against `ui/print.js`) — when `node` is on PATH;
//! without Node the test prints a skip line and passes, so a machine without
//! it still builds and tests the Rust side.

use std::process::Command;

#[test]
fn the_ui_modules_pass_their_node_tests() {
    let tests = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/query.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/theme.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/files.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/print.test.js"),
    ];
    let output = match Command::new("node").arg("--test").args(tests).output() {
        Ok(output) => output,
        Err(err) => {
            eprintln!("skipped: node is not available ({err})");
            return;
        }
    };
    assert!(
        output.status.success(),
        "node --test failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
