//! The engine reads every unit with the method language's one reader
//! (docs/briefs/F8.md, F8-a1): a design's machine name (`Kilometre`), a
//! release's symbol (`km`), or a unit the reader reads that the engine
//! holds. Anything else refuses its row by name, and is never read as a pure
//! number: before, `km` reached today's design as a pure number, its range
//! read as metres by a factor of a thousand, and `furlong` as a number of
//! nothing.
//!
//! A row of `design/` in kilometres is written, in memory, in the other
//! spellings a design may hold.

#![cfg(feature = "std")]

use std::path::PathBuf;

use vleo_modules::core_engine::fault::Fault;
use vleo_modules::opened;
use vleo_sheet::method::unit_named;
use vleo_units::Unit;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn every_spelling_of_a_unit_is_read_by_the_one_reader() {
    assert_eq!(unit_named("Kilometre"), Ok(Unit::Kilometre));
    assert_eq!(unit_named("km"), Ok(Unit::Kilometre));
    assert_eq!(unit_named(" km "), Ok(Unit::Kilometre));
    assert_eq!(unit_named("kg.m/s^2"), Ok(Unit::Newton));
    assert_eq!(unit_named(""), Ok(Unit::One));
    assert_eq!(unit_named("1"), Ok(Unit::One));
    // A symbol is the unit it names, before any other of its dimension and
    // factor: rad is the radian, not the pure number.
    assert_eq!(unit_named("rad"), Ok(Unit::Radian));
    assert_eq!(unit_named("dB"), Ok(Unit::Decibel));
    let e = unit_named("furlong").unwrap_err();
    assert!(e.contains("the unit «furlong» does not read"), "{e}");
    let e = unit_named("mN/kW").unwrap_err();
    assert!(
        e.contains("the unit «mN/kW» reads as s/m") && e.contains("the engine holds no such unit"),
        "{e}"
    );
}

#[test]
fn a_row_in_kilometres_reads_alike_in_every_spelling_and_refuses_in_none() {
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    let id = tree
        .sheets
        .iter()
        .find(|(_, sh)| {
            sh.unit == "Kilometre"
                && sh.is_declared()
                && sh.inputs.is_empty()
                && sh.lower < sh.upper
        })
        .map(|(id, _)| id.clone())
        .expect("a stated row of design/ in kilometres");
    let read = |tree: &vleo_sheet::Tree| {
        let g = opened::graph(tree).unwrap();
        let k = g.find(&id).unwrap();
        let limit = &g.vars[k as usize].limit;
        (
            (limit.lower, limit.upper),
            g.vars[k as usize].unit,
            g.probe(k, &[]),
        )
    };
    let (range, unit, value) = read(&tree);
    assert_eq!(unit, Unit::Kilometre);
    let value = value.expect("the row states its value");
    // A release's symbol, the same row: the same range in metres, the same value.
    tree.sheets.get_mut(&id).unwrap().unit = "km".into();
    let (r, u, v) = read(&tree);
    assert_eq!((r, u), (range, Unit::Kilometre), "{id} in km");
    assert_eq!(v.unwrap(), value, "{id} in km");
    // A unit the reader does not read, and one the engine holds none of:
    // each refuses the row by name, and nothing reads it as a pure number.
    for (unit, says) in [
        ("furlong", "the unit «furlong» does not read"),
        ("mN/kW", "the engine holds no such unit"),
    ] {
        tree.sheets.get_mut(&id).unwrap().unit = unit.into();
        let (_, _, v) = read(&tree);
        match v {
            Err(Fault::Refused { node, reason }) => {
                assert_eq!(node, id);
                assert!(reason.contains(says), "{unit}: {reason}");
            }
            other => panic!("{unit}: {other:?}"),
        }
    }
}

#[test]
fn the_page_shows_a_unit_it_does_not_read_as_written_and_no_number_converted_by_it() {
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    let id = tree
        .sheets
        .iter()
        .find(|(_, sh)| sh.unit == "Kilometre" && sh.lower < sh.upper)
        .map(|(id, _)| id.clone())
        .expect("a row of design/ in kilometres");
    let row = |tree: &vleo_sheet::Tree| {
        let json = vleo_sheet::page::index_json(tree);
        let at = json.find(&format!("\"id\":\"{id}\"")).unwrap();
        let line = &json[at..];
        let line = &line[..line.find('\n').unwrap()];
        let field = |k: &str| {
            let from = line.find(&format!("\"{k}\":")).unwrap() + k.len() + 3;
            line[from..]
                .split(',')
                .next()
                .unwrap()
                .trim_matches('"')
                .to_string()
        };
        (field("unit"), field("lo"), field("hi"))
    };
    let (unit, lo, hi) = row(&tree);
    assert_eq!(unit, "km");
    tree.sheets.get_mut(&id).unwrap().unit = "km".into();
    assert_eq!(row(&tree), (unit, lo, hi), "the same in either spelling");
    tree.sheets.get_mut(&id).unwrap().unit = "furlong".into();
    assert_eq!(row(&tree), ("furlong".into(), "null".into(), "null".into()));
}
