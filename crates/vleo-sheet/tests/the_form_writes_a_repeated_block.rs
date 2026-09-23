//! Adding, changing and removing a `[[table]]` block.
//!
//! A block is not a field. A field is replaced; a block is added and removed as
//! well, and either can break something a scalar edit never could — a numbered
//! hole in generated Rust, a hand-written body binding a name, an edge that
//! closes a loop. `form::block_text` is the textual half, which is what is
//! exercised here; the half that needs the tree and the hole bodies is in
//! `the_save_leaves_the_tree_consistent`.

use vleo_sheet::form;

/// A sheet shaped like the real ones: comments between the blocks, prose in a
/// multi-line body, and a table after the arrays.
const SHEET: &str = r#"id = "demo_row"
label = "A row"
kind = "computed"

[question]
text = "How much?"

[maths]
# The relation came from a paper.
expression = "y = 2*x"
source = "somebody1999"

[output]
symbol = "y"
type = "Length"
unit = "Metre"
lower = 0.0
upper = 10.0
reason_lower = "a length cannot be negative"
reason_upper = "above this the model is unusable"

[[input]]
# The consumer declares what it expects.
binding = "x"
var = "other_row"
type = "Length"

[[input]]
binding = "k"
var = "a_constant"
type = "Ratio"

[[algorithm.step]]
number = 1
text = "scale it"
binds = "scaled"
type = "Length"

[[algorithm.step]]
number = 2
text = "add the offset"
binds = "out"
type = "Length"

[view]
kind = "number"
"#;

fn v(pairs: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
    pairs.iter().map(|(k, x)| (*k, x.to_string())).collect()
}

fn parse(s: &str) -> toml::Value {
    s.parse()
        .unwrap_or_else(|e| panic!("not TOML any more: {e}\n{s}"))
}

#[test]
fn a_block_is_added_after_the_last_one_of_its_kind() {
    let out = form::block_text(
        SHEET,
        "input",
        "add",
        0,
        &v(&[("binding", "t"), ("var", "a_time"), ("type", "Time")]),
    )
    .unwrap();
    let t = parse(&out);
    let ins = t["input"].as_array().unwrap();
    assert_eq!(ins.len(), 3, "the new one is there");
    assert_eq!(ins[2]["binding"].as_str(), Some("t"), "and it is last");
    assert_eq!(
        ins[0]["binding"].as_str(),
        Some("x"),
        "the existing ones are where they were"
    );
    assert_eq!(
        t["algorithm"]["step"].as_array().unwrap().len(),
        2,
        "the steps are untouched"
    );
    assert_eq!(t["view"]["kind"].as_str(), Some("number"));
    // AND IT LANDED WHERE A PERSON WOULD HAVE PUT IT. Appending at the end of
    // the file parses just as well and is wrong: the sheet then reads
    // `[view]` and then an input, and a second add to a different array
    // interleaves the two. The order of a sheet is how it is read.
    let (last_in, first_step, view) = (
        out.rfind("[[input]]").unwrap(),
        out.find("[[algorithm.step]]").unwrap(),
        out.find("\n[view]").unwrap(),
    );
    assert!(
        last_in < first_step && first_step < view,
        "the new input belongs with the other inputs, before the steps and the view:\n{out}"
    );
    assert!(
        out.contains("# The consumer declares what it expects."),
        "and the comment between the blocks survives"
    );
}

#[test]
fn a_numbered_block_is_numbered_by_the_form() {
    // The number is the identity of a hole in generated Rust. A person typing it
    // is a person reattaching somebody's code to a different step.
    let out = form::block_text(
        SHEET,
        "algorithm",
        "add",
        0,
        &v(&[
            ("number", "99"),
            ("text", "round it"),
            ("binds", "rounded"),
            ("type", "Length"),
        ]),
    )
    .unwrap();
    let steps = parse(&out)["algorithm"]["step"].as_array().unwrap().clone();
    assert_eq!(steps.len(), 3);
    assert_eq!(
        steps[2]["number"].as_integer(),
        Some(3),
        "one past the last, not the 99 that was sent"
    );

    let e = form::block_text(SHEET, "algorithm", "set", 0, &v(&[("number", "7")])).unwrap_err();
    assert!(e.contains("never typed"), "{e}");
}

#[test]
fn only_the_last_numbered_block_can_be_removed() {
    let e = form::block_text(SHEET, "algorithm", "remove", 0, &[]).unwrap_err();
    assert!(
        e.contains("only the last"),
        "removing from the middle renumbers the holes after it: {e}"
    );
    let out = form::block_text(SHEET, "algorithm", "remove", 1, &[]).unwrap();
    let t = parse(&out);
    assert_eq!(t["algorithm"]["step"].as_array().unwrap().len(), 1);
    assert_eq!(
        t["view"]["kind"].as_str(),
        Some("number"),
        "and nothing after it was taken with it"
    );
    assert_eq!(
        t["input"].as_array().unwrap().len(),
        2,
        "nor anything before"
    );
}

#[test]
fn an_unnumbered_block_can_be_removed_from_anywhere() {
    let out = form::block_text(SHEET, "input", "remove", 0, &[]).unwrap();
    let ins = parse(&out)["input"].as_array().unwrap().clone();
    assert_eq!(ins.len(), 1);
    assert_eq!(
        ins[0]["binding"].as_str(),
        Some("k"),
        "the other one remains"
    );
}

#[test]
fn one_key_of_one_block_is_changed_and_the_rest_is_not() {
    let out =
        form::block_text(SHEET, "input", "set", 1, &v(&[("var", "a_different_row")])).unwrap();
    let ins = parse(&out)["input"].as_array().unwrap().clone();
    assert_eq!(ins[1]["var"].as_str(), Some("a_different_row"));
    assert_eq!(
        ins[1]["binding"].as_str(),
        Some("k"),
        "its siblings keep theirs"
    );
    assert_eq!(
        ins[0]["var"].as_str(),
        Some("other_row"),
        "and so does the other block"
    );
}

#[test]
fn an_optional_key_is_added_and_a_required_one_may_not_be_emptied() {
    // `math` is optional on a theory step and this sheet has no theory at all,
    // so the table has to be created and then written into.
    let out = form::block_text(
        SHEET,
        "theory",
        "add",
        0,
        &v(&[("text", "the first step of the argument")]),
    )
    .unwrap();
    let t = parse(&out);
    assert_eq!(
        t["theory"]["step"][0]["text"].as_str(),
        Some("the first step of the argument")
    );
    assert!(
        t["theory"]["step"][0].get("math").is_none(),
        "an optional key left blank is left out, not written empty"
    );
    let out2 = form::block_text(&out, "theory", "set", 0, &v(&[("math", "y = 2x")])).unwrap();
    assert_eq!(
        parse(&out2)["theory"]["step"][0]["math"].as_str(),
        Some("y = 2x")
    );

    let e = form::block_text(&out, "theory", "set", 0, &v(&[("text", "  ")])).unwrap_err();
    assert!(
        e.contains("Remove the whole block"),
        "emptying a required key leaves a block that says nothing: {e}"
    );
}

#[test]
fn a_block_with_a_required_key_missing_is_refused() {
    let e = form::block_text(SHEET, "input", "add", 0, &v(&[("binding", "t")])).unwrap_err();
    assert!(e.contains("var"), "should name what is missing: {e}");
}

#[test]
fn a_quantity_that_does_not_exist_is_refused_before_anything_is_written() {
    // The same rule as a scalar `type`, for the same reason: it goes straight
    // into the generated signature.
    let e = form::block_text(
        SHEET,
        "input",
        "add",
        0,
        &v(&[("binding", "t"), ("var", "a_time"), ("type", "Nonsense")]),
    )
    .unwrap_err();
    assert!(e.contains("not a quantity"), "{e}");
}

#[test]
fn a_prose_key_holding_a_paragraph_survives_being_a_block() {
    // An assumption's `fails_when` is a paragraph, and a block's keys go through
    // the same writer a field's do.
    let out = form::block_text(
        SHEET,
        "assumption",
        "add",
        0,
        &v(&[
            ("text", "the scale factor is constant"),
            (
                "fails_when",
                "the temperature moves.\nBelow 200 K it is not.\n",
            ),
        ]),
    )
    .unwrap();
    let t = parse(&out);
    assert_eq!(
        t["assumption"][0]["fails_when"].as_str(),
        Some("the temperature moves.\nBelow 200 K it is not.\n")
    );
    // And a second one after it, which is the case that would break if the
    // block scanner read a line of that paragraph as a table header.
    let out2 = form::block_text(
        &out,
        "assumption",
        "add",
        0,
        &v(&[("text", "the second"), ("fails_when", "never")]),
    )
    .unwrap();
    assert_eq!(parse(&out2)["assumption"].as_array().unwrap().len(), 2);
}

#[test]
fn a_header_inside_somebody_s_paragraph_is_not_a_block() {
    // The scanner has to know the difference. A reason that quotes a table name
    // at the start of a line is prose, and counting it would make the form
    // remove the wrong thing.
    let odd = SHEET.replace(
        "reason_upper = \"above this the model is unusable\"",
        "reason_upper = \"\"\"\nabove this the model is unusable. The earlier draft wrote it as\n[[input]]\nwhich it is not.\n\"\"\"",
    );
    let out = form::block_text(&odd, "input", "remove", 0, &[]).unwrap();
    let t = parse(&out);
    assert_eq!(
        t["input"].as_array().unwrap().len(),
        1,
        "two real blocks, one removed — not three counted and the wrong one gone"
    );
    assert_eq!(t["input"][0]["binding"].as_str(), Some("k"));
    assert!(
        out.contains("The earlier draft wrote it as"),
        "and the paragraph is untouched: {out}"
    );
}

#[test]
fn every_array_declares_what_each_of_its_keys_is_for() {
    for a in form::ARRAYS {
        assert!(
            !a.why.is_empty(),
            "{}: a block with no stated purpose",
            a.name
        );
        for c in a.columns {
            assert!(
                c.managed || !c.ask.is_empty(),
                "{}.{}: a question with no wording is one a face cannot ask",
                a.name,
                c.key
            );
        }
    }
}

#[test]
fn a_view_that_carries_a_comment_or_a_key_it_does_not_know_is_refused() {
    // `save_view` is the one writer here that REWRITES A WHOLE TABLE rather
    // than replacing a value, because a view's kind decides which other keys
    // exist. That makes it the one place a comment can be lost without the loss
    // appearing anywhere, so a table it does not fully understand is refused.
    //
    // Driven through `form::view_rewrite`, which is the textual half — the half
    // that needs the tree, to check the row being drawn against exists, is in
    // `the_save_leaves_the_tree_consistent`.
    let commented = SHEET.replace(
        "[view]\nkind = \"number\"",
        "[view]\n# a line here would need a row to sweep, and there is not one yet\nkind = \"number\"",
    );
    let e = form::view_rewrite(&commented, "number", "", "").unwrap_err();
    assert!(
        e.contains("comment"),
        "every comment in a sheet is somebody's reason: {e}"
    );

    let strange = SHEET.replace(
        "[view]\nkind = \"number\"",
        "[view]\nkind = \"number\"\nscale = \"log\"",
    );
    let e = form::view_rewrite(&strange, "number", "", "").unwrap_err();
    assert!(
        e.contains("scale"),
        "it must name the key it does not know: {e}"
    );

    let heat = SHEET.replace(
        "[view]\nkind = \"number\"",
        "[view]\nkind = \"heatmap\"\nover_x = \"a\"\nover_y = \"b\"\npoints = 40",
    );
    let e = form::view_rewrite(&heat, "line", "other_row", "60").unwrap_err();
    assert!(
        e.contains("heatmap"),
        "turning a heatmap into a line throws an axis away: {e}"
    );

    // And the ordinary case still goes through, with the table replaced rather
    // than appended to.
    let out = form::view_rewrite(SHEET, "line", "other_row", "60").unwrap();
    let v: toml::Value = out.parse().unwrap();
    assert_eq!(v["view"]["kind"].as_str(), Some("line"));
    assert_eq!(v["view"]["over"].as_str(), Some("other_row"));
    assert_eq!(v["view"]["points"].as_integer(), Some(60));

    let e = form::view_rewrite(SHEET, "line", "other_row", "1").unwrap_err();
    assert!(e.contains("two points"), "one point is a number: {e}");
    let e = form::view_rewrite(SHEET, "number", "other_row", "").unwrap_err();
    assert!(
        e.contains("not drawn against anything"),
        "a number has no row to sweep: {e}"
    );
}
