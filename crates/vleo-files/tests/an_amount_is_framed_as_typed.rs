//! An amount is framed as a person types it (docs/briefs/F8.md, F8-a, item
//! 7): a port filled, or an allocation, is given `55 ± 10% mN` or
//! `2.5e-11 to 1.5e-10 kg/m^3`, read by the method language's `amount::read`
//! and written as typed, its unit and its numbers, never converted.
//!
//! The systems' file of a small programme of invented people, Pat the
//! programme manager, Sam the system engineer and Rui a subsystem engineer,
//! holds two blocks invented for the check: `sys_thrust`, an open leaf Sam
//! fills, and `sys_drag`, with a density in kg/m^3 and a power in W, which
//! Sam mounts for Rui and allocates to. Nothing here is computed: every
//! expected value is written in the test, as typed.
//!
//! - filled: a spread's nominal is the port's value and its ends its range
//!   (question 67), with its symbol and both reasons; filled again with less,
//!   the port keeps what it is not given; refused by name when it would keep
//!   a value or a range in another unit, when its value is outside its
//!   range, and when an end is not a number;
//! - allocated: a range as typed; a spread gives its range only, and says
//!   so; an allocation in kg bounding an output in W is refused, both
//!   dimensions named;
//! - what does not read is refused by name, with the reader's own fix.

use vleo_files::breakdown::{self, Edit, Part};
use vleo_files::chain::{self, Registry, FILE};
use vleo_files::keys::SigningKey;
use vleo_files::meta::Kind;
use vleo_files::model::*;

const APP: &str = "vleo 1.0.0 (test)";
const AT: &str = "2026-10-19T09:00:00Z";
const WHY: &str = "The thruster and the drag it answers are framed with their bands.";

fn s(v: &str) -> String {
    v.to_string()
}

fn key(seed: u8) -> SigningKey {
    SigningKey::from_seed([seed; 32])
}

fn register(f: &mut File, name: &str, role: &str, k: &SigningKey) {
    f.people.push(Person {
        name: s(name),
        role: s(role),
        deputy_for: s(""),
    });
    f.keys.push(PersonKey {
        person: s(name),
        public_key: k.public().to_hex(),
        registered_at: s("2026-01-01"),
        registered_by: s("Pat Morgan"),
        revoked_from: s(""),
        proof: vleo_files::people::proof(k, name),
    });
}

fn block(uid: &str, behaviour: &str, question: &str) -> Block {
    Block {
        uid: s(uid),
        id: s(uid),
        behaviour: s(behaviour),
        question: s(question),
        contract_version: 1,
        ..Block::default()
    }
}

fn out_port(uid: &str, name: &str, unit: &str) -> Port {
    Port {
        block_uid: s(uid),
        direction: s("out"),
        name: s(name),
        port_type: s("number"),
        unit: s(unit),
        state: s("open"),
        ..Port::default()
    }
}

fn group_file(id: &str, writer: &str) -> File {
    let mut f = File::new(Kind::Group, APP);
    f.meta.insert(s("group_id"), s(id));
    f.meta.insert(s("writer"), s(writer));
    f.meta.insert(s("based_on"), s(""));
    f
}

fn signed(mut f: File, who: &str, k: &SigningKey) -> File {
    let row = chain::sign(&f, FILE, who, k, "2026-02-01", "ok", "").unwrap();
    f.signatures.push(row);
    f
}

// Pat (11), Sam (12), Rui (15).
fn programme() -> Registry {
    let mut f = group_file("programme", "Pat Morgan");
    register(&mut f, "Pat Morgan", "programme manager", &key(11));
    register(&mut f, "Sam Engel", "system engineer", &key(12));
    register(&mut f, "Rui Ray", "subsystem engineer", &key(15));
    f.blocks.push(block("root", "children", ""));
    f.mounts.push(Mount {
        block_uid: s("root"),
        group_id: s("systems"),
        release: s(""),
    });
    f.assignments.push(Assignment {
        block_uid: s("root"),
        person: s("Sam Engel"),
        contract_version: 1,
    });
    chain::open_programme(
        signed(f, "Pat Morgan", &key(11)),
        &key(11).public().fingerprint(),
    )
    .unwrap()
}

fn systems_file() -> File {
    let mut f = group_file("systems", "Sam Engel");
    f.blocks.push(block(
        "sys_thrust",
        "open",
        "What thrust does the thruster give?",
    ));
    f.blocks.push(block(
        "sys_drag",
        "open",
        "What density and power does the drag group answer within?",
    ));
    f.ports.push(out_port("sys_drag", "density", "kg/m^3"));
    f.ports.push(out_port("sys_drag", "power", "W"));
    signed(f, "Sam Engel", &key(12))
}

/// `systems` with `e` made by Sam.
fn framed(systems: &File, e: &Edit) -> Result<File, String> {
    let pr = programme();
    let elsewhere = vec![pr.file.clone()];
    breakdown::edit(
        systems,
        &[],
        &elsewhere,
        e,
        WHY,
        "Sam Engel",
        &key(12),
        AT,
        &pr,
        &pr,
    )
    .map(|(f, _)| f)
    .map_err(|e| e.message().to_string())
}

/// `sys_thrust`'s output `thrust`, given `typed` as its amount, and the rest
/// of what a fill gives.
fn thrust(typed: &str, symbol: &str, range_reason: &str, says: &str) -> (Port, String) {
    let p = Port {
        direction: s("out"),
        name: s("thrust"),
        port_type: s("number"),
        state: s("open"),
        symbol: s(symbol),
        range_reason: s(range_reason),
        says: s(says),
        ..Port::default()
    };
    if typed.is_empty() {
        (p, String::new())
    } else {
        breakdown::with_amount(&p, typed, false).expect("the amount reads")
    }
}

fn fill(f: &File, p: Port) -> Result<File, String> {
    framed(
        f,
        &Edit::Fill {
            block: s("sys_thrust"),
            part: Part::Port(Box::new(p)),
        },
    )
}

fn port<'a>(f: &'a File, block: &str, name: &str) -> &'a Port {
    f.ports
        .iter()
        .find(|p| p.block_uid == block && p.name == name)
        .unwrap()
}

fn held(p: &Port) -> [&str; 4] {
    [&p.unit, &p.value, &p.lower, &p.upper]
}

#[test]
fn a_port_is_filled_with_an_amount_as_typed_and_keeps_what_it_is_not_given() {
    let (p, said) = thrust(
        "55 ± 10% mN",
        "T",
        "the thruster's datasheet band",
        "the thrust it gives",
    );
    assert!(said.contains("a Force"), "{said}");
    let f = fill(&systems_file(), p).expect("Sam fills the thrust");
    let t = port(&f, "sys_thrust", "thrust");
    assert_eq!(held(t), ["mN", "55", "49.5", "60.5"]);
    assert_eq!(
        [t.symbol.as_str(), t.range_reason.as_str(), t.says.as_str()],
        ["T", "the thruster's datasheet band", "the thrust it gives"]
    );
    chain::open_systems(&f, &programme()).expect("it opens");

    // Filled again with its state only: it keeps its unit, value, range,
    // symbol and both reasons.
    let (mut again, _) = thrust("", "", "", "");
    again.state = s("decided");
    let g = fill(&f, again).expect("filled again");
    let t = port(&g, "sys_thrust", "thrust");
    assert_eq!(
        (held(t), t.state.as_str(), t.symbol.as_str()),
        (["mN", "55", "49.5", "60.5"], "decided", "T")
    );
    assert_eq!(t.range_reason, "the thruster's datasheet band");

    // A value in another unit, keeping its range in mN: refused, never
    // converted.
    let (newton, _) = thrust("0.05 N", "", "", "");
    let e = fill(&f, newton).unwrap_err();
    assert!(
        e.contains("keeps its range in [mN], and is filled in [N]"),
        "{e}"
    );
    // A range its kept value is outside of.
    let (narrow, _) = thrust("50 to 52 mN", "", "", "");
    let e = fill(&f, narrow).unwrap_err();
    assert!(
        e.contains("has the value 55, outside its range 50 to 52"),
        "{e}"
    );
    // An end that is not a number, typed apart.
    let (mut bad, _) = thrust("", "", "", "");
    (bad.lower, bad.upper) = (s("abc"), s("60"));
    let e = fill(&systems_file(), bad).unwrap_err();
    assert!(
        e.contains("has the lower end \"abc\", which is not a number"),
        "{e}"
    );
    // A value outside a range typed with it.
    let (mut out, _) = thrust("", "", "", "");
    (out.unit, out.value, out.lower, out.upper) = (s("mN"), s("70"), s("49.5"), s("60.5"));
    let e = fill(&systems_file(), out).unwrap_err();
    assert!(e.contains("has the value 70, outside its range"), "{e}");
}

fn drag_mounted() -> File {
    framed(
        &systems_file(),
        &Edit::Mount {
            block: s("sys_drag"),
            group: s("l3_drag"),
            owner: s("Rui Ray"),
        },
    )
    .expect("Sam mounts drag for Rui")
}

fn allocate(name: &str, typed: &str, bounds: &str) -> Result<(File, String), String> {
    let p = Port {
        name: s(name),
        port_type: s("number"),
        range_reason: s("the drag group's design band"),
        ..Port::default()
    };
    let (p, said) = breakdown::with_amount(&p, typed, true).map_err(|e| e.message().to_string())?;
    framed(
        &drag_mounted(),
        &Edit::Allocate {
            block: s("sys_drag"),
            port: Box::new(p),
            bounds: s(bounds),
            sense: s("<="),
            says: s("the drag group stays within what it is given"),
        },
    )
    .map(|f| (f, said))
}

#[test]
fn an_allocation_is_typed_as_an_amount_of_the_output_s_dimension() {
    let (f, said) =
        allocate("density_allocated", "2.5e-11 to 1.5e-10 kg/m^3", "density").expect("allocated");
    let a = port(&f, "sys_drag", "density_allocated");
    assert_eq!(held(a), ["kg/m^3", "", "2.5e-11", "1.5e-10"]);
    assert_eq!(a.range_reason, "the drag group's design band");
    assert!(said.contains("a MassDensity"), "{said}");
    chain::open_systems(&f, &programme()).expect("it opens");

    // A spread: an allocation is a value or a range, so it takes the range,
    // and says so.
    let (f, said) = allocate("power_allocated", "120 ± 5% W", "power").expect("allocated");
    assert_eq!(
        held(port(&f, "sys_drag", "power_allocated")),
        ["W", "", "114", "126"]
    );
    assert!(said.contains("it takes the range, 114 to 126 W"), "{said}");
    // And a value the port held before is not kept beside the range.
    let held_one = Port {
        name: s("power_allocated"),
        port_type: s("number"),
        value: s("7"),
        ..Port::default()
    };
    let (p, _) = breakdown::with_amount(&held_one, "120 ± 5% W", true).unwrap();
    assert_eq!(held(&p), ["W", "", "114", "126"]);

    // A mass bounding a power: refused, both dimensions named.
    let e = allocate("power_allocated", "40 kg", "power").unwrap_err();
    assert!(
        e.contains("is in [kg], of dimension kg, and bounds power, in [W], of dimension")
            && e.contains("an allocation is of the dimension of the output it bounds"),
        "{e}"
    );
    // A pure number bounding a density, the same.
    let e = allocate("density_allocated", "3", "density").unwrap_err();
    assert!(e.contains("is in no unit"), "{e}");
}

#[test]
fn an_amount_that_does_not_read_is_refused_with_the_reader_s_fix() {
    let p = Port {
        name: s("thrust"),
        port_type: s("number"),
        ..Port::default()
    };
    for (typed, says) in [
        ("1 to 2 kg/m3", "kg/m3"),
        ("1,5 mN", "comma"),
        ("2 km to 1 km", "thrust's amount \"2 km to 1 km\""),
    ] {
        let e = breakdown::with_amount(&p, typed, false)
            .unwrap_err()
            .message()
            .to_string();
        assert!(e.contains(says), "{typed}: {e}");
    }
}
