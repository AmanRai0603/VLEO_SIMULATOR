//! Every case the design holds passes, run by the engine that reads the
//! design from its files.
//!
//! Each node crate's `evidence.rs` asks this of one node, of its compiled
//! code: each fixture within its tolerance, each of its author's cases as the
//! author's code answered or refused it, the three properties from its
//! declared domain, and the prior implementation's grid. Those crates go
//! (docs/PLAN_1_0.md, phase E), and the questions must not go with them. So
//! they are asked here of every node at once, of the graph read from
//! `design/` and run in the interpreter alone, in the same words and to the
//! same tolerances. A disagreement is a physics disagreement, for the node's
//! engineer; a tolerance is never the thing to change.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vleo_modules::core_engine::fault::Fault;
use vleo_modules::{opened, Graph, MAX_OUTPUTS};
use vleo_sheet::files::{Disk, Files};
use vleo_sheet::load::Tree;
use vleo_sheet::model::Sheet;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn relative_error(got: f64, expected: f64) -> f64 {
    if expected == 0.0 {
        got.abs()
    } else {
        ((got - expected) / expected).abs()
    }
}

/// What every check found, and how many of each kind it asked.
#[derive(Default)]
struct Found {
    wrong: Vec<String>,
    fixtures: usize,
    cases: usize,
    properties: usize,
    grids: usize,
}

type Answer = Result<[f64; MAX_OUTPUTS], Fault>;

/// One node's inputs, in the order it reads them, from SI values by binding.
fn by_binding(sh: &Sheet, given: &[(String, f64)]) -> Vec<f64> {
    sh.inputs
        .iter()
        .map(|i| {
            given
                .iter()
                .find(|(k, _)| *k == i.binding)
                .map_or(0.0, |(_, v)| *v)
        })
        .collect()
}

#[test]
fn every_case_of_the_design_passes() {
    let root = root();
    let (on_disk, _) =
        vleo_files::convert::read_folder(&root.join("design")).expect("design/ reads");
    let served = vleo_files::convert::Served::new(&root, &on_disk, Arc::new(Disk))
        .expect("design/ is served");
    let tree: Tree = vleo_sheet::load::load_all_from(&served, &root).expect("design/ loads");
    let g = opened::interpreting(&tree).expect("design/ makes a graph");

    let mut found = Found::default();
    for sh in tree.ordered() {
        let k = g.find(&sh.id).expect("every row is in the graph");
        fixtures(g, k, sh, &mut found);
        author_cases(g, k, sh, &mut found);
        properties(g, k, sh, &mut found);
        parity(g, k, sh, &served, &mut found);
    }

    // Each kind asked as often as the generated tests ask it today, so a
    // reader that lost the cases cannot pass by asking nothing.
    assert!(
        found.fixtures >= 300,
        "only {} fixtures asked",
        found.fixtures
    );
    assert!(
        found.cases >= 250,
        "only {} author's cases asked",
        found.cases
    );
    assert!(
        found.properties >= 60,
        "only {} nodes' properties asked",
        found.properties
    );
    assert!(found.grids >= 10, "only {} parity grids asked", found.grids);
    assert!(
        found.wrong.is_empty(),
        "{} disagreement(s) in the design — physics disagreements, for each node's engineer; \
         do not widen a tolerance:\n  {}",
        found.wrong.len(),
        found.wrong.join("\n  ")
    );
}

/// Each fixture, within its declared tolerance, and never refused.
fn fixtures(g: &Graph, k: u16, sh: &Sheet, found: &mut Found) {
    for (f, got) in g.nodes[k as usize]
        .fixtures
        .iter()
        .zip(g.fixture_verdicts(k))
    {
        found.fixtures += 1;
        if got.got.is_nan() && got.expected.is_finite() {
            found.wrong.push(format!(
                "{}: fixture «{}» was refused, and must not be",
                sh.id, f.label
            ));
        } else if !got.passed {
            found.wrong.push(format!(
                "{}: fixture «{}»: got {} want {}, relative error {} exceeds the declared \
                 tolerance {}",
                sh.id, f.label, got.got, f.expected, got.relative_error, f.tolerance
            ));
        }
    }
}

/// Each of the author's cases, as the author's own code answered or refused it.
fn author_cases(g: &Graph, k: u16, sh: &Sheet, found: &mut Found) {
    let has_method = vleo_sheet::method::node_program(sh).is_some();
    for c in &sh.cases {
        found.cases += 1;
        let got = g.probe(k, &by_binding(sh, &c.inputs));
        match (c.expect, got) {
            (None, Err(Fault::Refused { .. })) => {}
            (None, Err(_)) if !has_method => {}
            (None, got) => found.wrong.push(format!(
                "{}: case «{}»: the author's code refuses it and the node gave {got:?}",
                sh.id, c.label
            )),
            (Some(_), Err(f)) => found.wrong.push(format!(
                "{}: case «{}»: the author's code answers it and the node refused it: {f}",
                sh.id, c.label
            )),
            (Some(want), Ok(v)) => {
                let mut held = vec![(sh.symbol.as_str(), v[0], want)];
                for (member, want) in &c.also {
                    let at = sh
                        .publishes
                        .iter()
                        .position(|pb| pb.symbol == *member)
                        .map(|n| n + 1);
                    match at {
                        Some(n) => held.push((member.as_str(), v[n], *want)),
                        None => found.wrong.push(format!(
                            "{}: case «{}» gives {member}, which the node does not publish",
                            sh.id, c.label
                        )),
                    }
                }
                for (what, got, want) in held {
                    let err = relative_error(got, want);
                    if err > c.tolerance {
                        found.wrong.push(format!(
                            "{}: case «{}»: {what} is {got} and the author's code gave {want}; \
                             relative error {err} is more than their tolerance {}",
                            sh.id, c.label, c.tolerance
                        ));
                    }
                }
            }
        }
    }
}

/// The three properties from the declared domain, around the first fixture.
fn properties(g: &Graph, k: u16, sh: &Sheet, found: &mut Found) {
    if sh.is_declared() || sh.inputs.is_empty() || sh.fixtures.is_empty() {
        return;
    }
    found.properties += 1;
    let def = &g.nodes[k as usize];
    let base = by_binding(sh, &sh.fixtures[0].inputs);
    let at = |which: usize, scale: f64| -> Answer {
        let mut x = base.clone();
        x[which] *= scale;
        g.probe(k, &x)
    };

    // One per cent either side of the known-good point, it still answers.
    for (i, input) in sh.inputs.iter().enumerate() {
        for scale in [0.99, 1.01] {
            if let Err(f) = at(i, scale) {
                found.wrong.push(format!(
                    "{}: refuses near its own known-good point, {} x{scale}: {f}",
                    sh.id, input.binding
                ));
            }
        }
    }
    // Every answer is a number inside each member's own declared domain.
    for i in 0..sh.inputs.len() {
        for scale in [0.001, 0.1, 1.0, 10.0, 1000.0] {
            let Ok(v) = at(i, scale) else { continue };
            for (n, var) in def.outputs.iter().enumerate() {
                let var = &g.vars[*var as usize];
                let (x, lo, hi) = (v[n], var.limit.lower, var.limit.upper);
                if !x.is_finite() || x < lo || x > hi {
                    found.wrong.push(format!(
                        "{}: answered {x} for {} with input {} x{scale}, outside its declared \
                         domain {lo} … {hi}",
                        sh.id, var.id, sh.inputs[i].binding
                    ));
                }
            }
        }
    }
    // The same inputs give the same answer, to the bit.
    let same = match (g.probe(k, &base), g.probe(k, &base)) {
        (Ok(a), Ok(b)) => a
            .iter()
            .zip(&b)
            .take(def.outputs.len())
            .all(|(x, y)| x.to_bits() == y.to_bits()),
        (Err(_), Err(_)) => true,
        _ => false,
    };
    if !same {
        found
            .wrong
            .push(format!("{}: the same inputs gave two answers", sh.id));
    }
}

/// The prior implementation's grid, within the row's own parity tolerance.
fn parity(g: &Graph, k: u16, sh: &Sheet, files: &dyn Files, found: &mut Found) {
    let path: &Path = &sh.dir.join("parity.csv");
    if !files.is_file(path) {
        return;
    }
    found.grids += 1;
    let grid = files.read_to_string(path).expect("parity.csv reads");
    let tol = sh.parity_tolerance;
    let mut lines = grid
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'));
    let finding = |found: &mut Found, what: String| {
        found.wrong.push(format!(
            "{}: {what} — a finding about one of the two implementations, for the node's \
             engineer; do not widen parity_tolerance or edit the grid",
            sh.id
        ))
    };
    let header: Vec<&str> = lines
        .next()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .collect();

    // A row with nothing to sweep: the last column of the first data row is
    // the prior implementation's one number.
    if sh.is_declared() || sh.inputs.is_empty() {
        let expected: Option<f64> = lines
            .next()
            .and_then(|r| r.rsplit(',').next())
            .and_then(|c| c.trim().parse().ok());
        match (expected, g.probe(k, &[])) {
            (Some(want), Ok(v)) if relative_error(v[0], want) <= tol => {}
            (Some(want), Ok(v)) => finding(
                found,
                format!("this row says {} and the prior implementation {want}", v[0]),
            ),
            (None, _) => finding(found, "parity.csv has no number to compare".into()),
            (_, Err(f)) => finding(found, format!("the row refused: {f}")),
        }
        return;
    }

    let col: Option<Vec<usize>> = sh
        .inputs
        .iter()
        .map(|i| header.iter().position(|h| *h == i.binding))
        .collect();
    let Some(col) = col else {
        return finding(
            found,
            format!("parity.csv lacks a column this node reads: {header:?}"),
        );
    };
    let out = header.len() - 1;
    if !header[out].starts_with("matlab_") {
        return finding(
            found,
            format!(
                "the grid's last column is '{}', not matlab_<symbol>",
                header[out]
            ),
        );
    }
    let mut rows = 0;
    for (n, line) in lines.enumerate() {
        let row: Vec<f64> = line
            .split(',')
            .filter_map(|c| c.trim().parse().ok())
            .collect();
        if row.len() != header.len() {
            finding(
                found,
                format!("grid line {} is not {} numbers", n + 2, header.len()),
            );
            continue;
        }
        rows += 1;
        let x: Vec<f64> = col.iter().map(|c| row[*c]).collect();
        match g.probe(k, &x) {
            Err(f) => finding(
                found,
                format!(
                    "grid line {}: the prior implementation answered, this engine refused: {f}",
                    n + 2
                ),
            ),
            Ok(v) if relative_error(v[0], row[out]) > tol => finding(
                found,
                format!(
                    "grid line {}: this engine {}, the prior implementation {}",
                    n + 2,
                    v[0],
                    row[out]
                ),
            ),
            Ok(_) => {}
        }
    }
    if rows == 0 {
        finding(found, "parity.csv compares nothing".into());
    }
}
