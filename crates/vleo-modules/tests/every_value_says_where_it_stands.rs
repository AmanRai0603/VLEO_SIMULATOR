//! Every value carries its state and its maturity, and a parameter the level
//! that owns it; every closure says, over the open values behind it, whether
//! it closes for all of their range, part of it or none of it, ranks them in a
//! tornado, and names the least mature value it rests on.
//!
//! docs/SYSTEM_MODEL.md, section 4. No port in the design states its state or
//! maturity yet, so what a sheet can say is proved on the design with sheets
//! changed in memory. The design on disk is not touched.

#![cfg(feature = "std")]

use std::path::PathBuf;
use std::sync::OnceLock;

use vleo_bus::Case;
use vleo_modules::core_engine::graph::{Kind, Maturity, PortState, State};
use vleo_modules::health::{health, range, Verdict};
use vleo_modules::{opened, Graph, COMPILED};
use vleo_sheet::gate::{gate_node, Verdict as Gate};
use vleo_sheet::load::Tree;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> Tree {
    vleo_sheet::load_all(&root()).unwrap()
}

fn with_data(supply: &[(&str, f64)]) -> Case {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let scratch = std::env::temp_dir().join(format!("vleo-ports-{}", std::process::id()));
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
        supply: supply.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        ..Default::default()
    }
}

/// An output made open in memory: its owner, the gate it is due by, and, when
/// given, the range it may still take.
fn open(t: &mut Tree, id: &str, range: Option<(f64, f64)>) {
    let sh = t.sheets.get_mut(id).unwrap();
    sh.port.state = "open".into();
    sh.port.open_owner = "the solar group".into();
    sh.port.open_due = "the design review".into();
    if let Some((lo, hi)) = range {
        sh.lower = lo;
        sh.upper = hi;
    }
}

fn verdict_of(t: &Tree, case: &Case) -> (Verdict, Vec<String>, Option<f64>) {
    let g: &'static Graph = opened::graph(t).unwrap();
    let map = health(g, case);
    let k = g.find("l3_solar_ach_01").unwrap();
    let r = range(g, case, &map, k);
    let names = r
        .tornado
        .iter()
        .map(|l| g.nodes[l.node as usize].id.to_string())
        .collect();
    (
        r.verdict,
        names,
        r.tornado.first().and_then(|l| l.closes_at),
    )
}

fn port_check(t: &Tree, id: &str) -> Gate {
    gate_node(&t.sheets[id], t)
        .into_iter()
        .find(|c| c.name == "port")
        .map(|c| c.verdict)
        .unwrap_or(Gate::Pass)
}

#[test]
fn every_value_has_a_state_read_alike_by_the_generator_and_the_reader() {
    let t = tree();
    let read = opened::graph(&t).unwrap();
    for (v, var) in COMPILED.vars.iter().enumerate() {
        assert_eq!(read.vars[v].port, var.port, "{}", var.id);
    }
    // When a sheet says nothing, its row says it: a stated value is decided,
    // a requirement allocated, a row not decided yet open, the rest achieved.
    for def in COMPILED.nodes.iter() {
        let port = COMPILED.vars[def.outputs[0] as usize].port;
        let expected = if def.state == State::Empty {
            PortState::Open
        } else {
            match def.kind {
                Kind::Declared => PortState::Decided,
                Kind::Required => PortState::Allocated,
                _ => PortState::Achieved,
            }
        };
        assert_eq!(port.state, expected, "{}", def.id);
        assert_eq!(port.maturity, Maturity::Unstated, "{}", def.id);
        assert_eq!(port.parameter, None, "{}", def.id);
    }
}

#[test]
fn a_closure_says_whether_it_closes_over_the_range_of_its_open_values() {
    // Nothing behind it is open in the design: its margin is what it says.
    let t = tree();
    let (v, tornado, _) = verdict_of(&t, &with_data(&[]));
    assert_eq!((v, tornado.len()), (Verdict::NoOpenValue, 0));

    // The spread open across its declared range: it holds at both ends.
    let mut t = tree();
    open(&mut t, "sw_mean_band_spread", None);
    assert_eq!(port_check(&t, "sw_mean_band_spread"), Gate::Pass);
    let (v, tornado, _) = verdict_of(&t, &with_data(&[]));
    assert_eq!(v, Verdict::ClosesForAll);
    assert_eq!(tornado, ["sw_mean_band_spread"]);

    // And the requirement open too: it holds for part of its range, and the
    // crossing is where the requirement meets what the design sustains — the
    // tornado's first bar, the one that moves the margin most.
    open(&mut t, "l3_solar_req_01", None);
    let case = with_data(&[]);
    let (v, tornado, crossing) = verdict_of(&t, &case);
    assert_eq!(v, Verdict::ClosesForPart);
    assert_eq!(tornado, ["l3_solar_req_01", "sw_mean_band_spread"]);
    let g = opened::graph(&t).unwrap();
    let achieved = g
        .evaluate(
            &Case {
                target: "sw_f107_design_long".into(),
                mode: vleo_bus::RunMode::Branch,
                ..case.clone()
            },
            &mut vleo_modules::Scratch::for_graph(g),
        )
        .unwrap()
        .values
        .iter()
        .find(|v| v.id == "sw_f107_design_long")
        .unwrap()
        .value;
    let at = crossing.expect("the requirement crosses");
    assert!(
        (at - achieved).abs() <= 1e-9 * achieved,
        "{at} vs {achieved}"
    );

    // The requirement open only below what the design sustains, and set
    // there: it fails for all of the range.
    let mut t = tree();
    open(&mut t, "l3_solar_req_01", Some((50.0, 90.0)));
    let (v, _, _) = verdict_of(&t, &with_data(&[("l3_solar_req_01", 80.0)]));
    assert_eq!(v, Verdict::FailsForAll);
}

#[test]
fn a_closure_names_the_least_mature_value_it_rests_on() {
    // Every value measured but the spread, which is an estimate.
    let mut t = tree();
    for sh in t.sheets.values_mut() {
        sh.port.maturity = "measured".into();
    }
    t.sheets
        .get_mut("sw_mean_band_spread")
        .unwrap()
        .port
        .maturity = "estimated".into();
    assert_eq!(port_check(&t, "sw_mean_band_spread"), Gate::Pass);
    let g = opened::graph(&t).unwrap();
    let case = with_data(&[]);
    let map = health(g, &case);
    let r = range(g, &case, &map, g.find("l3_solar_ach_01").unwrap());
    let (m, rows) = &r.least_mature;
    assert_eq!(*m, Maturity::Estimated);
    let rows: Vec<&str> = rows.iter().map(|&k| g.nodes[k as usize].id).collect();
    assert_eq!(rows, ["sw_mean_band_spread"]);
}

#[test]
fn a_port_that_does_not_hold_together_is_refused_by_the_gate_by_name() {
    let refused = |edit: &dyn Fn(&mut Tree), id: &str, says: &str| {
        let mut t = tree();
        edit(&mut t);
        match port_check(&t, id) {
            Gate::Fail(why) => assert!(why.contains(says), "{says}: {why}"),
            other => panic!("{says}: {other:?}"),
        }
    };
    refused(
        &|t| t.sheets.get_mut("sw_f107_design_long").unwrap().port.state = "decided".into(),
        "sw_f107_design_long",
        "decided is a value stated at this level, and this row is computed",
    );
    refused(
        &|t| t.sheets.get_mut("sw_mean_band_spread").unwrap().port.state = "open".into(),
        "sw_mean_band_spread",
        "names who owns it (open_owner) and the gate it is due by (open_due)",
    );
    refused(
        &|t| open(t, "sw_mean_band_spread", Some((0.0, f64::INFINITY))),
        "sw_mean_band_spread",
        "carries the range it may still take",
    );
    refused(
        &|t| {
            t.sheets
                .get_mut("sw_mean_band_spread")
                .unwrap()
                .port
                .maturity = "guessed".into()
        },
        "sw_mean_band_spread",
        "maturity = 'guessed' is not one of estimated, calculated, measured",
    );
    refused(
        &|t| {
            t.sheets
                .get_mut("sw_f107_design_long")
                .unwrap()
                .port
                .parameter = "system".into()
        },
        "sw_f107_design_long",
        "a parameter is a stated value, and this row is computed",
    );
    // A stated value may be a parameter, and the reader says whose.
    let mut t = tree();
    t.sheets
        .get_mut("sw_mean_band_spread")
        .unwrap()
        .port
        .parameter = "subsystem".into();
    assert_eq!(port_check(&t, "sw_mean_band_spread"), Gate::Pass);
    let g = opened::graph(&t).unwrap();
    let v = g.vars[g.find("sw_mean_band_spread").unwrap() as usize].port;
    assert_eq!(v.parameter.map(|l| l.name()), Some("subsystem"));
}
