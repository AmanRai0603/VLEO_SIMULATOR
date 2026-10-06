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
use vleo_core::graph::{Kind, NodeDef, NodeIdx, NodeTable, VarDef};
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

pub use tables::{
    CaseDef, CycleDef, GroupDef, CASES, GROUPS, MAX_INPUTS, MAX_OUTPUTS, NODES, NODE_COUNT,
    RELATIONS, VARS, VAR_COUNT,
};

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
}

/// The graph compiled into this build, from the sheets as they were.
pub static COMPILED: Graph = Graph {
    nodes: &NODES,
    vars: &VARS,
    dispatch: &tables::DISPATCH,
    run: &[],
    cases: &CASES,
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

/// Run the engine on `graph` from now on (docs/PLAN_1_0.md, phase D: every
/// face is re-pointed).
///
/// Refused unless the graph is laid out as the compiled one is — the same
/// rows, variables and cases, in the same order — because a face still reads
/// the compiled tables to name and arrange what the engine answers, and a
/// graph laid out otherwise would be answered under the wrong names. A design
/// that differs from this build runs when the faces read it from the graph as
/// well; until then it is refused here, by name.
#[cfg(feature = "std")]
pub fn run_on(graph: &'static Graph) -> Result<(), Error> {
    if let Some(why) = laid_out_otherwise(graph) {
        return Err(Error::new(
            ErrorKind::Invalid,
            alloc::format!(
                "this design is laid out otherwise than this build of the engine: {why}"
            ),
        ));
    }
    *ENGINE.write().unwrap_or_else(|e| e.into_inner()) = Some(graph);
    Ok(())
}

/// Run the engine on the compiled graph again.
#[cfg(feature = "std")]
pub fn run_compiled() {
    *ENGINE.write().unwrap_or_else(|e| e.into_inner()) = None;
}

/// The first way `graph` is laid out otherwise than the compiled one, if any.
#[cfg(feature = "std")]
fn laid_out_otherwise(graph: &Graph) -> Option<String> {
    let c = &COMPILED;
    if graph.nodes.len() != c.nodes.len() {
        return Some(alloc::format!(
            "{} rows, where this build has {}",
            graph.nodes.len(),
            c.nodes.len()
        ));
    }
    for (a, b) in graph.nodes.iter().zip(c.nodes) {
        if a.id != b.id || a.inputs != b.inputs || a.outputs != b.outputs {
            return Some(alloc::format!(
                "the row {} is not where this build has {}",
                a.id,
                b.id
            ));
        }
    }
    if graph.vars.len() != c.vars.len() {
        return Some(alloc::format!(
            "{} values, where this build has {}",
            graph.vars.len(),
            c.vars.len()
        ));
    }
    for (a, b) in graph.vars.iter().zip(c.vars) {
        if a.id != b.id {
            return Some(alloc::format!(
                "the value {} is not where this build has {}",
                a.id,
                b.id
            ));
        }
    }
    let ids = |g: &Graph| g.cases.iter().map(|k| k.id).collect::<Vec<_>>();
    if ids(graph) != ids(c) {
        return Some("its cases are not this build's".into());
    }
    None
}

impl Graph {
    /// Run node `i`'s relation: the graph's own when it has one, else the
    /// compiled function.
    pub fn call(&self, i: usize, inputs: &[f64], outputs: &mut [f64]) -> Result<(), Fault> {
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

// MAX_INPUTS and MAX_OUTPUTS come from `tables`, measured off the tree by the
// generator rather than written here by hand. Both were hand-written once — 16
// and 4 — and both were outgrown. The output cap panicked on the first row whose
// answer was a set, which is a loud failure and an acceptable one. The INPUT cap
// did not: `eval` sliced to it, so a row declaring more inputs than the cap
// silently received fewer, and what surfaced was the generated length guard
// refusing the short slice — reported as a node blocked on "an input that has
// never run", which is not what had happened. A constant that can be outgrown
// by a sheet belongs to the sheet, so it is generated.

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
        for f in def.fixtures {
            let mut out = [0.0f64; MAX_OUTPUTS];
            match self.call(node as usize, f.inputs, &mut out[..def.outputs.len()]) {
                Ok(()) => {
                    ran = true;
                    // The slot the fixture named. A set row's fixture checks the
                    // member it is about; a row with one answer has slot 0.
                    if !f.check(out[f.slot]).passed() {
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

    /// One fixture, executed against the live engine ([`fixture_verdicts`]).
    pub fn fixture_verdicts(&self, node: NodeIdx) -> Vec<vleo_bus::VerdictOut> {
        let def = &self.nodes[node as usize];
        let mut out = Vec::new();
        for f in def.fixtures {
            let mut o = [0.0f64; MAX_OUTPUTS];
            let (got, passed, err) =
                match self.call(node as usize, f.inputs, &mut o[..def.outputs.len()]) {
                    Ok(()) => {
                        let got = o[f.slot];
                        let e = match f.check(got) {
                            Verdict::Pass { relative_error } => relative_error,
                            Verdict::Fail { relative_error, .. } => relative_error,
                            _ => f64::NAN,
                        };
                        (got, f.check(got).passed(), e)
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
