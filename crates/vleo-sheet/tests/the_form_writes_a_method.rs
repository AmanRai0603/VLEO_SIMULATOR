//! A node's method, its author's code, their cases and flight software, written
//! into a real sheet and read back exactly.
//!
//! Code is the one kind of value these sheets never held before, and it breaks
//! every assumption prose could make: its first line's indentation matters, a
//! backslash in it is a character and not an escape, and a line of it can look
//! exactly like a TOML assignment or a table header. Each of those is here.

use vleo_sheet::{form, method};

// A copy of orbit_velocity's sheet as it was before any method, kept here so
// the test does not change meaning the day that node's owner sends one.

const SHEET: &str = include_str!("data/orbit_velocity.node.toml");

const METHOD: &str = "\
# Vallado (2013), eq. 1-18: the circular two-body speed.
if r <= R_EARTH then
  refuse \"the orbit is inside the Earth\"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v";

const MATLAB: &str = "    function v = orbit_speed(r)
    % not a TOML table: [output]
    mu = 3.986004418e14; % x = 1 is not an assignment either
    fprintf('%g\\n', r);  \"\"\" three quotes, and a backslash \\ alone
    v = sqrt(mu ./ r);
end";

fn filled() -> String {
    let mut t = form::set(SHEET, "method_text", METHOD).unwrap();
    for (f, v) in [
        ("author_name", "Ana Rao"),
        ("author_language", "MATLAB"),
        ("author_entry", "orbit_speed"),
        ("author_code", MATLAB),
        (
            "author_test_code",
            "for r = [6628137 6778137]\n  disp(orbit_speed(r))\nend",
        ),
        ("author_how_run", "MATLAB R2023b, Ana's laptop, 2026-09-20"),
    ] {
        t = form::set(&t, f, v).unwrap_or_else(|e| panic!("{f}: {e}"));
    }
    for (label, refuse, expect, r) in [
        ("250 km", "no", "7754.84549737", "6628137.0"),
        ("400 km", "no", "7668.55817541", "6778137.0"),
        ("1000 km", "no", "7350.13862961", "7378137.0"),
        ("inside the Earth", "yes", "", "6000000"),
    ] {
        t = form::block_text(
            &t,
            "case",
            "add",
            0,
            &[
                ("label", label.into()),
                ("refuse", refuse.into()),
                ("expect", expect.into()),
                (
                    "tolerance",
                    if refuse == "yes" {
                        String::new()
                    } else {
                        "1e-6".into()
                    },
                ),
                ("inputs", format!("{{ r = {r} }}")),
            ],
        )
        .unwrap_or_else(|e| panic!("{label}: {e}"));
    }
    form::block_text(
        &t,
        "flight",
        "add",
        0,
        &[
            ("name", "orbit_speed.c".into()),
            ("language", "C".into()),
            (
                "purpose",
                "the on-board speed estimate for the drag model".into(),
            ),
            (
                "code",
                "double orbit_speed(double r) {\n    return sqrt(MU / r); /* \\n */\n}".into(),
            ),
            (
                "test_code",
                "assert(fabs(orbit_speed(6628137.0) - 7754.845) < 1e-3);".into(),
            ),
            ("test_result", "".into()),
        ],
    )
    .unwrap()
}

#[test]
fn code_comes_back_exactly_as_it_was_typed() {
    let t = filled();
    let v: toml::Value = t.parse().unwrap_or_else(|e| panic!("{e}\n{t}"));
    assert_eq!(v["method"]["text"].as_str().unwrap().trim_end(), METHOD);
    assert_eq!(v["author"]["code"].as_str().unwrap().trim_end(), MATLAB);
    assert_eq!(v["author"]["language"].as_str().unwrap(), "MATLAB");
    let fl = &v["flight"][0];
    assert!(fl["code"].as_str().unwrap().contains("/* \\n */"));
    // Nothing that looks like a table header inside the code became one.
    assert!(v.get("output").unwrap().get("symbol").is_some());
    // The sections sit where a reader expects them: the method after the
    // relation, the author's code after the method, the cases after that.
    let at = |needle: &str| t.find(needle).unwrap_or_else(|| panic!("{needle} missing"));
    assert!(at("[maths]") < at("[method]"));
    assert!(at("[method]") < at("[author]"));
    assert!(at("[author]") < at("[[case]]"));
    assert!(at("[[case]]") < at("[[flight]]"));
    // And the author's keys in the order the form asks them, not backwards.
    assert!(at("name = \"Ana Rao\"") < at("language = \"MATLAB\""));
    assert!(at("language = \"MATLAB\"") < at("how_run = "));
}

#[test]
fn a_second_write_replaces_the_code_it_wrote() {
    let t = filled();
    let t2 = form::set(&t, "author_code", "v = 1").unwrap();
    let v: toml::Value = t2.parse().unwrap();
    assert_eq!(v["author"]["code"].as_str().unwrap().trim_end(), "v = 1");
    // Everything else stays.
    assert_eq!(v["method"]["text"].as_str().unwrap().trim_end(), METHOD);
    assert_eq!(v["case"].as_array().unwrap().len(), 4);
}

#[test]
fn the_cases_run_against_the_method_as_written() {
    let r = method::report_toml(&filled()).unwrap();
    assert!(
        r.diags
            .iter()
            .all(|d| d.severity != method::Severity::Error),
        "{:?}",
        r.diags
    );
    assert!(r.shortfall.is_empty(), "{:?}", r.shortfall);
    for (c, v) in &r.cases {
        assert!(v.agrees(), "{}: {}", c.label, v.text(c));
    }
    assert!(r.sound());
}

#[test]
fn a_case_is_refused_as_it_is_written() {
    let bad = |vals: &[(&str, &str)]| {
        let vals: Vec<(&str, String)> = vals.iter().map(|(k, v)| (*k, v.to_string())).collect();
        let e = form::block_text(SHEET, "case", "add", 0, &vals).unwrap_err();
        // An answer the form will not write is refused, never taken as a broken sheet.
        assert_eq!(e.kind(), vleo_sheet::ErrorKind::Refused, "{e}");
        e.message().to_string()
    };
    assert!(
        bad(&[("label", "x"), ("refuse", "maybe"), ("inputs", "{ r = 1 }")]).contains("one of")
    );
    assert!(
        bad(&[("label", "x"), ("refuse", "no"), ("inputs", "r = 1")])
            .contains("not a set of inputs")
    );
    assert!(bad(&[
        ("label", "x"),
        ("refuse", "no"),
        ("inputs", "{ r = \"a\" }")
    ])
    .contains("not a number"));
    assert!(bad(&[
        ("label", "x"),
        ("refuse", "no"),
        ("expect", "fast"),
        ("inputs", "{ r = 1 }")
    ])
    .contains("is a number"));
}

#[test]
fn only_a_refused_case_may_give_an_input_no_node_takes() {
    // Every generated node refuses an input that is not a number at its door,
    // and a case saying so is evidence of that refusal: it goes in as written,
    // in TOML's own spelling, and the method is held to refusing it too.
    let case = |refuse: &str, r: &str| -> Vec<(&str, String)> {
        vec![
            ("label", format!("r = {r}")),
            ("refuse", refuse.into()),
            ("expect", if refuse == "yes" { "" } else { "7000.0" }.into()),
            (
                "tolerance",
                if refuse == "yes" { "" } else { "1e-6" }.into(),
            ),
            ("inputs", format!("{{ r = {r} }}")),
        ]
    };
    let mut t = filled();
    for r in ["nan", "inf", "-inf"] {
        t = form::block_text(&t, "case", "add", 0, &case("yes", r))
            .unwrap_or_else(|e| panic!("a refused case with r = {r}: {e}"));
        assert!(
            t.contains(&format!("inputs = {{ r = {r} }}")),
            "written as TOML reads it: {r}"
        );
    }
    let rep = method::report_toml(&t).unwrap();
    for (c, v) in &rep.cases {
        assert!(v.agrees(), "{}: {}", c.label, v.text(c));
    }
    assert!(rep.sound());

    // A case with an answer may not: there is no answer to a value that is not one.
    let e = form::block_text(&filled(), "case", "add", 0, &case("no", "nan")).unwrap_err();
    assert_eq!(e.kind(), vleo_sheet::ErrorKind::Refused, "{e}");
    assert!(
        e.message().contains("Only a case the node must refuse"),
        "{e}"
    );

    // Nor may a refused case become an answered one while it still gives one.
    let n = method::report_toml(&t).unwrap().cases.len();
    let e = form::block_text(&t, "case", "set", n - 1, &[("refuse", "no".into())]).unwrap_err();
    assert!(e.message().contains("not finite"), "{e}");
    // Its inputs may be changed, and stay a refusal's.
    form::block_text(
        &t,
        "case",
        "set",
        n - 1,
        &[("inputs", "{ r = nan }".into())],
    )
    .unwrap();
}
