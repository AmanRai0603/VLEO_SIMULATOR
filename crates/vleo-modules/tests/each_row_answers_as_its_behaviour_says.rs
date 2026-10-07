//! Every row's answer is exactly one behaviour (docs/SYSTEM_MODEL.md): a
//! method, a built-in relation, a stated value, a lookup, its children, or
//! open. The generator and the graph read at run time name the same one for
//! every row; an open row refuses by its own id; a table is read by the engine
//! and refuses outside its rows; and a row answered by its children takes
//! their port, keeping its own relation as its estimate.
//!
//! No row of the design is a lookup or answered by its children yet, so those
//! two are proved on the design with one sheet changed in memory. The design
//! on disk is not touched.

#![cfg(feature = "std")]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::fault::{Edge, Fault};
use vleo_modules::core_engine::graph::Behaviour;
use vleo_modules::{opened, Graph, Scratch, COMPILED};
use vleo_sheet::gate::{gate_node, Verdict};
use vleo_sheet::load::Tree;
use vleo_sheet::model::{ChildrenOf, Lookup};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> Tree {
    vleo_sheet::load_all(&root()).unwrap()
}

/// The case with the reference data, verified once for every test here.
fn with_data() -> Case {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let scratch = std::env::temp_dir().join(format!("vleo-behaviour-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::env::set_var("VLEO_DATA", scratch.join("data"));
        let mut store = vleo_data::Store::open(&scratch.join("data"));
        store
            .sync(&vleo_data::Source::Shipped(root().join("bundles")))
            .expect("the shipped bundles verify");
        store.verified_names()
    });
    Case {
        data: names.clone(),
        ..Default::default()
    }
}

/// One row's answer and its fault, if it refused, on `graph`.
fn run(graph: &'static Graph, id: &str, supply: &[(&str, f64)]) -> (Option<f64>, Option<String>) {
    let case = Case {
        target: id.into(),
        mode: RunMode::Branch,
        supply: supply.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        ..with_data()
    };
    let r = graph
        .evaluate(&case, &mut Scratch::for_graph(graph))
        .unwrap();
    (
        r.values.iter().find(|v| v.id == id).map(|v| v.value),
        r.blocked
            .iter()
            .find(|b| b.id == id)
            .map(|b| b.message.clone()),
    )
}

fn gate(tree: &Tree, id: &str, check: &str) -> Verdict {
    gate_node(&tree.sheets[id], tree)
        .into_iter()
        .find(|c| c.name == check)
        .unwrap_or_else(|| panic!("{id} has no '{check}' check"))
        .verdict
}

#[test]
fn every_row_has_one_behaviour_and_the_generator_and_the_reader_agree() {
    let t = tree();
    let read = opened::graph(&t).unwrap();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (k, def) in COMPILED.nodes.iter().enumerate() {
        let sh = &t.sheets[def.id];
        assert_eq!(def.behaviour.name(), sh.behaviour(), "{}", def.id);
        assert_eq!(read.nodes[k].behaviour, def.behaviour, "{}", def.id);
        *counts.entry(def.behaviour.name()).or_default() += 1;
    }
    // Counted from the sheets, by what each says of itself.
    let seeded = t.sheets.values().filter(|s| s.is_seeded()).count();
    let with_method = t
        .sheets
        .values()
        .filter(|s| !s.is_seeded() && !s.is_declared() && !s.method.text.trim().is_empty())
        .count();
    assert_eq!(counts["open"], seeded);
    assert_eq!(counts["method"], with_method);
    assert_eq!(counts.get("lookup"), None, "no row is a lookup yet");
    assert_eq!(
        counts.get("children"),
        None,
        "no row is answered by its children yet"
    );
    assert_eq!(counts.values().sum::<usize>(), COMPILED.nodes.len());
}

#[test]
fn an_open_row_refuses_by_its_own_id() {
    let (k, def) = COMPILED
        .nodes
        .iter()
        .enumerate()
        .find(|(_, d)| d.behaviour == Behaviour::Open && !d.inputs.is_empty())
        .expect("an open row that reads something");
    let mut out = [0.0; 8];
    let ins = vec![1.0; def.inputs.len()];
    let e = COMPILED
        .call(k, &ins, &mut out[..def.outputs.len()])
        .expect_err("an open row answered");
    assert_eq!(e, Fault::NotRun { node: def.id });
}

/// `sw_f107_design_long`, its method replaced in memory by a table read along
/// its spread, a declared input a case can set.
fn with_a_table(x: Vec<f64>, y: Vec<f64>, read: &str) -> Tree {
    let mut t = tree();
    let sh = t.sheets.get_mut("sw_f107_design_long").unwrap();
    sh.method.text.clear();
    sh.steps.clear();
    sh.lookup = Some(Lookup {
        by: "spread".into(),
        x,
        y,
        read: read.into(),
    });
    t
}

#[test]
fn a_table_is_read_by_the_engine_and_refuses_outside_its_rows() {
    let spread = "sw_mean_band_spread";
    // A straight line from (0, 1) to (40, 100): at 20, halfway, 50.5. (The
    // spread's own declared range ends at 45 sfu.)
    let t = with_a_table(vec![0.0, 40.0], vec![1.0, 100.0], "linear");
    assert_eq!(gate(&t, "sw_f107_design_long", "lookup"), Verdict::Pass);
    let g = opened::interpreting(&t).unwrap();
    let k = g.find("sw_f107_design_long").unwrap() as usize;
    assert_eq!(g.nodes[k].behaviour.name(), "lookup");
    assert_eq!(
        run(g, "sw_f107_design_long", &[(spread, 20.0)]).0,
        Some(50.5)
    );
    // At a row, the row's own answer.
    assert_eq!(
        run(g, "sw_f107_design_long", &[(spread, 40.0)]).0,
        Some(100.0)
    );

    // Read in the logarithm of the answer, halfway from 1 to 100 is 10.
    let t = with_a_table(vec![0.0, 40.0], vec![1.0, 100.0], "log");
    let g = opened::interpreting(&t).unwrap();
    let (v, _) = run(g, "sw_f107_design_long", &[(spread, 20.0)]);
    assert!((v.unwrap() - 10.0).abs() < 1e-12, "{v:?}");

    // Outside its rows it says nothing, and the row says so, with the edge
    // it broke: never the value at its end.
    let (v, why) = run(g, "sw_f107_design_long", &[(spread, 42.0)]);
    assert_eq!(v, None);
    let why = why.expect("the row refused");
    assert!(
        why.contains("sw_f107_design_long") && why.contains("outside its first and last row"),
        "{why}"
    );
    let k = g.find("sw_f107_design_long").unwrap() as usize;
    let def = &g.nodes[k];
    let by = def
        .inputs
        .iter()
        .position(|&v| g.vars[v as usize].id == spread)
        .unwrap();
    let mut ins = vec![0.0; def.inputs.len()];
    ins[by] = 42.0;
    let mut out = [0.0; 1];
    match g.call(k, &ins, &mut out) {
        Err(Fault::OutOfDomain { edge, bound, .. }) => {
            assert_eq!((edge, bound), (Edge::Upper, 40.0));
        }
        other => panic!("{other:?}"),
    }
    // And what reads it is blocked by it, by name.
    let (_, why) = run(g, "l3_solar_ach_01", &[(spread, 42.0)]);
    assert!(why.is_some_and(|w| w.contains("sw_f107_design_long")));
}

#[test]
fn a_table_that_does_not_hold_together_is_refused_by_the_gate_by_name() {
    for (x, y, read, says) in [
        (
            vec![0.0, 100.0, 50.0],
            vec![1.0, 2.0, 3.0],
            "linear",
            "x must rise",
        ),
        (vec![0.0, 100.0], vec![1.0], "linear", "the same length"),
        (
            vec![0.0, 100.0],
            vec![0.0, 100.0],
            "log",
            "needs every y above zero",
        ),
        (
            vec![0.0, 100.0],
            vec![1.0, 100.0],
            "cubic",
            "neither 'linear' nor 'log'",
        ),
    ] {
        let t = with_a_table(x, y, read);
        match gate(&t, "sw_f107_design_long", "lookup") {
            Verdict::Fail(why) => assert!(why.contains(says), "{why}"),
            other => panic!("{says}: {other:?}"),
        }
    }
    // By a binding it has not got, and with a method beside the table.
    let mut t = with_a_table(vec![0.0, 1.0], vec![1.0, 2.0], "linear");
    {
        let sh = t.sheets.get_mut("sw_f107_design_long").unwrap();
        sh.lookup.as_mut().unwrap().by = "altitude".into();
        sh.method.text = "return central".into();
    }
    match gate(&t, "sw_f107_design_long", "lookup") {
        Verdict::Fail(why) => {
            assert!(
                why.contains("by = 'altitude' names none of its inputs"),
                "{why}"
            );
            assert!(why.contains("has one behaviour"), "{why}");
        }
        other => panic!("{other:?}"),
    }
}

/// `sys_space_environment_f10_7`, a pass-through of the solar group's
/// crossing, answered by its children in the solar group — and, so the two
/// can be told apart, given an estimate of its own that doubles it.
fn answered_by_children(group: &str) -> Tree {
    let mut t = tree();
    let sh = t.sheets.get_mut("sys_space_environment_f10_7").unwrap();
    sh.method.text = "return crossing * 2".into();
    sh.children = Some(ChildrenOf {
        group: group.into(),
        from: vec!["crossing".into()],
    });
    t
}

#[test]
fn a_row_answered_by_its_children_takes_their_port_and_keeps_its_estimate() {
    let t = answered_by_children("l3_solar");
    assert_eq!(
        gate(&t, "sys_space_environment_f10_7", "children"),
        Verdict::Pass
    );
    let g = opened::interpreting(&t).unwrap();
    let k = g.find("sys_space_environment_f10_7").unwrap() as usize;
    assert_eq!(g.nodes[k].behaviour.name(), "children");
    let (answer, why) = run(g, "sys_space_environment_f10_7", &[]);
    let (port, _) = run(g, "l3_solar_interface", &[]);
    assert!(why.is_none(), "{why:?}");
    assert_eq!(answer, port, "the children's port is the answer");
    // Its own relation stays, as its estimate: here, twice the port.
    let ins = [port.unwrap()];
    let mut out = [0.0; 1];
    g.estimate(k, &ins, &mut out).unwrap();
    assert_eq!(out[0], 2.0 * port.unwrap());
    // Its own cases test the estimate it kept — which no longer reproduces
    // them — not the copy of a port.
    assert!(g.fixture_verdicts(k as u16).iter().any(|v| !v.passed));
}

#[test]
fn children_that_are_not_the_group_s_are_refused_by_the_gate_by_name() {
    // A group the crossing's producer is not in.
    let t = answered_by_children("l3_power");
    match gate(&t, "sys_space_environment_f10_7", "children") {
        Verdict::Fail(why) => {
            assert!(
                why.contains("is not a port of a child in l3_power"),
                "{why}"
            )
        }
        other => panic!("{other:?}"),
    }
    // A group that does not exist, and a port it does not read.
    let mut t = answered_by_children("l3_nowhere");
    t.sheets
        .get_mut("sys_space_environment_f10_7")
        .unwrap()
        .children
        .as_mut()
        .unwrap()
        .from = vec!["flux".into()];
    match gate(&t, "sys_space_environment_f10_7", "children") {
        Verdict::Fail(why) => {
            assert!(why.contains("'l3_nowhere' is not a group"), "{why}");
            assert!(
                why.contains("from names 'flux', none of its inputs"),
                "{why}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn each_graph_keeps_its_own_cases_answers_and_no_other_s() {
    // A graph runs a row's cases once and keeps what they gave. Two graphs
    // with the same row, one with its method changed, give that row's
    // cases different answers, whichever is asked first and however often.
    let id = "sys_space_environment_f10_7";
    let same = opened::interpreting(&tree()).unwrap();
    let mut t = tree();
    t.sheets.get_mut(id).unwrap().method.text = "return crossing * 2".into();
    let changed = opened::interpreting(&t).unwrap();
    let k = same.find(id).unwrap();
    assert_eq!(changed.find(id), Some(k));
    let passes = |g: &Graph| g.fixture_verdicts(k).iter().all(|v| v.passed);
    assert!(passes(same), "today's method reproduces its cases");
    assert!(
        !passes(changed),
        "a doubled identity reproduces none of them"
    );
    assert!(passes(same), "and asking again gives the first graph's own");
}
