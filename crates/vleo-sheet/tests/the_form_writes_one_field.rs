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
    s.lines()
        .filter(|l| l.trim_start().starts_with('#'))
        .count()
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
        assert_eq!(
            got.and_then(|x| x.as_str()),
            Some(value),
            "{field} did not read back"
        );
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
fn a_key_that_is_not_there_is_refused_unless_the_field_says_it_may_be_added() {
    // `sense` is a real field and this sheet has no line for it. A sense belongs
    // to a requirement, the gate puts one on every row of that kind, so a row
    // without one is not one — and inventing it would make this row claim to be
    // a requirement it is not.
    let e = form::set(SHEET, "sense", ">=").unwrap_err();
    assert!(e.contains("sense"), "should name the key: {e}");
    assert!(
        e.contains("requirement"),
        "and say why this row has not got one: {e}"
    );
}

#[test]
fn a_field_that_may_be_added_is_added_under_its_own_table() {
    // The other half of the rule. Most sheets have no `note` — 124 of 1396 — so
    // a form that could only replace could never write one, and the question
    // would be unanswerable through every face.
    let out = form::set(SHEET, "note", "what this row is not for").unwrap();
    let v: toml::Value = out.parse().expect("must still be TOML");
    assert_eq!(
        v["question"]["note"].as_str(),
        Some("what this row is not for")
    );
    assert_eq!(
        v["output"]["note"].as_str().map(|s| s.contains("laid-out")),
        Some(true),
        "the identically named key under [output] must be untouched"
    );
    assert_eq!(
        count_comments(&out),
        count_comments(SHEET),
        "and no comment is lost"
    );
}

#[test]
fn a_missing_theory_table_is_created_where_the_authored_sheets_put_it() {
    // 1322 of 1396 rows have no [theory] at all, so this is the ordinary case
    // and not an edge one.
    let out = form::set(SHEET, "theory_why", "because the ceiling is the bound").unwrap();
    let v: toml::Value = out.parse().expect("must still be TOML");
    assert_eq!(
        v["theory"]["why"].as_str(),
        Some("because the ceiling is the bound")
    );
    let (theory, output) = (
        out.find("[theory]").expect("a [theory] table"),
        out.find("[output]").expect("the [output] table"),
    );
    assert!(
        theory < output,
        "it belongs after [maths] and before [output], where every authored sheet has it"
    );
    // And it can be written twice — the second edit replaces rather than adding
    // a second table.
    let again = form::set(
        &out,
        "theory_reading",
        "read it as a bound, not a description",
    )
    .unwrap();
    let v: toml::Value = again.parse().unwrap();
    assert_eq!(
        v["theory"]["why"].as_str(),
        Some("because the ceiling is the bound")
    );
    assert_eq!(again.matches("[theory]").count(), 1, "one table, not two");
}

#[test]
fn a_bound_is_written_bare_and_normalised() {
    // `lower = "40"` parses as a string and the loader reads it back as zero, so
    // a guard the whole row rests on would silently become "not below nothing".
    let s = SHEET.replace(
        "[output]\n",
        "[output]\nlower = 0.0\nupper = 1.0   # the declared domain\n",
    );
    let out = form::set(&s, "lower", "40").unwrap();
    assert!(
        out.contains("lower = 40.0"),
        "bare, and carrying its decimal point: {out}"
    );
    let v: toml::Value = out.parse().unwrap();
    assert_eq!(v["output"]["lower"].as_float(), Some(40.0));

    let out = form::set(&s, "upper", "140").unwrap();
    assert!(
        out.contains("upper = 140.0   # the declared domain"),
        "and the comment beside it survives: {out}"
    );

    let e = form::set(&s, "lower", "quite low").unwrap_err();
    assert!(
        e.contains("not a number"),
        "a bound that is not a number is refused, not written: {e}"
    );
}

#[test]
fn a_comment_beside_a_value_survives_the_edit() {
    // 27 of the fields this form writes carry one, and they are instructions to
    // whoever fills the row: `# REQUIRED — an agent may never supply mathematics`
    // sits on the relation of the rows where that matters most.
    let s = SHEET.replace(
        "expression = \"y = 2*x\"",
        "expression = \"y = 2*x\"   # REQUIRED — an agent may never supply mathematics",
    );
    let out = form::set(&s, "expression", "y = 3*x").unwrap();
    assert!(
        out.contains(
            "expression = \"y = 3*x\"   # REQUIRED — an agent may never supply mathematics"
        ),
        "the instruction must travel with the field it is about: {out}"
    );
}

#[test]
fn a_prose_block_is_replaced_whole_and_not_truncated() {
    // The shape that was refused outright before. A `[theory]` paragraph is
    // always a multi-line block, so refusing them meant the derivation could
    // never be written through a form at all.
    let s = SHEET.replace(
        "[output]",
        "[theory]\nwhy = \"\"\"\nThe first argument.\nOver two lines.\n\"\"\"\nreading = \"short\"\n\n[output]",
    );
    let out = form::set(
        &s,
        "theory_why",
        "A different argument.\nAlso over two lines.",
    )
    .unwrap();
    let v: toml::Value = out.parse().expect("must still be TOML");
    assert_eq!(
        v["theory"]["why"].as_str(),
        Some("A different argument.\nAlso over two lines.\n"),
        "the whole block is replaced"
    );
    assert!(
        !out.contains("The first argument"),
        "and none of the old one is left as stray TOML: {out}"
    );
    assert_eq!(
        v["theory"]["reading"].as_str(),
        Some("short"),
        "its neighbour is untouched"
    );
}

#[test]
fn a_one_line_field_refuses_a_paragraph_and_a_closed_set_refuses_a_label() {
    let e = form::set(SHEET, "expression", "y = 2*x\nand also y = 3*x").unwrap_err();
    assert!(e.contains("one line"), "{e}");

    let withsense = SHEET.replace("order = 42", "order = 42\nsense = \">=\"");
    assert!(form::set(&withsense, "sense", "<=").is_ok());
    let e = form::set(&withsense, "sense", "less than").unwrap_err();
    assert!(
        e.contains("closed set"),
        "a sense is one of two things, not a phrase: {e}"
    );
}

#[test]
fn the_nine_that_block_generation_are_the_nine() {
    // `unfilled` and the question list used to be two lists that had to agree,
    // with an error raised when they did not. They are one list now, so what is
    // worth asserting is that the list still says nine and says which.
    let blocking: Vec<&str> = form::FIELDS
        .iter()
        .filter(|f| f.blocks)
        .map(|f| f.field)
        .collect();
    assert_eq!(
        blocking,
        vec![
            "label",
            "question",
            "expression",
            "source",
            "symbol",
            "type",
            "unit",
            "reason_lower",
            "reason_upper"
        ],
        "these are what `docs` refuses to emit a scaffold without"
    );
    for f in form::FIELDS {
        assert!(
            !f.asked || (!f.ask.is_empty() && !f.why.is_empty() && !f.group.is_empty()),
            "{}: a question with no stated consequence is one somebody fills with anything",
            f.field
        );
    }
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
    let twice = SHEET.replace("unit = \"Metre\"", "unit = \"Metre\"\nunit = \"Metre\"");
    let e = form::set(&twice, "unit", "Kelvin").unwrap_err();
    assert!(e.contains("2 times"), "should say how many: {e}");
}

#[test]
fn a_key_inside_a_multi_line_body_is_never_replaced() {
    // A line of prose that happens to look like `unit = ...` is not an
    // assignment. The first version of this counted it as one and refused the
    // edit as ambiguous, which meant a row whose reason mentioned a key could
    // not have that key changed at all. The blocks are tracked, so the real
    // assignment is found and the paragraph is left alone.
    let s = "id = \"x\"\n\n[output]\nreason_lower = \"\"\"\nunit = \"WRONG\" is what an earlier draft said.\n\"\"\"\nunit = \"Metre\"\n";
    let out = form::set(s, "unit", "Kelvin").unwrap();
    let v: toml::Value = out.parse().expect("must still be TOML");
    assert_eq!(
        v["output"]["unit"].as_str(),
        Some("Kelvin"),
        "the assignment is the one outside the block"
    );
    assert!(
        out.contains("unit = \"WRONG\" is what an earlier draft said."),
        "and the paragraph is somebody's prose, not a field: {out}"
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

// ── pasting another row's sheet ──────────────────────────────────────────────

fn sheet() -> vleo_sheet::model::Sheet {
    // The loader is the only thing that builds a Sheet, so a real row is used
    // rather than a hand-made one: a preview against a fabricated sheet would
    // prove the preview agrees with the fabrication.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let t = vleo_sheet::load::load_all(root).unwrap();
    t.sheets.get("gnc_alignment_error").unwrap().clone()
}

#[test]
fn a_paste_reports_what_would_change_and_writes_nothing() {
    let sh = sheet();
    let before = std::fs::read_to_string(sh.dir.join("node.toml")).unwrap();
    let out = form::preview(&sh, "[maths]\nexpression = \"totally_new = 1\"\n").unwrap();
    assert!(out.contains("\"field\": \"expression\""), "{out}");
    assert!(
        out.contains("totally_new = 1"),
        "the new value is shown: {out}"
    );
    assert_eq!(
        std::fs::read_to_string(sh.dir.join("node.toml")).unwrap(),
        before,
        "a preview must never write"
    );
}

#[test]
fn a_pasted_attribution_is_refused_by_name() {
    // The one that matters. `confirmed_by` IS writable, so without a rule of its
    // own a paste would carry somebody's name across onto mathematics they have
    // never read.
    let out = form::preview(
        &sheet(),
        "[maths]\nexpression = \"x = 1\"\nconfirmed_by = \"Someone Else / 2026-01-01\"\n",
    )
    .unwrap();
    assert!(
        !out.contains("Someone Else"),
        "a pasted attribution must never appear as a change: {out}"
    );
    assert!(
        out.contains("confirmed_by") && out.contains("not pasted"),
        "and it must be named as dropped, with the reason: {out}"
    );
}

#[test]
fn a_pasted_structural_key_is_dropped_with_its_reason() {
    let out = form::preview(&sheet(), "id = \"other\"\norder = 999\nparent = \"l3_x\"\n").unwrap();
    for k in ["id", "order", "parent"] {
        assert!(out.contains(k), "{k} must be named: {out}");
    }
    assert!(
        out.contains("renumbers its neighbours"),
        "with the reason: {out}"
    );
    assert!(
        !out.contains("\"field\": \"id\""),
        "and never as a change: {out}"
    );
}

#[test]
fn a_table_the_form_writes_into_is_not_itself_reported_as_dropped() {
    // `[maths]` and `[output]` are containers the form writes INTO. Reporting
    // them as "not a field this form writes" told a reader their whole relation
    // had been ignored when only an extra key beside it had.
    let out = form::preview(&sheet(), "[maths]\nexpression = \"x = 1\"\nwhatever = 2\n").unwrap();
    assert!(
        !out.contains("\"maths — not a field"),
        "the table itself must not be reported as dropped: {out}"
    );
    assert!(
        out.contains("maths.whatever"),
        "its unknown key must be: {out}"
    );
}

#[test]
fn a_paste_that_is_not_toml_is_refused_rather_than_half_read() {
    let e = form::preview(&sheet(), "this is not = = toml [[[").unwrap_err();
    assert!(e.contains("not TOML"), "{e}");
}
