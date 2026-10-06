//! The graph, read from the design's files when the engine opens.
//!
//! docs/PLAN_1_0.md, phase D: "The graph is built from the design's files when
//! the application opens, not compiled into it." This is that graph, in the
//! same shape as the compiled one ([`crate::COMPILED`]), so it runs through
//! the one engine — the resolver, the adapter that moves values to and from a
//! node, the fixtures, the credibility — with nothing of its own to differ.
//!
//! Each node, variable and case is built from the tree exactly as the
//! generator writes them into the compiled tables (`vleo_sheet::emit`,
//! `tables_rs`): the same order, the same indices, the same units turned to
//! SI, the same fixtures. A node's relation is found by its id:
//!
//! - **built-in** — a relation still in compiled code is the compiled one,
//!   taken only when its implementation is the one the sheet was built with
//!   (its `impl_hash`); its guards are the compiled guards, so its parity is
//!   exact;
//! - **seeded** — a row nobody has specified refuses, as the compiled one does;
//! - anything else — a row this build has no code for, or code from another
//!   sheet — refuses by name, rather than running something that is not it.
//!
//! The 32 rows whose relation is a method still run as translated code here;
//! running them in the interpreter is the next step (D2), held to the same
//! answers.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::BTreeMap;

use vleo_core::credibility::Tier;
use vleo_core::evidence::{Fixture, Provenance};
use vleo_core::fault::Fault;
use vleo_core::graph::{Kind, Limit, NodeDef, Retirement, State, VarDef, View};
use vleo_sheet::load::Tree;
use vleo_sheet::model as sheet;
use vleo_units::Unit;

use crate::{
    CaseDef, CycleDef, Error, ErrorKind, Graph, NodeFn, COMPILED, MAX_INPUTS, MAX_OUTPUTS,
};

/// Text that lives as long as the graph does: the run's whole life.
fn text(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn slice<T>(v: Vec<T>) -> &'static [T] {
    Box::leak(v.into_boxed_slice())
}

fn unit_of(name: &str) -> Unit {
    Unit::from_name(name).unwrap_or(Unit::One)
}

fn to_si(v: f64, unit: &str) -> f64 {
    v * unit_of(unit).si_factor()
}

fn kind_of(k: &str) -> Kind {
    match k {
        "declared" => Kind::Declared,
        "required" => Kind::Required,
        "achieved" => Kind::Achieved,
        "kpi" => Kind::Kpi,
        _ => Kind::Computed,
    }
}

fn state_of(s: &str) -> State {
    match s {
        "" | "empty" => State::Empty,
        "specified" => State::Specified,
        "implemented" => State::Implemented,
        "verified" => State::Verified,
        "deprecated" => State::Deprecated,
        _ => State::Published,
    }
}

fn tier_of(t: &str) -> Tier {
    match t {
        "A+" => Tier::APlus,
        "A" => Tier::A,
        "B" => Tier::B,
        "C" => Tier::C,
        _ => Tier::Unset,
    }
}

fn provenance_of(p: &str) -> Provenance {
    match p {
        "independent-derivation" => Provenance::IndependentDerivation,
        "published-source" => Provenance::PublishedSource,
        "independent-tool" => Provenance::IndependentTool,
        "physical-bound" => Provenance::PhysicalBound,
        "self-snapshot" => Provenance::SelfSnapshot,
        _ => Provenance::AgentGenerated,
    }
}

fn view_of(v: &sheet::View) -> View {
    match v {
        sheet::View::Number => View::Number,
        sheet::View::Line { over, points } => View::Line {
            over: text(over),
            y: "",
            points: *points,
        },
        sheet::View::Heatmap {
            over_x,
            over_y,
            points,
        } => View::Heatmap {
            over_x: text(over_x),
            over_y: text(over_y),
            z: "",
            points: *points,
        },
        sheet::View::Bar { y } => View::Bar { y: text(y) },
    }
}

/// A row the tree knows about that nobody has specified yet: it refuses by
/// name, as the compiled table's does.
fn unspecified(_: &[f64], _: &mut [f64]) -> Result<(), Fault> {
    Err(Fault::NotRun {
        node: "seeded, not yet specified",
    })
}

/// A row this build of the engine has no relation for, or whose relation was
/// built from another sheet: refused by name, never run as something else.
fn not_in_this_build(_: &[f64], _: &mut [f64]) -> Result<(), Fault> {
    Err(Fault::NotRun {
        node: "its relation is not in this build of the engine",
    })
}

/// The relation each row runs, by its id: the compiled one when it is this
/// sheet's.
fn relation(sh: &sheet::Sheet) -> NodeFn {
    if sh.is_seeded() {
        return unspecified;
    }
    match COMPILED.find(&sh.id) {
        Some(k) if COMPILED.nodes[k as usize].impl_hash == sh.impl_hash => {
            COMPILED.dispatch[k as usize]
        }
        _ => not_in_this_build,
    }
}

/// The graph of `tree`, built as the generator builds the compiled one. It
/// lives as long as the engine that opened it.
pub fn graph(tree: &Tree) -> Result<&'static Graph, Error> {
    let sheets = tree.ordered();
    let n = sheets.len();
    let mut idx: BTreeMap<String, usize> = sheets
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.clone(), i))
        .collect();
    // Published members after every primary, as the generator orders them:
    // node i's own answer stays at index i.
    let mut extras: Vec<(usize, &sheet::Publish)> = Vec::new();
    for (i, sh) in sheets.iter().enumerate() {
        for pb in &sh.publishes {
            idx.insert(format!("{}.{}", sh.id, pb.id), n + extras.len());
            extras.push((i, pb));
        }
    }
    let at = |name: &str| idx.get(name).copied();

    // The engine's scratch is sized by this build's tree; a design wider than
    // it is refused rather than truncated.
    let max_in = sheets.iter().map(|sh| sh.inputs.len()).max().unwrap_or(0);
    let max_out = sheets
        .iter()
        .map(|sh| 1 + sh.publishes.len())
        .max()
        .unwrap_or(1);
    if max_in > MAX_INPUTS || max_out > MAX_OUTPUTS {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "the design has a row with {max_in} inputs and {max_out} outputs; this build \
                 of the engine runs at most {MAX_INPUTS} and {MAX_OUTPUTS}"
            ),
        ));
    }

    let mut nodes = Vec::with_capacity(n);
    let mut dispatch: Vec<NodeFn> = Vec::with_capacity(n);
    for sh in &sheets {
        let inputs: Vec<u16> = sh
            .inputs
            .iter()
            .map(|i| at(&i.var).unwrap_or(0) as u16)
            .collect();
        let mut outputs = vec![idx[sh.id.as_str()] as u16];
        for pb in &sh.publishes {
            outputs.push(idx[format!("{}.{}", sh.id, pb.id).as_str()] as u16);
        }
        let fixtures: Vec<Fixture> = sh
            .fixtures
            .iter()
            .map(|f| {
                let ins: Vec<f64> = sh
                    .inputs
                    .iter()
                    .map(|i| {
                        f.inputs
                            .iter()
                            .find(|(k, _)| *k == i.binding)
                            .map_or(0.0, |(_, v)| *v)
                    })
                    .collect();
                let slot = if f.variable.is_empty() {
                    0
                } else {
                    sh.publishes
                        .iter()
                        .position(|pb| pb.id == f.variable)
                        .map_or(0, |k| k + 1)
                };
                Fixture {
                    label: text(&f.label),
                    expected: f.expect,
                    tolerance: f.tolerance,
                    provenance: provenance_of(&f.provenance),
                    source: text(&f.source),
                    inputs: slice(ins),
                    slot,
                }
            })
            .collect();
        nodes.push(NodeDef {
            id: text(&sh.id),
            label: text(&sh.label),
            subsystem: text(&sh.subsystem),
            folder: text(&format!("crates/{}/nodes/{}", sh.crate_name, sh.folder)),
            parent: text(&sh.parent),
            layer: sh.layer,
            order: sh.order,
            crosses_to: text(&sh.crosses_to),
            kind: kind_of(&sh.kind),
            state: state_of(&sh.state),
            retirement: Retirement::Live,
            owner: text(&sh.owner),
            tier: tier_of(&sh.tier),
            question: text(&sh.question),
            expression: text(&sh.expression),
            source: text(&sh.source),
            relation_by: text(&sh.relation_by),
            derived: !sh.theory.is_empty(),
            assumptions: slice(
                sh.assumptions
                    .iter()
                    .map(|a| (text(&a.text), text(&a.fails_when)))
                    .collect(),
            ),
            steps: slice(sh.steps.iter().map(|s| text(&s.text)).collect()),
            inputs: slice(inputs),
            outputs: slice(outputs),
            contributes: slice(sh.kpis.iter().map(|k| text(k)).collect()),
            bundles: slice(sh.bundles.iter().map(|b| text(b)).collect()),
            fixtures: slice(fixtures),
            sheet_hash: sh.sheet_hash,
            impl_hash: sh.impl_hash,
            view: view_of(&sh.view),
        });
        dispatch.push(relation(sh));
    }

    let limit = |lower: f64, upper: f64, unit: &str, rl: &str, ru: &str| Limit {
        lower: to_si(lower, unit),
        upper: to_si(upper, unit),
        reason_lower: text(rl),
        reason_upper: text(ru),
    };
    let mut vars = Vec::with_capacity(n + extras.len());
    for (i, sh) in sheets.iter().enumerate() {
        vars.push(VarDef {
            id: text(&sh.id),
            symbol: text(&sh.symbol),
            label: text(&sh.label),
            unit: unit_of(&sh.unit),
            producer: i as u16,
            limit: limit(
                sh.lower,
                sh.upper,
                &sh.unit,
                &sh.reason_lower,
                &sh.reason_upper,
            ),
        });
    }
    for (producer, pb) in &extras {
        vars.push(VarDef {
            id: text(&format!("{}.{}", sheets[*producer].id, pb.id)),
            symbol: text(&pb.symbol),
            label: text(&pb.label),
            unit: unit_of(&pb.unit),
            producer: *producer as u16,
            limit: limit(
                pb.lower,
                pb.upper,
                &pb.unit,
                &pb.reason_lower,
                &pb.reason_upper,
            ),
        });
    }

    let pairs = |v: &[(String, f64)]| -> &'static [(u16, f64)] {
        slice(
            v.iter()
                .filter_map(|(k, x)| at(k).map(|i| (i as u16, *x)))
                .collect(),
        )
    };
    let cases: Vec<CaseDef> = tree
        .cases
        .values()
        .map(|c| CaseDef {
            id: text(&c.id),
            label: text(&c.label),
            note: text(&c.note),
            supply: pairs(&c.supply),
            cycles: slice(
                c.cycles
                    .iter()
                    .map(|cy| CycleDef {
                        nodes: slice(
                            cy.nodes
                                .iter()
                                .filter_map(|n| at(n).map(|i| i as u16))
                                .collect(),
                        ),
                        converge_on: at(&cy.converge_on).unwrap_or(0) as u16,
                        tolerance: cy.tolerance,
                        max_iter: cy.max_iter,
                        seeds: pairs(&cy.seeds),
                    })
                    .collect(),
            ),
            conditions: slice(
                c.conditions
                    .iter()
                    .filter_map(|k| at(k).map(|i| i as u16))
                    .collect(),
            ),
        })
        .collect();

    Ok(Box::leak(Box::new(Graph {
        nodes: slice(nodes),
        vars: slice(vars),
        dispatch: slice(dispatch),
        cases: slice(cases),
    })))
}

/// The graph of the design's files under `root`: the tree read by the one
/// loader, then built as above.
pub fn read(root: &std::path::Path) -> Result<&'static Graph, Error> {
    let tree = vleo_sheet::load_all(root).map_err(|e| {
        Error::new(ErrorKind::Malformed, e.to_string()).within("the design does not load")
    })?;
    graph(&tree)
}
