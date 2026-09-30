//! Text typed into a form lands in comments of the generated code. A line
//! break in it used to end the comment, and whatever followed was compiled.

use std::collections::BTreeMap;
use vleo_sheet::emit;
use vleo_sheet::model::Sheet;

fn hostile() -> Sheet {
    Sheet {
        id: "x_row".into(),
        symbol: "x".into(),
        label: "a label\npub fn injected_label() {}".into(),
        question: "a question\npub fn injected() {} */ /* \r\nstill text".into(),
        expression: "x = 1\nfn injected_expr() {}".into(),
        source: "somewhere\u{2028}fn injected_src() {}".into(),
        unit: "1".into(),
        ty: "f64".into(),
        ..Default::default()
    }
}

/// Every line of generated code that mentions an injected name is a comment.
fn only_in_comments(code: &str) {
    for line in code.lines() {
        if line.contains("injected") {
            let t = line.trim_start();
            assert!(
                t.starts_with("//"),
                "sheet text escaped its comment:\n{line}"
            );
        }
    }
}

#[test]
fn a_line_break_in_sheet_text_stays_inside_the_comment() {
    let sh = hostile();
    only_in_comments(&emit::model_rs(&sh, &BTreeMap::new()));
    only_in_comments(&emit::mod_rs(&sh));
    only_in_comments(&emit::contract_rs(&sh));
}
