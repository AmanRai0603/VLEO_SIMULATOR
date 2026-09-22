//! The graph types.
//!
//! Three graphs, never merged — the mistake that would follow from merging them
//! is specific: execute the union and a KPI gets computed as though it were
//! derived; check coverage against the union and a navigation link counts as
//! evidence.
//!
//! | graph | edge | says | used for |
//! |---|---|---|---|
//! | derivation | variable → node | this is computed from that | execution |
//! | contribution | variable → KPI | this feeds that target | coverage |
//! | relation | group → group | these bear on each other | navigation, impact |
//!
//! Every edge is declared exactly once, by the end the edge *changes*: a
//! derivation edge by the consuming node (knowing its inputs changes its
//! implementation), a contribution edge by the contributing variable, a
//! relation edge by the layer file. Everything else — who consumes this, what
//! feeds a KPI, what a change reaches — is derived on assembly and never
//! stored, so it cannot go stale.
//!
//! None of these types is ever written by hand. They are generated into
//! `vleo-graph` from the sheets, and this crate holds only their shape.

use crate::credibility::Tier;
use crate::evidence::Fixture;
use crate::fault::Fault;
use crate::value::Store;
use vleo_units::Unit;

/// Index into the node table.
pub type NodeIdx = u16;
/// Index into the variable table.
pub type VarIdx = u16;

/// What kind of row this is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// It works out a number from its declared inputs. It has an algorithm,
    /// holes and fixtures.
    Computed,
    /// A person picked the number. It still needs a unit, a range, a reason per
    /// bound, a source, and who confirmed it — two-thirds of the tree is this,
    /// and every margin in the design is built out of them.
    Declared,
    /// A target handed down from the layer above, mirrored by an achieved row.
    Required,
    /// What a subsystem returned against a required row. Nothing closes until
    /// achieved meets required.
    Achieved,
    /// A key performance indicator: the thing a customer is promised.
    Kpi,
}

impl Kind {
    pub const fn name(self) -> &'static str {
        match self {
            Kind::Computed => "computed",
            Kind::Declared => "declared",
            Kind::Required => "required",
            Kind::Achieved => "achieved",
            Kind::Kpi => "kpi",
        }
    }
    pub fn from_name(s: &str) -> Option<Kind> {
        Some(match s {
            "computed" => Kind::Computed,
            "declared" => Kind::Declared,
            "required" => Kind::Required,
            "achieved" => Kind::Achieved,
            "kpi" => Kind::Kpi,
            _ => return None,
        })
    }
}

/// Where a node is in its life. The state lives in `meta.json`, written by the
/// gate — a person who can type `verified` can skip it, and then the colour on
/// the tree is a claim rather than a fact.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum State {
    /// The folder exists; the required fields are not filled.
    Empty,
    /// The sheet validates and both reviews have passed. Nothing is generated
    /// from an unreviewed sheet.
    Specified,
    /// Generated, the holes are filled, and it compiles.
    Implemented,
    /// The gate passes on this node. It is runnable.
    Verified,
    /// Merged. Other nodes may depend on it.
    Published,
    /// Being retired. What happens to consumers depends on why — see
    /// [`Retirement`].
    Deprecated,
}

impl State {
    pub const fn name(self) -> &'static str {
        match self {
            State::Empty => "empty",
            State::Specified => "specified",
            State::Implemented => "implemented",
            State::Verified => "verified",
            State::Published => "published",
            State::Deprecated => "deprecated",
        }
    }
    pub fn from_name(s: &str) -> Option<State> {
        Some(match s {
            "empty" => State::Empty,
            "specified" => State::Specified,
            "implemented" => State::Implemented,
            "verified" => State::Verified,
            "published" => State::Published,
            "deprecated" => State::Deprecated,
            _ => return None,
        })
    }
    /// Can the resolver run it?
    pub const fn runnable(self) -> bool {
        matches!(self, State::Verified | State::Published | State::Deprecated)
    }
}

/// Why a node was retired — and therefore what happens downstream.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Retirement {
    /// Not retired.
    Live,
    /// A better model exists. Consumers keep running and migrate when
    /// convenient — nothing computed so far is wrong.
    Superseded { replacement: &'static str },
    /// Folded into another node. Same treatment.
    Merged { replacement: &'static str },
    /// The physics was found to be incorrect. Every consumer is marked suspect
    /// and their runs refuse, naming the replacement — retirement must not
    /// protect people from finding out.
    Wrong { replacement: &'static str },
}

/// One declared limit on a variable, with the reason it exists.
///
/// The reason is not decoration. A guard whose reason is not written down gets
/// deleted by the next person who finds it inconvenient.
#[derive(Clone, Copy, Debug)]
pub struct Limit {
    pub lower: f64,
    pub upper: f64,
    pub reason_lower: &'static str,
    pub reason_upper: &'static str,
}

impl Limit {
    pub const UNBOUNDED: Limit = Limit {
        lower: f64::NEG_INFINITY,
        upper: f64::INFINITY,
        reason_lower: "",
        reason_upper: "",
    };
}

/// A variable: one named, typed quantity that some node produces.
#[derive(Clone, Copy, Debug)]
pub struct VarDef {
    /// Stable identifier, e.g. `prop_intake_capture_ratio`.
    pub id: &'static str,
    /// The symbol as it appears in the mathematics, e.g. `R`.
    pub symbol: &'static str,
    /// What it is, in ordinary words.
    pub label: &'static str,
    /// The unit it is stored and published in.
    pub unit: Unit,
    /// The node that produces it.
    pub producer: NodeIdx,
    /// The range over which it is meant to be valid, and why.
    pub limit: Limit,
}

/// A node: one small question with one answer.
#[derive(Clone, Copy, Debug)]
pub struct NodeDef {
    /// The module path, which *is* the identifier — the graph is walked from
    /// the module tree, never from a registry that can drift.
    pub id: &'static str,
    /// The row label shown on the tree.
    pub label: &'static str,
    /// The subsystem crate it lives in.
    pub subsystem: &'static str,
    /// Where the node's directory is, relative to the repository root.
    ///
    /// Carried rather than reconstructed. A face that rebuilds this path from
    /// the id and the subsystem is a second implementation of the layout rule,
    /// and it goes wrong on the first row whose folder is not its id minus a
    /// prefix — silently, as a 404 nobody attributes to a layout change.
    pub folder: &'static str,
    /// The layer group it hangs under.
    pub parent: &'static str,
    /// Which of the four layers this row lives in.
    pub layer: u8,
    /// Where the row sits among its siblings, as whoever wrote them ordered
    /// them. The node table is folder-ordered so that generation is
    /// deterministic; a face that draws the tree sorts by this instead, or it
    /// draws an alphabetised list nobody wrote.
    pub order: u32,
    /// The layer group this node crosses to, or empty for an ordinary row.
    ///
    /// Exactly one node crosses between any two layers, and it is the only
    /// route in: a subsystem is reached through its interface node, never by
    /// reaching into it.
    pub crosses_to: &'static str,
    pub kind: Kind,
    pub state: State,
    pub retirement: Retirement,
    /// The team that owns the path. Reviews route here.
    pub owner: &'static str,
    pub tier: Tier,
    /// The question, in ordinary words. An equation with no stated question
    /// gets reused for the wrong thing.
    pub question: &'static str,
    /// The relation, as it is written on the page.
    pub expression: &'static str,
    /// The identifier of a row in `sources/`.
    pub source: &'static str,
    /// Who supplied this relation, and when — the sheet's `[maths]
    /// confirmed_by`.
    ///
    /// A citation says where a relation was found. This says a person read it
    /// there. Only the second rules out a relation an agent invented, and an
    /// invented relation that runs cleanly looks exactly like a cited one — so
    /// this is what the mathematics factor of the credibility vector is scored
    /// on, not the citation.
    ///
    /// It does not decide whether the row answers. That is `derived`.
    pub relation_by: &'static str,
    /// Whether the sheet carries the relation's derivation — its `[theory]`:
    /// why it is this relation, what the answer means, and the steps it comes
    /// in.
    ///
    /// The prose itself stays on the page; the fact is here, because the fact
    /// is what decides whether the row answers. A relation that is stated and
    /// never derived has an expression, a citation and a number it would
    /// happily print, and nothing a reader can check it against. See
    /// [`NodeDef::is_defined`] and [`crate::fault::Fault::Undefined`].
    pub derived: bool,
    /// Each assumption, and the condition under which it stops holding.
    pub assumptions: &'static [(&'static str, &'static str)],
    /// The numbered algorithm. Each step becomes one hole.
    pub steps: &'static [&'static str],
    /// Variables it reads. This is the derivation graph, declared by the
    /// consumer.
    pub inputs: &'static [VarIdx],
    /// Variables it publishes.
    pub outputs: &'static [VarIdx],
    /// KPIs it contributes to. This is the contribution graph, declared by the
    /// variable.
    pub contributes: &'static [&'static str],
    /// Reference-data bundles it needs before it can run at all.
    pub bundles: &'static [&'static str],
    /// Its expected values, with provenance.
    pub fixtures: &'static [Fixture],
    /// Hash of the sheet this node was generated from. A page whose sheet hash
    /// differs from the engine's refuses to run and says so.
    pub sheet_hash: u64,
    /// Hash of the filled implementation. Part of the chain hash, which is what
    /// makes a cached result notice that the arithmetic under it changed.
    pub impl_hash: u64,
    /// How the result is drawn. `None` is the default: a number with its units,
    /// its verdict and its provenance — which is what all but a handful of
    /// nodes want.
    pub view: View,
}

impl NodeDef {
    /// Is this row a function — something that works a number out — as against
    /// a row that states one?
    ///
    /// Asked of the algorithm rather than of `kind`, because the algorithm is
    /// the thing that runs. A row with steps has a body; a row without one
    /// publishes what it was handed.
    pub fn is_function(&self) -> bool {
        !self.steps.is_empty()
    }

    /// Has a person defined it?
    ///
    /// The two halves of the tree answer this differently, and they should.
    ///
    /// An **input** is defined by carrying a value. A default is a definition:
    /// somebody chose it, `[value] confirmed_by` says who, and the face lets it
    /// be edited — that is the whole point of an input.
    ///
    /// A **function** is defined by its derivation. Not by its expression,
    /// which is one line anybody can type, and not by its citation, which says
    /// a paper exists rather than that this relation came out of it. The
    /// derivation is the part that says where the relation came from and what
    /// its answer means, and it is the one thing a reader cannot recover from
    /// any other field — so a function without it is a relation nobody has
    /// accounted for, and the number it would print is the scaffold's.
    ///
    /// This is deliberately a weaker bar than `relation_by`, which is a second
    /// person reading the relation against its source. That one is worth
    /// credibility, not silence: a derived-but-unconfirmed row answers, and
    /// says on its page that nobody has checked it. A row that is neither does
    /// not answer at all. See [`crate::fault::Fault::Undefined`].
    pub fn is_defined(&self) -> bool {
        !self.is_function() || self.derived
    }
}

/// How a node's result is drawn. Declared on the sheet, in the same file as the
/// physics, so a node author never writes display code and one component in the
/// shell draws every node in the tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// A formatted number with its unit, verdict and provenance.
    Number,
    /// A curve of one output against one swept input.
    Line {
        over: &'static str,
        y: &'static str,
        points: u32,
    },
    /// A field of one output over two swept inputs.
    Heatmap {
        over_x: &'static str,
        over_y: &'static str,
        z: &'static str,
        points: u32,
    },
    /// A bar per named contribution — budgets and error trees.
    Bar { y: &'static str },
}

/// What the resolver needs from the generated tables.
///
/// The resolver lives here and the tables live in `vleo-graph` and
/// `vleo-modules`, so this trait is the seam between a kernel that can be
/// reviewed on its own and a graph that changes every time a node is added.
pub trait NodeTable {
    fn nodes(&self) -> &[NodeDef];
    fn vars(&self) -> &[VarDef];
    /// Run one node against the store. Implemented by generated code that
    /// reads the declared inputs out of the store, calls the node's typed
    /// function, and writes the declared outputs back.
    fn eval(&self, node: NodeIdx, store: &mut Store<'_>) -> Result<(), Fault>;

    fn node(&self, i: NodeIdx) -> &NodeDef {
        &self.nodes()[i as usize]
    }
    fn var(&self, i: VarIdx) -> &VarDef {
        &self.vars()[i as usize]
    }
    /// Find a node by its identifier. Linear — the tables are small, and a
    /// perfect hash would be a second thing that can be wrong.
    fn find_node(&self, id: &str) -> Option<NodeIdx> {
        self.nodes()
            .iter()
            .position(|n| n.id == id)
            .map(|i| i as NodeIdx)
    }
    fn find_var(&self, id: &str) -> Option<VarIdx> {
        self.vars()
            .iter()
            .position(|v| v.id == id)
            .map(|i| i as VarIdx)
    }
}
