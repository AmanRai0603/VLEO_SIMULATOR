//! The probe evaluates the relation, and nothing else.
//!
//! `/v1/probe` exists because the design question and the relation question
//! stopped having the same answer. Once `env_f107` read the solar subsystem the
//! flux was no longer free, so the engine correctly refuses a supplied one —
//! and the coefficients of Jacchia 1971 are still a fact about Jacchia 1971,
//! which something has to be able to ask.
//!
//! These tie it to the one external oracle in the system. A probe at a
//! fixture's inputs must reproduce that fixture's expected value, which came
//! from a published source rather than from this code.

use vleo_modules::{probe, Vleo, NODES};

/// Every fixture in the tree, asked of the probe.
///
/// Not a sample and not one node: if the probe ever stops being the relation —
/// reads a store, picks up a declared value, reorders its inputs — every
/// fixture in the tree is the thing that notices.
#[test]
fn a_probe_reproduces_every_fixture() {
    let mut checked = 0usize;
    for (i, def) in NODES.iter().enumerate() {
        if !def.state.runnable() {
            continue;
        }
        for fx in def.fixtures {
            if fx.inputs.len() != def.inputs.len() {
                continue; // a set row's fixture names a member; slot handling is run_fixture's
            }
            let got = probe(i as u16, fx.inputs).expect("the fixture's own inputs must not refuse");
            let want = fx.expected;
            let err = if want == 0.0 {
                got[fx.slot].abs()
            } else {
                ((got[fx.slot] - want) / want).abs()
            };
            assert!(
                err <= fx.tolerance,
                "{}: probe gave {} where the fixture expects {} ({})",
                def.id,
                got[fx.slot],
                want,
                fx.label
            );
            checked += 1;
        }
    }
    assert!(checked > 50, "only {checked} fixtures were reached");
}

/// The probe does not read the graph. Asked at the OLD declared drivers, the
/// thermosphere relation must still give the old answer, even though the design
/// now sizes itself somewhere else entirely.
#[test]
fn a_probe_ignores_what_the_design_is_wired_to() {
    let k = Vleo::find("env_exospheric_temperature").expect("the row exists");
    let old = probe(k, &[150.0, 150.0, 3.0]).expect("the old declared point");
    assert!(
        (old[0] - 949.6025661076957).abs() < 1e-9,
        "the relation at 150/150/3 moved: {}",
        old[0]
    );
    // And it is genuinely sensitive to each driver, or the assertion above
    // would pass against a probe that returned a constant.
    let hotter = probe(k, &[150.0, 150.0, 7.0]).expect("a storm");
    assert!(
        hotter[0] > old[0] + 100.0,
        "Kp did not move it: {}",
        hotter[0]
    );
    let brighter = probe(k, &[250.0, 250.0, 3.0]).expect("a bright sky");
    assert!(
        brighter[0] > old[0] + 300.0,
        "the flux did not move it: {}",
        brighter[0]
    );
}

/// Inputs go in the node's declared order. Three drivers of the same type is
/// exactly where a positional mix-up produces a plausible number and no error.
#[test]
fn the_input_order_is_the_declared_order() {
    let k = Vleo::find("env_exospheric_temperature").expect("the row exists");
    let def = &NODES[k as usize];
    let names: Vec<&str> = def
        .inputs
        .iter()
        .map(|&v| vleo_modules::VARS[v as usize].id)
        .collect();
    assert_eq!(names, vec!["env_f107", "env_f107a", "env_kp"]);

    // F10.7 enters as a departure weighted 1.3 and F10.7A as the mean weighted
    // 3.24, so swapping the pair is not symmetric — and by how much is
    // arithmetic rather than a guessed threshold. Swapping x and y moves the
    // answer by (3.24 - 2*1.3)*(y - x), which at 100 and 200 is 64 K exactly.
    let a = probe(k, &[200.0, 100.0, 3.0]).expect("one way round");
    let b = probe(k, &[100.0, 200.0, 3.0]).expect("the other");
    let want = (3.24 - 2.0 * 1.3) * 100.0;
    assert!(
        ((b[0] - a[0]) - want).abs() < 1e-9,
        "swapping the flux pair should move the answer by exactly {want} K, and moved it by {}",
        b[0] - a[0]
    );
}
