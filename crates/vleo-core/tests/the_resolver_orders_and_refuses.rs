//! The resolver, on small graphs built here rather than the design's tree.
//!
//! Every run of the tool goes through `resolver::evaluate`, and until these it
//! was tested only through the whole tree — where a wrong order, a loop swept
//! twice or an entry dropped from a full stack shows up, if at all, as some
//! node far away blocking for a reason that is not the real one. Each property
//! is asked of a graph small enough to know the answer to by hand.

use vleo_core::credibility::{CredVec, Tier};
use vleo_core::fault::Fault;
use vleo_core::graph::{Kind, Limit, NodeDef, NodeTable, Retirement, State, VarDef, View};
use vleo_core::resolver::{evaluate, CycleSpec, Mode, RunReport, Workspace};
use vleo_core::units::Unit;
use vleo_core::value::{Slot, SlotStatus, Store};

type Rule = fn(&[f64]) -> f64;

/// A table of one-output nodes: node i publishes variable i.
struct Graph {
    nodes: Vec<NodeDef>,
    vars: Vec<VarDef>,
    rules: Vec<Rule>,
}

fn leak(v: Vec<u16>) -> &'static [u16] {
    Box::leak(v.into_boxed_slice())
}

fn name(i: usize) -> &'static str {
    Box::leak(format!("n{i}").into_boxed_str())
}

impl Graph {
    /// `spec[i]` is node i's inputs (as node indices) and its rule.
    fn new(spec: Vec<(Vec<u16>, Rule)>) -> Graph {
        let mut g = Graph {
            nodes: Vec::new(),
            vars: Vec::new(),
            rules: Vec::new(),
        };
        for (i, (inputs, rule)) in spec.into_iter().enumerate() {
            let id = name(i);
            g.nodes.push(NodeDef {
                id,
                label: id,
                subsystem: "",
                folder: "",
                parent: "",
                layer: 3,
                order: i as u32,
                crosses_to: "",
                kind: Kind::Computed,
                state: State::Published,
                retirement: Retirement::Live,
                owner: "",
                tier: Tier::A,
                question: "",
                expression: "",
                source: "",
                relation_by: "",
                derived: true,
                assumptions: &[],
                steps: &[],
                inputs: leak(inputs),
                outputs: leak(vec![i as u16]),
                contributes: &[],
                bundles: &[],
                fixtures: &[],
                sheet_hash: i as u64,
                impl_hash: i as u64,
                view: View::Number,
            });
            g.vars.push(VarDef {
                id,
                symbol: id,
                label: id,
                unit: Unit::One,
                producer: i as u16,
                limit: Limit::UNBOUNDED,
            });
            g.rules.push(rule);
        }
        g
    }
}

impl NodeTable for Graph {
    fn nodes(&self) -> &[NodeDef] {
        &self.nodes
    }
    fn vars(&self) -> &[VarDef] {
        &self.vars
    }
    fn eval(&self, node: u16, store: &mut Store<'_>) -> Result<(), Fault> {
        let def = &self.nodes[node as usize];
        let ins: Vec<f64> = def.inputs.iter().map(|&v| store.get(v).value).collect();
        let value = (self.rules[node as usize])(&ins);
        store.slots[node as usize] = Slot {
            value,
            unit: Unit::One,
            status: SlotStatus::Computed,
            producer: node,
            cred: CredVec::ZERO,
        };
        Ok(())
    }
}

/// Everything one run needs, with the stack a chosen size.
struct Run {
    slots: Vec<Slot>,
    order: Vec<u16>,
    mark: Vec<u8>,
    stack: Vec<u16>,
    ran: Vec<u16>,
    blocked: Vec<u16>,
    faults: Vec<Fault>,
}

impl Run {
    fn new(n: usize, stack: usize) -> Run {
        Run {
            slots: vec![Slot::EMPTY; n],
            order: vec![0; n + 1],
            mark: vec![0; n + 1],
            stack: vec![0; stack],
            ran: vec![0; n + 1],
            blocked: vec![0; n + 1],
            faults: vec![Fault::NotRun { node: "" }; n + 1],
        }
    }

    fn go(&mut self, g: &Graph, mode: Mode, cycles: &[CycleSpec<'_>]) -> Result<RunReport, Fault> {
        let mut store = Store::new(&mut self.slots);
        let mut ws = Workspace {
            order: &mut self.order,
            mark: &mut self.mark,
            stack: &mut self.stack,
            ran: &mut self.ran,
            blocked: &mut self.blocked,
            blocked_fault: &mut self.faults,
        };
        evaluate(g, &mut store, mode, cycles, &mut ws)
    }

    fn value(&self, i: u16) -> f64 {
        self.slots[i as usize].value
    }
}

fn one(_: &[f64]) -> f64 {
    1.0
}
fn plus_one(x: &[f64]) -> f64 {
    x[0] + 1.0
}
fn double(x: &[f64]) -> f64 {
    2.0 * x[0]
}
fn sum(x: &[f64]) -> f64 {
    x.iter().sum()
}
fn half_plus_one(x: &[f64]) -> f64 {
    0.5 * x[0] + 1.0
}
fn same(x: &[f64]) -> f64 {
    x[0]
}

#[test]
fn a_diamond_runs_each_node_once_after_its_inputs() {
    // 0 → 1, 0 → 2, (1, 2) → 3
    let g = Graph::new(vec![
        (vec![], one),
        (vec![0], plus_one),
        (vec![0], double),
        (vec![1, 2], sum),
    ]);
    let mut r = Run::new(4, 16);
    let rep = r.go(&g, Mode::Branch(3), &[]).unwrap();
    assert_eq!((rep.ran, rep.blocked), (4, 0));
    assert_eq!(r.value(3), 4.0);
    let at = |n: u16| r.ran[..4].iter().position(|&x| x == n).unwrap();
    assert!(at(0) < at(1) && at(0) < at(2) && at(1) < at(3) && at(2) < at(3));
}

#[test]
fn an_undeclared_loop_is_named_not_followed() {
    let g = Graph::new(vec![(vec![1], plus_one), (vec![0], plus_one)]);
    let mut r = Run::new(2, 8);
    match r.go(&g, Mode::Branch(0), &[]) {
        Err(Fault::UndeclaredCycle { .. }) => {}
        other => panic!("an undeclared loop gave {other:?}"),
    }
}

#[test]
fn a_declared_loop_converges_to_its_fixed_point() {
    // x = y/2 + 1, y = x  →  x = 2
    let g = Graph::new(vec![(vec![1], half_plus_one), (vec![0], same)]);
    let cy = [CycleSpec {
        nodes: &[0, 1],
        converge_on: 0,
        tolerance: 1e-12,
        max_iter: 200,
        seeds: &[(1, 0.0)],
    }];
    let mut r = Run::new(2, 8);
    let rep = r.go(&g, Mode::Branch(0), &cy).unwrap();
    assert_eq!((rep.ran, rep.blocked), (2, 0));
    assert!((r.value(0) - 2.0).abs() < 1e-9, "{}", r.value(0));
}

#[test]
fn a_loop_that_does_not_settle_says_so_for_every_member() {
    // x = y + 1, y = x: grows without bound.
    let g = Graph::new(vec![(vec![1], plus_one), (vec![0], same)]);
    let cy = [CycleSpec {
        nodes: &[0, 1],
        converge_on: 0,
        tolerance: 1e-9,
        max_iter: 20,
        seeds: &[(1, 0.0)],
    }];
    let mut r = Run::new(2, 8);
    let rep = r.go(&g, Mode::Branch(0), &cy).unwrap();
    assert_eq!((rep.ran, rep.blocked), (0, 2));
    assert!(matches!(rep.first_fault, Some(Fault::NotConverged { .. })));
}

#[test]
fn more_than_thirty_two_loops_are_each_swept_once() {
    // Forty independent two-node loops. A fixed table of 32 once let each
    // loop past the 32nd be swept a second time when its other member came up.
    const N: usize = 40;
    let mut spec = Vec::new();
    for k in 0..N {
        let a = (2 * k) as u16;
        spec.push((vec![a + 1], half_plus_one as Rule));
        spec.push((vec![a], same as Rule));
    }
    let g = Graph::new(spec);
    let cycles: Vec<CycleSpec<'_>> = (0..N)
        .map(|k| {
            let a = (2 * k) as u16;
            CycleSpec {
                nodes: leak(vec![a, a + 1]),
                converge_on: a,
                tolerance: 1e-12,
                max_iter: 200,
                seeds: Box::leak(vec![(a + 1, 0.0)].into_boxed_slice()),
            }
        })
        .collect();
    let mut one_loop = Run::new(2 * N, 8 * N);
    let single = one_loop
        .go(&g, Mode::Branch(0), &cycles)
        .unwrap()
        .iterations;
    let mut r = Run::new(2 * N, 8 * N);
    let rep = r.go(&g, Mode::All, &cycles).unwrap();
    assert_eq!((rep.ran, rep.blocked), (2 * N, 0));
    assert_eq!(
        rep.iterations,
        single * N as u32,
        "some loop was swept more than once"
    );
}

#[test]
fn a_stack_too_small_for_the_graph_is_named_not_truncated() {
    // The root reads every leaf and a middle node that reads every leaf too,
    // so each leaf waits on the stack twice: more entries than nodes.
    const K: u16 = 6;
    let mut spec: Vec<(Vec<u16>, Rule)> = (0..K).map(|_| (vec![], one as Rule)).collect();
    spec.push(((0..K).collect(), sum)); // middle = K
    let mut root: Vec<u16> = (0..K).collect();
    root.push(K);
    spec.push((root, sum)); // root = K + 1
    let g = Graph::new(spec);
    let n = K as usize + 2;
    let mut tight = Run::new(n, n + 1);
    match tight.go(&g, Mode::Branch(K + 1), &[]) {
        Err(Fault::WorkspaceTooSmall { .. }) => {}
        other => panic!("a full stack gave {other:?}"),
    }
    let mut roomy = Run::new(n, 4 * n);
    let rep = roomy.go(&g, Mode::Branch(K + 1), &[]).unwrap();
    assert_eq!((rep.ran, rep.blocked), (n, 0));
    assert_eq!(roomy.value(K + 1), 2.0 * K as f64);
}
