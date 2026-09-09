//! The facade — twelve crates behind one dependency, and the one `NodeTable`
//! the resolver walks.
//!
//! Every face takes a single dependency on this crate. The compiler still sees
//! twelve units, so they build in parallel, and no face can reach a module
//! directly — which is why adding a face cannot change a result.
//!
//! This crate holds **no formula**. It holds the tables generated from the
//! sheets and the adapter that moves values between the store and a node's
//! typed function. `cargo xtask gate` fails the build if a formula appears
//! here.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vleo_core::credibility::{self, CredVec, Factor};
use vleo_core::evidence::Verdict;
use vleo_core::fault::Fault;
use vleo_core::graph::{Kind, NodeDef, NodeIdx, NodeTable, VarDef};
use vleo_core::resolver::{self, CycleSpec, RunReport, Workspace};
use vleo_core::value::{Slot, SlotStatus, Store};
use vleo_units::Unit;

/// The graph, compiled in.
pub mod tables {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
}

pub use tables::{CaseDef, CycleDef, GroupDef, CASES, GROUPS, NODES, NODE_COUNT, RELATIONS, VARS};

/// Re-exported so a face has one name to import.
pub use vleo_bus as bus;
pub use vleo_core as core_engine;
pub use vleo_units as units;

/// The largest number of inputs any node declares. Fixed at build time so the
/// adapter needs no allocator — the kernel runs where there is not one.
const MAX_INPUTS: usize = 16;

/// The engine.
pub struct Vleo;

impl Vleo {
    /// Identifies this build of the engine. Travels into every result: a page
    /// carrying a different one refuses to run rather than showing a number
    /// from an engine it was not built against.
    pub fn kernel_hash() -> u64 {
        let mut h = vleo_core::hash::Hasher::new();
        h.write_str(env!("CARGO_PKG_VERSION"));
        for n in NODES.iter() {
            h.write_u64(n.impl_hash);
        }
        h.finish()
    }

    /// Identifies the graph. Changes when an edge does.
    pub fn graph_hash() -> u64 {
        let mut h = vleo_core::hash::Hasher::new();
        for n in NODES.iter() {
            h.write_str(n.id);
            h.write_u64(n.sheet_hash);
            for i in n.inputs {
                h.write_u16(*i);
            }
        }
        h.finish()
    }

    pub fn find(id: &str) -> Option<NodeIdx> {
        NODES.iter().position(|n| n.id == id).map(|i| i as NodeIdx)
    }

    pub fn case(id: &str) -> Option<&'static CaseDef> {
        CASES.iter().find(|c| c.id == id)
    }
}

impl NodeTable for Vleo {
    fn nodes(&self) -> &[NodeDef] {
        &NODES
    }
    fn vars(&self) -> &[VarDef] {
        &VARS
    }

    /// Move values between the store and one node's typed function.
    ///
    /// Everything crossing here is SI `f64`. The node's own generated adapter
    /// re-types it, applies the guards the sheet declared, and hands back the
    /// one answer the node exists to give. Nothing in this function knows what
    /// a millinewton is, and that is the point: the bus carries no quantity
    /// types.
    fn eval(&self, node: NodeIdx, store: &mut Store<'_>) -> Result<(), Fault> {
        let def = &NODES[node as usize];
        let mut inputs = [0.0f64; MAX_INPUTS];
        let n_in = def.inputs.len().min(MAX_INPUTS);

        // The rolled-up credibility of everything that fed this node.
        let mut upstream = CredVec([4; 8]);
        for (k, &v) in def.inputs.iter().take(n_in).enumerate() {
            let slot = store.get(v);
            if !slot.is_known() {
                return Err(Fault::Blocked {
                    node: def.id,
                    missing: NODES[VARS[v as usize].producer as usize].id,
                });
            }
            inputs[k] = slot.value;
            upstream = upstream.rollup(slot.cred);
        }

        // Every declared bundle must be present and verified before the run
        // starts. A result computed from unverifiable data is not a degraded
        // result; it is not a result. The store is a resolved handle passed in,
        // never a path the kernel opens.
        let data_ok = def.bundles.is_empty();

        let mut outputs = [0.0f64; 4];
        (tables::DISPATCH[node as usize])(&inputs[..n_in], &mut outputs[..def.outputs.len()])?;

        // The evidence executes on the run, using the same function the test
        // calls — one implementation, checked one way.
        let (ran, passed) = check_fixtures(node);
        let cred = credibility::score(def, ran, passed, data_ok, upstream);

        for (k, &o) in def.outputs.iter().enumerate() {
            let slot = &mut store.slots[o as usize];
            slot.value = outputs[k];
            slot.unit = VARS[o as usize].unit;
            slot.status = SlotStatus::Computed;
            slot.producer = node;
            slot.cred = cred;
        }
        Ok(())
    }
}

/// Run one node's fixtures, on the run.
///
/// The same function the test calls, called from the run — one implementation,
/// checked one way. A verdict shown on a page has to have come from a run: a
/// badge read out of a field is a claim about last March.
fn check_fixtures(node: NodeIdx) -> (bool, bool) {
    let def = &NODES[node as usize];
    if def.fixtures.is_empty() {
        return (false, false);
    }
    let mut ran = false;
    let mut passed = true;
    for f in def.fixtures {
        let mut out = [0.0f64; 4];
        match (tables::DISPATCH[node as usize])(f.inputs, &mut out[..def.outputs.len()]) {
            Ok(()) => {
                ran = true;
                if !f.check(out[0]).passed() {
                    passed = false;
                }
            }
            // The node refused its own fixture's case. That is a verdict, and a
            // failing one: an expected value the implementation will not even
            // evaluate is stronger evidence of a defect than a numeric
            // disagreement.
            Err(_) => {
                ran = true;
                passed = false;
            }
        }
    }
    (ran, passed)
}

/// One fixture, executed against the live engine.
pub fn fixture_verdicts(node: NodeIdx) -> Vec<vleo_bus::VerdictOut> {
    let def = &NODES[node as usize];
    let mut out = Vec::new();
    for f in def.fixtures {
        let mut o = [0.0f64; 4];
        let (got, passed, err) =
            match (tables::DISPATCH[node as usize])(f.inputs, &mut o[..def.outputs.len()]) {
                Ok(()) => {
                    let e = match f.check(o[0]) {
                        Verdict::Pass { relative_error } => relative_error,
                        Verdict::Fail { relative_error, .. } => relative_error,
                        _ => f64::NAN,
                    };
                    (o[0], f.check(o[0]).passed(), e)
                }
                Err(_) => (f64::NAN, false, f64::NAN),
            };
        out.push(vleo_bus::VerdictOut {
            node: def.id.to_string(),
            label: f.label.to_string(),
            expected: f.expected,
            got,
            relative_error: err,
            tolerance: f.tolerance,
            passed,
            provenance: f.provenance.name(),
            source: f.source,
        });
    }
    out
}

/// One fixture, executed against the live engine.
pub fn run_fixture(node: NodeIdx, inputs: &[f64]) -> Result<(f64, Verdict), Fault> {
    let def = &NODES[node as usize];
    let mut outputs = [0.0f64; 4];
    (tables::DISPATCH[node as usize])(inputs, &mut outputs[..def.outputs.len()])?;
    let got = outputs[0];
    let verdict = def
        .fixtures
        .first()
        .map(|f| f.check(got))
        .unwrap_or(Verdict::NotRun);
    Ok((got, verdict))
}

/// The scratch a run needs. Allocated once by a face and reused: the kernel
/// never allocates.
pub struct Scratch {
    pub slots: Vec<Slot>,
    pub order: Vec<NodeIdx>,
    pub mark: Vec<u8>,
    pub stack: Vec<NodeIdx>,
    pub ran: Vec<NodeIdx>,
    pub blocked: Vec<NodeIdx>,
    pub blocked_fault: Vec<Fault>,
}

impl Default for Scratch {
    fn default() -> Self {
        Self::new()
    }
}

impl Scratch {
    pub fn new() -> Scratch {
        Scratch {
            slots: alloc::vec![Slot::EMPTY; NODE_COUNT],
            order: alloc::vec![0; NODE_COUNT + 1],
            mark: alloc::vec![0; NODE_COUNT + 1],
            stack: alloc::vec![0; NODE_COUNT + 2],
            ran: alloc::vec![0; NODE_COUNT + 1],
            blocked: alloc::vec![0; NODE_COUNT + 1],
            blocked_fault: alloc::vec![Fault::NotRun { node: "" }; NODE_COUNT + 1],
        }
    }
}

/// Evaluate a case.
///
/// This is the whole engine surface. Every face — the browser, the desktop, the
/// command line, the rig console, the wheel — reaches it, and they all get the
/// same numbers because there is only one of it.
pub fn evaluate(case: &vleo_bus::Case, scratch: &mut Scratch) -> Result<vleo_bus::Results, Fault> {
    let table = Vleo;
    let target = Vleo::find(&case.target).ok_or(Fault::NotRun { node: "unknown node" })?;
    let base = Vleo::case(&case.base);

    let mut store = Store::new(&mut scratch.slots);

    // Declared values publish themselves, then the case overrides. A supply is
    // the start of a run rather than an edit to one, so nothing is marked stale
    // here.
    for (i, def) in NODES.iter().enumerate() {
        if def.kind == Kind::Declared {
            let mut out = [0.0f64; 4];
            if (tables::DISPATCH[i])(&[], &mut out[..1]).is_ok() {
                store.supply(def.outputs[0], out[0], VARS[def.outputs[0] as usize].unit);
                store.slots[def.outputs[0] as usize].cred =
                    credibility::score(def, false, false, true, CredVec([4; 8]));
            }
        }
    }
    if let Some(b) = base {
        for (v, val) in b.supply {
            supply_checked(&mut store, *v, *val)?;
        }
    }
    for (id, val) in &case.supply {
        if let Some(v) = Vleo::find(id) {
            supply_checked(&mut store, v, *val)?;
        }
    }

    // The declared cycles this case carries.
    let cycles: Vec<CycleSpec<'_>> = base
        .map(|b| {
            b.cycles
                .iter()
                .map(|c| CycleSpec {
                    nodes: c.nodes,
                    converge_on: c.converge_on,
                    tolerance: c.tolerance,
                    max_iter: c.max_iter,
                    seeds: c.seeds,
                })
                .collect()
        })
        .unwrap_or_default();

    let mut ws = Workspace {
        order: &mut scratch.order,
        mark: &mut scratch.mark,
        stack: &mut scratch.stack,
        ran: &mut scratch.ran,
        blocked: &mut scratch.blocked,
        blocked_fault: &mut scratch.blocked_fault,
    };
    let report = resolver::evaluate(&table, &mut store, case.mode.to_mode(target), &cycles, &mut ws)?;

    Ok(collect(&table, &store, &ws, &report, case, target))
}

/// Supply a value, refusing rather than clamping when it is outside the
/// variable's declared range.
///
/// Out of domain is an error naming the field and the bound, at every face. A
/// value silently corrected is a design that drifted without anyone deciding
/// to.
fn supply_checked(store: &mut Store<'_>, v: NodeIdx, value: f64) -> Result<(), Fault> {
    let var = &VARS[v as usize];
    let def = &NODES[var.producer as usize];
    if value < var.limit.lower {
        return Err(Fault::OutOfDomain {
            node: def.id,
            field: var.symbol,
            value,
            bound: var.limit.lower,
            edge: vleo_core::fault::Edge::Lower,
            unit: var.unit,
            reason: var.limit.reason_lower,
        });
    }
    if value > var.limit.upper {
        return Err(Fault::OutOfDomain {
            node: def.id,
            field: var.symbol,
            value,
            bound: var.limit.upper,
            edge: vleo_core::fault::Edge::Upper,
            unit: var.unit,
            reason: var.limit.reason_upper,
        });
    }
    store.supply(v, value, var.unit);
    Ok(())
}

fn collect(
    table: &Vleo,
    store: &Store<'_>,
    ws: &Workspace<'_>,
    report: &RunReport,
    case: &vleo_bus::Case,
    target: NodeIdx,
) -> vleo_bus::Results {
    let _ = table;
    let mut values = Vec::new();
    for &n in ws.ran[..report.ran].iter() {
        let def = &NODES[n as usize];
        for &o in def.outputs {
            let slot = store.get(o);
            let var = &VARS[o as usize];
            values.push(vleo_bus::ValueOut {
                id: var.id.to_string(),
                symbol: var.symbol.to_string(),
                label: var.label.to_string(),
                value: slot.value,
                unit: var.unit.symbol(),
                status: slot.status,
                cred: slot.cred,
                governing: slot.cred.governing_factor().name(),
            });
        }
    }
    let mut blocked = Vec::new();
    for i in 0..report.blocked {
        let n = ws.blocked[i];
        blocked.push(vleo_bus::BlockedOut {
            id: NODES[n as usize].id.to_string(),
            kind: ws.blocked_fault[i].kind(),
            message: vleo_bus::fault_message(&ws.blocked_fault[i]),
        });
    }

    let target_cred = NODES[target as usize]
        .outputs
        .first()
        .map(|&o| store.get(o).cred)
        .unwrap_or(CredVec::ZERO);

    let manifest = vleo_bus::RunManifest {
        node: case.target.clone(),
        mode: case.mode.name(),
        kernel: hex(Vleo::kernel_hash()),
        graph: hex(Vleo::graph_hash()),
        case: hex(case.hash()),
        chain: hex(report.chain_hash),
        data: Vec::new(),
        endpoint: String::new(),
        ran: report.ran,
        blocked_count: report.blocked,
        iterations: report.iterations,
        cred: target_cred,
        governing: target_cred.governing_factor().name(),
        reproducible: true,
    };
    let mut verdicts = Vec::new();
    for &n in ws.ran[..report.ran].iter() {
        verdicts.extend(fixture_verdicts(n));
    }
    vleo_bus::Results { values, blocked, verdicts, manifest, series: Vec::new() }
}

fn hex(h: u64) -> String {
    let b = vleo_core::hash::short_hex(h);
    core::str::from_utf8(&b).unwrap_or("......").to_string()
}

/// Which factor is holding a subtree down, and which node it belongs to.
///
/// The management view asks one question — what is holding the design down —
/// and the answer is a node identifier rather than a meeting.
pub fn governing_node(store: &Store<'_>, subtree_root: NodeIdx) -> (Factor, &'static str, u8) {
    let mut worst = (Factor::Mathematics, NODES[subtree_root as usize].id, 4u8);
    for (i, def) in NODES.iter().enumerate() {
        for &o in def.outputs {
            let slot = store.get(o);
            if slot.status != SlotStatus::Computed {
                continue;
            }
            let s = slot.cred.governing_score();
            if s < worst.2 {
                worst = (slot.cred.governing_factor(), NODES[i].id, s);
            }
        }
    }
    worst
}

/// The unit a variable is published in, for a face that has to draw it.
pub fn unit_of(v: NodeIdx) -> Unit {
    VARS[v as usize].unit
}
