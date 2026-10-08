//! Text from a sheet never becomes code.
//!
//! A row's source and method are written by people outside the repository,
//! through the node form, and the translation writes them into Rust comments.
//! A line break in sheet text once ended a comment and put its next line into
//! a generated file as code.

use std::path::Path;
use vleo_sheet::load::load_all;
use vleo_sheet::method::node_rust;

const EVIL: &str = "an honest {first} line\nfn injected() { std::process::exit(1) }\n*/ still text";

#[test]
fn every_line_of_sheet_text_is_a_comment_line() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tree = load_all(&root).unwrap();
    let mut sh = tree
        .sheets
        .values()
        .find(|s| node_rust(s).is_some())
        .expect("a row with a method the translation takes")
        .clone();
    sh.source = EVIL.into();
    sh.method.text = format!("# {}\n{}", EVIL.replace('\n', "\n# "), sh.method.text);
    let code = node_rust(&sh).expect("the method still translates");
    let mut seen = 0;
    for line in code.lines() {
        if line.contains("injected") || line.contains("still text") {
            seen += 1;
            let t = line.trim_start();
            assert!(
                t.starts_with("//") || t.contains('"'),
                "sheet text became code: {line}"
            );
        }
    }
    assert!(seen > 0, "the sheet text was not written at all:\n{code}");
}
