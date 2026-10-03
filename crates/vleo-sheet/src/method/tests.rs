//! The method language, tested.

use super::*;

fn sig(inputs: &[(&str, &str)], out: &str) -> Signature {
    Signature {
        inputs: inputs
            .iter()
            .map(|(n, q)| (n.to_string(), quantity_dim(q).unwrap()))
            .collect(),
        output: quantity_dim(out).unwrap(),
        publishes: Vec::new(),
    }
}

const ORBIT: &str = "\
# The speed of a circular orbit at altitude h.
let r = R_EARTH + h
if h < 0 [m] then
  refuse \"the altitude is below the ground\"
end
let v : Velocity = sqrt(MU_EARTH / r)
return v
";

#[test]
fn a_sound_method_checks_and_runs() {
    let s = sig(&[("h", "Length")], "Velocity");
    let p = compile(ORBIT, &s).unwrap();
    let o = run(&p, &[("h".into(), 400e3)]).unwrap();
    let Outcome::Answer(v) = o else {
        panic!("{o:?}")
    };
    assert!((v - 7668.6).abs() < 1.0, "{v}");
    let o = run(&p, &[("h".into(), -1.0)]).unwrap();
    assert!(matches!(o, Outcome::Refused { line: 4, .. }), "{o:?}");
}

#[test]
fn units_are_read_and_converted() {
    assert_eq!(parse_unit("km").unwrap().0, 1e3);
    let (f, d) = parse_unit("m^3/s^2").unwrap();
    assert_eq!(f, 1.0);
    assert_eq!(d, Dim::new(3, 0, -2, 0, 0, 0, 0));
    assert_eq!(
        parse_unit("1/m^3").unwrap().1,
        Dim::new(-3, 0, 0, 0, 0, 0, 0)
    );
    let e = parse_unit("kN").unwrap_err();
    assert_eq!(e.kind(), ErrorKind::Malformed, "{e}");
    assert!(e.message().contains("kN"), "{e}");
    assert!((parse_unit("deg").unwrap().0 - core::f64::consts::PI / 180.0).abs() < 1e-15);
    assert_eq!(dim_text(Dim::new(3, 0, -2, 0, 0, 0, 0)), "m^3/s^2");
    assert_eq!(dim_text(Dim::new(-3, 0, 0, 0, 0, 0, 0)), "1/m^3");
}

fn errors(src: &str, s: &Signature) -> Vec<String> {
    match parse(src) {
        Err(d) => vec![d.to_string()],
        Ok(p) => check(&p, s)
            .into_iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| d.to_string())
            .collect(),
    }
}

#[test]
fn every_unit_mistake_is_refused_by_line() {
    let s = sig(&[("h", "Length"), ("t", "Time")], "Velocity");
    let e = errors("let x = h + t\nreturn h / t", &s);
    assert!(
        e[0].starts_with("line 1: a sum of unlike quantities: m and s"),
        "{e:?}"
    );
    let e = errors("return exp(h) * h / t", &s);
    assert!(e[0].contains("exp of a m value"), "{e:?}");
    let e = errors("return h", &s);
    assert!(
        e[0].contains("the answer must be m/s and this is m"),
        "{e:?}"
    );
    let e = errors("return sqrt(h) / t", &s);
    assert!(e[0].contains("sqrt of m is not a quantity"), "{e:?}");
    let e = errors("if h < 5 then\n return h / t\nend\nreturn h / t", &s);
    assert!(e[0].contains("a comparison of unlike quantities"), "{e:?}");
    // A bare zero is zero of anything.
    assert!(errors("if h < 0 then\n refuse \"low\"\nend\nreturn h / t", &s).is_empty());
}

#[test]
fn every_path_must_end() {
    let s = sig(&[("h", "Length"), ("t", "Time")], "Velocity");
    let e = errors("if h > 0 [m] then\n return h / t\nend", &s);
    assert!(
        e.iter().any(|m| m.contains("without return or refuse")),
        "{e:?}"
    );
    let ok = "if h > 0 [m] then\n return h / t\nelse\n refuse \"no\"\nend";
    assert!(errors(ok, &s).is_empty());
    let e = errors("return h / t\nlet x = h", &s);
    assert!(e[0].contains("can never run"), "{e:?}");
}

#[test]
fn names_are_checked() {
    let s = sig(&[("h", "Length")], "Length");
    assert!(errors("return q", &s)[0].contains("«q» is not an input"));
    assert!(errors("let h = 3 [m]\nreturn h", &s)[0].contains("an input of this node"));
    assert!(errors("set h = 3 [m]\nreturn h", &s)[0].contains("cannot be changed"));
    assert!(errors("let R_EARTH = h\nreturn h", &s)[0].contains("kernel constant"));
    assert!(errors("return foo(h)", &s)[0].contains("not a function"));
    let p = parse("return 2 [m]").unwrap();
    let notes = check(&p, &s);
    assert!(notes
        .iter()
        .any(|d| d.severity == Severity::Warning && d.msg.contains("never reads")));
}

#[test]
fn loops_tables_and_powers() {
    let s = sig(&[("x", "Ratio")], "Ratio");
    let series =
        "let total = 0\nfor n = 1 to 20\n  set total = total + x ^ n / n\nend\nreturn total";
    let p = compile(series, &s).unwrap();
    let Outcome::Answer(v) = run(&p, &[("x".into(), 0.5)]).unwrap() else {
        panic!()
    };
    assert!((v - core::f64::consts::LN_2).abs() < 1e-6, "{v}");

    let s = sig(&[("h", "Length")], "MassDensity");
    let table = "return interp(h, [200, 300, 400] [km], [2.5e-10, 1.9e-11, 2.8e-12] [kg/m^3])";
    let p = compile(table, &s).unwrap();
    let Outcome::Answer(v) = run(&p, &[("h".into(), 250e3)]).unwrap() else {
        panic!()
    };
    assert!((v - (2.5e-10 + 1.9e-11) / 2.0).abs() < 1e-18, "{v}");

    let s = sig(&[("a", "Area")], "Length");
    assert!(errors("return a ^ 0.5", &s).is_empty());
    let s = sig(&[("r", "Length")], "Volume");
    assert!(errors("return 4 / 3 * PI * r ^ 3", &s).is_empty());
    assert!(errors("return r ^ 1.5", &s)[0].contains("not a quantity"));
}

#[test]
fn runtime_faults_are_errors_not_refusals() {
    let s = sig(&[("x", "Ratio")], "Ratio");
    let p = compile("return sqrt(x)", &s).unwrap();
    let e = run(&p, &[("x".into(), -1.0)]).unwrap_err();
    assert!(e.msg.contains("no square root"), "{e}");
    let p = compile("return 1 / x", &s).unwrap();
    assert!(run(&p, &[("x".into(), 0.0)])
        .unwrap_err()
        .msg
        .contains("division by zero"));
}

#[test]
fn the_grammar_says_where_it_went_wrong() {
    let s = sig(&[("x", "Ratio")], "Ratio");
    assert!(errors("let = 3", &s)[0].starts_with("line 1"));
    assert!(errors("if x > 0 then\n return x", &s)[0].contains("needs its «end»"));
    assert!(errors("return x x", &s)[0].contains("one statement per line"));
    assert!(errors("return x $ 2", &s)[0].contains("not part of the method language"));
    assert!(errors("print x", &s)[0].contains("not «print»"));
    assert!(errors("const c = x\nreturn x", &s)[0].contains("one number with its unit"));
    // A table may run across lines inside its brackets.
    let s = sig(&[("h", "Length")], "Ratio");
    let multi = "return interp(h, [1, 2,\n 3] [m], [0, 1,\n 2] [1])";
    assert!(errors(multi, &s).is_empty(), "{:?}", errors(multi, &s));
}

#[test]
fn cases_are_judged_against_the_method() {
    let sheet = format!(
        "[method]\ntext = \"\"\"\n{ORBIT}\"\"\"\n[output]\ntype = \"Velocity\"\n\n[[input]]\nbinding = \"h\"\ntype = \"Length\"\n\n\
         [[case]]\nlabel = \"250 km\"\nexpect = 7754.84549737\ntolerance = 1e-6\ninputs = {{ h = 250000.0 }}\n\n\
         [[case]]\nlabel = \"400 km\"\nexpect = 7668.55817541\ntolerance = 1e-6\ninputs = {{ h = 400000.0 }}\n\n\
         [[case]]\nlabel = \"wrong\"\nexpect = 7000.0\ntolerance = 1e-6\ninputs = {{ h = 400000.0 }}\n\n\
         [[case]]\nlabel = \"underground\"\nrefuse = \"yes\"\ninputs = {{ h = -5.0 }}\n"
    );
    let r = report_toml(&sheet).unwrap();
    assert!(r.shortfall.is_empty(), "{:?}", r.shortfall);
    let v: Vec<bool> = r.cases.iter().map(|(_, v)| v.agrees()).collect();
    assert_eq!(v, [true, true, false, true]);
    assert!(!r.sound());
    assert!(r.cases[2]
        .1
        .text(&r.cases[2].0)
        .contains("more than your tolerance"));
    let j = r.json();
    assert!(j.starts_with("{\"sound\": false"), "{j}");
    // Too few, and no refusal: said, not guessed.
    let r = report("return 1 [m/s]", &sig(&[], "Velocity"), &[]);
    assert_eq!(r.shortfall.len(), 2);
}

#[test]
fn every_function_has_one_implementation_for_both_runner_and_translator() {
    for f in FUNCTIONS {
        if f.name == "interp" {
            continue;
        }
        assert!(
            implementation(f.name).is_some(),
            "{} has no implementation",
            f.name
        );
    }
}

#[test]
fn the_plain_form_reads_as_the_sheet_does() {
    let plain = format!(
        "output Velocity\ninput r Length\ncase 0 7754.84549737 1e-6 r=6628137 250 km\n\
         case 0 7000 1e-6 r=6778137 wrong\ncase 1 - - r=6000000 inside\nmethod\n{}",
        crate::example::METHOD
    );
    let r = report_plain(&plain).unwrap();
    let v: Vec<bool> = r.cases.iter().map(|(_, v)| v.agrees()).collect();
    assert_eq!(v, [true, false, true]);
    assert_eq!(r.cases[0].0.label, "250 km");
    assert!(report_plain("output Nonsense\nmethod\nreturn 1").is_err());
}

#[test]
fn the_plain_form_takes_units_as_a_group_contract_gives_them() {
    // A group's contract names units, not quantity types: the same method,
    // read with `[m]` and `[m/s]`, checks exactly as it does with the types.
    let plain = format!(
        "output [m/s]\ninput r [m]\ncase 0 7754.84549737 1e-6 r=6628137 250 km\nmethod\n{}",
        crate::example::METHOD
    );
    let r = report_plain(&plain).unwrap();
    assert!(r.diags.is_empty(), "{:?}", r.diags);
    assert!(r.cases[0].1.agrees());
    // The wrong unit for the answer is a unit error, said by line.
    let wrong = format!(
        "output [s]\ninput r [m]\nmethod\n{}",
        crate::example::METHOD
    );
    assert!(!report_plain(&wrong).unwrap().diags.is_empty());
    assert!(report_plain("output [furlong]\nmethod\nreturn 1").is_err());
}

#[test]
fn the_reference_covers_every_function_and_constant() {
    let md = reference_md();
    for f in FUNCTIONS {
        assert!(md.contains(&format!("`{}`", f.name)));
    }
    for c in KERNEL_CONSTANTS {
        assert!(md.contains(&format!("`{}`", c.name)));
        assert!(
            parse_unit(c.unit).is_ok(),
            "{} has an unreadable unit",
            c.name
        );
    }
}

/// Newton's method for the square root: a loop that settles, and refuses
/// rather than answer when it is not given the passes to settle in.
const NEWTON: &str = "\
let r = a
while abs(r * r - a) > 1e-12 * a at most N times
  set r = (r + a / r) / 2
end
return r
";

#[test]
fn a_loop_settles_or_refuses() {
    let s = sig(&[("a", "Ratio")], "Ratio");
    let p = compile(&NEWTON.replace('N', "40"), &s).unwrap();
    let Outcome::Answer(r) = run(&p, &[("a".into(), 2.0)]).unwrap() else {
        panic!("Newton's method on 2 did not settle in 40 passes")
    };
    assert!((r - 2f64.sqrt()).abs() < 1e-12, "{r}");
    // Two passes are not enough from a = 1e6: the loop refuses, by name,
    // instead of answering with where it had got to.
    let p = compile(&NEWTON.replace('N', "2"), &s).unwrap();
    let o = run(&p, &[("a".into(), 1e6)]).unwrap();
    match o {
        Outcome::Refused { line: 2, reason } => assert!(
            reason.contains("did not settle within 2 passes"),
            "{reason}"
        ),
        o => panic!("{o:?}"),
    }
}

#[test]
fn a_loop_says_how_often_it_may_run() {
    let s = sig(&[("a", "Ratio")], "Ratio");
    let e = errors(
        "let r = a\nwhile r > 1 times\n  set r = r / 2\nend\nreturn r",
        &s,
    );
    assert!(e.iter().any(|m| m.contains("at most N times")), "{e:?}");
    let e = errors(
        "let r = a\nwhile r > 1 at most 0 times\n  set r = r / 2\nend\nreturn r",
        &s,
    );
    assert!(
        e.iter()
            .any(|m| m.contains("at most 0 times, which is never")),
        "{e:?}"
    );
    let e = errors(
        "let r = a\nwhile r at most 5 times\n  set r = r / 2\nend\nreturn r",
        &s,
    );
    assert!(
        e.iter().any(|m| m.contains("while needs a condition")),
        "{e:?}"
    );
}

#[test]
fn a_kernel_function_is_called_by_name_with_its_units_checked() {
    let s = sig(&[("h", "Length"), ("t", "Temperature")], "MassDensity");
    let p = compile("return thermosphere_density(h, t)", &s).unwrap();
    let Outcome::Answer(rho) = run(&p, &[("h".into(), 250e3), ("t".into(), 1000.0)]).unwrap()
    else {
        panic!("no answer")
    };
    let kernel = vleo_core::physics::env::mass_density(
        vleo_units::Length::new(250e3),
        vleo_units::Temperature::new(1000.0),
    )
    .get();
    assert_eq!(
        rho.to_bits(),
        kernel.to_bits(),
        "the interpreter runs the kernel itself"
    );
    // An argument in the wrong unit, and the answer in the wrong quantity.
    let e = errors("return thermosphere_density(t, h)", &s);
    assert!(
        e.iter()
            .any(|m| m.contains("h is in [K] here, and the kernel takes it in [m]")),
        "{e:?}"
    );
    let s2 = sig(&[("h", "Length"), ("t", "Temperature")], "Length");
    let e = errors("return thermosphere_density(h, t)", &s2);
    assert!(!e.is_empty(), "a density returned as a length is refused");
    let e = errors("return thermosphere_density(h)", &s);
    assert!(
        e.iter().any(|m| m.contains("takes 2 arguments, not 1")),
        "{e:?}"
    );
}

/// A node that publishes two members beside its answer.
fn publishing() -> Signature {
    let mut s = sig(&[("x", "Length")], "Length");
    s.publishes = vec![
        ("Twice".into(), quantity_dim("Length").unwrap()),
        ("Square".into(), quantity_dim("Area").unwrap()),
    ];
    s
}

#[test]
fn a_node_publishes_every_member_before_it_answers() {
    let s = publishing();
    let src = "if x < 0 [m] then\n  refuse \"negative\"\nend\n\
               publish Twice = 2 * x\npublish Square = x * x\nreturn x";
    let p = compile(src, &s).unwrap_or_else(|e| panic!("{e:?}"));
    let (o, members) = run_all(&p, &[("x".into(), 3.0)]).unwrap();
    assert_eq!(o, Outcome::Answer(3.0));
    assert_eq!(
        members,
        vec![("Twice".to_string(), 6.0), ("Square".to_string(), 9.0)]
    );
    let (o, members) = run_all(&p, &[("x".into(), -1.0)]).unwrap();
    assert!(matches!(o, Outcome::Refused { .. }));
    assert!(members.is_empty(), "a refusal publishes nothing");

    // The translation returns the members in the order the sheet declares
    // them, whatever order the method publishes them in.
    let swapped = "publish Square = x * x\npublish Twice = 2 * x\nreturn x";
    let p = compile(swapped, &s).unwrap();
    let rust = to_rust_publishing(
        &p,
        "n",
        "src",
        swapped,
        &["x".into()],
        &["Twice".into(), "Square".into()],
    );
    assert!(
        rust.contains("-> Result<(f64, [f64; 2]), MethodError>"),
        "{rust}"
    );
    assert!(rust.contains("published[1] = rt::fin("), "{rust}");
    assert!(rust.contains("// Square"), "{rust}");
    assert!(rust.contains("return Ok((rt::fin("), "{rust}");
}

#[test]
fn a_member_missing_twice_nested_or_unknown_is_refused() {
    let s = publishing();
    let e = errors("publish Twice = 2 * x\nreturn x", &s);
    assert!(
        e.iter()
            .any(|m| m.contains("returns before publishing Square")),
        "{e:?}"
    );
    let e = errors(
        "publish Twice = 2 * x\npublish Twice = x\npublish Square = x * x\nreturn x",
        &s,
    );
    assert!(
        e.iter().any(|m| m.contains("«Twice» is published twice")),
        "{e:?}"
    );
    let e = errors(
        "if x > 0 [m] then\n  publish Twice = 2 * x\nend\npublish Square = x * x\nreturn x",
        &s,
    );
    assert!(e.iter().any(|m| m.contains("top level")), "{e:?}");
    let e = errors(
        "publish Thrice = 3 * x\npublish Twice = 2 * x\npublish Square = x * x\nreturn x",
        &s,
    );
    assert!(e.iter().any(|m| m.contains("no member «Thrice»")), "{e:?}");
    // Each member in its own quantity.
    let e = errors(
        "publish Twice = x * x\npublish Square = x * x\nreturn x",
        &s,
    );
    assert!(e.iter().any(|m| m.contains("«Twice» must be")), "{e:?}");
    // A node with one answer publishes nothing.
    let e = errors(
        "publish Twice = x\nreturn x",
        &sig(&[("x", "Length")], "Length"),
    );
    assert!(e.iter().any(|m| m.contains("publishes nothing")), "{e:?}");
}

#[test]
fn a_case_holds_every_member_to_the_authors_value() {
    let s = publishing();
    let p = compile(
        "publish Twice = 2 * x\npublish Square = x * x\nreturn x",
        &s,
    )
    .unwrap();
    let case = |also: &[(&str, f64)]| Case {
        label: "three".into(),
        inputs: vec![("x".into(), 3.0)],
        expect: Some(3.0),
        tolerance: 1e-12,
        also: also.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    };
    assert_eq!(
        judge(&p, &s, &case(&[("Twice", 6.0), ("Square", 9.0)])),
        Verdict::Agrees
    );
    match judge(&p, &s, &case(&[("Twice", 6.0), ("Square", 10.0)])) {
        Verdict::MemberDiffers { member, got, .. } => {
            assert_eq!(member, "Square");
            assert_eq!(got, 9.0);
        }
        v => panic!("{v:?}"),
    }
    assert!(matches!(
        judge(&p, &s, &case(&[("Twice", 6.0)])),
        Verdict::Malformed(m) if m.contains("«Square»")
    ));
}
