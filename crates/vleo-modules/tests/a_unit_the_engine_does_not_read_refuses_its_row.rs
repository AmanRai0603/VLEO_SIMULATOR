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

/// `id`'s answer refused, by its own name, its reason saying `says`.
fn refuses(g: &vleo_modules::Graph, id: &str, says: &str) {
    let k = g.find(id).unwrap();
    let inputs = vec![0.0; g.nodes[k as usize].inputs.len()];
    match g.probe(k, &inputs) {
        Err(Fault::Refused { node, reason }) => {
            assert_eq!(node, id);
            assert!(reason.contains(says), "{id}: {reason}");
        }
        other => panic!("{id}: {other:?}"),
    }
}

#[test]
fn every_way_a_row_answers_or_is_given_a_value_refuses_by_an_unread_unit() {
    use vleo_modules::core_engine::graph::Behaviour;
    use vleo_sheet::model::{ChildrenOf, Lookup};
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    // No row of design/ is a lookup or answered by its children yet, so one
    // of each is made in memory as `each_row_answers_as_its_behaviour_says`
    // makes them; a method, and a row that publishes a value beside its
    // answer, are found in design/ as they are.
    let (lookup, children) = ("sw_f107_design_long", "sys_space_environment_f10_7");
    {
        let sh = tree.sheets.get_mut(lookup).unwrap();
        sh.method.text.clear();
        sh.steps.clear();
        sh.lookup = Some(Lookup {
            by: "spread".into(),
            x: vec![0.0, 1.0],
            y: vec![1.0, 2.0],
            read: "linear".into(),
        });
        let sh = tree.sheets.get_mut(children).unwrap();
        sh.method.text = "return crossing * 2".into();
        sh.children = Some(ChildrenOf {
            group: "l3_solar".into(),
            from: vec!["crossing".into()],
        });
    }
    let (method, publishes) = {
        let g = opened::graph(&tree).unwrap();
        let of = |want: &dyn Fn(usize) -> bool| -> String {
            (0..g.nodes.len())
                .find(|&k| want(k) && !tree.sheets[g.nodes[k].id].is_seeded())
                .map(|k| g.nodes[k].id.to_string())
                .expect("a row of design/ of that kind")
        };
        for (id, name) in [(lookup, "lookup"), (children, "children")] {
            let k = g.find(id).unwrap() as usize;
            assert_eq!(g.nodes[k].behaviour.name(), name, "{id}");
            assert!(
                g.unread[k].is_none(),
                "{id} reads before its unit is changed"
            );
        }
        (
            of(&|k| {
                g.nodes[k].behaviour == Behaviour::Method
                    && matches!(g.run.get(k), Some(Some(_)))
                    && !g.nodes[k].inputs.is_empty()
            }),
            of(&|k| g.nodes[k].outputs.len() > 1),
        )
    };
    let (lookup, children) = (lookup.to_string(), children.to_string());
    for id in [&method, &lookup, &children] {
        let was = std::mem::replace(
            &mut tree.sheets.get_mut(id.as_str()).unwrap().unit,
            "furlong".into(),
        );
        let g = opened::graph(&tree).unwrap();
        refuses(g, id, "the unit «furlong» does not read");
        // And as its kept estimate, for a row its children answer.
        let k = g.find(id).unwrap() as usize;
        let mut out = vec![0.0; g.nodes[k].outputs.len()];
        let ins = vec![0.0; g.nodes[k].inputs.len()];
        assert!(
            matches!(g.estimate(k, &ins, &mut out), Err(Fault::Refused { .. })),
            "{id}'s estimate"
        );
        tree.sheets.get_mut(id.as_str()).unwrap().unit = was;
    }
    let pb = tree.sheets.get_mut(publishes.as_str()).unwrap();
    let symbol = pb.publishes[0].symbol.clone();
    pb.publishes[0].unit = "furlong".into();
    refuses(
        opened::graph(&tree).unwrap(),
        &publishes,
        &format!("the value {symbol} it publishes: the unit «furlong» does not read"),
    );
}

#[test]
fn a_value_given_to_a_row_whose_unit_does_not_read_is_refused_never_taken() {
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    let id = tree
        .sheets
        .iter()
        .find(|(_, sh)| sh.unit == "Kilometre" && sh.is_declared() && sh.inputs.is_empty())
        .map(|(id, _)| id.clone())
        .unwrap();
    tree.sheets.get_mut(&id).unwrap().unit = "mN/kW".into();
    let g = opened::graph(&tree).unwrap();
    let case = vleo_modules::bus::Case {
        target: id.clone(),
        supply: vec![(id.clone(), 20.0)],
        ..Default::default()
    };
    match g.evaluate(&case, &mut vleo_modules::Scratch::for_graph(g)) {
        Err(Fault::Refused { node, reason }) => {
            assert_eq!(node, id);
            assert!(reason.contains("«mN/kW»"), "{reason}");
        }
        other => panic!("{id} given 20: {other:?}"),
    }
}

#[test]
fn a_row_nobody_has_specified_is_not_run_whatever_its_unit() {
    let mut tree = vleo_files::convert::open(&root()).unwrap().0;
    let id = tree
        .sheets
        .iter()
        .find(|(_, sh)| sh.is_seeded())
        .map(|(id, _)| id.clone())
        .expect("a row of design/ nobody has specified");
    tree.sheets.get_mut(&id).unwrap().unit = "furlong".into();
    let g = opened::graph(&tree).unwrap();
    let k = g.find(&id).unwrap();
    assert!(g.unread[k as usize].is_none(), "{id} is seeded");
    let inputs = vec![0.0; g.nodes[k as usize].inputs.len()];
    match g.probe(k, &inputs) {
        Err(Fault::NotRun { node }) => assert_eq!(node, id),
        other => panic!("{id}: {other:?}"),
    }
}

#[test]
fn a_per_cent_is_a_hundredth_and_a_unit_two_units_are_is_named_neither() {
    assert_eq!(unit_named("%"), Ok(Unit::Percent));
    assert_eq!(unit_named("%").unwrap().si_factor(), 0.01);
    assert_eq!(unit_named("Hz"), Ok(Unit::Hertz));
    assert_eq!(unit_named("J"), Ok(Unit::Joule));
    for t in ["1/s", "kg.m^2/s^2"] {
        let e = unit_named(t).unwrap_err();
        assert!(e.contains(&format!("the unit «{t}» is each of")), "{e}");
    }
}
