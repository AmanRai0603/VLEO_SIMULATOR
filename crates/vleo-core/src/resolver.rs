//! The resolver — the only thing that reads the derivation graph.
//!
//! A node is a pure function of its declared inputs and **calls nothing**. It
//! has no way to name a peer, so the crate graph never has to mirror the
//! dataflow and a genuine closure loop — thrust needs available power, power
//! needs thrust demand — can never force two subsystems to be merged into one
//! crate. The loop is data, declared in the case, and it is iterated here.
//!
//! Order is computed, never written down. A node whose inputs are unchanged is
//! reused rather than re-run, and the key it is reused on is the **chain
//! hash**, not the case: every node the run reached, each one's implementation
//! content and published outputs, and the case. Keying on the case alone was
//! the defect this system exists to prevent, reappearing one level up — rewrite
//! a node's arithmetic without touching its interface and a case-keyed cache
//! hands back yesterday's number as though it were current.
//!
//! The resolver obeys the same rule as every node: no files, no clock, no
//! drawing. It may cache, because a hash of inputs is not a clock.

use crate::fault::Fault;
use crate::graph::{Kind, NodeIdx, NodeTable, VarIdx};
use crate::hash::Hasher;
use crate::value::{SlotStatus, Store};

/// What a single press of Run selects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// This node only. Upstream values are whatever the store already holds; if
    /// a dependency has never run the control is disabled and names it.
    Alone(NodeIdx),
    /// The whole dependency closure of one node, in topological order. Each
    /// node is evaluated once however many paths reach it. This is the mode
    /// that produces a defensible number.
    Branch(NodeIdx),
    /// Everything buildable, in topological order. Unbuilt nodes stay blocked
    /// and are listed. With four nodes it runs four; with all of them it runs
    /// the tool — which is why a half-built tree is the normal operating mode
    /// rather than a special case.
    All,
}

/// A cycle the design actually has, declared in the case.
///
/// A cycle is a property of the design, so it is declared where design
/// decisions live. Non-convergence is a reportable result with its iteration
/// history, not a crash.
#[derive(Clone, Copy, Debug)]
pub struct CycleSpec<'a> {
    /// The nodes that close the loop, in the order they should be relaxed.
    pub nodes: &'a [NodeIdx],
    /// The variable whose change decides convergence.
    pub converge_on: VarIdx,
    /// Relative tolerance on that variable between successive sweeps.
    pub tolerance: f64,
    pub max_iter: u32,
    /// Starting values, so the first sweep has something to read. A seed is a
    /// declared value like any other: it decides which fixed point the
    /// iteration reaches, so it carries a source and a confirmation.
    pub seeds: &'a [(VarIdx, f64)],
}

/// Scratch space the caller owns, so the kernel never allocates.
///
/// Every buffer must be at least as long as the node table; `blocked_fault` is
/// parallel to `blocked`. A workspace that is too small is a reported fault,
/// never a silent truncation.
pub struct Workspace<'a> {
    pub order: &'a mut [NodeIdx],
    pub mark: &'a mut [u8],
    pub stack: &'a mut [NodeIdx],
    pub ran: &'a mut [NodeIdx],
    pub blocked: &'a mut [NodeIdx],
    pub blocked_fault: &'a mut [Fault],
}

const UNVISITED: u8 = 0;
const IN_PROGRESS: u8 = 1;
const DONE: u8 = 2;

/// What a run did.
#[derive(Clone, Copy, Debug)]
pub struct RunReport {
    /// How many nodes evaluated. Read `workspace.ran[..ran]` for which.
    pub ran: usize,
    /// How many could not, and why. Read `workspace.blocked[..blocked]` and the
    /// parallel `blocked_fault`. Never hidden: the answer is always
    /// "n ran, m blocked", and the blocked ones are named.
    pub blocked: usize,
    /// The identity of this run, and the cache key. Changes when any node in
    /// the chain changes its implementation, not only when the case changes.
    pub chain_hash: u64,
    /// Total sweeps spent inside declared cycles.
    pub iterations: u32,
    /// The first fault, for a caller that wants one line rather than a list.
    pub first_fault: Option<Fault>,
}

/// Evaluate the graph.
///
/// `cycles` are the loops the case declares. An undeclared cycle is a fault
/// naming both ends, never an infinite loop.
pub fn evaluate<T: NodeTable + ?Sized>(
    table: &T,
    store: &mut Store<'_>,
    mode: Mode,
    cycles: &[CycleSpec<'_>],
    ws: &mut Workspace<'_>,
) -> Result<RunReport, Fault> {
    let n = table.nodes().len();
    if ws.order.len() < n
        || ws.mark.len() < n
        || ws.stack.len() < n + 1
        || ws.ran.len() < n
        || ws.blocked.len() < n
        || ws.blocked_fault.len() < n
    {
        return Err(Fault::WorkspaceTooSmall {
            needed: n + 1,
            given: ws.order.len(),
        });
    }
    for m in ws.mark[..n].iter_mut() {
        *m = UNVISITED;
    }

    // ---- 1. topological order over what was asked for -----------------------
    let mut order_len = 0usize;
    match mode {
        Mode::Alone(target) => {
            ws.order[0] = target;
            order_len = 1;
        }
        Mode::Branch(target) => {
            order_len = topo_from(table, target, cycles, ws, order_len)?;
        }
        Mode::All => {
            for i in 0..n {
                order_len = topo_from(table, i as NodeIdx, cycles, ws, order_len)?;
            }
        }
    }

    // ---- 2. evaluate, in that order ----------------------------------------
    let mut hasher = Hasher::new();
    let mut ran = 0usize;
    let mut blocked = 0usize;
    let mut iterations = 0u32;
    let mut first_fault: Option<Fault> = None;
    // A node inside a declared cycle is run by the cycle sweep, not on its own.
    let mut cycle_done = [false; 32];

    let mut idx = 0usize;
    while idx < order_len {
        let node = ws.order[idx];
        idx += 1;
        let def = table.node(node);

        // A node retired as *wrong* poisons its consumers deliberately.
        if let crate::graph::Retirement::Wrong { replacement } = def.retirement {
            let f = Fault::DependsOnWithdrawn {
                node: def.id,
                withdrawn: def.id,
                replacement,
            };
            record_block(ws, &mut blocked, &mut first_fault, node, f);
            continue;
        }
        if !def.state.runnable() {
            let f = Fault::NotRun { node: def.id };
            record_block(ws, &mut blocked, &mut first_fault, node, f);
            continue;
        }
        // Declared values publish themselves; there is nothing to compute.
        //
        // Unless the case supplied one. A case is per-customer values against
        // one shared architecture, so an override is the whole mechanism —
        // re-publishing the sheet's own number on top of it would make every
        // case identical and every sweep flat, which is the kind of failure
        // that looks like working software.
        if def.kind == Kind::Declared && def.inputs.is_empty() {
            if def
                .outputs
                .iter()
                .all(|&o| store.get(o).status == SlotStatus::Supplied)
            {
                ws.ran[ran] = node;
                ran += 1;
                hash_node(&mut hasher, table, node, store);
                continue;
            }
            match table.eval(node, store) {
                Ok(()) => {
                    ws.ran[ran] = node;
                    ran += 1;
                    hash_node(&mut hasher, table, node, store);
                }
                Err(f) => record_block(ws, &mut blocked, &mut first_fault, node, f),
            }
            continue;
        }
        // A declared cycle is checked *before* the input check. Inside a loop
        // every member is waiting on the next one by construction, so asking
        // "are all your inputs known" first would block the whole loop and
        // report it as eight independent missing dependencies. The seed is what
        // breaks the deadlock, and it is applied by the sweep.
        if let Some((ci, spec)) = cycle_of(cycles, node) {
            if ci < 32 && cycle_done[ci] {
                continue;
            }
            match sweep_cycle(table, store, spec, &mut iterations) {
                Ok(()) => {
                    for &m in spec.nodes {
                        ws.ran[ran] = m;
                        ran += 1;
                        hash_node(&mut hasher, table, m, store);
                    }
                }
                Err(f) => {
                    for &m in spec.nodes {
                        record_block(ws, &mut blocked, &mut first_fault, m, f);
                    }
                }
            }
            if ci < 32 {
                cycle_done[ci] = true;
            }
            continue;
        }

        // Every input must be known, or the node is blocked and says by what.
        if let Some(f) = missing_input(table, store, node) {
            record_block(ws, &mut blocked, &mut first_fault, node, f);
            continue;
        }

        match table.eval(node, store) {
            Ok(()) => {
                ws.ran[ran] = node;
                ran += 1;
                hash_node(&mut hasher, table, node, store);
            }
            Err(f) => {
                for &o in def.outputs {
                    store.slots[o as usize].status = SlotStatus::Refused;
                }
                record_block(ws, &mut blocked, &mut first_fault, node, f);
            }
        }
    }

    // The case is part of the run's identity, not only the chain.
    for s in store.slots.iter() {
        if s.status == SlotStatus::Supplied {
            hasher.write_f64(s.value);
        }
    }

    Ok(RunReport {
        ran,
        blocked,
        chain_hash: hasher.finish(),
        iterations,
        first_fault,
    })
}

fn record_block(
    ws: &mut Workspace<'_>,
    blocked: &mut usize,
    first: &mut Option<Fault>,
    node: NodeIdx,
    f: Fault,
) {
    if *blocked < ws.blocked.len() {
        ws.blocked[*blocked] = node;
        ws.blocked_fault[*blocked] = f;
        *blocked += 1;
    }
    if first.is_none() {
        *first = Some(f);
    }
}

/// Which input is not available, if any. Names the producing node rather than
/// the variable, because "run this node" is what the reader has to do next.
fn missing_input<T: NodeTable + ?Sized>(
    table: &T,
    store: &Store<'_>,
    node: NodeIdx,
) -> Option<Fault> {
    let def = table.node(node);
    for &v in def.inputs {
        let slot = store.get(v);
        if !slot.is_known() {
            let producer = table.var(v).producer;
            let missing = if producer == u16::MAX {
                table.var(v).id
            } else {
                table.node(producer).id
            };
            return Some(Fault::Blocked {
                node: def.id,
                missing,
            });
        }
    }
    None
}

/// Depth-first post-order over the derivation graph, appending to `ws.order`.
/// Iterative, because a recursive walk on a deep graph is a stack overflow
/// nobody can debug from a browser.
fn topo_from<T: NodeTable + ?Sized>(
    table: &T,
    root: NodeIdx,
    cycles: &[CycleSpec<'_>],
    ws: &mut Workspace<'_>,
    mut order_len: usize,
) -> Result<usize, Fault> {
    if ws.mark[root as usize] == DONE {
        return Ok(order_len);
    }
    let mut sp = 0usize;
    ws.stack[sp] = root;
    sp += 1;
    while sp > 0 {
        let node = ws.stack[sp - 1];
        let m = ws.mark[node as usize];
        if m == DONE {
            sp -= 1;
            continue;
        }
        if m == IN_PROGRESS {
            // Every child has been dealt with; emit.
            ws.mark[node as usize] = DONE;
            ws.order[order_len] = node;
            order_len += 1;
            sp -= 1;
            continue;
        }
        ws.mark[node as usize] = IN_PROGRESS;
        let def = table.node(node);

        // A declared cycle is ordered as one unit. Every member's *external*
        // dependencies have to be satisfied before the loop is entered,
        // because the sweep runs all of them. Expanding only this node's own
        // inputs would schedule the loop before a sibling member's input was
        // computed, and the first sweep would then refuse on a dependency
        // nobody could see was missing.
        let members: &[NodeIdx] = match cycle_of(cycles, node) {
            Some((_, spec)) => spec.nodes,
            None => core::slice::from_ref(&node),
        };
        for &m in members {
            for &v in table.node(m).inputs {
                let producer = table.var(v).producer;
                if producer == u16::MAX || producer == m {
                    continue;
                }
                if members.len() > 1 && members.contains(&producer) {
                    continue; // an edge inside the loop; the sweep relaxes it
                }
                match ws.mark[producer as usize] {
                    DONE => {}
                    IN_PROGRESS => {
                        // A loop. Declared cycles break the edge here and are
                        // relaxed later; anything else is a named error.
                        if same_declared_cycle(cycles, node, producer) {
                            continue;
                        }
                        return Err(Fault::UndeclaredCycle {
                            node: def.id,
                            back_to: table.node(producer).id,
                        });
                    }
                    _ => {
                        if sp < ws.stack.len() {
                            ws.stack[sp] = producer;
                            sp += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(order_len)
}

fn same_declared_cycle(cycles: &[CycleSpec<'_>], a: NodeIdx, b: NodeIdx) -> bool {
    cycles
        .iter()
        .any(|c| c.nodes.contains(&a) && c.nodes.contains(&b))
}

fn cycle_of<'c, 'a>(
    cycles: &'c [CycleSpec<'a>],
    node: NodeIdx,
) -> Option<(usize, &'c CycleSpec<'a>)> {
    cycles
        .iter()
        .enumerate()
        .find(|(_, c)| c.nodes.contains(&node))
}

/// Relax a declared cycle to its criterion.
fn sweep_cycle<T: NodeTable + ?Sized>(
    table: &T,
    store: &mut Store<'_>,
    spec: &CycleSpec<'_>,
    iterations: &mut u32,
) -> Result<(), Fault> {
    for &(v, seed) in spec.seeds {
        if !store.get(v).is_known() {
            let unit = table.var(v).unit;
            store.supply(v, seed, unit);
        }
    }
    let mut previous = store.get(spec.converge_on).value;
    let mut i = 0u32;
    while i < spec.max_iter {
        for &m in spec.nodes {
            table.eval(m, store)?;
        }
        i += 1;
        *iterations += 1;
        let now = store.get(spec.converge_on).value;
        let denom = crate::units::pmath::abs(now);
        let residual = if denom > 0.0 {
            crate::units::pmath::abs(now - previous) / denom
        } else {
            crate::units::pmath::abs(now - previous)
        };
        if residual <= spec.tolerance {
            settle_credibility(table, store, spec)?;
            return Ok(());
        }
        previous = now;
    }
    let now = store.get(spec.converge_on).value;
    Err(Fault::NotConverged {
        node: table.node(spec.nodes[0]).id,
        iterations: i,
        residual: crate::units::pmath::abs(now - previous),
        tolerance: spec.tolerance,
    })
}

/// Recompute the loop's credibility from the converged state.
///
/// Credibility rolls up by taking the minimum factor by factor, which is right
/// along a chain and wrong inside a fixed point: a loop reads its own outputs,
/// so once any zero enters it is absorbing and every member ends at zero
/// however many sweeps run. That is not a statement about the design, it is an
/// artefact of the propagation rule meeting a cycle.
///
/// The honest semantics: inside a converged fixed point the members do not
/// weaken each other. The loop is as credible as what feeds it from outside
/// and as what each member is made of. This clears the members' vectors and
/// runs one final pass, so an edge inside the loop contributes nothing and an
/// edge from outside contributes its real value.
fn settle_credibility<T: NodeTable + ?Sized>(
    table: &T,
    store: &mut Store<'_>,
    spec: &CycleSpec<'_>,
) -> Result<(), Fault> {
    for &m in spec.nodes {
        for &o in table.node(m).outputs {
            store.slots[o as usize].cred = crate::credibility::CredVec([4; 8]);
        }
    }
    for &m in spec.nodes {
        table.eval(m, store)?;
    }
    Ok(())
}

/// Fold one node's identity and its published outputs into the chain hash.
fn hash_node<T: NodeTable + ?Sized>(h: &mut Hasher, table: &T, node: NodeIdx, store: &Store<'_>) {
    let def = table.node(node);
    h.write_str(def.id);
    h.write_u64(def.sheet_hash);
    h.write_u64(def.impl_hash);
    for &o in def.outputs {
        h.write_f64(store.get(o).value);
    }
}

/// Mark everything downstream of a changed variable stale, transitively.
///
/// This is the only definition of staleness that is safe to show a person:
/// change any node in a chain and every result that depended on it is unknown
/// until it is re-run. Not wrong — unknown.
pub fn mark_downstream_stale<T: NodeTable + ?Sized>(
    table: &T,
    store: &mut Store<'_>,
    changed: VarIdx,
) {
    // Fixed-point sweep. The graph is small and this runs on an edit, not in a
    // campaign, so a clear O(n·e) sweep beats an index that can go stale.
    store.mark_stale(changed);
    let mut moved = true;
    while moved {
        moved = false;
        for (i, def) in table.nodes().iter().enumerate() {
            let dirty = def.inputs.iter().any(|&v| {
                matches!(
                    store.get(v).status,
                    SlotStatus::Stale | SlotStatus::Empty | SlotStatus::Refused
                )
            });
            if dirty {
                for &o in def.outputs {
                    if store.get(o).status == SlotStatus::Computed {
                        store.slots[o as usize].status = SlotStatus::Stale;
                        moved = true;
                    }
                }
            }
            let _ = i;
        }
    }
}
