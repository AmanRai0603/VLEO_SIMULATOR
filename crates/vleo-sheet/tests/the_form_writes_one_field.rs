//! What `form::set` must do to a sheet, and what it must refuse to do.
//!
//! This is the only code that writes a hand-written sheet on somebody else's
//! behalf, so its failure mode matters more than its success one: a sheet
//! edited approximately — a comment dropped, a multi-line body truncated, the
//! wrong `source` key found — is worse than a sheet not edited. Every case here
//! is one of those.

use vleo_sheet::form;

/// A sheet shaped like the real ones: more comment than content, two tables
/// carrying a `source`, and one multi-line body.
const SHEET: &str = r#"# Why this row exists at all.
#
# Two lines of somebody's reasoning.
id = "demo_row"
label = "The old label"
kind = "computed"
order = 42

[question]
# What this asks, and why it is asked this way.
text = "How much?"

[maths]
# The relation came from a paper, and this comment says which figure.
expression = "y = 2*x"
source = "somebody1999"

[output]
symbol = "y"
type = "Length"
unit = "Metre"
# A bound with no reason gets deleted by the next person.
reason_lower = "a length cannot be negative"
reason_upper = "above this the model is unusable"
note = """
A laid-out body
over several lines.
"""

[[input]]
var = "x"
source = "another_table_with_a_source"
"#;

fn count_comments(s: &str) -> usize {
    s.lines().filter(|l| l.trim_start().starts_with('#')).count()
}

#[test]
fn a_written_field_reads_back_and_every_comment_survives() {
    for (field, value) in [
        ("label", "A new label"),
        ("question", "How much, exactly?"),
        ("expression", "y = 3*x"),
        ("unit", "Kelvin"),
        ("reason_lower", "a rewritten reason"),
    ] {
        let out = form::set(SHEET, field, value)
            .unwrap_or_else(|e| panic!("{field} should be writable: {e}"));
        let v: toml::Value = out.parse().expect("the result must still be TOML");
        let (t, k) = form::place(field).unwrap();
        let got = if t.is_empty() {
            v.get(k)
        } else {
            v.get(t).and_then(|x| x.get(k))
        };
        assert_eq!(got.and_then(|x| x.as_str()), Some(value), "{field} did not read back");
        assert_eq!(
            count_comments(&out),
            count_comments(SHEET),
            "{field}: a comment was lost, and every comment is somebody's reason"
        );
    }
}

#[test]
fn a_value_with_a_quote_in_it_survives_the_round_trip() {
    let awkward = r#"a label with "quotes" and a \backslash"#;
    let out = form::set(SHEET, "label", awkward).unwrap();
    let v: toml::Value = out.parse().expect("must still be TOML");
    assert_eq!(v.get("label").and_then(|x| x.as_str()), Some(awkward));
}

#[test]
fn the_source_under_maths_is_written_not_the_one_under_input() {
    let out = form::set(SHEET, "source", "newsource2026").unwrap();
    let v: toml::Value = out.parse().unwrap();
    assert_eq!(
        v["maths"]["source"].as_str(),
        Some("newsource2026"),
        "the [maths] source is the one this field means"
    );
    assert_eq!(
        v["input"][0]["source"].as_str(),
        Some("another_table_with_a_source"),
        "an identically named key in another table must be untouched"
    );
}

#[test]
fn a_structural_field_is_refused_with_a_reason() {
    for field in ["order", "parent", "id", "kind", "layer", "owner", "state"] {
        assert!(
            form::structural(field).is_some(),
            "{field} must be named structural, or a form will offer it"
        );
        assert!(
            form::set(SHEET, field, "anything").is_err(),
            "{field} must not be writable through the form"
        );
    }
}

#[test]
fn an_unknown_field_is_refused_rather_than_appended() {
    let e = form::set(SHEET, "not_a_field", "x").unwrap_err();
    assert!(e.contains("not a field"), "unhelpful refusal: {e}");
}

#[test]
fn a_key_that_is_not_there_is_refused_rather_than_invented() {
    // `confirmed_by` is a real field and this sheet has no line for it. The form
    // replaces a value that exists; deciding where a new key belongs — above or
    // below the comment explaining the relation — is not its call.
    let e = form::set(SHEET, "confirmed_by", "A. Person / 2026-01-01").unwrap_err();
    assert!(e.contains("confirmed_by"), "should name the key: {e}");
}

#[test]
fn a_multi_line_body_is_refused_rather_than_truncated() {
    // `note` is not a form field, so reach the guard through a sheet whose
    // `unit` is written as a multi-line string — the shape the guard is for.
    let odd = SHEET.replace("unit = \"Metre\"", "unit = \"\"\"\nMetre\n\"\"\"");
    let e = form::set(&odd, "unit", "Kelvin").unwrap_err();
    assert!(
        e.contains("multi-line"),
        "a multi-line value must be refused by name, not cut at the first newline: {e}"
    );
}

#[test]
fn a_duplicated_key_is_refused_rather_than_half_written() {
    let twice = SHEET.replace(
        "unit = \"Metre\"",
        "unit = \"Metre\"\nunit = \"Metre\"",
    );
    let e = form::set(&twice, "unit", "Kelvin").unwrap_err();
    assert!(e.contains("2 times"), "should say how many: {e}");
}

#[test]
fn a_key_inside_a_multi_line_body_is_never_replaced() {
    // The real reason the duplicate refusal exists. `note` holds prose, and a
    // line of prose that happens to look like `unit = ...` is not an
    // assignment. Replacing it would edit somebody's paragraph and leave the
    // actual field alone, so two hits refuse.
    let s = "id = \"x\"\n\n[output]\nnote = \"\"\"\nunit = \"WRONG\"\n\"\"\"\nunit = \"Metre\"\n";
    let e = form::set(s, "unit", "Kelvin").unwrap_err();
    assert!(
        e.contains("2 times"),
        "a key inside a body must make this ambiguous and refuse: {e}"
    );
}

#[test]
fn a_commented_out_assignment_is_not_mistaken_for_the_real_one() {
    let with_comment = SHEET.replace(
        "unit = \"Metre\"",
        "# unit = \"Foot\"   an earlier decision, kept as a note\nunit = \"Metre\"",
    );
    let out = form::set(&with_comment, "unit", "Kelvin").unwrap();
    let v: toml::Value = out.parse().unwrap();
    assert_eq!(v["output"]["unit"].as_str(), Some("Kelvin"));
    assert!(
        out.contains("# unit = \"Foot\""),
        "the commented-out line is a note and must survive"
    );
}
