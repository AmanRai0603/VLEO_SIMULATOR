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
//!   taken only when it was built from this sheet (its `impl_hash` and its
//!   `sheet_hash` both the same); its guards are the compiled guards, so its
//!   parity is exact;
//! - **seeded** — a row nobody has specified refuses, as the compiled one does;
//! - anything else — a row this build has no code for, or code from another
//!   sheet — refuses by name, rather than running something that is not it.
//!
//! - **a method** — a row whose relation is a method the node engineer wrote
//!   runs in the method language's interpreter, the same one that checked it
//!   against their cases, with no compiled code at all. Around it is what the
//!   compiled row does, in the same order and the same words: the length
//!   check, an input that is not a number refused at the door, the method's
//!   own refusal and its undefined values as the same faults, and the guards
//!   on every value it publishes. Its arithmetic is the same portable maths,
//!   so it is held to today's answers exactly.
//!
//!   A method this build was made from — the very sheet — runs as its
//!   translation instead, the fast path a sweep needs, held equal to the
//!   interpreter by the parity gate ([`interpreting`] builds the graph that
//!   gate runs). A method the build has not seen runs in the interpreter.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::BTreeMap;

use vleo_core::credibility::Tier;
use vleo_core::evidence::{Fixture, Provenance};
use vleo_core::fault::Fault;
use vleo_core::graph::{
    Behaviour, Children, Kind, Limit, Lookup, NodeDef, Read, Retirement, State, VarDef, View,
};
use vleo_core::math::table::Table1;
use vleo_sheet::load::Tree;
use vleo_sheet::model as sheet;
use vleo_units::Unit;

use crate::{
    CaseDef, CycleDef, Error, ErrorKind, Graph, NodeFn, Relation, COMPILED, MAX_INPUTS, MAX_OUTPUTS,
};
use vleo_core::fault::Edge;
use vleo_sheet::method::{self, Outcome, Program};

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

/// What a row's answer is, as its sheet says (`Sheet::behaviour`): a table or
/// the children that answer it held as data the engine reads, every other
/// behaviour by name.
fn behaviour_of(sh: &sheet::Sheet) -> Behaviour {
    let at = |binding: &str| {
        sh.inputs
            .iter()
            .position(|i| i.binding == binding)
            .unwrap_or(0) as u8
    };
    match sh.behaviour() {
        "open" => Behaviour::Open,
        "stated" => Behaviour::Stated,
        "method" => Behaviour::Method,
        "lookup" => {
            let l = sh.lookup.as_ref().expect("a lookup row has a table");
            Behaviour::Lookup(Box::leak(Box::new(Lookup {
                by: at(&l.by),
                table: Table1 {
                    x: slice(l.x.clone()),
                    y: slice(l.y.clone()),
                },
                read: if l.read == "log" {
                    Read::Log
                } else {
                    Read::Linear
                },
            })))
        }
        "children" => {
            let c = sh.children.as_ref().expect("a children row has children");
            Behaviour::Children(Box::leak(Box::new(Children {
                group: text(&c.group),
                from: slice(c.from.iter().map(|b| at(b)).collect()),
            })))
        }
        _ => Behaviour::BuiltIn,
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

/// A sentence a refusal says, kept once for the life of the engine: a fault
/// carries its reason by reference, and a sweep that refuses ten thousand
/// times must not keep ten thousand copies of the same words.
fn said(s: String) -> &'static str {
    use std::sync::Mutex;
    static SAID: Mutex<std::collections::BTreeSet<&'static str>> =
        Mutex::new(std::collections::BTreeSet::new());
    let mut kept = SAID.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(k) = kept.get(s.as_str()) {
        return k;
    }
    let k: &'static str = Box::leak(s.into_boxed_str());
    kept.insert(k);
    k
}

/// One value a method publishes, and the guards the compiled row puts on it.
struct Guarded {
    symbol: &'static str,
    unit: Unit,
    lower: f64,
    upper: f64,
    reason_lower: &'static str,
    reason_upper: &'static str,
}

impl Guarded {
    #[allow(clippy::too_many_arguments)]
    fn new(
        symbol: &str,
        ty: &str,
        unit: &str,
        lower: f64,
        upper: f64,
        rl: &str,
        ru: &str,
    ) -> Guarded {
        Guarded {
            symbol: text(symbol),
            unit: vleo_units::quantity_unit(ty).unwrap_or(Unit::One),
            lower: to_si(lower, unit),
            upper: to_si(upper, unit),
            reason_lower: text(rl),
            reason_upper: text(ru),
        }
    }

    /// The compiled row's guards on one value, in its order: not a number,
    /// then below its lower bound, then above its upper.
    fn check(&self, node: &'static str, v: f64) -> Result<(), Fault> {
        if !v.is_finite() {
            return Err(Fault::Degenerate {
                node,
                field: self.symbol,
                reason: "the computation produced a value that is not a number",
            });
        }
        if self.lower.is_finite() && v < self.lower {
            return Err(Fault::OutOfDomain {
                node,
                field: self.symbol,
                value: v,
                bound: self.lower,
                edge: Edge::Lower,
                unit: self.unit,
                reason: self.reason_lower,
            });
        }
        if self.upper.is_finite() && v > self.upper {
            return Err(Fault::OutOfDomain {
                node,
                field: self.symbol,
                value: v,
                bound: self.upper,
                edge: Edge::Upper,
                unit: self.unit,
                reason: self.reason_upper,
            });
        }
        Ok(())
    }
}

/// A row's method, run by the interpreter, with the compiled row's contract
/// around it.
struct Interpreted {
    node: &'static str,
    program: Program,
    /// The method's names for its inputs, in the contract's order.
    inputs: Vec<String>,
    /// The answer, then each published member, in declared order.
    outputs: Vec<Guarded>,
}

impl Interpreted {
    fn call(&self, inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
        let node = self.node;
        if inputs.len() < self.inputs.len() || outputs.len() < self.outputs.len() {
            return Err(Fault::Blocked {
                node,
                missing: "an input the contract declares",
            });
        }
        // The door, as the compiled method has it: an input that is not a
        // number is refused before the first line.
        if inputs[..self.inputs.len()].iter().any(|v| !v.is_finite()) {
            return Err(Fault::Refused {
                node,
                reason: "an input is not a finite number",
            });
        }
        let named: Vec<(String, f64)> = self
            .inputs
            .iter()
            .cloned()
            .zip(inputs.iter().copied())
            .collect();
        let field = self.outputs[0].symbol;
        let (answer, published) = match method::run_all(&self.program, &named) {
            Ok((Outcome::Answer(v), p)) => (v, p),
            Ok((Outcome::Refused { reason, .. }, _)) => {
                return Err(Fault::Refused {
                    node,
                    reason: said(reason),
                })
            }
            Err(d) => {
                return Err(Fault::Degenerate {
                    node,
                    field,
                    reason: said(d.msg),
                })
            }
        };
        let mut values = Vec::with_capacity(self.outputs.len());
        values.push(answer);
        for g in &self.outputs[1..] {
            match published.iter().find(|(s, _)| s == g.symbol) {
                Some((_, v)) => values.push(*v),
                None => {
                    return Err(Fault::Degenerate {
                        node,
                        field: g.symbol,
                        reason: "the method did not publish this value",
                    })
                }
            }
        }
        for (g, v) in self.outputs.iter().zip(&values) {
            g.check(node, *v)?;
        }
        outputs[..values.len()].copy_from_slice(&values);
        Ok(())
    }
}

/// The row's method, run by the interpreter, when it has one the tool can run.
fn interpreted(sh: &sheet::Sheet) -> Option<&'static Relation> {
    if sh.is_seeded() {
        return None;
    }
    let program = method::node_program(sh)?;
    let mut outputs = vec![Guarded::new(
        &sh.symbol,
        &sh.ty,
        &sh.unit,
        sh.lower,
        sh.upper,
        &sh.reason_lower,
        &sh.reason_upper,
    )];
    for pb in &sh.publishes {
        outputs.push(Guarded::new(
            &pb.symbol,
            &pb.ty,
            &pb.unit,
            pb.lower,
            pb.upper,
            &pb.reason_lower,
            &pb.reason_upper,
        ));
    }
    let m: &'static Interpreted = Box::leak(Box::new(Interpreted {
        node: text(&sh.id),
        program,
        inputs: sh.inputs.iter().map(|i| i.binding.clone()).collect(),
        outputs,
    }));
    Some(Box::leak(Box::new(move |i: &[f64], o: &mut [f64]| {
        m.call(i, o)
    })))
}

/// The relation each row runs, by its id: the compiled one when it is this
/// sheet's.
///
/// The compiled function carries the guards generated from the sheet it was
/// built from — its units, its limits and their reasons — so it is this
/// sheet's only when both are the same: its implementation (`impl_hash`) and
/// the sheet itself (`sheet_hash`). A sheet whose range has moved, run on the
/// compiled code, would be guarded by the range it no longer declares.
fn relation(sh: &sheet::Sheet) -> NodeFn {
    if sh.is_seeded() {
        return unspecified;
    }
    this_build(sh).unwrap_or(not_in_this_build)
}

/// The compiled code for this sheet, when this build was made from it: the
/// same implementation and the same sheet — and so, for a method, the same
/// method, translated.
fn this_build(sh: &sheet::Sheet) -> Option<NodeFn> {
    let k = COMPILED.find(&sh.id)? as usize;
    (COMPILED.nodes[k].impl_hash == sh.impl_hash && COMPILED.nodes[k].sheet_hash == sh.sheet_hash)
        .then(|| COMPILED.dispatch[k])
}

/// How the graph runs a row whose relation is a method.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Methods {
    /// Its translation, when this build was made from the very sheet; the
    /// interpreter otherwise. docs/PLAN_1_0.md, phase D: "If a sweep is too
    /// slow, translated code stays as the fast path, held equal to the
    /// interpreter." A sweep runs a method thousands of times, and one call
    /// costs about 1.7 µs translated and 40 µs interpreted: the design
    /// panel's F10.7 view took 3.3 s interpreted and 0.16 s translated.
    Translated,
    /// Always in the interpreter — how the parity gate holds every
    /// translation equal to it.
    Interpreted,
}

/// The graph of `tree`, built as the generator builds the compiled one. It
/// lives as long as the engine that opened it.
///
/// A method this build was made from runs as its translation, held equal to
/// the interpreter by the parity gate; a method the build has not seen — a
/// group's release changed it — runs in the interpreter.
pub fn graph(tree: &Tree) -> Result<&'static Graph, Error> {
    build(tree, Methods::Translated)
}

/// The graph of `tree` with every method run by the interpreter, whatever
/// code the build has for it: the graph the parity gate holds every
/// translation to.
pub fn interpreting(tree: &Tree) -> Result<&'static Graph, Error> {
    build(tree, Methods::Interpreted)
}

fn build(tree: &Tree, methods: Methods) -> Result<&'static Graph, Error> {
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
    let mut run: Vec<Option<&'static Relation>> = Vec::with_capacity(n);
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
            behaviour: behaviour_of(sh),
        });
        // A method runs in the interpreter, and needs no compiled code — or,
        // when this build was made from this very sheet, as its translation.
        let translated = methods == Methods::Translated && this_build(sh).is_some();
        let interp = if translated { None } else { interpreted(sh) };
        dispatch.push(if interp.is_some() {
            not_in_this_build
        } else {
            relation(sh)
        });
        run.push(interp);
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
        run: slice(run),
        cases: slice(cases),
    })))
}

/// The graph of the design's files under `root`: the tree read by the one
/// loader, then built as above.
pub fn read(root: &std::path::Path) -> Result<&'static Graph, Error> {
    graph(&load(root)?)
}

/// The same, with every method run by the interpreter ([`interpreting`]).
pub fn read_interpreting(root: &std::path::Path) -> Result<&'static Graph, Error> {
    interpreting(&load(root)?)
}

fn load(root: &std::path::Path) -> Result<Tree, Error> {
    vleo_sheet::load_all(root).map_err(|e| {
        Error::new(ErrorKind::Malformed, e.to_string()).within("the design does not load")
    })
}
