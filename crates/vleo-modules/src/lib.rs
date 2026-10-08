//! The facade — the subsystem crates behind one dependency, and the one `NodeTable`
//! the resolver walks.
//!
//! Every face takes a single dependency on this crate. The compiler still sees
//! one unit per subsystem, so they build in parallel, and no face can reach a module
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
use vleo_core::graph::{Behaviour, Kind, NodeDef, NodeIdx, NodeTable, VarDef};
use vleo_core::resolver::{self, CycleSpec, RunReport, Workspace};
use vleo_core::value::{Slot, SlotStatus, Store};
use vleo_units::Unit;

/// The graph, compiled in.
///
/// A declared limit that happens to equal pi/2 — an inclination bound, a beta
/// angle — is a bound, not an approximation standing in for a named constant,
/// so the lint that would rewrite it is switched off across the generated
/// tables.
#[allow(clippy::approx_constant)]
pub mod tables {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
}

include!(concat!(env!("OUT_DIR"), "/engine_source.rs"));

pub use tables::{CASES, GROUPS, NODES, NODE_COUNT, RELATIONS, VARS, VAR_COUNT};

/// The most inputs a row may declare: the size of the engine's scratch.
///
/// Both caps were once 16 and 4, and both were outgrown. The output cap
/// panicked on the first row whose answer was a set, which was loud. The
/// input cap was not: `eval` sliced to it, and a row declaring more inputs
/// received fewer, which surfaced as a row "blocked on an input that has never
/// run". A design is read at run time now, so it cannot size the engine; a
/// design wider than these is refused by name when it is opened, never cut
/// short (`opened`).
pub const MAX_INPUTS: usize = 32;
/// The most values a row may publish, its own answer included.
pub const MAX_OUTPUTS: usize = 32;

/// A loop the design actually has, declared where design decisions live.
pub struct CycleDef {
    pub nodes: &'static [u16],
    pub converge_on: u16,
    pub tolerance: f64,
    pub max_iter: u32,
    pub seeds: &'static [(u16, f64)],
}

/// The case: the one multipayload design. See `cases/`.
pub struct CaseDef {
    pub id: &'static str,
    pub label: &'static str,
    pub note: &'static str,
    pub supply: &'static [(u16, f64)],
    pub cycles: &'static [CycleDef],
    /// The inputs in the Condition group; every other declared input is Customer.
    pub conditions: &'static [u16],
}

/// One heading in the tree.
pub struct GroupDef {
    pub id: &'static str,
    pub label: &'static str,
    pub parent: &'static str,
    pub owner: &'static str,
    /// 1 management · 2 the system · 3 subsystem · 4 the run.
    pub layer: u8,
    /// Where the heading sits among its siblings, as it was written.
    /// A face that draws the tree sorts by this; the table itself is
    /// folder-ordered so that generation stays deterministic.
    pub order: u32,
    /// Drawn as a nested box on the diagonal. A mark inside a box is
    /// coupling that subtree owns; a mark outside it crosses a boundary.
    pub is_box: bool,
    /// The colour family the branch is drawn in — what makes a branch
    /// findable on a tree of thirteen hundred rows.
    pub tone: &'static str,
    /// The cases this branch is in play for. Empty means every case.
    pub cases: &'static [&'static str],
}

/// One node's relation, as the engine calls it: SI values in, SI values out,
/// its sheet's guards applied.
pub type NodeFn = fn(&[f64], &mut [f64]) -> Result<(), Fault>;

/// A relation the graph runs itself rather than as compiled code — a node's
/// method, run by the interpreter — with the same contract as a [`NodeFn`].
pub type Relation = dyn Fn(&[f64], &mut [f64]) -> Result<(), Fault> + Send + Sync;

/// The graph the engine runs: its nodes, its variables, each node's relation
/// and the cases.
///
/// The graph compiled into this build is [`COMPILED`]. One read from the
/// design's files when the engine opens has the same shape (`opened`,
/// docs/PLAN_1_0.md phase D), so both run through this one engine and each is
/// the other's check: there is no second resolver, adapter or fixture runner
/// for a parity gate to compare against itself.
pub struct Graph {
    pub nodes: &'static [NodeDef],
    pub vars: &'static [VarDef],
    pub dispatch: &'static [NodeFn],
    /// Each node's relation when the graph runs it itself, in place of its
    /// entry in `dispatch`. Empty for the compiled graph.
    pub run: &'static [Option<&'static Relation>],
    pub cases: &'static [CaseDef],
    /// The headings of the tree the rows hang from.
    pub groups: &'static [GroupDef],
    /// The labelled relations between headings: (from, to, why).
    pub relations: &'static [(&'static str, &'static str, &'static str)],
    /// Each recorded row's current version and the release that carried it:
    /// (row, version, release).
    pub versions: &'static [(&'static str, u32, &'static str)],
    /// What each node gave on its own cases, worked out the first time they
    /// are asked for ([`Graph::fixture_runs`]).
    pub cases_run: CasesRun,
}

/// Each node's answers to its own cases, kept by the graph that ran them.
#[cfg(feature = "std")]
pub struct CasesRun(std::sync::OnceLock<Box<[std::sync::OnceLock<Answers>]>>);

/// One node's answers to its cases, in order: `None` where it refused one.
#[cfg(feature = "std")]
type Answers = Vec<Option<f64>>;

#[cfg(feature = "std")]
impl CasesRun {
    pub const fn new() -> Self {
        CasesRun(std::sync::OnceLock::new())
    }
}

/// Without the standard library nothing is kept, and every ask runs the cases.
#[cfg(not(feature = "std"))]
pub struct CasesRun;

#[cfg(not(feature = "std"))]
impl CasesRun {
    pub const fn new() -> Self {
        CasesRun
    }
}

impl Default for CasesRun {
    fn default() -> Self {
        Self::new()
    }
}

/// The graph compiled into this build, from the sheets as they were.
pub static COMPILED: Graph = Graph {
    nodes: &NODES,
    vars: &VARS,
    dispatch: &tables::DISPATCH,
    run: &[],
    cases: &CASES,
    groups: &GROUPS,
    relations: &RELATIONS,
    versions: tables::NODE_VERSIONS,
    cases_run: CasesRun::new(),
};

/// The graph this process's engine runs, when a face has installed one.
#[cfg(feature = "std")]
static ENGINE: std::sync::RwLock<Option<&'static Graph>> = std::sync::RwLock::new(None);

/// The graph the engine runs: the one a face installed from the design's
/// files ([`run_on`]), or the compiled one until it does. Every function a
/// face calls — [`evaluate`], [`probe`], [`fixture_verdicts`], [`Vleo::find`]
/// and the rest — runs on this one.
pub fn engine() -> &'static Graph {
    #[cfg(feature = "std")]
    {
        if let Some(g) = *ENGINE.read().unwrap_or_else(|e| e.into_inner()) {
            return g;
        }
    }
    &COMPILED
}

/// The rows of the graph the engine runs, in its order.
pub fn nodes() -> &'static [NodeDef] {
    engine().nodes
}

/// The variables of the graph the engine runs, in its order.
pub fn vars() -> &'static [VarDef] {
    engine().vars
}

/// The cases of the graph the engine runs.
pub fn cases() -> &'static [CaseDef] {
    engine().cases
}

/// The headings of the graph the engine runs.
pub fn groups() -> &'static [GroupDef] {
    engine().groups
}

/// The labelled relations between the headings of the graph the engine runs.
pub fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
    engine().relations
}

/// Run the engine on `graph` from now on (docs/PLAN_1_0.md, phase D: every
/// face is re-pointed).
///
/// Any graph the design's files make: every face reads the rows, values,
/// cases and headings it names from the graph that runs ([`nodes`], [`vars`],
/// [`cases`], [`groups`]), so a design laid out otherwise than this build is
/// answered under its own names.
#[cfg(feature = "std")]
pub fn run_on(graph: &'static Graph) {
    *ENGINE.write().unwrap_or_else(|e| e.into_inner()) = Some(graph);
}

/// Run the engine on the compiled graph again.
#[cfg(feature = "std")]
pub fn run_compiled() {
    *ENGINE.write().unwrap_or_else(|e| e.into_inner()) = None;
}

impl Graph {
    /// Run node `i` as its behaviour says: an open row refuses by its own id;
    /// a lookup is read along its table and its children's ports are its
    /// answer, both by the engine itself; any other row runs its relation —
    /// the graph's own when it has one, else the compiled function.
    pub fn call(&self, i: usize, inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
        let def = &self.nodes[i];
        match def.behaviour {
            Behaviour::Open => Err(Fault::NotRun { node: def.id }),
            Behaviour::Lookup(l) => {
                let x = *inputs.get(l.by as usize).ok_or(Fault::Blocked {
                    node: def.id,
                    missing: "the input its table is read along",
                })?;
                let var = &self.vars[def.inputs[l.by as usize] as usize];
                let y = l.read(def.id, var.id, var.unit, x)?;
                *outputs.first_mut().ok_or(Fault::Degenerate {
                    node: def.id,
                    field: "",
                    reason: "a lookup answers one output, and this row has none",
                })? = y;
                Ok(())
            }
            Behaviour::Children(c) => {
                for (k, out) in outputs.iter_mut().enumerate() {
                    let from = c.from.get(k).ok_or(Fault::Degenerate {
                        node: def.id,
                        field: "",
                        reason: "an output no child answers",
                    })?;
                    *out = *inputs.get(*from as usize).ok_or(Fault::Blocked {
                        node: def.id,
                        missing: "the child port that answers it",
                    })?;
                }
                Ok(())
            }
            _ => self.estimate(i, inputs, outputs),
        }
    }

    /// What a row's own cases are run against: its answer, except for a row
    /// answered by its children, whose cases test the estimate it kept.
    /// Whether the children together reproduce them is the integration check
    /// a group makes of its breakdown.
    fn against_its_cases(
        &self,
        i: usize,
        inputs: &[f64],
        outputs: &mut [f64],
    ) -> Result<(), Fault> {
        match self.nodes[i].behaviour {
            Behaviour::Children(_) => self.estimate(i, inputs, outputs),
            _ => self.call(i, inputs, outputs),
        }
    }

    /// Node `i`'s own relation, whatever its behaviour: for a row answered by
    /// its children, the estimate it kept from before it was broken down.
    pub fn estimate(&self, i: usize, inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
        match self.run.get(i) {
            Some(Some(r)) => r(inputs, outputs),
            _ => (self.dispatch[i])(inputs, outputs),
        }
    }

    /// How many nodes the graph runs itself rather than as compiled code.
    pub fn run_by_the_graph(&self) -> usize {
        self.run.iter().filter(|r| r.is_some()).count()
    }
}

/// Re-exported so a face has one name to import.
pub use vleo_bus as bus;
pub use vleo_core as core_engine;
pub use vleo_units as units;

/// The engine, holding the resolved data handle for one run.
///
/// It is constructed per run rather than being a unit struct so that the
/// bundles a run was given travel with it. Nothing here is global: a sweep is
/// a parallel map with no mutex, because there is nothing shared to protect.
pub struct Vleo {
    data: Vec<String>,
    graph: &'static Graph,
}

impl Vleo {
    /// The engine with no reference data. Every node that declares a bundle
    /// refuses, by name.
    pub fn bare() -> Vleo {
        Vleo::on(engine(), Vec::new())
    }
    /// The engine given the bundles a face verified before the run.
    pub fn with_data(data: Vec<String>) -> Vleo {
        Vleo::on(engine(), data)
    }
    /// The engine on a graph, given the bundles a face verified.
    pub fn on(graph: &'static Graph, data: Vec<String>) -> Vleo {
        Vleo { data, graph }
    }

    /// Identifies this build of the engine (`Graph::kernel_hash`).
    pub fn kernel_hash() -> u64 {
        engine().kernel_hash()
    }

    /// Identifies the graph (`Graph::graph_hash`).
    pub fn graph_hash() -> u64 {
        engine().graph_hash()
    }

    pub fn find(id: &str) -> Option<NodeIdx> {
        engine().find(id)
    }

    pub fn case(id: &str) -> Option<&'static CaseDef> {
        engine().case(id)
    }

    /// The case a run starts from when nobody names one (`Graph::default_case`).
    pub fn default_case() -> Option<&'static CaseDef> {
        engine().default_case()
    }

    /// The case a run names, or the default when it names none.
    pub fn case_of(case: &vleo_bus::Case) -> Option<&'static CaseDef> {
        engine().case_of(case)
    }
}

impl Graph {
    /// Identifies this build of the engine. Travels into every result: a page
    /// carrying a different one refuses to run rather than showing a number
    /// from an engine it was not built against.
    pub fn kernel_hash(&self) -> u64 {
        let mut h = vleo_core::hash::Hasher::new();
        h.write_str(env!("CARGO_PKG_VERSION"));
        // The relations every node calls, not only each node's own code: a
        // saved result is reused on this identity, so a corrected formula
        // must be a different engine.
        h.write_u64(ENGINE_SOURCE);
        for n in self.nodes.iter() {
            h.write_u64(n.impl_hash);
        }
        h.finish()
    }

    /// Identifies the graph. Changes when an edge does.
    pub fn graph_hash(&self) -> u64 {
        let mut h = vleo_core::hash::Hasher::new();
        for n in self.nodes.iter() {
            h.write_str(n.id);
            h.write_u64(n.sheet_hash);
            for i in n.inputs {
                h.write_u16(*i);
            }
        }
        h.finish()
    }

    /// The design as this graph runs it, in one number: every row — its id,
    /// kind, state, behaviour and the table or children it reads, its sheet
    /// and implementation hashes, its wiring, bundles and cases — every
    /// variable's unit, limits and port, and every case's supply and loops.
    /// Two graphs with the same fingerprint run the same design; it is what
    /// today's design is compared by across computers (docs/PLAN_1_0.md,
    /// phase D: "built identically on two computers from the same drive").
    /// Outside it: labels and prose, which change no answer.
    pub fn design_fingerprint(&self) -> u64 {
        let mut h = vleo_core::hash::Hasher::new();
        for n in self.nodes.iter() {
            h.write_str(n.id);
            h.write_str(n.kind.name());
            h.write_u8(n.state as u8);
            h.write_str(n.behaviour.name());
            match n.behaviour {
                Behaviour::Lookup(l) => {
                    h.write_u8(l.by);
                    h.write_str(l.read.name());
                    for v in l.table.x.iter().chain(l.table.y) {
                        h.write_f64(*v);
                    }
                }
                Behaviour::Children(c) => {
                    h.write_str(c.group);
                    h.write_bytes(c.from);
                }
                _ => {}
            }
            h.write_u64(n.sheet_hash);
            h.write_u64(n.impl_hash);
            h.write_u8(n.derived as u8);
            for i in n.inputs.iter().chain(n.outputs) {
                h.write_u16(*i);
            }
            h.write_u8(0x1e);
            for b in n.bundles {
                h.write_str(b);
            }
            for f in n.fixtures {
                h.write_str(f.label);
                h.write_f64(f.expected);
                h.write_f64(f.tolerance);
                for x in f.inputs {
                    h.write_f64(*x);
                }
            }
        }
        for v in self.vars.iter() {
            h.write_str(v.id);
            h.write_str(v.unit.name());
            h.write_f64(v.limit.lower);
            h.write_f64(v.limit.upper);
            h.write_str(v.port.state.name());
            h.write_str(v.port.maturity.name());
            h.write_str(v.port.parameter.map_or("", |l| l.name()));
            h.write_str(v.port.open_owner);
            h.write_str(v.port.open_due);
        }
        for c in self.cases.iter() {
            h.write_str(c.id);
            for (k, x) in c.supply {
                h.write_u16(*k);
                h.write_f64(*x);
            }
            for cy in c.cycles {
                for k in cy.nodes {
                    h.write_u16(*k);
                }
                h.write_u16(cy.converge_on);
                h.write_f64(cy.tolerance);
                h.write_u64(cy.max_iter as u64);
                for (k, x) in cy.seeds {
                    h.write_u16(*k);
                    h.write_f64(*x);
                }
            }
        }
        h.finish()
    }

    /// What the design answers on `case`, in one number: one run of the whole
    /// design, every value it gives by its bits and every row it refuses with
    /// its reason, in the order they ran. Two computers that agree on this
    /// gave every answer alike, to the last bit — which is what portable
    /// maths (`vleo_units::pmath`) is for.
    pub fn answers_fingerprint(&'static self, case: &vleo_bus::Case) -> Result<u64, Fault> {
        let mut all = case.clone();
        all.mode = vleo_bus::RunMode::All;
        if self.find(&all.target).is_none() {
            all.target = self.nodes[0].id.into();
        }
        let r = self.evaluate(&all, &mut Scratch::for_graph(self))?;
        let mut h = vleo_core::hash::Hasher::new();
        for v in &r.values {
            h.write_str(&v.id);
            h.write_bytes(&v.value.to_bits().to_le_bytes());
        }
        h.write_u8(0x1e);
        for b in &r.blocked {
            h.write_str(&b.id);
            h.write_str(&b.message);
        }
        Ok(h.finish())
    }

    pub fn find(&self, id: &str) -> Option<NodeIdx> {
        self.nodes
            .iter()
            .position(|n| n.id == id)
            .map(|i| i as NodeIdx)
    }

    pub fn case(&self, id: &str) -> Option<&'static CaseDef> {
        self.cases.iter().find(|c| c.id == id)
    }

    /// The case a run starts from when nobody names one: the first, and today
    /// the only one.
    ///
    /// Read from the table rather than written into each face, because five
    /// faces each holding the string "nominal" is how a renamed file turned
    /// every default run into a run of nothing in particular.
    pub fn default_case(&self) -> Option<&'static CaseDef> {
        self.cases.first()
    }

    /// The case a run names, or the default when it names none.
    pub fn case_of(&self, case: &vleo_bus::Case) -> Option<&'static CaseDef> {
        if case.base.is_empty() {
            self.default_case()
        } else {
            self.case(&case.base)
        }
    }
}

/// Why a run cannot start from the case it names, if it cannot.
///
/// Every face asks this before it evaluates. An unknown name is refused rather
/// than ignored: the engine used to run the bare declared design for a case it
/// had never heard of, which is a number with somebody else's name on it — a
/// test and two tools named cases that did not exist and passed for years.
pub fn case_refusal(case: &vleo_bus::Case) -> Option<String> {
    engine().case_refusal(case)
}

impl Graph {
    /// Why a run cannot start from the case it names ([`case_refusal`]).
    pub fn case_refusal(&self, case: &vleo_bus::Case) -> Option<String> {
        // A value for a row that does not exist was skipped, and the run went on
        // without it — the same substitution, one level down.
        for (id, v) in &case.supply {
            if self.find(id).is_none() {
                return Some(alloc::format!(
                    "the case sets '{id}', and there is no such row"
                ));
            }
            if !v.is_finite() {
                return Some(alloc::format!("the value set for '{id}' is not a number"));
            }
        }
        if self.case_of(case).is_some() {
            return None;
        }
        let names = self
            .cases
            .iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
            .join(", ");
        Some(if case.base.is_empty() {
            "cases/ holds no case, so there is nothing to run".to_string()
        } else {
            alloc::format!("there is no case '{}'. The cases are: {}", case.base, names)
        })
    }
}

/// Why a value supplied to `id` cannot be applied, or `None` when it can.
///
/// A supplied value survives only on a row that declares its own number; every
/// other kind works its answer out during the run and overwrites what was
/// supplied, so the run would report a number nobody asked for. The command
/// line and the server each had their own copy of this rule, and the two had
/// already drifted once; this is the one both call.
pub fn why_not_suppliable(id: &str) -> Option<String> {
    engine().why_not_suppliable(id)
}

impl Graph {
    /// Why a value supplied to `id` cannot be applied ([`why_not_suppliable`]).
    pub fn why_not_suppliable(&self, id: &str) -> Option<String> {
        let Some(k) = self.find(id) else {
            return Some(alloc::format!("there is no row called '{id}'"));
        };
        let def = &self.nodes[k as usize];
        let what = match def.kind {
            Kind::Declared => return None,
            Kind::Computed => "computed from its inputs",
            Kind::Required => "a target handed down from the layer above",
            Kind::Achieved => "what a subsystem returned",
            Kind::Kpi => "a key performance indicator",
        };
        Some(alloc::format!(
            "'{}' is {what}, so a supplied value would be overwritten the moment it is \
         evaluated. Set one of the declared numbers it reads instead.",
            def.id
        ))
    }
}

pub mod design;
mod error;
pub use error::{Error, ErrorKind};
pub mod figure;
/// The health map (phase D): every node's state, rolled up valve by valve,
/// and a closure traced to its cause. It runs the whole design, so only with
/// the std feature.
#[cfg(feature = "std")]
pub mod health;
pub mod inputs;
/// The graph read from the design's files when the engine opens (phase D):
/// it needs the loader, so only with the std feature.
#[cfg(feature = "std")]
pub mod opened;
/// The numbers the record's figures draw (phase 10): read from the reference
/// bundle, so only with the store that holds it.
#[cfg(feature = "std")]
pub mod record;
pub mod results;
pub mod thermo;

impl NodeTable for Vleo {
    fn nodes(&self) -> &[NodeDef] {
        self.graph.nodes
    }
    fn vars(&self) -> &[VarDef] {
        self.graph.vars
    }

    /// Move values between the store and one node's typed function.
    ///
    /// Everything crossing here is SI `f64`. The node's own generated adapter
    /// re-types it, applies the guards the sheet declared, and hands back the
    /// one answer the node exists to give. Nothing in this function knows what
    /// a millinewton is, and that is the point: the bus carries no quantity
    /// types.
    fn eval(&self, node: NodeIdx, store: &mut Store<'_>) -> Result<(), Fault> {
        let def = &self.graph.nodes[node as usize];
        let mut inputs = [0.0f64; MAX_INPUTS];
        let n_in = def.inputs.len().min(MAX_INPUTS);

        // The rolled-up credibility of everything that fed this node.
        let mut upstream = CredVec([4; 8]);
        for (k, &v) in def.inputs.iter().take(n_in).enumerate() {
            let slot = store.get(v);
            if !slot.is_known() {
                return Err(Fault::Blocked {
                    node: def.id,
                    missing: self.graph.nodes[self.graph.vars[v as usize].producer as usize].id,
                });
            }
            inputs[k] = slot.value;
            upstream = upstream.rollup(slot.cred);
        }

        // Every declared bundle must be present and verified before the run
        // starts. A result computed from unverifiable data is not a degraded
        // result; it is not a result. The store is a resolved handle passed in,
        // never a path the kernel opens.
        let data_ok = def.bundles.iter().all(|b| self.data.iter().any(|d| d == b));
        if !data_ok {
            if let Some(missing) = def
                .bundles
                .iter()
                .find(|b| !self.data.iter().any(|d| d == *b))
            {
                return Err(Fault::DataMissing {
                    node: def.id,
                    bundle: missing,
                });
            }
        }

        let mut outputs = [0.0f64; MAX_OUTPUTS];
        self.graph.call(
            node as usize,
            &inputs[..n_in],
            &mut outputs[..def.outputs.len()],
        )?;

        // The evidence executes on the run, using the same function the test
        // calls — one implementation, checked one way.
        let (ran, passed) = self.graph.check_fixtures(node);
        let cred = credibility::score(def, ran, passed, data_ok, upstream);

        for (k, &o) in def.outputs.iter().enumerate() {
            let slot = &mut store.slots[o as usize];
            slot.value = outputs[k];
            slot.unit = self.graph.vars[o as usize].unit;
            slot.status = SlotStatus::Computed;
            slot.producer = node;
            slot.cred = cred;
        }
        Ok(())
    }
}

///
/// The same function the test calls, called from the run — one implementation,
/// checked one way. A verdict shown on a page has to have come from a run: a
/// badge read out of a field is a claim about last March.
impl Graph {
    /// Run one node's fixtures, on the run.
    ///
    /// The same function the test calls, called from the run — one implementation,
    /// checked one way. A verdict shown on a page has to have come from a run: a
    /// badge read out of a field is a claim about last March.
    fn check_fixtures(&self, node: NodeIdx) -> (bool, bool) {
        let def = &self.nodes[node as usize];
        if def.fixtures.is_empty() {
            return (false, false);
        }
        let mut ran = false;
        let mut passed = true;
        for (f, got) in def.fixtures.iter().zip(self.fixture_runs(node).iter()) {
            match got {
                Some(got) => {
                    ran = true;
                    if !f.check(*got).passed() {
                        passed = false;
                    }
                }
                // The node refused its own fixture's case. That is a verdict, and a
                // failing one: an expected value the implementation will not even
                // evaluate is stronger evidence of a defect than a numeric
                // disagreement.
                None => {
                    ran = true;
                    passed = false;
                }
            }
        }
        (ran, passed)
    }

    /// What node `node` gave on each of its own cases, in order: the slot the
    /// case names (a set row's case checks the member it is about; a row with
    /// one answer has slot 0), or `None` where the node refused it.
    fn run_fixtures(&self, node: NodeIdx) -> Vec<Option<f64>> {
        let def = &self.nodes[node as usize];
        def.fixtures
            .iter()
            .map(|f| {
                let mut out = [0.0f64; MAX_OUTPUTS];
                self.against_its_cases(node as usize, f.inputs, &mut out[..def.outputs.len()])
                    .ok()
                    .map(|()| out[f.slot])
            })
            .collect()
    }

    /// [`run_fixtures`], worked out once for each node and kept by the graph.
    /// A node's relation reads nothing but the inputs it is given, and a graph
    /// does not change once it is built, so its cases give the same answers
    /// every time they are asked: a sweep asks at every point, and with every
    /// method interpreted that asking was most of what a sweep cost. Every
    /// verdict is still the engine running the case, never a value read from a
    /// file.
    #[cfg(feature = "std")]
    fn fixture_runs(&self, node: NodeIdx) -> &[Option<f64>] {
        let all = self.cases_run.0.get_or_init(|| {
            (0..self.nodes.len())
                .map(|_| std::sync::OnceLock::new())
                .collect()
        });
        all[node as usize].get_or_init(|| self.run_fixtures(node))
    }

    #[cfg(not(feature = "std"))]
    fn fixture_runs(&self, node: NodeIdx) -> Vec<Option<f64>> {
        self.run_fixtures(node)
    }

    /// One fixture, executed against the live engine ([`fixture_verdicts`]).
    pub fn fixture_verdicts(&self, node: NodeIdx) -> Vec<vleo_bus::VerdictOut> {
        let def = &self.nodes[node as usize];
        let mut out = Vec::new();
        for (f, got) in def.fixtures.iter().zip(self.fixture_runs(node).iter()) {
            let (got, passed, err) = match *got {
                Some(got) => {
                    let e = match f.check(got) {
                        Verdict::Pass { relative_error } => relative_error,
                        Verdict::Fail { relative_error, .. } => relative_error,
                        _ => f64::NAN,
                    };
                    (got, f.check(got).passed(), e)
                }
                None => (f64::NAN, false, f64::NAN),
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
}

/// One fixture, executed against the live engine.
pub fn fixture_verdicts(node: NodeIdx) -> Vec<vleo_bus::VerdictOut> {
    engine().fixture_verdicts(node)
}

/// Evaluate ONE node's relation at supplied inputs, with no graph at all.
///
/// The design question and the relation question are different, and the engine
/// answers only the first. *What is the design's number* walks the graph: every
/// driver comes from whatever produces it, and a value supplied for a computed
/// row is rightly refused because the run would overwrite it. *What does this
/// relation do at these inputs* has no graph in it — it is what a fixture asks,
/// and `run_fixture` below has always asked it this way.
///
/// It became worth exposing when `env_f107` stopped being declared and started
/// reading the solar subsystem. The flux is no longer free, so a sweep over it
/// is meaningless to the resolver and correctly refused — but the coefficients
/// of the thermosphere relation are still a fact about the relation, and
/// checking one still means moving one driver while holding the others.
///
/// It is deliberately NOT a way to fake a design number. It returns the
/// relation's own outputs and touches no store, so nothing downstream can see
/// what a probe computed, and no run's provenance can contain one.
pub fn probe(node: NodeIdx, inputs: &[f64]) -> Result<[f64; MAX_OUTPUTS], Fault> {
    engine().probe(node, inputs)
}

/// One fixture, executed against the live engine.
pub fn run_fixture(node: NodeIdx, inputs: &[f64]) -> Result<(f64, Verdict), Fault> {
    engine().run_fixture(node, inputs)
}

impl Graph {
    /// One node's relation at supplied inputs, with no graph at all ([`probe`]).
    pub fn probe(&self, node: NodeIdx, inputs: &[f64]) -> Result<[f64; MAX_OUTPUTS], Fault> {
        let def = &self.nodes[node as usize];
        let mut outputs = [0.0f64; MAX_OUTPUTS];
        self.call(node as usize, inputs, &mut outputs[..def.outputs.len()])?;
        Ok(outputs)
    }

    /// One fixture, executed against the live engine ([`run_fixture`]).
    pub fn run_fixture(&self, node: NodeIdx, inputs: &[f64]) -> Result<(f64, Verdict), Fault> {
        let def = &self.nodes[node as usize];
        let mut outputs = [0.0f64; MAX_OUTPUTS];
        self.call(node as usize, inputs, &mut outputs[..def.outputs.len()])?;
        // The first fixture's own slot, so a set row's ad-hoc run reports the member
        // its first expected value is about rather than whichever came first.
        let slot = def.fixtures.first().map(|f| f.slot).unwrap_or(0);
        let got = outputs[slot];
        let verdict = def
            .fixtures
            .first()
            .map(|f| f.check(got))
            .unwrap_or(Verdict::NotRun);
        Ok((got, verdict))
    }
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
        Scratch::for_graph(engine())
    }

    /// The scratch a run on `graph` needs.
    pub fn for_graph(graph: &Graph) -> Scratch {
        let (node_count, var_count) = (graph.nodes.len(), graph.vars.len());
        Scratch {
            // VAR_COUNT, not NODE_COUNT. A slot is one VARIABLE, and a row
            // whose conclusion is a set publishes several — so the two counts
            // are only equal while every row publishes one. Sized at NODE_COUNT
            // this panicked on the first set row rather than returning a wrong
            // answer, which is the right failure, but it is not a failure worth
            // having: the array is indexed by a variable index everywhere.
            slots: alloc::vec![Slot::EMPTY; var_count],
            order: alloc::vec![0; node_count + 1],
            mark: alloc::vec![0; node_count + 1],
            // Every expansion pushes the inputs of its node (or of its whole
            // cycle), and a producer can be waiting more than once, so the
            // bound is the inputs, times the widest cycle, not the node count.
            stack: alloc::vec![0; node_count + 2 + input_edges(graph) * widest_cycle(graph)],
            ran: alloc::vec![0; node_count + 1],
            blocked: alloc::vec![0; node_count + 1],
            blocked_fault: alloc::vec![Fault::NotRun { node: "" }; node_count + 1],
        }
    }
}

/// Every input of every row: the edges of the dependency graph.
fn input_edges(graph: &Graph) -> usize {
    graph.nodes.iter().map(|d| d.inputs.len()).sum()
}

/// The most rows any declared cycle holds, and at least one.
fn widest_cycle(graph: &Graph) -> usize {
    graph
        .cases
        .iter()
        .flat_map(|c| c.cycles.iter())
        .map(|c| c.nodes.len())
        .max()
        .unwrap_or(1)
        .max(1)
}

/// Evaluate a case.
///
/// This is the whole engine surface. Every face — the browser, the desktop, the
/// command line, the rig console, the wheel — reaches it, and they all get the
/// same numbers because there is only one of it.
pub fn evaluate(case: &vleo_bus::Case, scratch: &mut Scratch) -> Result<vleo_bus::Results, Fault> {
    engine().evaluate(case, scratch)
}

impl Graph {
    /// Evaluate a case on this graph ([`evaluate`]). The graph lives for the
    /// run, as every graph the engine is given does.
    pub fn evaluate(
        &'static self,
        case: &vleo_bus::Case,
        scratch: &mut Scratch,
    ) -> Result<vleo_bus::Results, Fault> {
        let table = Vleo::on(self, case.data.clone());
        let target = self.find(&case.target).ok_or(Fault::NotRun {
            node: "unknown node",
        })?;
        // Asked again here, not only by the faces: a caller that skipped the
        // question must not get the declared design under another case's name.
        if self.case_refusal(case).is_some() {
            return Err(Fault::NotRun {
                node: "the case named",
            });
        }
        let base = self.case_of(case);

        let mut store = Store::new(&mut scratch.slots);

        // Declared values publish themselves, then the case overrides. A supply is
        // the start of a run rather than an edit to one, so nothing is marked stale
        // here.
        for (i, def) in self.nodes.iter().enumerate() {
            if def.kind == Kind::Declared {
                let mut out = [0.0f64; MAX_OUTPUTS];
                // Every slot the row declares, not the first. A declared row has one
                // today and the gate refuses a second, but this loop is the same
                // shape as the one in `eval` and costs nothing: a hard-coded 1 here
                // would be a silent truncation the day that gate rule is relaxed.
                if self.call(i, &[], &mut out[..def.outputs.len()]).is_ok() {
                    for (k, &o) in def.outputs.iter().enumerate() {
                        store.supply(o, out[k], self.vars[o as usize].unit);
                        store.slots[o as usize].cred =
                            credibility::score(def, false, false, true, CredVec([4; 8]));
                    }
                }
            }
        }
        // Declared, then the case, then whatever the face sends — the saved
        // inputs first and a person's unsaved edits after, so the last word is
        // always the one typed most recently.
        if let Some(b) = base {
            for (v, val) in b.supply {
                supply_checked(self, &mut store, *v, *val)?;
            }
        }
        for (id, val) in &case.supply {
            match self.find(id) {
                Some(v) => supply_checked(self, &mut store, v, *val)?,
                // `case_refusal` names the row; a face that did not ask it still
                // gets a refusal rather than a run without the value.
                None => {
                    return Err(Fault::Refused {
                        node: "the case",
                        reason: "it sets a row that does not exist",
                    })
                }
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
        let report = resolver::evaluate(
            &table,
            &mut store,
            case.mode.to_mode(target),
            &cycles,
            &mut ws,
        )?;

        Ok(collect(self, &store, &ws, &report, case, target))
    }
}

/// Supply a value, refusing rather than clamping when it is outside the
/// variable's declared range.
///
/// Out of domain is an error naming the field and the bound, at every face. A
/// value silently corrected is a design that drifted without anyone deciding
/// to.
fn supply_checked(
    graph: &Graph,
    store: &mut Store<'_>,
    v: NodeIdx,
    value: f64,
) -> Result<(), Fault> {
    let var = &graph.vars[v as usize];
    let def = &graph.nodes[var.producer as usize];
    // NaN passes both range comparisons below, so it is refused first.
    if !value.is_finite() {
        return Err(Fault::Refused {
            node: def.id,
            reason: "the value supplied is not a finite number",
        });
    }
    if value < var.limit.lower {
        return Err(Fault::OutOfDomain {
            node: def.id,
            field: var.symbol,
            // Stated in the variable's own unit, which the fault names:
            // 150000 m printed as "150000 km" once told a reader the wrong number.
            value: value / var.unit.si_factor(),
            bound: var.limit.lower / var.unit.si_factor(),
            edge: vleo_core::fault::Edge::Lower,
            unit: var.unit,
            reason: var.limit.reason_lower,
        });
    }
    if value > var.limit.upper {
        return Err(Fault::OutOfDomain {
            node: def.id,
            field: var.symbol,
            // Stated in the variable's own unit, which the fault names:
            // 150000 m printed as "150000 km" once told a reader the wrong number.
            value: value / var.unit.si_factor(),
            bound: var.limit.upper / var.unit.si_factor(),
            edge: vleo_core::fault::Edge::Upper,
            unit: var.unit,
            reason: var.limit.reason_upper,
        });
    }
    store.supply(v, value, var.unit);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect(
    graph: &Graph,
    store: &Store<'_>,
    ws: &Workspace<'_>,
    report: &RunReport,
    case: &vleo_bus::Case,
    target: NodeIdx,
) -> vleo_bus::Results {
    let mut values = Vec::new();
    for &n in ws.ran[..report.ran].iter() {
        let def = &graph.nodes[n as usize];
        for &o in def.outputs {
            let slot = store.get(o);
            let var = &graph.vars[o as usize];
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
            id: graph.nodes[n as usize].id.to_string(),
            kind: ws.blocked_fault[i].kind(),
            message: vleo_bus::fault_message(&ws.blocked_fault[i]),
        });
    }

    let target_cred = graph.nodes[target as usize]
        .outputs
        .first()
        .map(|&o| store.get(o).cred)
        .unwrap_or(CredVec::ZERO);

    let manifest = vleo_bus::RunManifest {
        node: case.target.clone(),
        mode: case.mode.name(),
        kernel: hex(graph.kernel_hash()),
        graph: hex(graph.graph_hash()),
        case: hex(case.hash()),
        chain: hex(report.chain_hash),
        data: case.data_versions.clone(),
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
        verdicts.extend(graph.fixture_verdicts(n));
    }
    vleo_bus::Results {
        values,
        blocked,
        verdicts,
        manifest,
        series: Vec::new(),
    }
}

pub(crate) fn hex(h: u64) -> String {
    let b = vleo_core::hash::short_hex(h);
    core::str::from_utf8(&b).unwrap_or("......").to_string()
}

/// Which factor is holding a subtree down, and which node it belongs to.
///
/// The management view asks one question — what is holding the design down —
/// and the answer is a node identifier rather than a meeting.
pub fn governing_node(store: &Store<'_>, subtree_root: NodeIdx) -> (Factor, &'static str, u8) {
    let nodes = nodes();
    let mut worst = (Factor::Mathematics, nodes[subtree_root as usize].id, 4u8);
    for (i, def) in nodes.iter().enumerate() {
        for &o in def.outputs {
            let slot = store.get(o);
            if slot.status != SlotStatus::Computed {
                continue;
            }
            let s = slot.cred.governing_score();
            if s < worst.2 {
                worst = (slot.cred.governing_factor(), nodes[i].id, s);
            }
        }
    }
    worst
}

/// The unit a variable is published in, for a face that has to draw it.
pub fn unit_of(v: NodeIdx) -> Unit {
    vars()[v as usize].unit
}
