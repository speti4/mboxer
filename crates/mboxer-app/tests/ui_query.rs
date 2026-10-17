//! Runs the Node tests of the search field's query syntax
//! (`tests/ui/query.test.js`, against `ui/query.js`) when `node` is on PATH;
//! without Node the test prints a skip line and passes, so a machine without
//! it still builds and tests the Rust side.

use std::process::Command;

#[test]
fn the_ui_query_module_passes_its_node_tests() {
    let test = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/query.test.js");
    let output = match Command::new("node").args(["--test", test]).output() {
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
