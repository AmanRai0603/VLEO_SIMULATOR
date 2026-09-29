//! Text from a sheet never becomes code.
//!
//! A row's question, label, assumptions and fixture labels are written by
//! people outside the repository, through the node form, and every generator
//! writes them into Rust comments. A line break in one once ended the comment
//! and put its next line into the generated file as code.

use std::collections::BTreeMap;
use std::path::Path;
use vleo_sheet::emit;
use vleo_sheet::load::load_all;

const EVIL: &str = "an honest {first} line\nfn injected() { std::process::exit(1) }\n*/ still text";

#[test]
fn every_line_of_sheet_text_is_a_comment_line() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tree = load_all(&root).unwrap();
    let mut sh = tree
        .sheets
        .values()
        .find(|s| !s.fixtures.is_empty() && !s.assumptions.is_empty())
        .expect("a row with fixtures and assumptions")
        .clone();
    sh.question = EVIL.into();
    sh.label = EVIL.into();
    sh.assumptions[0].text = EVIL.into();
    sh.fixtures[0].label = EVIL.into();
    sh.author.name = EVIL.into();
    for (what, code) in [
        ("model.rs", emit::model_rs(&sh, &BTreeMap::new())),
        ("mod.rs", emit::mod_rs(&sh)),
        ("evidence.rs", emit::evidence_rs(&sh)),
    ] {
        for line in code.lines() {
            if line.contains("injected") || line.contains("still text") {
                let t = line.trim_start();
                assert!(
                    t.starts_with("//") || t.contains('"'),
                    "{what}: sheet text became code: {line}"
                );
            }
            // Inside a format string a brace from the sheet must be doubled,
            // or the generated test does not compile.
            if line.contains("assert!") && line.contains("honest") {
                assert!(
                    line.contains("{{first}}"),
                    "{what}: a brace was left bare: {line}"
                );
            }
        }
    }
}
