//! The health map says where exactly the design breaks: every node gets one
//! state, the states roll up group by group to the spacecraft, and a closure
//! that does not close is traced, by name, to the node that causes it.
//!
//! docs/OPERATING_1_0.md, section 6. The design is this repository's; the
//! reference data is the shipped bundles, verified into a store of the test's
//! own.

#![cfg(feature = "std")]

use std::path::PathBuf;
use std::sync::OnceLock;

use vleo_bus::{Case, RunMode};
use vleo_modules::core_engine::graph::NodeIdx;
use vleo_modules::health::{health, trace, Health, State};
use vleo_modules::{opened, Graph, Scratch, COMPILED};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The case with the reference data, verified once for every test here.
fn with_data() -> Case {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let scratch = std::env::temp_dir().join(format!("vleo-health-{}", std::process::id()));
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

fn k(graph: &Graph, id: &str) -> NodeIdx {
    graph
        .find(id)
        .unwrap_or_else(|| panic!("{id} is not in the design"))
}

fn state(graph: &Graph, map: &Health, id: &str) -> State {
    map.node(k(graph, id)).state
}

fn group(map: &Health, id: &str) -> State {
    map.groups.iter().find(|g| g.id == id).unwrap().state
}

const CLOSURES: [&str; 5] = [
    "l3_solar_ach_01",
    "l3_solar_ach_02",
    "l3_solar_ach_03",
    "l3_solar_ach_04",
    "l3_solar_ach_05",
];

#[test]
fn the_design_today_is_open_and_its_solar_closures_close() {
    let g: &'static Graph = &COMPILED;
    let map = health(g, &with_data());
    assert_eq!(map.nodes.len(), g.nodes.len());
    assert_eq!(
        map.counts().iter().map(|(_, n)| n).sum::<usize>(),
        g.nodes.len()
    );
    for id in CLOSURES {
        let n = map.node(k(g, id));
        assert_eq!(n.state, State::Closes, "{id}: {}", n.why);
        assert!(n.margin.is_some_and(|m| m > 0.0), "{id}: {:?}", n.margin);
    }
    // The solar group is open, by the two numbers it has not decided yet,
    // and by nothing else.
    let solar = map.groups.iter().find(|g| g.id == "l3_solar").unwrap();
    assert_eq!(solar.state, State::Open);
    let undecided: Vec<&str> = solar
        .worst
        .iter()
        .map(|&w| g.nodes[w as usize].id)
        .collect();
    assert_eq!(undecided, ["sw_band_confidence", "sw_kp_driving_slot"]);
    // A KPI whose relation is stated and never derived is open, and says so:
    // it is not a closure that closes.
    assert_eq!(state(g, &map, "kpi_gsd"), State::Open);
    assert!(map.node(k(g, "kpi_gsd")).why.contains("never derived"));
    assert_eq!(group(&map, "l3_x_closure"), State::Open);
    // The spacecraft is as bad as its worst node, and the programme's group
    // holds every node.
    assert_eq!(map.spacecraft, State::Open);
    assert_eq!(group(&map, "root"), State::Open);
    // Tight is not judged without a margin policy, and that is said.
    assert_eq!(state(g, &map, "l3_solar_ach_03"), State::Closes);
    assert!(map.notes.iter().any(|n| n.contains("no margin policy")));
}

#[test]
fn without_its_data_a_row_refuses_and_every_row_it_blocks_names_it() {
    let g: &'static Graph = &COMPILED;
    let map = health(g, &Case::default());
    let refused = map.node(k(g, "sw_central_expectation"));
    assert_eq!(refused.state, State::Refused);
    assert!(
        refused.why.contains("is not in the local store"),
        "{}",
        refused.why
    );
    // The closure two rows down is blocked, and names the row to look at, not
    // the first in the chain.
    let blocked = map.node(k(g, "l3_solar_ach_01"));
    assert_eq!(blocked.state, State::Blocked, "{}", blocked.why);
    assert_eq!(
        blocked.blocked_by.map(|b| g.nodes[b as usize].id),
        Some("sw_central_expectation")
    );
    // Refused is worse than open: it is what the solar group and the
    // spacecraft are.
    assert_eq!(group(&map, "l3_solar"), State::Refused);
    assert_eq!(map.spacecraft, State::Refused);
    // Traced from the closure, the refusal is its cause, with who owns it.
    let t = trace(g, &Case::default(), &map, k(g, "l3_solar_ach_01"));
    let c = t
        .causes
        .iter()
        .find(|c| g.nodes[c.node as usize].id == "sw_central_expectation")
        .expect("the refused row is a cause");
    assert_eq!(c.state, State::Refused);
    assert_eq!((c.group, c.owner), ("l3_solar", "environment"));
    assert!(t.levers.is_empty(), "a closure with no margin has no lever");
}

/// The design with nodes' methods changed in memory, each `(node, from, to)`,
/// run in the interpreter: the design on disk is not touched.
fn broken(edits: &[(&str, &str, &str)]) -> &'static Graph {
    let mut tree = vleo_sheet::load_all(&root()).unwrap();
    for (id, from, to) in edits {
        let m = &mut tree.sheets.get_mut(*id).unwrap().method.text;
        assert!(m.contains(from), "{id}'s method has no {from:?}");
        *m = m.replace(from, to);
    }
    opened::graph(&tree).unwrap()
}

/// The hot edge of the sustained F10.7 band with its multiplier read from
/// the wrong table.
const WRONG_Z: (&str, &str, &str) = (
    "sw_f107_design_long",
    "const z = 1.28 [1]",
    "const z = 14 [1]",
);

#[test]
fn a_deliberately_broken_node_is_traced_by_name_from_the_closure_it_breaks() {
    let g = broken(&[WRONG_Z]);
    let case = with_data();
    let map = health(g, &case);

    // The closure it feeds no longer closes.
    let closure = k(g, "l3_solar_ach_01");
    let n = map.node(closure);
    assert_eq!(n.state, State::Fails, "{}", n.why);
    assert!(n.margin.is_some_and(|m| m < 0.0), "{:?}", n.margin);
    // The node answers, but not its own cases: unproven, and why.
    let bad = map.node(k(g, "sw_f107_design_long"));
    assert_eq!(bad.state, State::Unproven, "{}", bad.why);
    assert!(
        bad.why.contains("of its own cases not reproduced"),
        "{}",
        bad.why
    );
    // It rolls up: the solar group fails, and so does the spacecraft.
    assert_eq!(group(&map, "l3_solar"), State::Fails);
    assert_eq!(map.spacecraft, State::Fails);
    // A closure it does not feed still closes: the map says where, not just
    // that.
    assert_eq!(state(g, &map, "l3_solar_ach_04"), State::Closes);

    // Traced from the closure, the broken node is named first among the
    // causes, with its group and its owner.
    let t = trace(g, &case, &map, closure);
    assert_eq!(t.state, State::Fails);
    let named: Vec<&str> = t
        .causes
        .iter()
        .map(|c| g.nodes[c.node as usize].id)
        .collect();
    assert_eq!(named.first(), Some(&"sw_f107_design_long"), "{named:?}");
    let c = &t.causes[0];
    assert_eq!(c.state, State::Unproven);
    assert_eq!((c.group, c.owner), ("l3_solar", "environment"));

    // The lever that moves the margin most is the requirement, and the level
    // at which the closure would close is the one the broken row answers.
    let lever = &t.levers[0];
    assert_eq!(g.nodes[lever.node as usize].id, "l3_solar_req_01");
    let achieved = g
        .evaluate(
            &Case {
                target: "sw_f107_design_long".into(),
                mode: RunMode::Branch,
                ..case.clone()
            },
            &mut Scratch::for_graph(g),
        )
        .unwrap()
        .values
        .iter()
        .find(|v| v.id == "sw_f107_design_long")
        .map(|v| v.value)
        .unwrap();
    let at = lever.closes_at.expect("the requirement crosses its range");
    assert!(
        (at - achieved).abs() <= 1e-9 * achieved,
        "closes at {at}, the row answers {achieved}"
    );
}

#[test]
fn two_broken_nodes_are_both_named_the_nearer_first() {
    // And the central expectation, one row further down, with its rotation
    // timescale read as a tenth of a rotation.
    let g = broken(&[
        WRONG_Z,
        (
            "sw_central_expectation",
            "const tau_days = 27 [1]",
            "const tau_days = 2.7 [1]",
        ),
    ]);
    let case = with_data();
    let map = health(g, &case);
    let t = trace(g, &case, &map, k(g, "l3_solar_ach_01"));
    let named: Vec<(&str, State)> = t
        .causes
        .iter()
        .map(|c| (g.nodes[c.node as usize].id, c.state))
        .collect();
    assert_eq!(
        named,
        [
            ("sw_f107_design_long", State::Unproven),
            ("sw_central_expectation", State::Unproven)
        ]
    );
}

#[test]
fn the_same_design_unbroken_in_the_interpreter_closes_as_compiled() {
    // The broken test's graph is the interpreter's; without the break it
    // gives the compiled map, so what the break shows is the break.
    let tree = vleo_sheet::load_all(&root()).unwrap();
    let g = opened::graph(&tree).unwrap();
    let case = with_data();
    let (a, b) = (health(g, &case), health(&COMPILED, &case));
    for (x, y) in a.nodes.iter().zip(&b.nodes) {
        assert_eq!(
            (x.state, x.margin),
            (y.state, y.margin),
            "{}",
            g.nodes[x.node as usize].id
        );
    }
}
