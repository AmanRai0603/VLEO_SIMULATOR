//! An amount and a formula are read as a person types them (docs/briefs/F8.md,
//! F8-a2): the examples of docs/OPERATING_1_0.md, section 7, read as they
//! say, and each thing the language does not read refused by name, never
//! guessed. Nothing is written and nothing runs.
//!
//! The drag node is invented for the check (F4's question 5): ports `rho`,
//! `v`, `cd` (symbol `Cd`) and `a` (symbol `A`), its output a force. The
//! formula is section 7's own example.

use vleo_sheet::method::amount::{self, Shape};
use vleo_sheet::method::formula::{self, Fit, Found, Port};
use vleo_sheet::method::{dim_text, parse, quantity_dim, Signature};

fn ok(typed: &str, field: &str) -> amount::Amount {
    amount::read(typed, field).unwrap_or_else(|e| panic!("{typed}: {e}"))
}

fn refused(typed: &str, field: &str, says: &str) {
    let e = amount::read(typed, field).expect_err(typed);
    assert!(e.contains(says), "{typed}: {e}");
}

#[test]
fn a_value_a_range_and_a_spread_are_read_as_typed() {
    for typed in ["250 km", "250 [km]", "250km"] {
        let a = ok(typed, "");
        assert_eq!(
            (a.shape, a.nominal.as_deref(), a.unit.as_str(), a.factor),
            (Shape::Value, Some("250"), "km", 1000.0),
            "{typed}"
        );
        assert!(
            a.said.contains("a Length") && a.said.contains("250000 m in SI"),
            "{}",
            a.said
        );
    }
    // A bare number: pure in a field with no unit, else the field's, said.
    let a = ok("2.2", "");
    assert_eq!(
        (a.unit.as_str(), a.dim),
        ("1", quantity_dim("Ratio").unwrap())
    );
    let a = ok("2.2", "km");
    assert_eq!(a.unit, "km");
    assert!(a.said.contains("the field's unit"), "{}", a.said);
    let a = ok("2.5e-11 to 1.5e-10 kg/m^3", "");
    assert_eq!(
        (
            a.shape,
            a.lower.as_deref(),
            a.upper.as_deref(),
            a.unit.as_str()
        ),
        (Shape::Range, Some("2.5e-11"), Some("1.5e-10"), "kg/m^3")
    );
    assert!(a.said.contains("a MassDensity"), "{}", a.said);
    // Each end its own unit, of one dimension.
    let a = ok("500 m to 2 km", "");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref()),
        (Some("0.5"), Some("2"))
    );
    // A spread's ends worked out from the digits, exactly.
    for typed in ["55 ± 10% mN", "55 mN ± 10%", "55 +/- 5.5 mN"] {
        let a = ok(typed, "");
        assert_eq!(
            (
                a.shape,
                a.nominal.as_deref(),
                a.lower.as_deref(),
                a.upper.as_deref(),
                a.unit.as_str()
            ),
            (Shape::Spread, Some("55"), Some("49.5"), Some("60.5"), "mN"),
            "{typed}"
        );
    }
    let a = ok("0.1 ± 3%", "");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref()),
        (Some("0.097"), Some("0.103"))
    );
    // A unit read as what it is, and called that: C is a coulomb, never Celsius.
    let a = ok("25 C", "");
    assert!(a.said.contains("a Charge"), "{}", a.said);
    // Section 7's examples as typed, each said back exactly, its value in SI
    // worked out from the digits: never 0.000054999999999999995.
    let a = ok("55 mN/kW", "");
    assert!(
        a.said
            .starts_with("55 mN/kW, of dimension s/m: 0.000055 s/m in SI"),
        "{}",
        a.said
    );
    let a = ok("2.5e-11 to 1.5e-10", "kg/m^3");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref(), a.unit.as_str()),
        (Some("2.5e-11"), Some("1.5e-10"), "kg/m^3")
    );
    assert!(a.said.contains("the field's unit"), "{}", a.said);
    let a = ok("55 ± 10%", "mN");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref(), a.unit.as_str()),
        (Some("49.5"), Some("60.5"), "mN")
    );
    // A pure number is said as one, without its unit 1.
    assert!(
        ok("2.2", "").said.starts_with("2.2, a pure number"),
        "{}",
        ok("2.2", "").said
    );
    // Numbers as the lexer reads them.
    for (typed, nominal) in [
        ("-5 m", "-5"),
        (".5 m", "0.5"),
        ("5. m", "5"),
        ("1e+3 m", "1000"),
        ("2.50 m", "2.5"),
        ("1e-30 m", "1e-30"),
        ("1e25 m", "1e25"),
    ] {
        assert_eq!(ok(typed, "").nominal.as_deref(), Some(nominal), "{typed}");
    }
    assert!(
        ok("1e-30 m", "").said.contains(": 1e-30 m in SI"),
        "{}",
        ok("1e-30 m", "").said
    );
    // A design file's machine name is a field's unit as its symbol is.
    assert_eq!(ok("250", "Kilometre").unit, "km");
    // Two units of one dimension a power of ten apart, the lower end in the
    // upper's.
    let a = ok("1 mN to 1500 uN", "");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref(), a.unit.as_str()),
        (Some("1000"), Some("1500"), "uN")
    );
    let a = ok("-5 ± 10%", "");
    assert_eq!(
        (a.lower.as_deref(), a.upper.as_deref()),
        (Some("-5.5"), Some("-4.5"))
    );
    assert!(
        ok("1 [m^2]", "").said.contains("an Area"),
        "{}",
        ok("1 [m^2]", "").said
    );
}

#[test]
fn what_an_amount_does_not_say_is_refused_by_name_with_the_fix() {
    refused("1 kg/m3", "", "write a power with ^");
    refused("20 °C", "", "kelvin");
    refused("1,5 km", "", "has a comma");
    refused("3 furlong", "", "«furlong» is not a unit this tool knows");
    refused(
        "55 ± 10%",
        "%",
        "a spread in per cent of a value in per cent",
    );
    refused("2 to 1 km", "", "the lower end 2 is above the upper end 1");
    refused("1 km to 2 kg", "", "both ends are of one dimension");
    refused("5 m²", "", "is a superscript");
    refused("1.5x10^-3", "", "write a power of ten with e");
    refused("250 km", "kg", "this field is in [kg]");
    refused("km", "", "does not start with a number");
    refused("", "", "nothing is typed");
    // A unit after the per-cent is the nominal's, or refused: never dropped.
    refused(
        "55 mN ± 10% N",
        "",
        "the nominal is in [mN] and the spread names [N]",
    );
    refused(
        "55 mN ± 10% kg",
        "",
        "the nominal is in [mN] and the spread names [kg]",
    );
    assert_eq!(ok("55 ± 10% [mN]", "").unit, "mN");
    refused(
        "55 mN ± 5.5 N",
        "",
        "the nominal is in [mN] and the spread in [N]",
    );
    // A number too large or too small to hold, however it is written.
    for typed in [
        "1e400 km",
        "1e-400 km",
        "1e2147483647 m",
        "10e2147483647 m",
        "1e9999999999 m",
    ] {
        refused(typed, "", "too large or too small a number to hold");
    }
    // An end without its unit takes no other's.
    refused("1 km to 2000", "m", "write the unit after the upper end");
    refused(
        "1 min to 2 h",
        "",
        "differ by a factor that is not a power of ten",
    );
    refused("55 ± -10%", "", "the spread in «55 ± -10%» is negative");
    refused("55 ± -1 mN", "", "is negative");
    refused("1 to 2 ± 1", "", "is both a range and a spread");
    refused("1to 2", "", "a range is written with to between its ends");
    refused("1 To 2", "", "a range is written with to between its ends");
    refused("−5 m", "", "a typographic minus");
    refused("5 µN", "", "write the micro prefix as u");
    refused("55 ± x%", "", "is not a number of per cent");
    refused("5", "furlong", "the field's own unit");
}

fn drag() -> (Signature, Vec<Port>, Port) {
    let d = |q: &str| quantity_dim(q).unwrap();
    let port = |name: &str, symbol: &str, unit: &str| Port {
        name: name.into(),
        symbol: symbol.into(),
        unit: unit.into(),
    };
    (
        Signature {
            inputs: vec![
                ("rho".into(), d("MassDensity")),
                ("v".into(), d("Velocity")),
                ("cd".into(), d("Ratio")),
                ("a".into(), d("Area")),
            ],
            output: d("Force"),
            publishes: vec![],
        },
        vec![
            port("rho", "", "kg/m^3"),
            port("v", "", "m/s"),
            port("cd", "Cd", "1"),
            port("a", "A", "m^2"),
        ],
        port("drag_force", "", "N"),
    )
}

#[test]
fn a_formula_typed_as_written_becomes_a_method_the_checker_holds_sound() {
    let (sig, ports, out) = drag();
    let r = formula::read("D = 0.5 * rho * v^2 * Cd * A", &sig, &ports, &out, None);
    assert!(r.ok(), "{:?}", r.diags);
    let found: Vec<(&str, Found, Option<&str>)> = r
        .inputs
        .iter()
        .map(|x| (x.name.as_str(), x.found, x.port.as_deref()))
        .collect();
    assert_eq!(
        found,
        [
            ("rho", Found::Name, Some("rho")),
            ("v", Found::Name, Some("v")),
            ("Cd", Found::Symbol, Some("cd")),
            ("A", Found::Symbol, Some("a")),
        ]
    );
    let m = r.method.as_deref().unwrap();
    assert_eq!(
        m,
        "# Typed as: D = 0.5 * rho * v^2 * Cd * A\nlet Cd = cd\nlet A = a\nreturn 0.5 * rho * v^2 * Cd * A\n"
    );
    assert!(
        r.note
            .as_deref()
            .is_some_and(|n| n.contains("names its answer «D»")),
        "{:?}",
        r.note
    );
    // Lines before the last are worked out first.
    let r = formula::read(
        "q = 0.5 * rho * v^2\nD = q * Cd * A",
        &sig,
        &ports,
        &out,
        None,
    );
    assert!(r.ok(), "{:?}", r.diags);
    assert!(r
        .method
        .unwrap()
        .contains("let q = 0.5 * rho * v^2\nreturn q * Cd * A"));
    // A spelling with one meaning is written back, and said.
    let r = formula::read(
        "D = 0.5 × rho * v**2 · Cd * A − 0 [N]",
        &sig,
        &ports,
        &out,
        None,
    );
    assert!(r.ok(), "{:?}", r.diags);
    // A sum, a call and a constant read as the language reads them: a
    // constant and a function are no inputs.
    let f = formula::read(
        "D = max(0.5 * rho * v^2 * Cd * A, 0 [N]) + 0 * G0 * rho * v * A",
        &sig,
        &ports,
        &out,
        None,
    );
    assert!(
        f.inputs.iter().all(|x| x.name != "G0" && x.name != "max"),
        "{:?}",
        f.inputs
    );
    assert_eq!(
        r.rewrites,
        [
            "«**» is written «^»",
            "«×» is written «*»",
            "«·» is written «*»",
            "«−» is written «-»"
        ]
    );
}

#[test]
fn what_a_formula_does_not_say_is_refused_by_name_and_never_guessed() {
    let (sig, ports, out) = drag();
    let says = |typed: &str, want: &str| {
        let r = formula::read(typed, &sig, &ports, &out, None);
        assert!(!r.ok(), "{typed}: read as {:?}", r.method);
        let all: Vec<String> = r.diags.iter().map(|d| d.to_string()).collect();
        assert!(all.iter().any(|d| d.contains(want)), "{typed}: {all:?}");
        r
    };
    says(
        "D = 0.5 * ρ * v^2 * Cd * A",
        "«ρ» is not part of the method language",
    );
    says("D = 0.5 * rho * v² * Cd * A", "write a power with ^");
    says(
        "D = 0.5 rho * v^2 * Cd * A",
        "«0.5» and «rho» stand side by side",
    );
    says(
        "D = q(rho) * v^2 * Cd * A",
        "«q» is not a function of the method language: write * for a product",
    );
    // A name found nowhere is missing; a near name offered, never taken.
    let r = says("D = 0.5 * RHO * v^2 * Cd * S", "did you mean «rho»?");
    assert!(r.method.is_none());
    assert!(r.diags[0].msg.contains("«S»"), "{}", r.diags[0].msg);
    says("D = pi * rho * v^2 * Cd * A", "did you mean «PI»?");
    // The checker's word on a formula of the wrong dimension, on its line.
    let r = says("D = 0.5 * rho * v^2 * Cd", "the answer must be");
    let d = r
        .diags
        .iter()
        .find(|d| d.msg.contains("the answer must be"))
        .unwrap();
    assert_eq!(d.line, 1, "on the formula's own line");
    let (force, pressure) = (
        dim_text(quantity_dim("Force").unwrap()),
        dim_text(quantity_dim("Pressure").unwrap()),
    );
    assert!(
        d.msg.contains(&force) && d.msg.contains(&pressure),
        "both dimensions named: {}",
        d.msg
    );
    let r = says("q = 0.5 * rho * v^2\nD = q * Cd", "the answer must be");
    assert!(
        r.diags
            .iter()
            .any(|d| d.line == 2 && d.msg.contains("the answer must be")),
        "{:?}",
        r.diags
    );
    says(
        "q = 0.5 * rho * v^2\nq * Cd * A\nD = q",
        "each line before the last names what it works out",
    );
    says("D = 2 pi r", "«2» and «pi» stand side by side");
    // A line never works out one of the node's inputs, nor a constant.
    says(
        "Cd = 2.2\nD = 0.5 * rho * v^2 * Cd * A",
        "«Cd» is the input cd",
    );
    says(
        "rho = 1.2 [kg/m^3]\nD = 0.5 * rho * v^2 * Cd * A",
        "«rho» is the input rho",
    );
    says(
        "G0 = 1\nD = 0.5 * rho * v^2 * Cd * A",
        "«G0» is a kernel constant: a line works out a name of its own",
    );
    says(
        "x = q * 2\nq = 0.5 * rho * v^2\nD = x * Cd * A",
        "«q» is read before the line that works it out",
    );
    says("D = rho * end", "«end» is a word of the method language");
    says(
        "end = 1\nD = 0.5 * rho * v^2 * Cd * A",
        "«end» is a word of the method language, not a name",
    );
    says("2x = 1\nD = 0.5 * rho * v^2 * Cd * A", "«2x» is not a name");
    says("D =", "nothing follows the =");
    says("", "nothing is typed");
    says(
        "D = 0.5 * rho * v^2 * Cd * A # Anderson 4.1",
        "a comment is not part of a formula",
    );
    says(
        "D = 0.5 * rho * v [m/s] * v * Cd * A",
        "«[» here: a unit is written straight after its number",
    );
    says("D = 6371 [km * rho", "is not closed with ]");
    says("D = Sqrt(rho) * v * Cd * A", "did you mean «sqrt»?");
    says("D = 1 ÷ rho", "«÷» is not part of the method language");
    says("D = rho == rho", "the answer is a number, not a condition");
    // A constant is the constant: a port that writes it as its symbol is
    // never taken for it unsaid.
    let mut clash = ports.clone();
    clash[3].symbol = "G0".into();
    let r = formula::read("D = 0.5 * rho * v^2 * Cd * G0", &sig, &clash, &out, None);
    assert!(
        r.diags.iter().any(|d| d
            .msg
            .contains("«G0» is a kernel constant and the symbol of the input a")),
        "{:?}",
        r.diags
    );
}

#[test]
fn an_empirical_fit_checks_once_the_units_it_holds_in_are_named() {
    let power = quantity_dim("Power").unwrap();
    let mass = quantity_dim("Mass").unwrap();
    let sig = Signature {
        inputs: vec![("power".into(), power)],
        output: mass,
        publishes: vec![],
    };
    let ports = [Port {
        name: "power".into(),
        symbol: "P".into(),
        unit: "W".into(),
    }];
    let out = Port {
        name: "mass".into(),
        symbol: "m".into(),
        unit: "kg".into(),
    };
    let fit = |input: &str, unit: &str, answer: &str| Fit {
        inputs: vec![(input.into(), unit.into())],
        answer: answer.into(),
    };
    // Without its units, a power of a dimensioned value is refused, by the
    // checker, on the formula's line.
    let r = formula::read("m = 1.2 * P^0.7", &sig, &ports, &out, None);
    assert!(!r.ok(), "{:?}", r.method);
    assert!(
        r.diags
            .iter()
            .any(|d| d.line == 1 && d.severity == vleo_sheet::method::Severity::Error),
        "{:?}",
        r.diags
    );
    // Over several lines, as over one.
    let r = formula::read(
        "x = P / 2\nm = 1.2 * x^0.7",
        &sig,
        &ports,
        &out,
        Some(&fit("P", "kW", "kg")),
    );
    assert!(r.ok(), "{:?}", r.diags);
    // An input given a unit twice is refused, by name.
    let twice = Fit {
        inputs: vec![("P".into(), "kW".into()), ("power".into(), "W".into())],
        answer: "kg".into(),
    };
    let r = formula::read("m = 1.2 * P^0.7", &sig, &ports, &out, Some(&twice));
    assert!(
        r.diags
            .iter()
            .any(|d| d.msg.contains("«P» is given a unit 2 times")),
        "{:?}",
        r.diags
    );
    // With them, each input is a number in its unit, and the answer one in its.
    let r = formula::read(
        "m = 1.2 * P^0.7",
        &sig,
        &ports,
        &out,
        Some(&fit("P", "kW", "kg")),
    );
    assert!(r.ok(), "{:?}", r.diags);
    assert_eq!(
        r.method.as_deref().unwrap(),
        "# Typed as: m = 1.2 * P^0.7\nlet P = power / 1 [kW]\nreturn (1.2 * P^0.7) * 1 [kg]\n"
    );
    assert!(r.note.is_none(), "its symbol is the output's: {:?}", r.note);
    // Read by the port's own name, it is renamed, never defined twice.
    let r = formula::read(
        "mass = 1.2 * power^0.7",
        &sig,
        &ports,
        &out,
        Some(&fit("power", "kW", "kg")),
    );
    assert!(r.ok(), "{:?}", r.diags);
    assert!(
        r.method.as_deref().unwrap().contains(
            "let power_number = power / 1 [kW]\nreturn (1.2 * power_number^0.7) * 1 [kg]"
        ),
        "{:?}",
        r.method
    );
    for (f, says) in [
        (fit("Q", "kW", "kg"), "name the unit «P» is read in"),
        (fit("P", "kg", "kg"), "[kg] is a Mass, and «P» is a Power"),
        (fit("P", "kW", ""), "name the unit its answer is in"),
        (fit("P", "kW", "W"), "and the node's output is a Mass"),
        (
            fit("Q", "kW", "kg"),
            "a unit is named for «Q», which the formula does not read",
        ),
        (fit("P", "furlong", "kg"), "«furlong» is not a unit"),
        (fit("P", "kW", "furlong"), "«furlong» is not a unit"),
    ] {
        let r = formula::read("m = 1.2 * P^0.7", &sig, &ports, &out, Some(&f));
        assert!(!r.ok(), "{says}");
        let all: Vec<String> = r.diags.iter().map(|d| d.to_string()).collect();
        assert!(all.iter().any(|d| d.contains(says)), "{says}: {all:?}");
    }
}

#[test]
fn the_method_language_names_a_character_it_does_not_read() {
    let e = parse("return ρ\n").unwrap_err();
    assert!(e.msg.contains("«ρ»"), "{}", e.msg);
}
