//! The files a run leaves under `~/.vleo` — a case's inputs, a saved result, a
//! saved sweep — name themselves as the one table of file kinds says
//! (`vleo-kinds`), so every reader tells them apart by the same rule.
//!
//! The facade is compiled into the pages too, so it does not depend on the
//! table; this holds its own words to the table's instead.

use vleo_kinds::{text, Store, CASE_LINE, RESULT_VERSION, SWEEP_VERSION};
use vleo_modules::{inputs, results};

fn first_bang(s: &str) -> &str {
    s.lines().find(|l| l.starts_with("#! ")).unwrap_or("")
}

#[test]
fn a_result_and_a_sweep_say_the_version_the_table_lists() {
    assert_eq!(results::FORMAT, RESULT_VERSION);
    assert_eq!(results::SWEEP_FORMAT, SWEEP_VERSION);
    for (line, version) in [("#! result", RESULT_VERSION), ("#! sweep", SWEEP_VERSION)] {
        assert_eq!(text(line).store, Store::Text { line, version }, "{line}");
    }
    let saved = results::csv(&results::Saved::default());
    assert_eq!(first_bang(&saved), format!("#! result {RESULT_VERSION}"));
}

#[test]
fn a_case_names_its_template_on_the_line_the_table_lists() {
    let case = inputs::csv(&[]);
    let line = first_bang(&case);
    assert!(
        line.starts_with(&format!("{CASE_LINE} ")),
        "the case's first #! line is {line:?}"
    );
    assert_eq!(&line[CASE_LINE.len() + 1..], inputs::template());
}
