//! Runs the Node tests of the UI modules — the search field's query syntax
//! (`tests/ui/query.test.js`, against `ui/query.js`), the theme preference
//! (`tests/ui/theme.test.js`, against `ui/theme.js`), the remembered files
//! (`tests/ui/files.test.js`, against `ui/files.js`), the macOS print helpers
//! (`tests/ui/print.test.js`, against `ui/print.js`) and the sanitized frame's
//! CSS scrub and head (`tests/ui/frame.test.js`, against `ui/frame.js`) — when `node` is on PATH;
//! without Node the test prints a skip line and passes, so a machine without
//! it still builds and tests the Rust side. A second test holds the
//! highlight's folding (`foldChar` in `ui/query.js`) to the index's analyzer.

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn the_ui_modules_pass_their_node_tests() {
    let tests = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/query.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/theme.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/files.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/print.test.js"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/ui/frame.test.js"),
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

/// The blocks whose letters and digits the highlight folds as the index does
/// (see the folding section of `ui/query.js`): Latin-1, Latin Extended-A,
/// Latin Extended Additional and the Latin ligatures.
const MIRRORED_BLOCKS: [(u32, u32); 3] = [(0x00A0, 0x017F), (0x1E00, 0x1EFF), (0xFB00, 0xFB06)];

/// Prints, for each line `hex<TAB>fold` on stdin, the characters whose
/// `foldChar` differs from the index's fold.
const COMPARE_FOLDS: &str = r#"
const Query = require(process.argv[1]);
const bad = [];
for (const line of require("fs").readFileSync(0, "utf8").split("\n")) {
  if (!line) continue;
  const [hex, index] = line.split("\t");
  const ch = String.fromCodePoint(parseInt(hex, 16));
  const ui = Query.foldChar(ch);
  if (ui !== index) bad.push(`U+${hex} ${ch}: the index folds to ${index}, the highlight to ${ui}`);
}
process.stdout.write(bad.join("\n"));
"#;

#[test]
fn the_highlight_folds_the_letters_the_index_folds() {
    let mut analyzer = mboxer_core::index::schema::fold_analyzer();
    let mut lines = String::new();
    for (first, last) in MIRRORED_BLOCKS {
        for ch in (first..=last).filter_map(char::from_u32) {
            if !ch.is_alphanumeric() {
                continue;
            }
            let text = ch.to_string();
            let mut stream = analyzer.token_stream(&text);
            let mut folded = Vec::new();
            while let Some(token) = stream.next() {
                folded.push(token.text.clone());
            }
            lines.push_str(&format!("{:04X}\t{}\n", u32::from(ch), folded.join(" ")));
        }
    }
    let query_js = concat!(env!("CARGO_MANIFEST_DIR"), "/ui/query.js");
    let child = Command::new("node")
        .args(["-e", COMPARE_FOLDS, query_js])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(err) => {
            eprintln!("skipped: node is not available ({err})");
            return;
        }
    };
    child
        .stdin
        .take()
        .expect("node's stdin")
        .write_all(lines.as_bytes())
        .expect("write the index's folds to node");
    let output = child.wait_with_output().expect("run node");
    assert!(
        output.status.success(),
        "node failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mismatches = String::from_utf8_lossy(&output.stdout);
    assert!(
        mismatches.is_empty(),
        "the highlight folds these characters unlike the index:\n{mismatches}"
    );
}
