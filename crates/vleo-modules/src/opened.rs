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
//! SI, the same fixtures. Every relation is the design's own (docs/PLAN_1_0.md,
//! phase E):
//!
//! - **a method** — a row whose relation is a method runs in the method
//!   language's interpreter, the same one that checked it against its cases.
//!   Around it is what the compiled row did, in the same order and the same
//!   words: the length check, an input that is not a number refused at the
//!   door, the method's own refusal and its undefined values as the same
//!   faults, and the guards on every value it publishes. Its arithmetic is the
//!   same portable maths, so it is held to today's answers exactly.
//!
//!   The interpreter is the only way a method runs: no translation of it
//!   compiled into this build runs in its place. docs/PLAN_1_0.md, phase D,
//!   kept translated code as a fast path only if a sweep proved too slow;
//!   measured, the solar design and closure figures are the same to within
//!   their timing noise, and the one heavy case — `l3_solar_interface` over
//!   100 001 points — takes 1.6 times as long, 0.27 ms a point against 0.17.
//!   No other compiled code runs for any row.
//! - **a stated value** — published as written, converted from its unit to
//!   its type's as the generated row did, and guarded by its declared domain.
//! - **a table, or its children** — read by the engine itself.
//! - **seeded** — a row nobody has specified refuses, as the compiled one does;
//!   anything else refuses by name, rather than running something that is not
//!   it.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::collections::BTreeMap;

use vleo_core::credibility::Tier;
use vleo_core::evidence::{Fixture, Provenance};
use vleo_core::fault::Fault;
use vleo_core::graph::{
    Behaviour, Children, Kind, Level, Limit, Lookup, Maturity, NodeDef, Port, PortState, Read,
    Retirement, State, VarDef, View,
};
use vleo_core::math::table::Table1;
use vleo_sheet::load::Tree;
use vleo_sheet::model as sheet;
use vleo_units::Unit;

use crate::{
    CaseDef, CycleDef, Error, ErrorKind, Graph, GroupDef, NodeFn, Relation, MAX_INPUTS, MAX_OUTPUTS,
};
use vleo_core::fault::Edge;
use vleo_sheet::method::{self, Compiled, Outcome};

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

/// What an output says of its value, as its sheet says (`Sheet::port_state`):
/// a word it does not know is no maturity and no parameter, which the gate
/// has already refused by name.
fn port_of(sh: &sheet::Sheet, p: &sheet::PortSheet) -> Port {
    Port {
        state: match sh.port_state(p) {
            "decided" => PortState::Decided,
            "allocated" => PortState::Allocated,
            "open" => PortState::Open,
            _ => PortState::Achieved,
        },
        maturity: match p.maturity.as_str() {
            "estimated" => Maturity::Estimated,
            "calculated" => Maturity::Calculated,
            "measured" => Maturity::Measured,
            _ => Maturity::Unstated,
        },
        parameter: match p.parameter.as_str() {
            "programme" => Some(Level::Programme),
            "system" => Some(Level::System),
            "subsystem" => Some(Level::Subsystem),
            _ => None,
        },
        open_owner: text(&p.open_owner),
        open_due: text(&p.open_due),
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
    /// The method, read once for its inputs: a sweep runs it thousands of
    /// times, and nothing is looked up by name on any of them.
    compiled: Compiled,
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
        let field = self.outputs[0].symbol;
        let (answer, published) = match self.compiled.run_all(&inputs[..self.inputs.len()]) {
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
    let inputs: Vec<String> = sh.inputs.iter().map(|i| i.binding.clone()).collect();
    let m: &'static Interpreted = Box::leak(Box::new(Interpreted {
        node: text(&sh.id),
        compiled: Compiled::new(&program, &inputs),
        inputs,
        outputs,
    }));
    Some(Box::leak(Box::new(move |i: &[f64], o: &mut [f64]| {
        m.call(i, o)
    })))
}

/// The relation a row runs when it has no method: a row nobody has specified
/// refuses as unspecified, and anything else refuses by name. Every relation
/// of the design is a method, a stated value, a table or its children.
fn relation(sh: &sheet::Sheet) -> NodeFn {
    if sh.is_seeded() {
        return unspecified;
    }
    not_in_this_build
}

/// A stated value, as the row publishes it: converted from the unit it was
/// written in to its type's, exactly as the generated row did
/// (`from_unit`), and guarded by its declared domain.
fn stated(sh: &sheet::Sheet) -> Option<&'static Relation> {
    if !sh.is_declared() || !sh.inputs.is_empty() {
        return None;
    }
    let node = text(&sh.id);
    let symbol = text(&sh.symbol);
    let value = vleo_units::quantity_unit(&sh.ty)
        .and_then(|ty| unit_of(&sh.unit).convert(sh.value.unwrap_or(0.0), ty));
    let guard: &'static Guarded = Box::leak(Box::new(Guarded::new(
        &sh.symbol,
        &sh.ty,
        &sh.unit,
        sh.lower,
        sh.upper,
        &sh.reason_lower,
        &sh.reason_upper,
    )));
    Some(Box::leak(Box::new(move |_: &[f64], o: &mut [f64]| {
        let v = value.ok_or(Fault::Degenerate {
            node,
            field: symbol,
            reason: "the declared unit does not match the declared type",
        })?;
        guard.check(node, v)?;
        *o.first_mut().ok_or(Fault::Blocked {
            node,
            missing: "an input the contract declares",
        })? = v;
        Ok(())
    })))
}

/// The graph of `tree`, built as the generator builds the compiled one, every
/// method run by the interpreter. It lives as long as the engine that opened
/// it.
pub fn graph(tree: &Tree) -> Result<&'static Graph, Error> {
    build(tree)
}

fn build(tree: &Tree) -> Result<&'static Graph, Error> {
    // Every name the design wires by must resolve before anything is built
    // from it. Below, a name that does not would run as the first variable,
    // or as zero, or be left out; the compiled build refused it at compile
    // time, and this graph refuses it here, by name (`vleo_sheet::wiring`).
    let wrong = vleo_sheet::wiring::errors(tree);
    if !wrong.is_empty() {
        return Err(Error::new(
            ErrorKind::Invalid,
            format!(
                "{} name(s) in the design resolve to nothing, so it is not run:\n  {}",
                wrong.len(),
                wrong.join("\n  ")
            ),
        ));
    }
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
        // A method runs in the interpreter, and a stated value is published
        // by the graph itself.
        let own = interpreted(sh).or_else(|| stated(sh));
        dispatch.push(if own.is_some() {
            not_in_this_build
        } else {
            relation(sh)
        });
        run.push(own);
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
            port: port_of(sh, &sh.port),
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
            port: port_of(sheets[*producer], &pb.port),
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

    // The headings and their relations, in the order the generator writes
    // them.
    let groups: Vec<GroupDef> = tree
        .groups
        .values()
        .map(|g| GroupDef {
            id: text(&g.id),
            label: text(&g.label),
            parent: text(&g.parent),
            owner: text(&g.owner),
            layer: g.layer,
            order: g.order,
            is_box: g.is_box,
            tone: text(&g.tone),
            cases: slice(g.cases.iter().map(|c| text(c)).collect()),
        })
        .collect();
    let relations: Vec<(&'static str, &'static str, &'static str)> = tree
        .relations
        .iter()
        .map(|r| (text(&r.from), text(&r.to), text(&r.why)))
        .collect();

    Ok(Box::leak(Box::new(Graph {
        nodes: slice(nodes),
        vars: slice(vars),
        dispatch: slice(dispatch),
        run: slice(run),
        cases: slice(cases),
        groups: slice(groups),
        relations: slice(relations),
        cases_run: crate::CasesRun::new(),
    })))
}

/// The graph of the design's files under `root`: the tree read by the one
/// loader, then built as above.
pub fn read(root: &std::path::Path) -> Result<&'static Graph, Error> {
    graph(&load(root)?)
}

/// The design under `root`: its files in `design/`, read as the folders they
/// were converted from. A checkout with no `design/` is read from its sheets.
fn load(root: &std::path::Path) -> Result<Tree, Error> {
    let refused =
        |e: String| Error::new(ErrorKind::Malformed, e).within("the design does not load");
    let dir = root.join("design");
    if dir.is_dir() {
        return read_folder(root, &dir).map_err(refused);
    }
    vleo_sheet::load_all(root).map_err(|e| refused(e.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_folder(root: &std::path::Path, dir: &std::path::Path) -> Result<Tree, String> {
    let (files, _) = vleo_files::convert::read_folder(dir).map_err(|e| e.to_string())?;
    let served = vleo_files::convert::Served::new(
        root,
        &files,
        std::sync::Arc::new(vleo_sheet::files::Disk),
    )
    .map_err(|e| e.to_string())?;
    vleo_sheet::load::load_all_from(&served, root).map_err(|e| e.to_string())
}

/// A page has no folders: it opens the design it carries ([`graph`]).
#[cfg(target_arch = "wasm32")]
fn read_folder(_: &std::path::Path, dir: &std::path::Path) -> Result<Tree, String> {
    Err(format!("{} cannot be read from a page", dir.display()))
}
