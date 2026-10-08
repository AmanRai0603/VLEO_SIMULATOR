//! The design the engine runs, and what it answers, each have a fingerprint:
//! two computers that print the same two run the same design and give every
//! answer alike, to the last bit (docs/PLAN_1_0.md, phase D: "today's design
//! is built identically on two computers from the same drive"). CI compares
//! them across Linux, Windows and macOS; this holds what they are made of.

#![cfg(feature = "std")]

use std::path::PathBuf;
use std::sync::OnceLock;

use vleo_bus::Case;
use vleo_modules::{opened, Graph};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tree() -> vleo_sheet::load::Tree {
    vleo_sheet::load_all(&root()).unwrap()
}

/// Today's design as the sheets hold it, read once.
fn today() -> &'static Graph {
    static G: OnceLock<&'static Graph> = OnceLock::new();
    G.get_or_init(|| opened::graph(&tree()).unwrap())
}

fn with_data() -> Case {
    static NAMES: OnceLock<Vec<String>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let scratch = std::env::temp_dir().join(format!("vleo-fingerprint-{}", std::process::id()));
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

#[test]
fn the_design_read_from_its_files_is_known_by_the_same_fingerprint_each_time() {
    // Its files in design/, read twice as the tool reads them, independently:
    // the same design, answering alike, with the reference data and without.
    let (one, two) = (
        opened::read(&root()).unwrap(),
        opened::read(&root()).unwrap(),
    );
    assert!(!std::ptr::eq(one, two), "read once, not twice");
    assert_eq!(one.design_fingerprint(), two.design_fingerprint());
    for case in [with_data(), Case::default()] {
        assert_eq!(
            one.answers_fingerprint(&case).unwrap(),
            two.answers_fingerprint(&case).unwrap()
        );
    }
    // The sheets they were converted from, likewise.
    assert_eq!(
        today().design_fingerprint(),
        opened::graph(&tree()).unwrap().design_fingerprint()
    );
    assert_ne!(
        today().answers_fingerprint(&with_data()).unwrap(),
        today().answers_fingerprint(&Case::default()).unwrap(),
        "the reference data changes what the design answers"
    );
}

/// One change to the design, made in memory.
type Edit = fn(&mut vleo_sheet::load::Tree);

#[test]
fn anything_the_design_runs_moves_its_fingerprint() {
    let base = today().design_fingerprint();
    let edits: [(&str, Edit); 5] = [
        ("a relation's implementation", |t| {
            t.sheets.get_mut("sw_f107_design_long").unwrap().impl_hash ^= 1;
        }),
        ("a sheet", |t| {
            t.sheets.get_mut("sw_f107_design_long").unwrap().sheet_hash ^= 1;
        }),
        ("a declared limit", |t| {
            t.sheets.get_mut("sw_mean_band_spread").unwrap().upper = 44.0;
        }),
        ("a value's maturity", |t| {
            t.sheets
                .get_mut("sw_mean_band_spread")
                .unwrap()
                .port
                .maturity = "measured".into();
        }),
        ("a loop's stopping rule", |t| {
            for c in t.cases.values_mut() {
                c.cycles[0].tolerance = 1e-7;
            }
        }),
    ];
    for (what, edit) in edits {
        let mut t = tree();
        edit(&mut t);
        assert_ne!(
            opened::graph(&t).unwrap().design_fingerprint(),
            base,
            "{what} changed and the fingerprint did not"
        );
    }
}

#[test]
fn an_answer_that_moves_moves_the_answers_fingerprint() {
    // The same sheets, one method run with its multiplier read from another
    // table: what the design is, as its hashes say, has not moved; what it
    // answers has.
    let mut t = tree();
    let m = &mut t.sheets.get_mut("sw_f107_design_long").unwrap().method.text;
    *m = m.replace("const z = 1.28 [1]", "const z = 1.29 [1]");
    let g = opened::graph(&t).unwrap();
    assert_eq!(g.design_fingerprint(), today().design_fingerprint());
    let case = with_data();
    assert_ne!(
        g.answers_fingerprint(&case).unwrap(),
        today().answers_fingerprint(&case).unwrap()
    );
}
