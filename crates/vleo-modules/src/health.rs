//! The health map: where exactly the design breaks.
//!
//! docs/OPERATING_1_0.md, section 6. Every node, group and closure gets one of
//! seven states, from one run of the whole design on the case asked for:
//!
//! | state | means |
//! |---|---|
//! | **closes** | answered, and every closure it feeds holds its margin |
//! | **tight** | closes, but within the margin its maturity demands |
//! | **fails** | a closure it feeds does not hold |
//! | **refused** | the node refused, and says why |
//! | **blocked** | it cannot run because a node it reads refused or is open; it names that node |
//! | **open** | not decided yet |
//! | **unproven** | it answers, but its own cases are not reproduced |
//!
//! A closure is a row whose answer is its signed margin — an achieved row or a
//! KPI, positive when the requirement is met in the sense it is stated — so a
//! closure fails when its margin is below zero, and nothing else is read to
//! say so. It rolls up group by group: a group is as bad as its worst node or
//! group, and the spacecraft as bad as its worst group, in the order
//! [`State`] lists them.
//!
//! **Trace to cause** ([`trace`]) starts at a closure that does not close and
//! walks down through what decides it. The nodes that cause it are named
//! first — a node upstream that refused, is open, or does not reproduce its own
//! cases, and, given what changed since the design last held
//! ([`trace_since`]), every node a release changed — with its group and its
//! owner. Then the declared inputs it reads are
//! ranked by how far each moves the margin across its declared range, each
//! with the value in that range that would make it close, when there is one.
//!
//! What it does not do, and says: **tight** is judged against the margin
//! policy the programme sets by maturity (section 9). The design declares none
//! yet, so no closure is judged tight; [`Health::notes`] says so rather than
//! inventing one. And **unproven** reads a node's own cases only: whether it
//! is signed, or behind its contract, is in the design's files (phase E).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use vleo_bus::{Case, RunMode};
use vleo_core::graph::{Kind, Maturity, PortState};

use crate::{Graph, NodeIdx, Scratch, GROUPS};

/// A node's state on the health map, worst first: a group is as bad as the
/// first of these any of its nodes has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    Fails,
    Refused,
    Blocked,
    Unproven,
    Open,
    Tight,
    Closes,
}

impl State {
    pub fn name(self) -> &'static str {
        match self {
            State::Fails => "fails",
            State::Refused => "refused",
            State::Blocked => "blocked",
            State::Unproven => "unproven",
            State::Open => "open",
            State::Tight => "tight",
            State::Closes => "closes",
        }
    }
}

/// One node on the map.
#[derive(Clone, Debug)]
pub struct Node {
    pub node: NodeIdx,
    pub state: State,
    /// Why, in a sentence.
    pub why: String,
    /// For a blocked node, the node it cannot run without that is itself
    /// refused or open — the one to look at, not the first in a chain.
    pub blocked_by: Option<NodeIdx>,
    /// For a closure, its margin as it answered.
    pub margin: Option<f64>,
}

/// One group on the map: its state, and the nodes that give it.
#[derive(Clone, Debug)]
pub struct Group {
    pub id: &'static str,
    pub state: State,
    /// The group's own nodes and every node below it with the group's state.
    pub worst: Vec<NodeIdx>,
}

/// The whole map.
#[derive(Clone, Debug)]
pub struct Health {
    pub nodes: Vec<Node>,
    pub groups: Vec<Group>,
    /// The spacecraft: as bad as its worst group.
    pub spacecraft: State,
    /// What the map could not judge, said rather than assumed.
    pub notes: Vec<String>,
}

impl Health {
    pub fn node(&self, k: NodeIdx) -> &Node {
        &self.nodes[k as usize]
    }

    /// How many nodes are in each state, worst first.
    pub fn counts(&self) -> Vec<(State, usize)> {
        let mut out = Vec::new();
        for s in [
            State::Fails,
            State::Refused,
            State::Blocked,
            State::Unproven,
            State::Open,
            State::Tight,
            State::Closes,
        ] {
            out.push((s, self.nodes.iter().filter(|n| n.state == s).count()));
        }
        out
    }
}

/// Whether a node is a closure: its answer is its own signed margin.
pub fn is_closure(graph: &Graph, k: NodeIdx) -> bool {
    matches!(graph.nodes[k as usize].kind, Kind::Achieved | Kind::Kpi)
}

/// The nodes `k` reads from directly: the producers of its inputs.
fn reads(graph: &Graph, k: usize) -> Vec<usize> {
    let mut out: Vec<usize> = graph.nodes[k]
        .inputs
        .iter()
        .map(|&v| graph.vars[v as usize].producer as usize)
        .filter(|&p| p != k)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Every node `k` depends on, nearest first.
fn upstream(graph: &Graph, k: usize) -> Vec<usize> {
    let mut seen = vec![false; graph.nodes.len()];
    seen[k] = true;
    let mut order = Vec::new();
    let mut frontier = vec![k];
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for n in frontier {
            for p in reads(graph, n) {
                if !seen[p] {
                    seen[p] = true;
                    order.push(p);
                    next.push(p);
                }
            }
        }
        frontier = next;
    }
    order
}

/// The health map of `graph` on `case` — its supply, base case and data; its
/// target and mode are the whole design's.
pub fn health(graph: &'static Graph, case: &Case) -> Health {
    let n = graph.nodes.len();
    let mut all = case.clone();
    all.mode = RunMode::All;
    if graph.find(&all.target).is_none() {
        all.target = graph.nodes[0].id.to_string();
    }
    let mut notes = Vec::new();
    let mut answer: Vec<Option<f64>> = vec![None; n];
    let mut fault: Vec<Option<(&'static str, String)>> = vec![None; n];
    match graph.evaluate(&all, &mut Scratch::for_graph(graph)) {
        Ok(r) => {
            for v in &r.values {
                if let Some(k) = graph.find(&v.id) {
                    answer[k as usize] = Some(v.value);
                }
            }
            for b in &r.blocked {
                if let Some(k) = graph.find(&b.id) {
                    fault[k as usize] = Some((b.kind, b.message.clone()));
                }
            }
        }
        Err(f) => notes.push(format!("the design did not run on this case: {f}")),
    }

    // Each node on its own: answered, refused, open, or waiting on another.
    let mut nodes: Vec<Node> = (0..n)
        .map(|k| {
            let def = &graph.nodes[k];
            let (state, why) = if !def.state.runnable() || !def.is_defined() {
                (
                    State::Open,
                    if !def.is_defined() {
                        "its relation is stated and never derived".to_string()
                    } else {
                        "it is not decided yet".to_string()
                    },
                )
            } else if let Some((kind, message)) = &fault[k] {
                match *kind {
                    "blocked" => (State::Blocked, message.clone()),
                    "not-run" => (State::Open, message.clone()),
                    _ => (State::Refused, message.clone()),
                }
            } else if answer[k].is_some() {
                (State::Closes, String::new())
            } else {
                (
                    State::Open,
                    "the run gave it no answer and no reason".to_string(),
                )
            };
            Node {
                node: k as NodeIdx,
                state,
                why,
                blocked_by: None,
                margin: None,
            }
        })
        .collect();

    // A blocked node names the one it waits on: the nearest node upstream that
    // is itself refused or open.
    for k in 0..n {
        if nodes[k].state == State::Blocked {
            nodes[k].blocked_by = upstream(graph, k)
                .into_iter()
                .find(|&p| matches!(nodes[p].state, State::Refused | State::Open))
                .map(|p| p as NodeIdx);
        }
    }

    // An answer its own cases do not reproduce is not one to stand on.
    for (k, node) in nodes.iter_mut().enumerate() {
        if node.state != State::Closes {
            continue;
        }
        let failed: Vec<String> = graph
            .fixture_verdicts(k as NodeIdx)
            .into_iter()
            .filter(|v| !v.passed)
            .map(|v| v.label)
            .collect();
        if !failed.is_empty() {
            node.state = State::Unproven;
            node.why = format!(
                "{} of its own cases not reproduced: {}",
                failed.len(),
                failed.join("; ")
            );
        }
    }

    // Each closure by its margin, and what it fails, everything it reads fails.
    let mut feeds_failing = vec![false; n];
    for k in 0..n {
        if !is_closure(graph, k as NodeIdx) {
            continue;
        }
        nodes[k].margin = answer[k];
        if let (State::Closes | State::Unproven, Some(m)) = (nodes[k].state, answer[k]) {
            if m < 0.0 {
                nodes[k].state = State::Fails;
                nodes[k].why = format!(
                    "its margin is {:.1}% — the requirement is not met",
                    m * 100.0
                );
                for p in upstream(graph, k) {
                    feeds_failing[p] = true;
                }
            }
        }
    }
    for (k, node) in nodes.iter_mut().enumerate() {
        if feeds_failing[k] && node.state == State::Closes {
            node.state = State::Fails;
            node.why = "a closure it feeds does not hold".to_string();
        }
    }
    notes.push(
        "no closure is judged tight: the design declares no margin policy by maturity \
         (docs/OPERATING_1_0.md, section 9), and the engine invents none"
            .to_string(),
    );

    // Valve by valve: a group is as bad as its worst node or group.
    let groups = roll_up(graph, &nodes);
    // Every group is under the programme's, so the spacecraft is as bad as
    // its worst node.
    let spacecraft = nodes.iter().map(|n| n.state).min().unwrap_or(State::Closes);
    Health {
        nodes,
        groups,
        spacecraft,
        notes,
    }
}

fn roll_up(graph: &Graph, nodes: &[Node]) -> Vec<Group> {
    // Every node under a group, at any depth.
    let under = |gid: &str| -> Vec<usize> {
        let mut ids = vec![gid];
        let mut i = 0;
        while i < ids.len() {
            for g in GROUPS.iter() {
                if g.parent == ids[i] && !ids.contains(&g.id) {
                    ids.push(g.id);
                }
            }
            i += 1;
        }
        (0..graph.nodes.len())
            .filter(|&k| ids.contains(&graph.nodes[k].parent))
            .collect()
    };
    GROUPS
        .iter()
        .map(|g| {
            let mine = under(g.id);
            let state = mine
                .iter()
                .map(|&k| nodes[k].state)
                .min()
                .unwrap_or(State::Closes);
            Group {
                id: g.id,
                state,
                worst: mine
                    .into_iter()
                    .filter(|&k| nodes[k].state == state)
                    .map(|k| k as NodeIdx)
                    .collect(),
            }
        })
        .collect()
}

/// A node that causes a closure not to close.
#[derive(Clone, Debug)]
pub struct Cause {
    pub node: NodeIdx,
    pub state: State,
    pub why: String,
    /// The group it is in, and who owns it.
    pub group: &'static str,
    pub owner: &'static str,
}

/// A declared input the closure reads, and what it does to the margin.
#[derive(Clone, Debug)]
pub struct Lever {
    pub node: NodeIdx,
    /// The margin at the bottom and the top of the input's declared range.
    pub at_lower: Option<f64>,
    pub at_upper: Option<f64>,
    /// The value in its declared range at which the closure begins to close,
    /// when there is one.
    pub closes_at: Option<f64>,
    pub group: &'static str,
    pub owner: &'static str,
}

impl Lever {
    /// How far it moves the margin across its range; zero when an end does
    /// not answer.
    pub fn swing(&self) -> f64 {
        match (self.at_lower, self.at_upper) {
            (Some(a), Some(b)) => (b - a).abs(),
            _ => 0.0,
        }
    }
}

/// Why a closure does not close, walked down to its causes.
#[derive(Clone, Debug)]
pub struct Trace {
    pub closure: NodeIdx,
    pub state: State,
    pub margin: Option<f64>,
    /// The nodes that cause it, nearest first.
    pub causes: Vec<Cause>,
    /// The declared inputs it reads, the one that moves the margin most first.
    pub levers: Vec<Lever>,
}

fn group_of(graph: &Graph, k: usize) -> (&'static str, &'static str) {
    let def = &graph.nodes[k];
    let g = GROUPS.iter().find(|g| g.id == def.parent);
    (
        g.map_or(def.parent, |g| g.id),
        if def.owner.is_empty() {
            g.map_or("", |g| g.owner)
        } else {
            def.owner
        },
    )
}

/// Trace a closure that does not close to the nodes that cause it.
pub fn trace(graph: &'static Graph, case: &Case, map: &Health, closure: NodeIdx) -> Trace {
    trace_since(graph, case, map, closure, &[])
}

/// Trace a closure, with what changed since the design last held: each node a
/// release changed, and which release, as `(node, said)`. A node that answers
/// and reproduces its own cases can still be what breaks a closure — a fault
/// its cases do not reach — and what tells it apart from every other row the
/// closure reads is that it changed. So a changed node upstream is a cause,
/// named with the change, whatever its state.
pub fn trace_since(
    graph: &'static Graph,
    case: &Case,
    map: &Health,
    closure: NodeIdx,
    changed: &[(NodeIdx, String)],
) -> Trace {
    let k = closure as usize;
    let mut causes = Vec::new();
    for p in core::iter::once(k).chain(upstream(graph, k)) {
        let s = map.nodes[p].state;
        let change = changed.iter().find(|(c, _)| *c as usize == p);
        if matches!(s, State::Refused | State::Open | State::Unproven) || change.is_some() {
            let (group, owner) = group_of(graph, p);
            let why = match (change, map.nodes[p].why.as_str()) {
                (Some((_, said)), "") => said.clone(),
                (Some((_, said)), why) => format!("{said}; {why}"),
                (None, why) => why.to_string(),
            };
            causes.push(Cause {
                node: p as NodeIdx,
                state: s,
                why,
                group,
                owner,
            });
        }
    }
    let levers = if map.nodes[k].margin.is_some() {
        levers_of(graph, case, k, |_| true)
    } else {
        Vec::new()
    };
    Trace {
        closure,
        state: map.nodes[k].state,
        margin: map.nodes[k].margin,
        causes,
        levers,
    }
}

/// The declared inputs closure `k` reads that `take` keeps, each with a finite
/// declared range, and what each does to its margin across that range, the
/// one that moves it most first.
fn levers_of(
    graph: &'static Graph,
    case: &Case,
    k: usize,
    take: impl Fn(usize) -> bool,
) -> Vec<Lever> {
    let margin_at = |var: usize, x: f64| -> Option<f64> {
        let mut c = case.clone();
        c.target = graph.nodes[k].id.to_string();
        c.mode = RunMode::Branch;
        c.supply.retain(|(id, _)| id != graph.vars[var].id);
        c.supply.push((graph.vars[var].id.to_string(), x));
        graph
            .evaluate(&c, &mut Scratch::for_graph(graph))
            .ok()?
            .values
            .iter()
            .find(|v| v.id == graph.nodes[k].id)
            .map(|v| v.value)
    };
    let mut levers = Vec::new();
    for p in upstream(graph, k) {
        let def = &graph.nodes[p];
        if def.kind != Kind::Declared || !take(p) {
            continue;
        }
        let var = def.outputs[0] as usize;
        let lim = graph.vars[var].limit;
        if !(lim.lower.is_finite() && lim.upper.is_finite() && lim.lower < lim.upper) {
            continue;
        }
        let (lo, hi) = (margin_at(var, lim.lower), margin_at(var, lim.upper));
        // Where in the range it begins to close: halve the interval on the
        // sign of the margin, from the end that fails to the end that holds.
        let closes_at = match (lo, hi) {
            (Some(a), Some(b)) if (a < 0.0) != (b < 0.0) => {
                let (mut fail, mut hold) = if a < 0.0 {
                    (lim.lower, lim.upper)
                } else {
                    (lim.upper, lim.lower)
                };
                for _ in 0..48 {
                    let mid = 0.5 * (fail + hold);
                    match margin_at(var, mid) {
                        Some(m) if m >= 0.0 => hold = mid,
                        Some(_) => fail = mid,
                        None => break,
                    }
                }
                Some(hold)
            }
            _ => None,
        };
        let (group, owner) = group_of(graph, p);
        levers.push(Lever {
            node: p as NodeIdx,
            at_lower: lo,
            at_upper: hi,
            closes_at,
            group,
            owner,
        });
    }
    levers.sort_by(|a, b| b.swing().total_cmp(&a.swing()));
    levers
}

/// What a closure says over the open values behind it (docs/SYSTEM_MODEL.md,
/// section 4, "Range"), one open value at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// No value it rests on is open: what it says is its margin.
    NoOpenValue,
    /// It holds at both ends of every open value's range: the decision can
    /// wait.
    ClosesForAll,
    /// It holds for part of the range: decide soon; each open value's
    /// crossing is in the tornado.
    ClosesForPart,
    /// It holds nowhere in the range: change the design, not the timing.
    FailsForAll,
}

impl Verdict {
    pub fn said(self) -> &'static str {
        match self {
            Verdict::NoOpenValue => "no value it rests on is open",
            Verdict::ClosesForAll => {
                "it closes for the whole range of every open value: the decision can wait"
            }
            Verdict::ClosesForPart => "it closes for part of the range: decide soon",
            Verdict::FailsForAll => {
                "it fails for all of the range: change the design, not the timing"
            }
        }
    }
}

/// A closure's range verdict, its tornado, and the least mature value it
/// rests on.
#[derive(Clone, Debug)]
pub struct Range {
    pub closure: NodeIdx,
    pub verdict: Verdict,
    /// The open values behind it, the one that moves its margin most first,
    /// each with its crossing when it has one.
    pub tornado: Vec<Lever>,
    /// How the least mature value behind it is known, and the rows known so,
    /// nearest first: it is the one whose margin the closure must carry.
    pub least_mature: (Maturity, Vec<NodeIdx>),
}

/// The range verdict of `closure` on `case`: the margin at both ends of every
/// open value's range, one at a time. Interactions between open values are
/// not judged; a variance-based index is the step after this one.
pub fn range(graph: &'static Graph, case: &Case, map: &Health, closure: NodeIdx) -> Range {
    let k = closure as usize;
    let open = |p: usize| {
        graph.nodes[p]
            .outputs
            .first()
            .is_some_and(|&v| graph.vars[v as usize].port.state == PortState::Open)
    };
    let tornado = levers_of(graph, case, k, open);
    let verdict = if tornado.is_empty() {
        Verdict::NoOpenValue
    } else {
        let ends: Vec<Option<f64>> = tornado
            .iter()
            .flat_map(|l| [l.at_lower, l.at_upper])
            .chain(core::iter::once(map.nodes[k].margin))
            .collect();
        if ends.iter().all(|m| m.is_some_and(|m| m >= 0.0)) {
            Verdict::ClosesForAll
        } else if ends.iter().all(|m| !m.is_some_and(|m| m >= 0.0)) {
            Verdict::FailsForAll
        } else {
            Verdict::ClosesForPart
        }
    };
    let mut least = (Maturity::Measured, Vec::new());
    for p in core::iter::once(k).chain(upstream(graph, k)) {
        for &v in graph.nodes[p].outputs.iter().take(1) {
            let m = graph.vars[v as usize].port.maturity;
            if m < least.0 {
                least = (m, Vec::new());
            }
            if m == least.0 {
                least.1.push(p as NodeIdx);
            }
        }
    }
    Range {
        closure,
        verdict,
        tornado,
        least_mature: least,
    }
}
